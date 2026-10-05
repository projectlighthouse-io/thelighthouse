//! One function per route, and no sql.
//!
//! They hand the pool to `store` and the reader's intent to a driver, and
//! nothing else — no statement, no column name, no provider's vocabulary.

use axum::{
    Extension,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use billing::{Customer, Event, Gateway, PlanId, Subscription};

use super::{
    membership::Membership,
    payload::ChosenPlan,
    refusal::{Refusal, refuse},
    store, track,
};
use crate::{
    api::AppState,
    cache::CachePolicy,
    response::{self, json},
    session::Session,
};

/// The driver a route parameter names, or a 404.
///
/// An unknown provider is a url that does not exist, not a malformed request.
fn driver_named<'s>(
    state: &'s AppState,
    provider: &str,
) -> Option<&'s dyn Gateway> {
    state.billing.providers.driver(provider)
}

/// Everything on sale, and what it costs.
///
/// Public and cacheable: it is the same answer for everyone, and it is the
/// pricing page's whole content. Nothing about a reader is consulted, which is
/// what lets the edge hold it.
///
/// The track is given alongside the plan so the page can group the two ways of
/// buying one thing without splitting the name itself — the api already knows
/// the rule, and a frontend re-deriving it is a second place to get it wrong.
/// Cloudflare's own country header, and the only one worth reading.
///
/// The edge sets it from the address it terminated and **overwrites** whatever
/// the client sent, so a caller cannot choose their own country by supplying
/// one. That is only true of traffic that actually came through Cloudflare —
/// direct to the origin there is no header at all, which reads as unknown.
const CF_COUNTRY: &str = "cf-ipcountry";

/// The country Cloudflare says this request came from, if it says anything.
///
/// `XX` is Cloudflare's "could not tell" and `T1` is Tor; both are answered as
/// unknown rather than passed on as if they were places. Anything that is not
/// two ascii letters is refused outright — the header is trusted only in the
/// shape it is documented to take.
fn country_of(headers: &HeaderMap) -> Option<String> {
    let code = headers.get(CF_COUNTRY)?.to_str().ok()?.trim();

    let plausible = code.len() == 2
        && code.bytes().all(|b| b.is_ascii_alphabetic())
        && !code.eq_ignore_ascii_case("XX")
        && !code.eq_ignore_ascii_case("T1");

    plausible.then(|| code.to_ascii_uppercase())
}

/// The discount `plan` offers a reader in `country`, as the page reads it:
/// the code, how much it takes off — `percent` or `amount_off` in minor
/// units, whichever the tier declares — and whether it is the plan's rest
/// tier, the price for everywhere no other tier names. `None` is the list
/// price.
fn offer(
    plan: &billing::Plan,
    country: Option<&str>,
) -> Option<serde_json::Value> {
    let tier = plan.for_country(country)?;

    let mut offered = serde_json::json!({
        "code": tier.code(),
        "rest": tier.is_rest(),
    });

    if let Some(fields) = offered.as_object_mut() {
        match tier.off() {
            billing::Off::Percent(percent) => {
                fields.insert("percent".to_owned(), percent.into());
            }
            billing::Off::Amount(amount) => {
                fields.insert("amount_off".to_owned(), amount.into());
            }
        }
    }

    Some(offered)
}

/// What is for sale, and where the asker is.
///
/// **`no-store`, where this used to be edge-cached.** The answer now depends on
/// the caller's country, and a shared cache holding one country's answer serves
/// it to the next country along. That is the same rule the lesson endpoint
/// follows for entitlement: the moment a response depends on who is asking, it
/// stops being cacheable.
pub(crate) async fn catalogue(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let country = country_of(&headers);

    let plans: Vec<_> = state
        .billing
        .plans
        .all()
        .map(|plan| {
            // Asked of the plan rather than of the catalogue, because a coupon
            // is declared inside the plan it comes off and restricted to that
            // plan's product at stripe. A page told only "this country gets
            // 70% off" would strike out a price nobody can ever pay.
            //
            // The same tier checkout applies — see `checkout` — so the price
            // a page shows is the price stripe asks for.
            let coupon = offer(plan, country.as_deref());

            serde_json::json!({
                "plan": plan.id.as_str(),
                // A grouping for the page, still parsed from the id. What the
                // plan actually unlocks is `includes`/`everything` below —
                // these are different questions now and the wire says both.
                "track": plan.id.as_str().rsplit_once('_')
                    .map_or(plan.id.as_str(), |(track, _)| track),
                "books": &plan.books,
                "everything": plan.everything,
                "recurring": plan.interval.recurs(),
                "amount": plan.money.as_ref().map(|money| money.amount),
                "currency": plan.money.as_ref().map(|money| &money.currency),
                "coupon": coupon,
            })
        })
        .collect();

    json(
        StatusCode::OK,
        serde_json::json!({
            "country": country,
            "plans": plans,
        }),
        CachePolicy::NoStore,
    )
}

