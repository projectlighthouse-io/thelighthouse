//! What the catalogue costs, reconciled against what stripe holds.
//!
//! ```text
//!   lighthouse-prices status   what stripe has, and where it differs
//!   lighthouse-prices apply    make stripe match, then write the api's config
//! ```
//!
//! A separate binary for the same reason `lighthouse-migrate` is one: creating
//! a price is a decision, not a side effect of starting a server. The api never
//! writes to stripe — it reads a config file this produces.
//!
//! # Why a reconciler rather than a config file of ids
//!
//! **A stripe price is immutable in its amount.** Changing what something costs
//! is not an edit; it is a new price object, and everything pointing at the old
//! one has to be repointed. Copying the new id into a config file by hand is
//! the step that gets skipped, and the symptom is a checkout charging last
//! quarter's price.
//!
//! So nothing here names a price id. Each plan carries a `lookup_key` — its own
//! name, `rust_yearly` — and stripe lets exactly one price hold a given key at
//! a time. Changing an amount creates a price and *transfers* the key onto it,
//! which is one call and cannot half-happen.
//!
//! # What is the truth
//!
//! `pricing.yaml` is. It says what a track costs; stripe is made to agree. The
//! opposite arrangement — amounts living at stripe, read back for display —
//! reads well until you need to know what the price *should* be, and the answer
//! is only in a dashboard nobody diffs.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    process::ExitCode,
};

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

/// Where the declaration lives, unless `PRICING` says otherwise.
const DEFAULT_DECLARATION: &str = "pricing.yaml";

/// Where the api reads its plans from, unless `BILLING_PLANS` says otherwise.
const DEFAULT_OUTPUT: &str = "billing.yaml";

/// Where the frontend reads the catalogue from, unless `CATALOGUE` says
/// otherwise. Relative to this repo's root, which is where the binary is run.
const DEFAULT_CATALOGUE: &str = "web/app/data/Catalogue.ts";

const STRIPE: &str = "https://api.stripe.com";

/// What a track costs, as declared.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Declaration {
    /// ISO 4217, lowercase. One currency for the whole catalogue: selling in
    /// several is a product decision with tax consequences, not a config key.
    currency: String,
    plans: Vec<Declared>,
    /// Purchasing-power tiers. Absent is none, which is what every declaration
    /// written before this field said.
    #[serde(default)]
    ppp: Vec<DeclaredPpp>,
}

/// One purchasing-power tier: a percentage, and the countries it is meant for.
///
/// **Two stripe objects, not one.** A coupon holds the percentage; a promotion
/// code is the string a reader types. The api advertises the code and stripe
/// resolves it — which is why the code is declared here and the coupon id is
/// not: the code is a decision, the id is stripe's answer to it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredPpp {
    /// What the reader types at checkout. Also the coupon's lookup id, so one
    /// name identifies the tier everywhere.
    code: String,
    /// Percent off, 1 to 99. A hundred is a free plan, which is a different
    /// decision and not this one.
    percent: u8,
    /// ISO 3166-1 alpha-2, the same codes cloudflare reports. Empty is every
    /// country, including a request cloudflare named no country for.
    #[serde(default)]
    countries: Vec<String>,
    /// The plan ids this tier discounts. Empty is all of them, which is what
    /// every declaration written before this field said.
    ///
    /// Becomes the coupon's `applies_to[products]` at stripe, so stripe
    /// enforces it — a reader who types the code against a plan it does not
    /// name is refused there rather than trusted here.
    #[serde(default)]
    plans: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Declared {
    /// `rust_yearly`. The track before the last underscore, the billing period
    /// after it — and the lookup key stripe holds the price under.
    id: String,
    /// Minor units. 4900 is $49.00.
    amount: i64,
    /// What a reader sees on the stripe page. Not used to match anything.
    name: String,
}

impl Declared {
    /// `year` for a plan that renews, `once` for one bought outright.
    ///
    /// Derived from the name rather than declared, so the id and the interval
    /// cannot disagree — the api derives the track from the same name the same
    /// way.
    fn interval(&self) -> Result<&'static str, String> {
        match self.id.rsplit_once('_') {
            Some((_, "yearly")) => Ok("year"),
            Some((_, "monthly")) => Ok("month"),
            Some((_, "lifetime")) => Ok("once"),
            _ => Err(format!(
                "{} does not end in _yearly, _monthly or _lifetime, so its \
                 billing period cannot be read from its name",
                self.id
            )),
        }
    }
}

