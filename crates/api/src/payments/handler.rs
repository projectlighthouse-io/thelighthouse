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
use billing::{Cancel, Customer, Event, Gateway, PlanId, Subscription};

use super::{
    membership::Membership,
    payload::{Cancellation, ChosenPlan},
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
    let plans: Vec<_> = state
        .billing
        .plans
        .all()
        .map(|plan| {
            serde_json::json!({
                "plan": plan.id.as_str(),
                "track": track::of(plan.id.as_str()),
                "recurring": plan.interval.recurs(),
                "amount": plan.money.as_ref().map(|money| money.amount),
                "currency": plan.money.as_ref().map(|money| &money.currency),
            })
        })
        .collect();

    let country = country_of(&headers);

    // How a country gets a different price: one coupon, declared in
    // `pricing.yaml` and reconciled at stripe, never a second price list — a
    // parallel set of amounts would be a second thing to keep in step, and the
    // point of `lighthouse-prices` is that there is one.
    //
    // Advertised, not applied. The reader types the code at stripe, which is
    // what keeps a spoofable header from being the only thing standing between
    // anybody and a discount.
    let coupon = country
        .as_deref()
        .and_then(|code| state.billing.plans.for_country(code))
        .map(|tier| {
            serde_json::json!({
                "code": tier.code(),
                "percent": tier.percent(),
            })
        });

    json(
        StatusCode::OK,
        serde_json::json!({
            "country": country,
            "plans": plans,
            "coupon": coupon,
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

    let track = bought
        .plan
        .as_ref()
        .map(|plan| track::of(plan.as_str()).to_owned());

    let snapshot = state.catalog.current();
    let books: Vec<_> = track
        .as_deref()
        .map(|track| {
            snapshot
                .books()
                .filter(|entry| track::covers(track, &entry.book))
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
            "track": track,
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
    let track = membership
        .as_ref()
        .filter(|membership| membership.grants_access())
        .map(|membership| track::of(&membership.plan).to_owned());

    let snapshot = state.catalog.current();

    let books: Vec<_> = snapshot
        .books()
        .filter(|entry| {
            track
                .as_deref()
                .is_some_and(|track| track::covers(track, &entry.book))
                || entry.book.id.is_some_and(|id| owned.contains(&id))
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
            "track": track,
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

    let who = Customer {
        reference: &reference,
        email: &email,
        existing: existing.as_deref(),
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

/// Stop a subscription renewing, or end it outright.
pub(crate) async fn cancel(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(provider): Path<String>,
    axum::Json(payload): axum::Json<Cancellation>,
) -> Response {
    let when = if payload.immediately {
        // Forfeits the rest of the paid period, so it is never the default.
        Cancel::Now
    } else {
        Cancel::AtPeriodEnd
    };

    let (driver, _, reference) =
        match subject(&state, session.user_id, &provider).await {
            Subject::Ready(driver, membership, reference) => {
                (driver, membership, reference)
            }
            Subject::Refused(response) => return response,
        };

    match driver.cancel(&reference, when).await {
        Ok(updated) => {
            settled(&state, session.user_id, &provider, &updated).await
        }
        Err(error) => {
            tracing::error!(%error, provider, "failed to cancel");
            refuse(Refusal::Unavailable)
        }
    }
}

/// Undo a cancellation that has not taken effect yet.
pub(crate) async fn resume(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(provider): Path<String>,
) -> Response {
    let (driver, membership, reference) =
        match subject(&state, session.user_id, &provider).await {
            Subject::Ready(driver, membership, reference) => {
                (driver, membership, reference)
            }
            Subject::Refused(response) => return response,
        };

    // Answered here rather than by asking the provider and relaying its
    // complaint: a subscription that is not ending has nothing to resume, and
    // that is knowable from the row.
    if !membership.is_cancelling() {
        return refuse(Refusal::NotCancelling);
    }

    match driver.resume(&reference).await {
        Ok(updated) => {
            settled(&state, session.user_id, &provider, &updated).await
        }
        Err(error) => {
            tracing::error!(%error, provider, "failed to resume");
            refuse(Refusal::Unavailable)
        }
    }
}

/// Move to another plan.
pub(crate) async fn swap(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(provider): Path<String>,
    axum::Json(payload): axum::Json<ChosenPlan>,
) -> Response {
    let Some(plan) = state.billing.plans.get(&PlanId::from(payload.plan))
    else {
        return refuse(Refusal::UnknownPlan);
    };

    let (driver, _, reference) =
        match subject(&state, session.user_id, &provider).await {
            Subject::Ready(driver, membership, reference) => {
                (driver, membership, reference)
            }
            Subject::Refused(response) => return response,
        };

    match driver.swap(&reference, plan).await {
        Ok(updated) => {
            settled(&state, session.user_id, &provider, &updated).await
        }
        Err(error) => {
            tracing::error!(%error, provider, "failed to swap a plan");
            refuse(Refusal::Unavailable)
        }
    }
}

/// The three things every lifecycle route needs: the driver, the membership,
/// and the provider's own reference for it.
///
/// An enum rather than `Result<_, Response>` for the reason clippy gives and
/// `bookmarks::Lesson` already follows: a whole `Response` is 128 bytes, and a
/// `Result` carries that width through the success path too.
enum Subject<'s> {
    Ready(&'s dyn Gateway, Membership, String),
    /// The response that replaces the work, already logged where it needed to
    /// be.
    Refused(Response),
}

async fn subject<'s>(
    state: &'s AppState,
    user_id: i64,
    provider: &str,
) -> Subject<'s> {
    let Some(driver) = driver_named(state, provider) else {
        return Subject::Refused(response::not_found());
    };

    let membership = match store::live(&state.db, user_id).await {
        Ok(Some(membership)) => membership,
        Ok(None) => return Subject::Refused(refuse(Refusal::NoMembership)),
        Err(error) => {
            tracing::error!(%error, "failed to read a membership");
            return Subject::Refused(response::server_error());
        }
    };

    // A scholarship has no reference because nothing was charged. There is
    // nobody to ask, and inventing one would be asking about somebody else's
    // subscription.
    let Some(reference) = membership.provider_ref.clone() else {
        return Subject::Refused(refuse(Refusal::NotPurchased));
    };

    Subject::Ready(driver, membership, reference)
}

/// Record what the provider said, and answer with the membership as it stands.
///
/// Written from the provider's answer rather than from what was asked for, so
/// a change that only partly took effect is stored as what actually happened.
async fn settled(
    state: &AppState,
    user_id: i64,
    provider: &str,
    updated: &Subscription,
) -> Response {
    if let Err(error) =
        store::record(&state.db, user_id, provider, updated).await
    {
        tracing::error!(%error, "failed to record a membership change");
        return response::server_error();
    }

    match store::live(&state.db, user_id).await {
        Ok(Some(membership)) => paid(&membership),
        Ok(None) => {
            response::empty(StatusCode::NO_CONTENT, CachePolicy::NoStore)
        }
        Err(error) => {
            tracing::error!(%error, "failed to read a membership back");
            response::server_error()
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
    let track = track::of_plan(plan);
    let books = track::books(&state.catalog.current(), track);

    if books.is_empty() {
        // Nothing to grant means the reader paid for a track that no longer
        // names any book with an id. Loud, because the money has been taken.
        tracing::error!(
            %plan, track, user_id,
            "a purchase granted nothing; the track has no books with ids"
        );
        return Ok(());
    }

    let granted = store::grant(&state.db, user_id, &books).await?;

    tracing::info!(
        %plan, track, user_id,
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
