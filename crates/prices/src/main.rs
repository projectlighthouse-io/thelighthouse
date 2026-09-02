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

use std::{collections::BTreeMap, fmt::Write as _, process::ExitCode};

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

/// Where the declaration lives, unless `PRICING` says otherwise.
const DEFAULT_DECLARATION: &str = "pricing.yaml";

/// Where the api reads its plans from, unless `BILLING_PLANS` says otherwise.
const DEFAULT_OUTPUT: &str = "billing.yaml";

const STRIPE: &str = "https://api.stripe.com";

/// What a track costs, as declared.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Declaration {
    /// ISO 4217, lowercase. One currency for the whole catalogue: selling in
    /// several is a product decision with tax consequences, not a config key.
    currency: String,
    plans: Vec<Declared>,
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

/// What has to happen to a plan for stripe to match the declaration.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// Stripe already holds this price, at this amount.
    Agrees(String),
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

    let key: SecretString = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| "STRIPE_SECRET_KEY is not set in .env or the environment")?
        .into();

    let declaration_path = std::env::var("PRICING")
        .unwrap_or_else(|_| DEFAULT_DECLARATION.to_owned());
    let declaration = read(&declaration_path)?;

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
            let output = std::env::var("BILLING_PLANS")
                .unwrap_or_else(|_| DEFAULT_OUTPUT.to_owned());

            write(&output, &declaration, &resolved)?;
            println!("wrote {output}");

            Ok(())
        }
        other => Err(format!(
            "unknown command {other:?}\n\n  \
             lighthouse-prices status   what stripe has, and where it differs\n  \
             lighthouse-prices apply    make stripe match, then write the config"
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

    Ok(declaration)
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
                    Verdict::Agrees(price.id.clone())
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
            Verdict::Agrees(id) => println!("agrees   {plan:24} {id}"),
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
        .filter(|(_, verdict)| !matches!(verdict, Verdict::Agrees(_)))
        .count();

    if pending == 0 {
        println!("\nstripe matches the declaration");
    } else {
        println!("\n{pending} to apply");
    }
}

/// Make stripe agree, and give back the price id for every plan.
async fn apply(
    client: &reqwest::Client,
    key: &SecretString,
    declaration: &Declaration,
    verdicts: &[(String, Verdict)],
) -> Result<BTreeMap<String, String>, String> {
    let mut resolved = BTreeMap::new();

    for (plan, verdict) in verdicts {
        let declared = declaration
            .plans
            .iter()
            .find(|candidate| &candidate.id == plan)
            .ok_or_else(|| format!("{plan} vanished from the declaration"))?;

        let id = match verdict {
            Verdict::Agrees(id) => {
                println!("agrees   {plan:24} {id}");

                id.clone()
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

                id
            }
            Verdict::Differs { held, product, .. } => {
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
                    "repriced {plan:24} {held} -> {} ({id})",
                    declared.amount
                );

                id
            }
        };

        resolved.insert(plan.clone(), id);
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
    resolved: &BTreeMap<String, String>,
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

    std::fs::write(path, out)
        .map_err(|error| format!("cannot write {path}: {error}"))
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
        }
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
            Verdict::Agrees("price_rust_yearly".to_owned())
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
