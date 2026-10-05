//! Stripe's json, as narrowly as this crate needs to read it.
//!
//! Every struct here names only the fields that are used. Stripe's objects
//! carry dozens more, and a driver that modelled all of them would need
//! changing every time Stripe adds one.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDateTime};
use serde::Deserialize;

use crate::{
    plan::PlanId,
    subscription::{Status, Subscription},
};

/// The metadata key the plan's name is stored under.
///
/// Written at checkout and read back on every subscription. The alternative —
/// mapping the price handle in a response back to a configured plan — breaks
/// the day a price is retired, because the handle on an existing subscription
/// is the old one and there is nothing left to match it against.
pub(crate) const PLAN_KEY: &str = "plan";

/// The metadata key the application's own account reference is stored under.
pub(crate) const REFERENCE_KEY: &str = "reference";

/// Seconds since the epoch, as Stripe writes every timestamp.
fn at(seconds: Option<i64>) -> Option<NaiveDateTime> {
    seconds
        .and_then(|seconds| DateTime::from_timestamp(seconds, 0))
        .map(|moment| moment.naive_utc())
}

#[derive(Debug, Deserialize)]
pub(crate) struct Customer {
    pub(crate) id: String,
}

/// A billing portal session. Unlike a checkout session its url is never null.
#[derive(Debug, Deserialize)]
pub(crate) struct Portal {
    pub(crate) url: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Session {
    pub(crate) url: Option<String>,
    /// `paid`, `unpaid`, or `no_payment_required`. Absent on the webhook
    /// payloads this crate also parses with this struct.
    pub(crate) payment_status: Option<String>,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
    pub(crate) customer: Option<String>,
    pub(crate) client_reference_id: Option<String>,
    pub(crate) subscription: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Sub {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) cancel_at: Option<i64>,
    pub(crate) ended_at: Option<i64>,
    /// Where the period end used to live.
    ///
    /// It moved onto the subscription items, and both shapes are still in
    /// circulation here: api responses use the version this driver pins, while
    /// **webhook payloads use the account's default version**, which is set in
    /// the dashboard and is not ours to choose. Reading both is three lines
    /// and removes an entire class of "the date is silently null" bug.
    pub(crate) current_period_end: Option<i64>,
    #[serde(default)]
    pub(crate) items: Items,
    #[serde(default)]
    pub(crate) metadata: BTreeMap<String, String>,
    /// Who pays for it. Kept so the application can find the owner of a
    /// subscription that carries none of this crate's metadata.
    #[serde(default)]
    pub(crate) customer: Option<CustomerRef>,
}

/// A customer as a subscription names it.
///
/// An id unless somebody asked Stripe to expand it. Nothing here does, but an
/// expanded object must not fail the whole subscription: it is read as an
/// unknown customer rather than as an unreadable event.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum CustomerRef {
    Id(String),
    Expanded(serde::de::IgnoredAny),
}