/// A price as stripe currently holds it.
#[derive(Debug, Deserialize)]
struct Price {
    id: String,
    unit_amount: Option<i64>,
    currency: String,
    product: String,
    lookup_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Prices {
    data: Vec<Price>,
}

#[derive(Debug, Deserialize)]
struct Created {
    id: String,
}

/// What stripe holds for one plan once `apply` is done with it.
///
/// Two ids because they are named by different things: a checkout session
/// takes the *price*, and a coupon's `applies_to` takes the *product*.
#[derive(Debug)]
struct Resolved {
    price: String,
    product: String,
}

/// What has to happen to a plan for stripe to match the declaration.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// Stripe already holds this price, at this amount.
    ///
    /// The product rides along because a coupon's `applies_to` names products,
    /// not prices, and this is the only place the held one is read.
    Agrees { id: String, product: String },
    /// No price holds this lookup key.
    Missing,
    /// A price holds the key, at a different amount or currency.
    Differs {
        held: i64,
        currency: String,
        id: String,
        product: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(reason) = run().await {
        eprintln!("{reason}");

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn run() -> Result<(), String> {
    // Absent is fine; the keys can come from the real environment.
    if let Err(error) = dotenvy::dotenv()
        && !error.not_found()
    {
        return Err(format!("cannot read .env: {error}"));
    }

    let command = std::env::args().nth(1).unwrap_or_default();

    let declaration_path = std::env::var("PRICING")
        .unwrap_or_else(|_| DEFAULT_DECLARATION.to_owned());
    let declaration = read(&declaration_path)?;

    // Before the key is even looked for: `catalogue` writes a file out of two
    // files, and asking for a stripe secret to do that would mean the frontend
    // could not be built without one.
    if command == "catalogue" {
        let output = std::env::var("CATALOGUE")
            .unwrap_or_else(|_| DEFAULT_CATALOGUE.to_owned());

        let written = catalogue(&declaration, &output)?;
        println!("wrote {output} — {written}");

        return Ok(());
    }

    let key: SecretString = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| "STRIPE_SECRET_KEY is not set in .env or the environment")?
        .into();

    let client = reqwest::Client::builder()
        .build()
        .map_err(|error| format!("cannot build an http client: {error}"))?;

    let held = fetch(&client, &key, &declaration).await?;
    let verdicts = compare(&declaration, &held);

    match command.as_str() {
        "status" => {
            report(&verdicts);

            Ok(())
        }
        "apply" => {
            let resolved =
                apply(&client, &key, &declaration, &verdicts).await?;

            if !declaration.ppp.is_empty() {
                println!("\npurchasing power");
            }

            let promotions =
                apply_ppp(&client, &key, &declaration.ppp, &resolved).await?;

            let output = std::env::var("BILLING_PLANS")
                .unwrap_or_else(|_| DEFAULT_OUTPUT.to_owned());

            write(&output, &declaration, &resolved, &promotions)?;
            println!("wrote {output}");

            Ok(())
        }
        other => Err(format!(
            "unknown command {other:?}\n\n  \
             lighthouse-prices status     what stripe has, and where it differs\n  \
             lighthouse-prices apply      make stripe match, then write the config\n  \
             lighthouse-prices catalogue  write the frontend's build-time catalogue"
        )),
    }
}

fn read(path: &str) -> Result<Declaration, String> {
    let document = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read {path}: {error}"))?;

    let declaration: Declaration = serde_norway::from_str(&document)
        .map_err(|error| format!("cannot read {path}: {error}"))?;

    if declaration.plans.is_empty() {
        return Err(format!("{path} declares no plans"));
    }

    // Every interval has to be readable before anything is sent, so a typo in
    // the last plan does not leave the first four already created.
    for plan in &declaration.plans {
        plan.interval()?;
    }

    check_ppp(&declaration.ppp, &declaration.plans, path)?;

    Ok(declaration)
}

/// Refuses a tier list that cannot mean one thing.
///
/// All of it before anything is sent, for the reason the interval check gives:
/// a mistake in the last tier should not leave the first two already created at
/// stripe.
///
/// **A country may appear once.** Twice and a reader in it has two coupons and
/// the api has to pick, which is a decision nobody wrote down — so it is
/// refused here rather than resolved at request time.
///
/// **A tier naming no countries is offered to everyone.** That is a list price
/// with a discount off it, not purchasing power, and it is the same two stripe
/// objects — so it is spelled as an empty country list rather than a second
/// mechanism. One of them at most, for the reason a country may appear once:
/// two catch-alls and the api is picking again.
fn check_ppp(
    tiers: &[DeclaredPpp],
    plans: &[Declared],
    path: &str,
) -> Result<(), String> {
    let mut claimed: BTreeMap<String, String> = BTreeMap::new();
    let mut codes: BTreeSet<String> = BTreeSet::new();
    let mut universal: Option<&str> = None;

    for tier in tiers {
        if tier.code.trim().is_empty() {
            return Err(format!("{path}: a ppp tier has no code"));
        }

        if !codes.insert(tier.code.clone()) {
            return Err(format!(
                "{path}: two ppp tiers use the code {}",
                tier.code
            ));
        }

        if !(1..=99).contains(&tier.percent) {
            return Err(format!(
                "{path}: {} is {} percent off, which is not between 1 and 99",
                tier.code, tier.percent
            ));
        }

        // A plan the declaration does not sell cannot be discounted, and
        // naming one is a typo that would otherwise reach stripe as a coupon
        // restricted to nothing — which discounts nothing, silently.
        for plan in &tier.plans {
            if !plans.iter().any(|declared| &declared.id == plan) {
                return Err(format!(
                    "{path}: {} applies to {plan}, which is not a declared plan",
                    tier.code
                ));
            }
        }

        if tier.countries.is_empty() {
            if let Some(first) = universal {
                return Err(format!(
                    "{path}: {first} and {} are both offered to every country",
                    tier.code
                ));
            }

            universal = Some(&tier.code);
            continue;
        }

        for country in &tier.countries {
            // The shape cloudflare reports, so the api can compare without
            // normalising either side.
            let shaped = country.len() == 2
                && country.bytes().all(|b| b.is_ascii_uppercase());

            if !shaped {
                return Err(format!(
                    "{path}: {country} is not a two letter uppercase country code"
                ));
            }

            if let Some(first) =
                claimed.insert(country.clone(), tier.code.clone())
            {
                return Err(format!(
                    "{path}: {country} is claimed by both {first} and {}",
                    tier.code
                ));
            }
        }
    }

    Ok(())
}

/// Every price stripe currently holds under one of our lookup keys.
///
/// One call rather than one per plan: `lookup_keys` takes a list, and a plan
/// with no price simply does not come back.
async fn fetch(
    client: &reqwest::Client,
    key: &SecretString,
    declaration: &Declaration,
) -> Result<BTreeMap<String, Price>, String> {
    let mut query: Vec<(String, String)> = declaration
        .plans
        .iter()
        .map(|plan| ("lookup_keys[]".to_owned(), plan.id.clone()))
        .collect();

    query.push(("limit".to_owned(), "100".to_owned()));

    let response = client
        .get(format!("{STRIPE}/v1/prices"))
        .basic_auth(key.expose_secret(), None::<&str>)
        .query(&query)
        .send()
        .await
        .map_err(|error| format!("cannot reach stripe: {error}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("cannot read stripe's answer: {error}"))?;

    if !status.is_success() {
        return Err(format!(
            "stripe refused the price listing: {status} {body}"
        ));
    }

    let prices: Prices = serde_json::from_str(&body)
        .map_err(|error| format!("cannot read stripe's answer: {error}"))?;

    Ok(prices
        .data
        .into_iter()
        .filter_map(|price| price.lookup_key.clone().map(|key| (key, price)))
        .collect())
}

fn compare(
    declaration: &Declaration,
    held: &BTreeMap<String, Price>,
) -> Vec<(String, Verdict)> {
    let mut verdicts = Vec::new();

    for plan in &declaration.plans {
        let verdict = match held.get(&plan.id) {
            None => Verdict::Missing,
            Some(price) => {
                let amount = price.unit_amount.unwrap_or_default();

                if amount == plan.amount
                    && price.currency == declaration.currency
                {
                    Verdict::Agrees {
                        id: price.id.clone(),
                        product: price.product.clone(),
                    }
                } else {
                    Verdict::Differs {
                        held: amount,
                        currency: price.currency.clone(),
                        id: price.id.clone(),
                        product: price.product.clone(),
                    }
                }
            }
        };

        verdicts.push((plan.id.clone(), verdict));
    }

    verdicts
}

fn report(verdicts: &[(String, Verdict)]) {
    for (plan, verdict) in verdicts {
        match verdict {
            Verdict::Agrees { id, .. } => {
                println!("agrees   {plan:24} {id}");
            }
            Verdict::Missing => {
                println!("missing  {plan:24} no price holds this key");
            }
            Verdict::Differs { held, currency, .. } => {
                println!("differs  {plan:24} stripe holds {held} {currency}");
            }
        }
    }

    let pending = verdicts
        .iter()
        .filter(|(_, verdict)| !matches!(verdict, Verdict::Agrees { .. }))
        .count();

    if pending == 0 {
        println!("\nstripe matches the declaration");
    } else {
        println!("\n{pending} to apply");
    }
}

/// Make stripe agree, and give back what it now holds for every plan.
async fn apply(
    client: &reqwest::Client,
    key: &SecretString,
    declaration: &Declaration,
    verdicts: &[(String, Verdict)],
) -> Result<BTreeMap<String, Resolved>, String> {
    let mut resolved = BTreeMap::new();

    for (plan, verdict) in verdicts {
        let declared = declaration
            .plans
            .iter()
            .find(|candidate| &candidate.id == plan)
            .ok_or_else(|| format!("{plan} vanished from the declaration"))?;

        let held = match verdict {
            Verdict::Agrees { id, product } => {
                println!("agrees   {plan:24} {id}");

                Resolved {
                    price: id.clone(),
                    product: product.clone(),
                }
            }
            Verdict::Missing => {
                let product = create_product(client, key, declared).await?;
                let id = create_price(
                    client,
                    key,
                    declaration,
                    declared,
                    &product,
                    false,
                )
                .await?;

                println!("created  {plan:24} {id}");

                Resolved { price: id, product }
            }
            Verdict::Differs {
                held: was, product, ..
            } => {
                // The existing product is reused: it is the thing being sold,
                // and only its price has changed. `transfer_lookup_key` moves
                // the key onto the new price in the same call that creates it,
                // so there is no moment where the key points at nothing.
                let id = create_price(
                    client,
                    key,
                    declaration,
                    declared,
                    product,
                    true,
                )
                .await?;

                println!(
                    "repriced {plan:24} {was} -> {} ({id})",
                    declared.amount
                );

                Resolved {
                    price: id,
                    product: product.clone(),
                }
            }
        };

        resolved.insert(plan.clone(), held);
    }

    Ok(resolved)
}

async fn create_product(
    client: &reqwest::Client,
    key: &SecretString,
    plan: &Declared,
) -> Result<String, String> {
    let created: Created = post(
        client,
        key,
        "/v1/products",
        &[
            ("name".to_owned(), plan.name.clone()),
            ("metadata[plan]".to_owned(), plan.id.clone()),
        ],
    )
    .await?;

    Ok(created.id)
}

async fn create_price(
    client: &reqwest::Client,
    key: &SecretString,
    declaration: &Declaration,
    plan: &Declared,
    product: &str,
    transfer: bool,
) -> Result<String, String> {
    let interval = plan.interval()?;

    let mut form = vec![
        ("product".to_owned(), product.to_owned()),
        ("unit_amount".to_owned(), plan.amount.to_string()),
        ("currency".to_owned(), declaration.currency.clone()),
        ("lookup_key".to_owned(), plan.id.clone()),
    ];

    if interval != "once" {
        form.push(("recurring[interval]".to_owned(), interval.to_owned()));
    }

    if transfer {
        form.push(("transfer_lookup_key".to_owned(), "true".to_owned()));
    }

    let created: Created = post(client, key, "/v1/prices", &form).await?;

    Ok(created.id)
}

/// Makes stripe hold the declared tiers, and answers what it now holds.
///
/// **The coupon id is the declared code.** Stripe lets a coupon's id be chosen,
/// unlike a price, so `LIGHTHOUSE-BD` is the coupon and the promotion code
/// both — one name for the tier everywhere, and nothing to look up.
///
/// **Neither the percentage nor `applies_to` can be edited.** Stripe allows
/// `name` and `metadata` and nothing else, so a change to either is a different
/// coupon and needs a different code. That is refused loudly here rather than
/// applied quietly, because the alternative — deleting the coupon and remaking
/// it — would revoke the discount from everyone already subscribed on it.
///
/// **`applies_to` names products, not prices.** A repricing makes a new price
/// against the same product, so a coupon restricted to a plan keeps applying
/// across price changes without being touched.
async fn apply_ppp(
    client: &reqwest::Client,
    key: &SecretString,
    tiers: &[DeclaredPpp],
    resolved: &BTreeMap<String, Resolved>,
) -> Result<BTreeMap<String, String>, String> {
    let mut promotion_codes = BTreeMap::new();

    for tier in tiers {
        // Sorted, because this is compared against what stripe answers with
        // and neither side promises an order.
        let mut wanted: Vec<String> = tier
            .plans
            .iter()
            .map(|plan| {
                resolved
                    .get(plan)
                    .map(|held| held.product.clone())
                    .ok_or_else(|| format!("{plan} was never resolved"))
            })
            .collect::<Result<_, _>>()?;
        wanted.sort();

        // `expand[]=applies_to`, and it is load bearing. The field is not in
        // a coupon's default json at all — not null, absent — so without this
        // every held coupon reads as unrestricted, and a restricted one that
        // agrees perfectly well is reported as a restriction that changed and
        // cannot. `apply` refuses itself the second time it is run.
        let held: Option<HeldCoupon> = maybe_get(
            client,
            key,
            &format!("/v1/coupons/{}?expand[]=applies_to", tier.code),
        )
        .await?;

        match held {
            Some(coupon)
                if coupon.percent_off == Some(f64::from(tier.percent))
                    && coupon.products() == wanted =>
            {
                println!(
                    "  {} agrees at {}% off{}",
                    tier.code,
                    tier.percent,
                    scope_of(&tier.plans)
                );
            }
            Some(coupon) if coupon.products() != wanted => {
                return Err(format!(
                    "{} applies to {:?} at stripe and {:?} in pricing.yaml. A \
                     coupon's applies_to cannot be changed — give the new \
                     restriction its own code, and stripe keeps honouring the \
                     old one for anybody already on it.",
                    tier.code,
                    coupon.products(),
                    wanted
                ));
            }
            Some(coupon) => {
                return Err(format!(
                    "{} is {}% off at stripe and {}% off in pricing.yaml. A \
                     coupon's percentage cannot be changed — give the new rate \
                     its own code, and stripe keeps honouring the old one for \
                     anybody already on it.",
                    tier.code,
                    coupon.percent_off.unwrap_or_default(),
                    tier.percent
                ));
            }
            None => {
                create_coupon(client, key, tier, &wanted).await?;
            }
        }

        // The code a reader types. Separate from the coupon, and the only half
        // of this a reader ever sees.
        let codes: Listed<HeldPromotion> = get(
            client,
            key,
            &format!("/v1/promotion_codes?code={}&limit=1", tier.code),
        )
        .await?;

        let id = if let Some(existing) = codes.data.into_iter().next() {
            existing.id
        } else {
            let form = vec![
                ("coupon".to_owned(), tier.code.clone()),
                ("code".to_owned(), tier.code.clone()),
            ];

            let created: Created =
                post(client, key, "/v1/promotion_codes", &form).await?;

            println!("  {} is now a promotion code", tier.code);

            created.id
        };

        promotion_codes.insert(tier.code.clone(), id);
    }

    Ok(promotion_codes)
}

#[derive(Debug, serde::Deserialize)]
struct HeldCoupon {
    percent_off: Option<f64>,
    /// Absent on a coupon that applies to everything, which is how stripe says
    /// "unrestricted" — not an empty product list.
    ///
    /// Also absent on a coupon that *is* restricted, unless the request asked
    /// for it: this is an expandable field. The read that fills this has to
    /// send `expand[]=applies_to` or the two cases are indistinguishable, and
    /// the indistinguishable answer is the wrong one.
    #[serde(default)]
    applies_to: Option<AppliesTo>,
}

impl HeldCoupon {
    /// The products this coupon is restricted to, sorted. Empty is every
    /// product, matching the declaration's empty `plans`.
    fn products(&self) -> Vec<String> {
        let mut products = self
            .applies_to
            .as_ref()
            .map(|to| to.products.clone())
            .unwrap_or_default();
        products.sort();

        products
    }
}

#[derive(Debug, serde::Deserialize)]
struct AppliesTo {
    #[serde(default)]
    products: Vec<String>,
}

/// Mint the coupon, restricted to the products the tier named.
///
/// Split out of `apply_ppp` for length, and it reads better here anyway: the
/// caller decides *whether* to create one, this decides what one looks like.
async fn create_coupon(
    client: &reqwest::Client,
    key: &SecretString,
    tier: &DeclaredPpp,
    products: &[String],
) -> Result<(), String> {
    let mut form = vec![
        ("id".to_owned(), tier.code.clone()),
        ("percent_off".to_owned(), tier.percent.to_string()),
        // Forever, because the tier is about where somebody lives rather than
        // when they arrived.
        ("duration".to_owned(), "forever".to_owned()),
        (
            "name".to_owned(),
            format!("{}% — purchasing power", tier.percent),
        ),
    ];

    // Left off entirely when the tier names no plans: an empty
    // `applies_to[products]` is not "everything" at stripe, it is a restriction
    // to nothing.
    for (i, product) in products.iter().enumerate() {
        form.push((format!("applies_to[products][{i}]"), product.clone()));
    }

    let _: Created = post(client, key, "/v1/coupons", &form).await?;

    println!(
        "  {} created at {}% off{}",
        tier.code,
        tier.percent,
        scope_of(&tier.plans)
    );

    Ok(())
}

/// `" on all_lifetime"` for a restricted tier, nothing for one that is not.
fn scope_of(plans: &[String]) -> String {
    if plans.is_empty() {
        String::new()
    } else {
        format!(" on {}", plans.join(", "))
    }
}

#[derive(Debug, serde::Deserialize)]
struct HeldPromotion {
    id: String,
}

#[derive(Debug, serde::Deserialize)]
struct Listed<T> {
    data: Vec<T>,
}

/// A GET that answers `None` for a 404 rather than failing.
///
/// Asking whether a coupon exists is a normal question, and stripe answers it
/// with a 404 — which is not an error here the way a refused create is.
async fn maybe_get<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    key: &SecretString,
    path: &str,
) -> Result<Option<T>, String> {
    let response = client
        .get(format!("{STRIPE}{path}"))
        .basic_auth(key.expose_secret(), None::<&str>)
        .send()
        .await
        .map_err(|error| format!("cannot reach stripe: {error}"))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("cannot read stripe's answer: {error}"))?;

    if !status.is_success() {
        return Err(format!("stripe refused {path}: {status} {body}"));
    }

    serde_json::from_str(&body)
        .map(Some)
        .map_err(|error| format!("cannot read stripe's answer: {error}"))
}

async fn get<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    key: &SecretString,
    path: &str,
) -> Result<T, String> {
    maybe_get(client, key, path)
        .await?
        .ok_or_else(|| format!("stripe has nothing at {path}"))
}

async fn post<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    key: &SecretString,
    path: &str,
    form: &[(String, String)],
) -> Result<T, String> {
    let response = client
        .post(format!("{STRIPE}{path}"))
        .basic_auth(key.expose_secret(), None::<&str>)
        .form(form)
        .send()
        .await
        .map_err(|error| format!("cannot reach stripe: {error}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("cannot read stripe's answer: {error}"))?;

    if !status.is_success() {
        return Err(format!("stripe refused {path}: {status} {body}"));
    }

    serde_json::from_str(&body)
        .map_err(|error| format!("cannot read stripe's answer: {error}"))
}

/// Write the file the api reads.
///
/// Generated rather than hand-kept, which is the whole point: the price ids in
/// it are stripe's, and the only way to have the right ones is to have just
/// asked.
fn write(
    path: &str,
    declaration: &Declaration,
    resolved: &BTreeMap<String, Resolved>,
    promotions: &BTreeMap<String, String>,
) -> Result<(), String> {
    let mut out = String::from(
        "# Generated by `lighthouse-prices apply`. Do not edit.\n\
         #\n\
         # The amounts are declared in pricing.yaml; the price ids are stripe's\n\
         # answer to that declaration. Editing this file by hand puts the two\n\
         # out of step, and the symptom is a checkout at the wrong price.\n\n\
         plans:\n",
    );

    for plan in &declaration.plans {
        let id = resolved
            .get(&plan.id)
            .map(|held| &held.price)
            .ok_or_else(|| format!("{} was never resolved", plan.id))?;

        let interval = plan.interval()?;
        // Writing to a String cannot fail; the result is discarded rather than
        // unwrapped so this stays a formatting detail.
        let _ = write!(
            out,
            "  - id: {}\n    price: {id}\n    interval: {interval}\n    \
             money:\n      amount: {}\n      currency: {}\n\n",
            plan.id, plan.amount, declaration.currency,
        );
    }

    // The tiers, so the api can answer "which coupon does this country get"
    // without asking stripe on every request. The percentage rides along
    // because the frontend shows the reduced price before anybody types
    // anything; the promotion code is what they type.
    //
    // `plans` rides along for the same reason the percentage does. Stripe is
    // what enforces the restriction, but the page has to know it before the
    // reader gets there — otherwise it strikes out the price of a plan the
    // coupon will not touch, which is a number nobody can ever pay.
    if !declaration.ppp.is_empty() {
        out.push_str("ppp:\n");

        for tier in &declaration.ppp {
            let promotion = promotions
                .get(&tier.code)
                .ok_or_else(|| format!("{} was never resolved", tier.code))?;

            let _ = write!(
                out,
                "  - code: {}\n    promotion: {promotion}\n    percent: {}\n    \
                 countries: [{}]\n    plans: [{}]\n\n",
                tier.code,
                tier.percent,
                tier.countries.join(", "),
                tier.plans.join(", "),
            );
        }
    }

    std::fs::write(path, out)
        .map_err(|error| format!("cannot write {path}: {error}"))
}

// ---------------------------------------------------------------------------
// the frontend's copy
// ---------------------------------------------------------------------------

/// One plan, in the shape the pricing page reads it.
///
/// Deliberately the same field names `/api/billing/plans` answers with, so the
/// page's own `Offer` type describes both and swapping a fetch for an import
/// changed no types. `price` — stripe's id — is *not* here: the page posts a
/// plan id and the api resolves it, so shipping the price id to a browser
/// would put a stripe identifier in a public bundle for no gain.
#[derive(Debug, Serialize)]
struct WirePlan {
    plan: String,
    track: String,
    recurring: bool,
    amount: i64,
    currency: String,
}

/// A book, reduced to what a price card needs: what it is called, and which
/// tracks it is on.
#[derive(Debug, Serialize)]
struct WireBook {
    slug: String,
    title: String,
    tracks: BTreeMap<String, i32>,
}

/// Write the catalogue the frontend compiles in.
///
/// **Why a file and not a fetch.** `/pricing` is prerendered — `routeRules` in
/// `nuxt.config.ts` says so — and the web image is built from `web/` alone,
/// with no api, no database and no network. A page that fetched its prices
/// during that build got a connection refused and baked its fallback, which is
/// how the live pricing page came to show an em dash where each amount belongs.
/// Reading a file that is already on disk cannot fail that way.
///
/// **Why it is generated and not written.** The same reason `billing.yaml` is:
/// the amount has one home, the declaration this reads. A number typed into a
/// `.ts` file is a number that drifts from what stripe charges, and the drift
/// is invisible until somebody compares a receipt with the page.
///
/// The shelf comes from ohara, filtered the way the api filters it — drafts
/// hidden — so a book being written does not appear on a price card.
///
/// **The ppp tiers are deliberately not written here.** Which country gets what
/// is already in `billing.yaml`, which is what the api answers from, and the
/// page asks the api for the reader's own coupon rather than working it out.
/// Shipping the table to the browser as well would put a second copy of it in a
/// public bundle, and the copy that drifted would be the one quoting a price.
///
/// # Errors
///
/// `CONTENT_PATH` unset, ohara refusing anything in it, a plan whose name does
/// not say its billing period, or the output path not being writable.
fn catalogue(declaration: &Declaration, path: &str) -> Result<String, String> {
    let content = std::env::var("CONTENT_PATH").map_err(|_| {
        "CONTENT_PATH is not set in .env or the environment, and the shelf \
         comes from it"
            .to_owned()
    })?;

    // `Drafts::Hidden`, always. This file is built into a public page, and the
    // env var that shows drafts locally must not be able to leak an unpublished
    // title into it.
    let snapshot = ohara::catalog::Snapshot::load(
        &ohara::Content::at(&content),
        ohara::Drafts::Hidden,
    )
    .map_err(|error| format!("cannot read {content}: {error}"))?;

    let mut plans = Vec::new();

    for plan in &declaration.plans {
        let interval = plan.interval()?;

        plans.push(WirePlan {
            // The track is read from the name the same way `payments::track`
            // reads it, rather than declared twice. Everything up to the last
            // underscore, so a track whose own name contains one still works.
            track: plan
                .id
                .rsplit_once('_')
                .map_or(plan.id.as_str(), |(track, _)| track)
                .to_owned(),
            plan: plan.id.clone(),
            recurring: interval != "once",
            amount: plan.amount,
            currency: declaration.currency.clone(),
        });
    }

    let shelf: Vec<WireBook> = snapshot
        .books()
        .map(|entry| WireBook {
            slug: entry.book.slug.clone(),
            title: entry.book.title.clone(),
            tracks: entry.book.tracks.clone(),
        })
        .collect();

    // Through serde_json rather than formatted by hand: json is a subset of
    // typescript's own literal syntax, and it escapes a title containing a
    // quote correctly, which `write!` would not.
    let plans_ts = serde_json::to_string_pretty(&plans)
        .map_err(|error| format!("cannot encode the plans: {error}"))?;
    let shelf_ts = serde_json::to_string_pretty(&shelf)
        .map_err(|error| format!("cannot encode the shelf: {error}"))?;

    let out = format!(
        "// Generated by `lighthouse-prices catalogue`. Do not edit.\n\
         //\n\
         // The amounts are declared in pricing.yaml and the shelf comes from\n\
         // ohara; both are read at generation time, not at build time, because\n\
         // the web image is built from `web/` alone and can reach neither.\n\
         //\n\
         // Regenerate with `make catalogue` in the thelighthouse repo, and\n\
         // commit the result — `/pricing` is prerendered, so this file is what\n\
         // the built page says. A price changed in stripe and not here is a\n\
         // page advertising the wrong number.\n\n\
         // One purchasable plan. The same shape `/api/billing/plans` answers\n\
         // with, so nothing downstream can tell which one it came from.\n\
         export interface CataloguePlan {{\n  \
           plan: string\n  track: string\n  recurring: boolean\n  \
           amount: number\n  currency: string\n\
         }}\n\n\
         // A book, and the tracks it is on with its position in each.\n\
         export interface CatalogueBook {{\n  \
           slug: string\n  title: string\n  tracks: Record<string, number>\n\
         }}\n\n\
         export const plans: CataloguePlan[] = {plans_ts}\n\n\
         export const shelf: CatalogueBook[] = {shelf_ts}\n"
    );

    if let Some(parent) = std::path::Path::new(path).parent()
        && !parent.as_os_str().is_empty()
        && !parent.is_dir()
    {
        return Err(format!(
            "{} is not a directory — is the web submodule checked out? \
             `make web` fetches it",
            parent.display()
        ));
    }

    std::fs::write(path, out)
        .map_err(|error| format!("cannot write {path}: {error}"))?;

    Ok(format!("{} plans, {} books", plans.len(), shelf.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(id: &str, amount: i64) -> Declared {
        Declared {
            id: id.to_owned(),
            amount,
            name: id.to_owned(),
        }
    }

    #[test]
    fn a_billing_period_is_read_from_the_plans_own_name() {
        assert_eq!(declared("rust_yearly", 1).interval().unwrap(), "year");
        assert_eq!(declared("rust_monthly", 1).interval().unwrap(), "month");
        assert_eq!(declared("rust_lifetime", 1).interval().unwrap(), "once");
    }

    #[test]
    fn a_name_with_no_period_in_it_is_refused_before_anything_is_sent() {
        // Caught while reading the file, so a typo in the last plan cannot
        // leave the first four already created at stripe.
        assert!(declared("rust", 1).interval().is_err());
        assert!(declared("rust_forever", 1).interval().is_err());
    }

    fn price(key: &str, amount: i64, currency: &str) -> Price {
        Price {
            id: format!("price_{key}"),
            unit_amount: Some(amount),
            currency: currency.to_owned(),
            product: "prod_1".to_owned(),
            lookup_key: Some(key.to_owned()),
        }
    }

    fn declaration() -> Declaration {
        Declaration {
            currency: "usd".to_owned(),
            plans: vec![declared("rust_yearly", 4900)],
            ppp: Vec::new(),
        }
    }

    fn tier(code: &str, percent: u8, countries: &[&str]) -> DeclaredPpp {
        DeclaredPpp {
            code: code.to_owned(),
            percent,
            countries: countries.iter().map(|c| (*c).to_owned()).collect(),
            plans: Vec::new(),
        }
    }

    /// A tier restricted to the plans it names.
    fn tier_for(code: &str, plans: &[&str]) -> DeclaredPpp {
        DeclaredPpp {
            code: code.to_owned(),
            percent: 50,
            countries: Vec::new(),
            plans: plans.iter().map(|p| (*p).to_owned()).collect(),
        }
    }

    #[test]
    fn a_country_may_belong_to_one_tier_only() {
        // Two claims and a reader in BD is offered whichever tier was listed
        // first, which is a decision nobody wrote down.
        let clash = [tier("A", 60, &["BD", "IN"]), tier("B", 40, &["BD"])];

        let refused = check_ppp(&clash, &[], "pricing.yaml").unwrap_err();

        assert!(refused.contains("BD"), "{refused}");
        assert!(refused.contains('A') && refused.contains('B'), "{refused}");
    }

    #[test]
    fn tiers_that_do_not_overlap_are_fine() {
        let clean = [tier("A", 60, &["BD", "IN"]), tier("B", 40, &["BR"])];

        assert!(check_ppp(&clean, &[], "pricing.yaml").is_ok());
    }

    #[test]
    fn no_tiers_at_all_is_fine() {
        assert!(check_ppp(&[], &[], "pricing.yaml").is_ok());
    }

    #[test]
    fn a_tier_naming_no_countries_is_offered_to_everyone() {
        // A list price with a discount off it. Allowed, and it does not claim
        // a country, so it cannot clash with one that does.
        let mixed = [tier("EVERYONE", 50, &[]), tier("A", 70, &["BD"])];

        assert!(check_ppp(&mixed, &[], "pricing.yaml").is_ok());
    }

    #[test]
    fn a_tier_may_name_the_plans_it_discounts() {
        let plans =
            [declared("rust_yearly", 4900), declared("all_lifetime", 1)];
        let tiers = [tier_for("HALF", &["all_lifetime"])];

        assert!(check_ppp(&tiers, &plans, "pricing.yaml").is_ok());
    }

    #[test]
    fn a_tier_naming_a_plan_that_is_not_sold_is_refused() {
        // Reaching stripe, this would be a coupon restricted to a product that
        // does not exist — which discounts nothing, and says nothing about it.
        let plans = [declared("rust_yearly", 4900)];
        let tiers = [tier_for("HALF", &["all_lifetme"])];

        let refused = check_ppp(&tiers, &plans, "pricing.yaml").unwrap_err();

        assert!(refused.contains("all_lifetme"), "{refused}");
    }

    #[test]
    fn a_coupon_with_no_applies_to_is_unrestricted() {
        // Stripe says "every product" by leaving the field off, not by sending
        // an empty list — so the two have to read the same way here.
        let unrestricted = HeldCoupon {
            percent_off: Some(50.0),
            applies_to: None,
        };

        assert!(unrestricted.products().is_empty());
    }

    #[test]
    fn a_coupons_products_are_compared_in_a_stable_order() {
        // Neither side promises an order, so both are sorted before comparing.
        let held = HeldCoupon {
            percent_off: Some(50.0),
            applies_to: Some(AppliesTo {
                products: vec!["prod_b".to_owned(), "prod_a".to_owned()],
            }),
        };

        assert_eq!(held.products(), ["prod_a", "prod_b"]);
    }

    #[test]
    fn two_tiers_offered_to_everyone_are_refused() {
        // Same ambiguity as a doubly claimed country: the api would be picking.
        let clash = [tier("A", 50, &[]), tier("B", 40, &[])];

        let refused = check_ppp(&clash, &[], "pricing.yaml").unwrap_err();

        assert!(refused.contains('A') && refused.contains('B'), "{refused}");
    }

    #[test]
    fn a_percentage_outside_one_to_ninety_nine_is_refused() {
        // A hundred is a free plan, which is a different decision.
        for wrong in [0, 100] {
            assert!(
                check_ppp(&[tier("A", wrong, &["BD"])], &[], "pricing.yaml")
                    .is_err(),
                "{wrong} percent was accepted"
            );
        }
    }

    #[test]
    fn a_country_code_is_two_uppercase_letters_or_it_is_refused() {
        // The shape cloudflare reports, so neither side normalises later.
        for wrong in ["bd", "BGD", "B", "12"] {
            assert!(
                check_ppp(&[tier("A", 50, &[wrong])], &[], "pricing.yaml")
                    .is_err(),
                "{wrong} was accepted as a country"
            );
        }
    }

    #[test]
    fn two_tiers_cannot_share_a_code() {
        let clash = [tier("A", 60, &["BD"]), tier("A", 40, &["BR"])];

        assert!(check_ppp(&clash, &[], "pricing.yaml").is_err());
    }

    #[test]
    fn a_price_stripe_does_not_hold_is_missing() {
        let verdicts = compare(&declaration(), &BTreeMap::new());

        assert_eq!(verdicts.first().unwrap().1, Verdict::Missing);
    }

    #[test]
    fn the_same_amount_under_the_same_key_agrees() {
        let held = BTreeMap::from([(
            "rust_yearly".to_owned(),
            price("rust_yearly", 4900, "usd"),
        )]);

        assert_eq!(
            compare(&declaration(), &held).first().unwrap().1,
            Verdict::Agrees {
                id: "price_rust_yearly".to_owned(),
                product: "prod_1".to_owned(),
            }
        );
    }

    #[test]
    fn a_changed_amount_differs_rather_than_agreeing() {
        // The case the whole tool exists for: stripe cannot be edited to the
        // new amount, so this has to become a new price with the key moved.
        let held = BTreeMap::from([(
            "rust_yearly".to_owned(),
            price("rust_yearly", 3900, "usd"),
        )]);

        assert!(matches!(
            compare(&declaration(), &held).first().unwrap().1,
            Verdict::Differs { held: 3900, .. }
        ));
    }

    #[test]
    fn a_changed_currency_differs_too() {
        let held = BTreeMap::from([(
            "rust_yearly".to_owned(),
            price("rust_yearly", 4900, "eur"),
        )]);

        assert!(matches!(
            compare(&declaration(), &held).first().unwrap().1,
            Verdict::Differs { .. }
        ));
    }
}