/// Where the provider sends the browser back.
///
/// `{CHECKOUT_SESSION_ID}` is a literal the provider substitutes for the real
/// session id on the way back — it must be written exactly, not interpolated.
///
/// The id is all that is handed to the browser, deliberately. What was bought
/// is then read from the provider by `bought` below, so a reader editing the
/// url gets somebody else's session refused rather than somebody else's
/// purchase displayed.
fn returns(state: &AppState) -> billing::Returns {
    billing::Returns {
        success: format!(
            "{}/billing/thanks?session={{CHECKOUT_SESSION_ID}}",
            state.config.app_url
        ),
        cancel: format!("{}/pricing", state.config.app_url),
    }
}

/// What a finished checkout was for, for the page a reader lands on.
///
/// **Display, not fulfilment.** The provider's own guidance is that a landing
/// page cannot be relied on — a reader can pay and close the tab — so nothing
/// here writes anything. The webhook remains the only writer; this only lets
/// the page say something true immediately instead of polling in silence.
///
/// The session is refused unless it names this reader. Without that check, a
/// guessed session id would read out a stranger's purchase.
pub(crate) async fn bought(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path((provider, checkout)): Path<(String, String)>,
) -> Response {
    let Some(driver) = driver_named(&state, &provider) else {
        return response::not_found();
    };

    let bought = match driver.bought(&checkout).await {
        Ok(bought) => bought,
        Err(error) => {
            tracing::warn!(%error, provider, "failed to read a checkout back");
            return response::not_found();
        }
    };

    // Somebody else's session, or one this application did not start.
    if bought.reference.as_deref().and_then(reader) != Some(session.user_id) {
        tracing::warn!(
            user_id = session.user_id,
            "a reader asked about a checkout that is not theirs"
        );
        return response::not_found();
    }

    let snapshot = state.catalog.current();
    let books: Vec<_> = bought
        .plan
        .as_ref()
        .map(|plan| {
            snapshot
                .books()
                .filter(|entry| {
                    track::covers(&state.billing.plans, plan, &entry.book.slug)
                })
                .map(|entry| {
                    serde_json::json!({
                        "slug": entry.book.slug,
                        "title": entry.book.title,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    json(
        StatusCode::OK,
        serde_json::json!({
            "plan": bought.plan.as_ref().map(billing::PlanId::as_str),
            "paid": bought.paid,
            "books": books,
        }),
        CachePolicy::NoStore,
    )
}

/// What this reader may now read, and what bought it.
///
/// The page a reader lands on after paying asks this. It answers for both
/// shapes a purchase can take, which is why it is not simply the membership:
/// a track subscribed to leaves a `memberships` row and no entitlements, and a
/// track bought outright leaves entitlements and no membership. A reader can
/// hold both at once.
///
/// **The books are derived, never frozen.** A subscription has to cover books
/// published after it was bought, so recording the list at purchase time would
/// be wrong within a release. This asks the catalogue every time.
pub(crate) async fn access(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    let membership = match store::live(&state.db, session.user_id).await {
        Ok(found) => found,
        Err(error) => {
            tracing::error!(%error, "failed to read a membership");
            return response::server_error();
        }
    };

    let owned = match store::owned(&state.db, session.user_id).await {
        Ok(owned) => owned,
        Err(error) => {
            tracing::error!(%error, "failed to read entitlements");
            return response::server_error();
        }
    };

    // Only a membership that grants access counts, so a reader in grace is
    // told what they bought without being shown it as readable.
    let held = membership
        .as_ref()
        .filter(|membership| membership.grants_access())
        .map(|membership| PlanId::from(membership.plan.as_str()));

    let snapshot = state.catalog.current();

    let books: Vec<_> = snapshot
        .books()
        .filter(|entry| {
            held.as_ref().is_some_and(|plan| {
                track::covers(&state.billing.plans, plan, &entry.book.slug)
            }) || entry.book.id.is_some_and(|id| owned.contains(&id))
        })
        .map(|entry| {
            serde_json::json!({
                "slug": entry.book.slug,
                "title": entry.book.title,
            })
        })
        .collect();

    json(
        StatusCode::OK,
        serde_json::json!({
            "plan": membership.as_ref().map(|membership| &membership.plan),
            "books": books,
        }),
        CachePolicy::NoStore,
    )
}

/// What this reader is currently paying for.
///
/// 204 rather than 404 for a reader with nothing: having no subscription is a
/// perfectly good answer to the question, and a 404 would make the client
/// treat an ordinary state as a failure.
pub(crate) async fn show(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    match store::live(&state.db, session.user_id).await {
        Ok(Some(membership)) => paid(&membership),
        Ok(None) => {
            response::empty(StatusCode::NO_CONTENT, CachePolicy::NoStore)
        }
        Err(error) => {
            tracing::error!(%error, "failed to read a membership");
            response::server_error()
        }
    }
}

/// Start paying for a plan.
pub(crate) async fn checkout(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    axum::Json(payload): axum::Json<ChosenPlan>,
) -> Response {
    let Some(driver) = driver_named(&state, &provider) else {
        return response::not_found();
    };

    let Some(plan) = state.billing.plans.get(&PlanId::from(payload.plan))
    else {
        return refuse(Refusal::UnknownPlan);
    };

    // Checked before the provider is asked, so a double-click cannot become a
    // second subscription in the window before the first webhook lands.
    //
    // Only for a recurring plan: buying a track outright is not something a
    // subscription should stand in the way of, and the two can be held at
    // once — a reader subscribed to Rust may still buy Foundation forever.
    if plan.interval.recurs() {
        match store::live(&state.db, session.user_id).await {
            Ok(Some(membership)) if membership.grants_access() => {
                return refuse(Refusal::AlreadySubscribed);
            }
            Ok(_) => {}
            Err(error) => {
                tracing::error!(%error, "failed to read a membership");
                return response::server_error();
            }
        }
    }

    let (email, existing) = match tokio::try_join!(
        store::email(&state.db, session.user_id),
        store::customer(&state.db, session.user_id),
    ) {
        Ok((Some(email), existing)) => (email, existing),
        // A session whose user is gone. The session is the bug, not this.
        Ok((None, _)) => return response::server_error(),
        Err(error) => {
            tracing::error!(%error, "failed to read a reader for checkout");
            return response::server_error();
        }
    };

    let reference = session.user_id.to_string();

    // The tier this reader's country is offered on this plan — the one the
    // catalogue advertised to them — applied up front, so the price they saw
    // is the price stripe asks for. The header is the same one the catalogue
    // read, and spoofing it buys nothing a shared code would not.
    let country = country_of(&headers);
    let promotion = plan
        .for_country(country.as_deref())
        .map(billing::Ppp::promotion);

    let who = Customer {
        reference: &reference,
        email: &email,
        existing: existing.as_deref(),
        promotion,
    };

    // The plan's own interval decides which kind of checkout this is. One
    // route rather than two, because from the reader's side it is the same
    // act — they picked a thing and they are going to pay for it.
    let back = returns(&state);

    let opened = if plan.interval.recurs() {
        driver.subscribe(plan, &who, &back).await
    } else {
        driver.purchase(plan, &who, &back).await
    };

    match opened {
        Ok(handoff) => json(
            StatusCode::OK,
            serde_json::json!({ "url": handoff.url }),
            CachePolicy::NoStore,
        ),
        Err(error) => {
            tracing::error!(%error, provider, "failed to open a checkout");
            refuse(Refusal::Unavailable)
        }
    }
}

/// Hand the reader to the provider's own billing page.
///
/// Cancelling, resuming, changing plan, cards and invoices all happen there,
/// and each change reaches `memberships` through the webhook like any other.
/// Anyone the provider has a customer for may go — a reader who bought a track
/// outright has invoices too.
pub(crate) async fn manage(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(provider): Path<String>,
) -> Response {
    let Some(driver) = driver_named(&state, &provider) else {
        return response::not_found();
    };

    let customer = match store::customer(&state.db, session.user_id).await {
        Ok(Some(customer)) => customer,
        Ok(None) => return refuse(Refusal::NoCustomer),
        Err(error) => {
            tracing::error!(%error, "failed to read a customer");
            return response::server_error();
        }
    };

    let back = format!("{}/settings/billing", state.config.app_url);

    match driver.manage(&customer, &back).await {
        Ok(handoff) => json(
            StatusCode::OK,
            serde_json::json!({ "url": handoff.url }),
            CachePolicy::NoStore,
        ),
        Err(error) => {
            tracing::error!(%error, provider, "failed to open a billing portal");
            refuse(Refusal::Unavailable)
        }
    }
}

/// A membership, and never in a cache.
fn paid(membership: &Membership) -> Response {
    json(StatusCode::OK, membership, CachePolicy::NoStore)
}

/// What a provider says happened.
///
/// **The only writer of subscription truth.** Every other route asks the
/// provider and records the answer; this is where a change nobody here
/// initiated — a renewal, a failed card, a cancellation from a receipt email —
/// arrives.
///
/// Outside the reader gate and the CSRF check, because a provider has neither
/// a session nor a token. It authenticates itself: the driver verifies the
/// signature over the raw body, in constant time.
pub(crate) async fn delivered(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(driver) = driver_named(&state, &provider) else {
        return response::not_found();
    };

    let signature = headers
        .get(driver.signature_header())
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    // The raw bytes, exactly as they arrived. A body that has been parsed and
    // re-serialised is a different sequence of bytes and will not verify.
    let event = match driver.settle(&body, signature) {
        Ok(event) => event,
        Err(error) => {
            tracing::warn!(%error, provider, "refused a webhook delivery");
            return response::empty(
                StatusCode::BAD_REQUEST,
                CachePolicy::NoStore,
            );
        }
    };

    match apply(&state, &provider, event).await {
        Ok(()) => response::empty(StatusCode::OK, CachePolicy::NoStore),
        // A 5xx, so the provider retries. Every write here is an upsert, so a
        // redelivery is the same statement again rather than a second row.
        Err(error) => {
            tracing::error!(%error, provider, "failed to apply a webhook");
            response::server_error()
        }
    }
}

/// Apply a verified event.
async fn apply(
    state: &AppState,
    provider: &str,
    event: Event,
) -> Result<(), sqlx::Error> {
    match event {
        Event::CheckoutCompleted {
            customer,
            reference,
            plan,
            subscription,
        } => {
            let Some(user_id) = reader(&reference) else {
                tracing::warn!(reference, "a checkout named no reader we know");
                return Ok(());
            };

            store::remember_customer(&state.db, user_id, &customer).await?;

            // A subscription checkout stops here: the event announcing the
            // subscription carries its dates and status, and writing a row
            // from this one would be guessing at both.
            if subscription.is_some() {
                return Ok(());
            }

            // A purchase has no such follow-up. This delivery is the only
            // notice that money moved, so the books are granted here or never.
            let Some(plan) = plan else {
                tracing::error!(
                    reference,
                    "a purchase completed naming no plan; nothing was granted"
                );
                return Ok(());
            };

            grant_track(state, user_id, &plan).await
        }
        Event::Started(subscription)
        | Event::Changed(subscription)
        | Event::Ended(subscription) => {
            let Some(user_id) = owner(state, provider, &subscription).await?
            else {
                tracing::warn!(
                    reference = subscription.reference,
                    "a subscription event named no reader we know"
                );
                return Ok(());
            };

            store::record(&state.db, user_id, provider, &subscription).await
        }
        // Logged and nothing else, which is what the laravel app did too. The
        // provider is already retrying the charge on its own schedule, and
        // locking the reader out on the first failure would lock out anyone
        // whose bank declined once.
        Event::PaymentFailed { customer } => {
            tracing::warn!(customer, provider, "a payment failed");
            Ok(())
        }
        Event::Ignored => Ok(()),
    }
}

/// Turn a paid-for track into the rows that open its books.
///
/// **Expanded at purchase, not consulted at read time.** One row per book
/// means the entitlement check stays a single lookup, and it means a book
/// leaving a track later cannot take away something somebody already bought.
/// That is the trade rebuild.md records: a bundle exists in the checkout path
/// and never in the read path.
async fn grant_track(
    state: &AppState,
    user_id: i64,
    plan: &PlanId,
) -> Result<(), sqlx::Error> {
    let books =
        track::books(&state.catalog.current(), &state.billing.plans, plan);

    if books.is_empty() {
        // Nothing to grant means the reader paid for a plan that names no book
        // with an id. Loud, because the money has been taken.
        tracing::error!(
            %plan, user_id,
            "a purchase granted nothing; the plan names no books with ids"
        );
        return Ok(());
    }

    let granted = store::grant(&state.db, user_id, &books).await?;

    tracing::info!(
        %plan, user_id,
        books = books.len(),
        granted,
        "granted a track"
    );

    Ok(())
}

/// Whose subscription this is.
///
/// The event's own reference first, because it works before any row exists —
/// deliveries have no guaranteed order, and the subscription can be announced
/// before the checkout that created it. The stored row is the fallback for a
/// subscription made outside this application, which carries no reference.
async fn owner(
    state: &AppState,
    provider: &str,
    subscription: &Subscription,
) -> Result<Option<i64>, sqlx::Error> {
    if let Some(user_id) = subscription.account.as_deref().and_then(reader) {
        return Ok(Some(user_id));
    }

    store::reader_of(&state.db, provider, &subscription.reference).await
}

/// Our own reference, back as the id it was made from.
fn reader(reference: &str) -> Option<i64> {
    reference.parse().ok()
}

#[cfg(test)]
mod offer_tests {
    use billing::Plans;

    use super::offer;

    const TIERS: &str = "
plans:
  - id: foundation_yearly
    price: price_x
    interval: year
    money:
      amount: 11900
      currency: usd
    ppp:
      - code: FOUNDATION-LOWER
        promotion: promo_lower
        amount_off: 7000
        countries: [IN]
      - code: FOUNDATION-PERCENT
        promotion: promo_percent
        percent: 40
        countries: [BR]
      - code: FOUNDATION-EVERYONE
        promotion: promo_everyone
        amount_off: 2000
        rest: true
";

    fn plans() -> Plans {
        Plans::from_yaml(TIERS).unwrap()
    }

    #[test]
    fn a_fixed_amount_tier_is_offered_as_an_amount() {
        let plans = plans();
        let plan = plans.all().next().unwrap();

        assert_eq!(
            offer(plan, Some("IN")),
            Some(serde_json::json!({
                "code": "FOUNDATION-LOWER",
                "amount_off": 7000,
                "rest": false,
            }))
        );
    }

    #[test]
    fn a_percentage_tier_is_offered_as_a_percentage() {
        let plans = plans();
        let plan = plans.all().next().unwrap();

        assert_eq!(
            offer(plan, Some("BR")),
            Some(serde_json::json!({
                "code": "FOUNDATION-PERCENT",
                "percent": 40,
                "rest": false,
            }))
        );
    }

    #[test]
    fn everywhere_else_and_nowhere_get_the_rest_tier() {
        let plans = plans();
        let plan = plans.all().next().unwrap();
        let everyone = Some(serde_json::json!({
            "code": "FOUNDATION-EVERYONE",
            "amount_off": 2000,
            "rest": true,
        }));

        assert_eq!(offer(plan, Some("GB")), everyone);
        assert_eq!(offer(plan, None), everyone);
    }
}