impl CustomerRef {
    fn id(self) -> Option<String> {
        match self {
            Self::Id(id) => Some(id),
            Self::Expanded(_) => None,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Items {
    #[serde(default)]
    pub(crate) data: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Item {
    pub(crate) id: String,
    pub(crate) current_period_end: Option<i64>,
    #[serde(default)]
    pub(crate) price: Option<ItemPrice>,
}

/// The price an item is on, for its lookup key and nothing else.
#[derive(Debug, Deserialize)]
pub(crate) struct ItemPrice {
    #[serde(default)]
    pub(crate) lookup_key: Option<String>,
}

impl Sub {
    /// The first item's id, which is what an update has to name to change a
    /// price.
    ///
    /// One item, because this crate sells single-price subscriptions. A
    /// subscription with several is one it did not create.
    pub(crate) fn item(&self) -> Option<&str> {
        self.items.data.first().map(|item| item.id.as_str())
    }
}

impl From<Sub> for Subscription {
    fn from(sub: Sub) -> Self {
        let period_ends_at = at(sub.current_period_end.or_else(|| {
            sub.items
                .data
                .first()
                .and_then(|item| item.current_period_end)
        }));

        let price_key = sub
            .items
            .data
            .first()
            .and_then(|item| item.price.as_ref())
            .and_then(|price| price.lookup_key.clone());

        Self {
            plan: sub
                .metadata
                .get(PLAN_KEY)
                .map(|name| PlanId::from(name.clone())),
            status: status(&sub.status),
            period_ends_at,
            cancel_at: at(sub.cancel_at).or_else(|| at(sub.ended_at)),
            account: sub.metadata.get(REFERENCE_KEY).cloned(),
            customer: sub.customer.and_then(CustomerRef::id),
            price_key,
            reference: sub.id,
        }
    }
}

/// Stripe's eight statuses onto the four worth telling apart.
///
/// An unrecognised one is [`Status::Incomplete`] rather than a parse error: a
/// status this driver has not heard of is a subscription that is certainly not
/// granting access, and failing to read the webhook would leave the stored row
/// saying `active` forever.
fn status(stripe: &str) -> Status {
    match stripe {
        // A trial is live access. Nothing here sells one, but a subscription
        // created by hand in the dashboard can have one.
        "active" | "trialing" => Status::Active,
        "past_due" | "unpaid" => Status::PastDue,
        "canceled" | "incomplete_expired" => Status::Canceled,
        _ => Status::Incomplete,
    }
}

/// Stripe's error envelope. Every non-2xx carries one.
#[derive(Debug, Deserialize)]
pub(crate) struct Failure {
    pub(crate) error: FailureDetail,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FailureDetail {
    /// Absent on some error types, where `type` is all there is.
    pub(crate) code: Option<String>,
    #[serde(rename = "type")]
    pub(crate) kind: String,
    pub(crate) message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Subscription {
        serde_json::from_str::<Sub>(json).unwrap().into()
    }

    #[test]
    fn the_period_end_is_read_from_the_item_where_it_now_lives() {
        let subscription = parse(
            r#"{
                "id": "sub_1",
                "status": "active",
                "cancel_at": null,
                "ended_at": null,
                "items": { "data": [{ "id": "si_1", "current_period_end": 1682288167 }] },
                "metadata": { "plan": "yearly", "reference": "41" }
            }"#,
        );

        assert_eq!(subscription.reference, "sub_1");
        assert_eq!(subscription.status, Status::Active);
        assert_eq!(subscription.plan, Some("yearly".into()));
        assert_eq!(subscription.account.as_deref(), Some("41"));
        assert!(subscription.period_ends_at.is_some());
    }

    #[test]
    fn the_period_end_is_still_read_where_it_used_to_live() {
        // Webhook payloads are serialised with the account's default api
        // version, not the one this driver pins. If that account is on an
        // older version, this is the only place the date appears.
        let subscription = parse(
            r#"{
                "id": "sub_1",
                "status": "active",
                "current_period_end": 1682288167,
                "cancel_at": null,
                "ended_at": null,
                "items": { "data": [] },
                "metadata": {}
            }"#,
        );

        assert!(subscription.period_ends_at.is_some());
        assert_eq!(subscription.plan, None);
    }

    #[test]
    fn every_stripe_status_lands_somewhere_deliberate() {
        assert_eq!(status("active"), Status::Active);
        assert_eq!(status("trialing"), Status::Active);
        assert_eq!(status("past_due"), Status::PastDue);
        assert_eq!(status("unpaid"), Status::PastDue);
        assert_eq!(status("canceled"), Status::Canceled);
        assert_eq!(status("incomplete_expired"), Status::Canceled);
        assert_eq!(status("incomplete"), Status::Incomplete);
        assert_eq!(status("paused"), Status::Incomplete);
        // The one that matters: a status added after this was written must not
        // read as access.
        assert_eq!(status("something_new"), Status::Incomplete);
        assert!(!status("something_new").is_live());
    }

    #[test]
    fn a_subscription_this_crate_did_not_create_still_parses() {
        // Made in the dashboard: no metadata, so no plan. It is still a real
        // subscription somebody is paying for, and refusing to read it would
        // lock them out.
        let subscription = parse(
            r#"{
                "id": "sub_manual",
                "status": "active",
                "items": { "data": [{ "id": "si_1", "current_period_end": 1682288167 }] }
            }"#,
        );

        assert_eq!(subscription.plan, None);
        assert_eq!(subscription.status, Status::Active);
    }

    #[test]
    fn the_customer_and_the_price_lookup_key_are_kept() {
        // What a subscription sold by the laravel app looks like: no metadata,
        // so the customer and the price are all there is to recognise it by.
        let subscription = parse(
            r#"{
                "id": "sub_laravel",
                "status": "active",
                "customer": "cus_123",
                "items": { "data": [{
                    "id": "si_1",
                    "current_period_end": 1682288167,
                    "price": { "id": "price_1", "lookup_key": "lighthouse_voyage_yearly" }
                }] }
            }"#,
        );

        assert_eq!(subscription.customer.as_deref(), Some("cus_123"));
        assert_eq!(
            subscription.price_key.as_deref(),
            Some("lighthouse_voyage_yearly")
        );
        assert_eq!(subscription.plan, None);
    }
}
