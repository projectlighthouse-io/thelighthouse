//! The Stripe driver.
//!
//! Six endpoints and a signature check. Everything Stripe can do that this
//! crate does not is absent rather than wrapped, which is why this is a few
//! hundred lines instead of a generated client.

mod client;
mod webhook;
mod wire;

use reqwest::Method;
use secrecy::SecretString;

use self::client::{Client, field};
use crate::{
    checkout::{Bought, Customer, Handoff, Returns},
    error::Error,
    event::Event,
    gateway::Gateway,
    plan::Plan,
    registry::Registration,
    request::RequestStrategy,
    subscription::{Cancel, Subscription},
};

/// The name this driver registers under, and the one a route parameter and a
/// stored row will carry.
pub const NAME: &str = "stripe";

/// Everything the driver needs to take money.
///
/// A struct rather than a parameter list, and with no `Default`, deliberately.
/// Two adjacent `String` parameters is a call where transposing the api key
/// and the webhook secret compiles — and then fails much later as "every
/// delivery is refused" rather than as a type error. Naming every field at the
/// literal makes both mistakes unwriteable, and a `Default` to spread over
/// would hand back the ability to omit one silently.
///
/// Both secrets are [`SecretString`], so `Debug` prints `[REDACTED]` and the
/// memory is zeroed on drop. Redaction is a property of the field's type, not
/// of somebody remembering to hand-write a `Debug`.
///
/// Where checkout returns to is *not* here: that is per checkout, because the
/// caller usually wants to say which plan was bought on the way back. See
/// [`Returns`].
#[derive(Debug)]
pub struct StripeConfig {
    /// The api key. `sk_live_…` or `sk_test_…`.
    pub secret_key: SecretString,
    /// The signing secret for *this* webhook endpoint — the `whsec_…` shown
    /// when the endpoint is created. Not the api key, and different per
    /// endpoint, so staging and production do not share one.
    pub webhook_secret: SecretString,
    /// How hard to try each request.
    pub strategy: RequestStrategy,
}

/// Which kind of checkout session to open.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Mode {
    /// Recurring. Creates a subscription.
    Subscription,
    /// A single payment. Creates nothing to manage afterwards.
    Payment,
}

impl Mode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Subscription => "subscription",
            Self::Payment => "payment",
        }
    }

    /// The parameter stripe copies metadata from onto whatever it creates.
    const fn data(self) -> &'static str {
        match self {
            Self::Subscription => "subscription_data",
            Self::Payment => "payment_intent_data",
        }
    }
}

/// Stripe, as a [`Gateway`].
#[derive(Debug)]
pub struct Stripe {
    client: Client,
    webhook_secret: SecretString,
}

impl Stripe {
    /// Build the driver.
    ///
    /// # Errors
    ///
    /// [`Error::Unreachable`] if an http client cannot be built at all, which
    /// is a broken tls configuration rather than anything to do with Stripe.
    pub fn new(config: StripeConfig) -> Result<Self, Error> {
        Ok(Self {
            client: Client::new(config.secret_key, config.strategy)?,
            webhook_secret: config.webhook_secret,
        })
    }

    /// The same driver, named, ready for [`providers`](crate::providers).
    ///
    /// # Errors
    ///
    /// [`Error::Unreachable`] if an http client cannot be built.
    pub fn register(config: StripeConfig) -> Result<Registration, Error> {
        Ok(Registration::new(NAME, Self::new(config)?))
    }

    /// The secret webhook deliveries are signed with.
    ///
    /// Exposed here and used immediately by the hmac. That is the whole point
    /// of the wrapper: the plain string exists for one expression rather than
    /// for the lifetime of a struct that something might print.
    pub(crate) fn webhook_secret(&self) -> &SecretString {
        &self.webhook_secret
    }

    /// Point the driver at another host. For tests.
    #[cfg(test)]
    pub(crate) fn with_base(mut self, base: impl Into<String>) -> Self {
        self.client = self.client.with_base(base);
        self
    }

    /// The customer to bill: the one we already have, or a new one.
    ///
    /// Reusing the existing id is what keeps a reader's payment history in one
    /// place. Without it, every checkout mints another customer and the
    /// account's history is split across all of them.
    pub(crate) async fn customer(
        &self,
        who: &Customer<'_>,
    ) -> Result<String, Error> {
        if let Some(existing) = who.existing {
            return Ok(existing.to_owned());
        }

        let created: wire::Customer = self
            .client
            .send(
                Method::POST,
                "/v1/customers",
                &[
                    field("email", who.email),
                    field(
                        format!("metadata[{}]", wire::REFERENCE_KEY),
                        who.reference,
                    ),
                ],
            )
            .await?;

        Ok(created.id)
    }

    /// Open a hosted checkout session for a plan.
    ///
    /// `mode` is what makes this a subscription or a single payment, and it
    /// decides where the metadata has to go: stripe copies
    /// `subscription_data[metadata]` onto the subscription it creates, and
    /// `payment_intent_data[metadata]` onto the payment intent. Neither exists
    /// in the other mode, so sending the wrong one silently drops it — and the
    /// metadata is how a delivery finds the account it belongs to.
    ///
    /// The session's own `metadata` carries the plan in both modes, because a
    /// one-time purchase has no subscription to read it back from later.
    pub(crate) async fn checkout(
        &self,
        to: &Plan,
        customer: &str,
        who: &Customer<'_>,
        mode: Mode,
        back: &Returns,
    ) -> Result<Handoff, Error> {
        let reference = who.reference;

        // The reader's tier applied up front, so the price the page showed is
        // the price stripe asks for without anybody typing a code. Stripe
        // refuses a session that both applies a discount and offers the code
        // box, so a tier replaces the box rather than joining it.
        //
        // Without a tier, the box: a coupon nobody can type is a coupon that
        // does not exist, and `lighthouse-prices` would go on creating codes no
        // reader could ever redeem.
        let discount = who.promotion.map_or_else(
            || field("allow_promotion_codes", "true"),
            |promotion| field("discounts[0][promotion_code]", promotion),
        );

        let session: wire::Session = self
            .client
            .send(
                Method::POST,
                "/v1/checkout/sessions",
                &[
                    field("mode", mode.as_str()),
                    field("customer", customer),
                    field("line_items[0][price]", to.price.clone()),
                    field("line_items[0][quantity]", "1"),
                    field("success_url", back.success.clone()),
                    field("cancel_url", back.cancel.clone()),
                    field("client_reference_id", reference),
                    discount,
                    // Written here and read back on every subscription and
                    // every webhook. It is what lets a delivery about
                    // `sub_123` find the account and the plan without a
                    // lookup table on our side.
                    field(
                        format!("metadata[{}]", wire::PLAN_KEY),
                        to.id.as_str(),
                    ),
                    field(
                        format!("metadata[{}]", wire::REFERENCE_KEY),
                        reference,
                    ),
                    field(
                        format!(
                            "{}[metadata][{}]",
                            mode.data(),
                            wire::PLAN_KEY
                        ),
                        to.id.as_str(),
                    ),
                    field(
                        format!(
                            "{}[metadata][{}]",
                            mode.data(),
                            wire::REFERENCE_KEY
                        ),
                        reference,
                    ),
                ],
            )
            .await?;

        // `url` is null for a session that is no longer active. There is
        // nowhere to send the reader, and pretending otherwise would be a
        // redirect to an empty string.
        session
            .url
            .map(|url| Handoff { url })
            .ok_or(Error::Refused {
                status: 200,
                code: "session_not_payable".to_owned(),
                message: "the checkout session came back with no url"
                    .to_owned(),
            })
    }

    /// Read a subscription back.
    pub(crate) async fn subscription(
        &self,
        reference: &str,
    ) -> Result<wire::Sub, Error> {
        self.client
            .send(Method::GET, &format!("/v1/subscriptions/{reference}"), &[])
            .await
    }

    /// Change a subscription, and return it as it now stands.
    pub(crate) async fn update(
        &self,
        reference: &str,
        fields: &[(String, String)],
    ) -> Result<Subscription, Error> {
        let updated: wire::Sub = self
            .client
            .send(
                Method::POST,
                &format!("/v1/subscriptions/{reference}"),
                fields,
            )
            .await?;

        Ok(updated.into())
    }

    /// End a subscription now, forfeiting the rest of the paid period.
    pub(crate) async fn end(
        &self,
        reference: &str,
    ) -> Result<Subscription, Error> {
        let ended: wire::Sub = self
            .client
            .send(
                Method::DELETE,
                &format!("/v1/subscriptions/{reference}"),
                &[],
            )
            .await?;

        Ok(ended.into())
    }

    /// Move a subscription onto another plan.
    ///
    /// Two calls, because Stripe changes a price by naming the *item* holding
    /// it, and the item id is only knowable by reading the subscription first.
    ///
    /// No proration: the reader is moved onto the new plan and charged for it
    /// at the next renewal rather than handed an immediate partial invoice.
    pub(crate) async fn move_to(
        &self,
        reference: &str,
        to: &Plan,
    ) -> Result<Subscription, Error> {
        let current = self.subscription(reference).await?;

        let item = current.item().ok_or_else(|| Error::Refused {
            status: 409,
            code: "no_item_to_move".to_owned(),
            message: "the subscription has no item whose price could change"
                .to_owned(),
        })?;

        self.update(
            reference,
            &[
                field("items[0][id]", item),
                field("items[0][price]", to.price.clone()),
                field("proration_behavior", "none"),
                // The stored plan has to move with the price, or every later
                // read reports the plan the reader used to be on.
                field(format!("metadata[{}]", wire::PLAN_KEY), to.id.as_str()),
            ],
        )
        .await
    }
}

#[async_trait::async_trait]
impl Gateway for Stripe {
    async fn subscribe(
        &self,
        to: &Plan,
        who: &Customer<'_>,
        back: &Returns,
    ) -> Result<Handoff, Error> {
        let customer = self.customer(who).await?;

        self.checkout(to, &customer, who, Mode::Subscription, back)
            .await
    }

    async fn purchase(
        &self,
        what: &Plan,
        who: &Customer<'_>,
        back: &Returns,
    ) -> Result<Handoff, Error> {
        let customer = self.customer(who).await?;

        self.checkout(what, &customer, who, Mode::Payment, back)
            .await
    }

    async fn cancel(
        &self,
        subscription: &str,
        when: Cancel,
    ) -> Result<Subscription, Error> {
        match when {
            // Not a deletion: the subscription stays live and stops renewing,
            // which is what leaves the reader the period they paid for.
            Cancel::AtPeriodEnd => {
                self.update(
                    subscription,
                    &[field("cancel_at_period_end", "true")],
                )
                .await
            }
            Cancel::Now => self.end(subscription).await,
            // Also not a deletion. Stripe holds the date and ends it itself,
            // so the answer does not depend on anything here still running
            // when the moment arrives.
            Cancel::At(when) => {
                self.update(
                    subscription,
                    &[field("cancel_at", when.timestamp().to_string())],
                )
                .await
            }
        }
    }

    async fn resume(&self, subscription: &str) -> Result<Subscription, Error> {
        self.update(subscription, &[field("cancel_at_period_end", "false")])
            .await
    }

    async fn swap(
        &self,
        subscription: &str,
        to: &Plan,
    ) -> Result<Subscription, Error> {
        self.move_to(subscription, to).await
    }

    async fn manage(
        &self,
        customer: &str,
        back: &str,
    ) -> Result<Handoff, Error> {
        let portal: wire::Portal = self
            .client
            .send(
                Method::POST,
                "/v1/billing_portal/sessions",
                &[field("customer", customer), field("return_url", back)],
            )
            .await?;

        Ok(Handoff { url: portal.url })
    }

    async fn bought(&self, session: &str) -> Result<Bought, Error> {
        let session: wire::Session = self
            .client
            .send(
                Method::GET,
                &format!("/v1/checkout/sessions/{session}"),
                &[],
            )
            .await?;

        Ok(Bought {
            plan: session
                .metadata
                .get(wire::PLAN_KEY)
                .map(|name| name.clone().into()),
            reference: session.client_reference_id,
            // `no_payment_required` is a zero-amount session — a full discount
            // — which is paid for as far as access goes.
            paid: session
                .payment_status
                .as_deref()
                .is_some_and(|status| status != "unpaid"),
            subscription: session.subscription,
        })
    }

    fn signature_header(&self) -> &'static str {
        "stripe-signature"
    }

    fn settle(&self, body: &[u8], signature: &str) -> Result<Event, Error> {
        webhook::settle(self.webhook_secret(), body, signature)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, method, path},
    };

    use super::*;
    use crate::subscription::Status;

    const SUBSCRIPTION: &str = r#"{
        "id": "sub_1",
        "status": "active",
        "items": { "data": [{ "id": "si_1", "current_period_end": 1682288167 }] },
        "metadata": { "plan": "yearly", "reference": "41" }
    }"#;

    fn config(strategy: RequestStrategy) -> StripeConfig {
        StripeConfig {
            secret_key: "sk_test".into(),
            webhook_secret: "whsec_test".into(),
            strategy,
        }
    }

    fn driver(server: &MockServer, strategy: RequestStrategy) -> Stripe {
        Stripe::new(config(strategy))
            .unwrap()
            .with_base(server.uri())
    }

    /// Where a test checkout comes back to. The urls are the caller's, and
    /// these assert nothing about them beyond being sent.
    fn back() -> Returns {
        Returns {
            success: "https://example.com/paid".to_owned(),
            cancel: "https://example.com/pricing".to_owned(),
        }
    }

    fn plan() -> Plan {
        Plan {
            id: "yearly".into(),
            price: "price_yearly".to_owned(),
            interval: crate::plan::Interval::Year,
            money: None,
            books: Vec::new(),
            everything: false,
            ppp: Vec::new(),
        }
    }

    fn reader(existing: Option<&'static str>) -> Customer<'static> {
        Customer {
            reference: "41",
            email: "reader@example.com",
            existing,
            promotion: None,
        }
    }

    /// Matches a request whose body does *not* contain `needle`. Wiremock has
    /// a matcher for presence only, and the absence is the assertion here.
    struct BodyLacks(&'static str);

    impl wiremock::Match for BodyLacks {
        fn matches(&self, request: &wiremock::Request) -> bool {
            !String::from_utf8_lossy(&request.body).contains(self.0)
        }
    }

    #[tokio::test]
    async fn a_readers_tier_is_applied_and_the_code_box_is_not_offered() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .and(body_string_contains(
                "discounts%5B0%5D%5Bpromotion_code%5D=promo_everyone",
            ))
            // Stripe refuses a session carrying both.
            .and(BodyLacks("allow_promotion_codes"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"url":"https://checkout.stripe.com/c/pay/cs_3"}"#,
            ))
            .mount(&server)
            .await;

        let who = Customer {
            promotion: Some("promo_everyone"),
            ..reader(Some("cus_existing"))
        };

        let handoff = driver(&server, RequestStrategy::Once)
            .subscribe(&plan(), &who, &back())
            .await
            .unwrap();

        assert_eq!(handoff.url, "https://checkout.stripe.com/c/pay/cs_3");
    }

    #[tokio::test]
    async fn checking_out_sends_the_plan_the_account_and_where_to_come_back_to()
    {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            // The assertions that matter are on the request, not the reply: a
            // driver that builds the wrong body still gets a canned 200.
            .and(body_string_contains("mode=subscription"))
            .and(body_string_contains(
                "line_items%5B0%5D%5Bprice%5D=price_yearly",
            ))
            .and(body_string_contains("customer=cus_existing"))
            .and(body_string_contains("client_reference_id=41"))
            .and(body_string_contains("allow_promotion_codes=true"))
            .and(body_string_contains(
                "subscription_data%5Bmetadata%5D%5Bplan%5D=yearly",
            ))
            .and(body_string_contains(
                "subscription_data%5Bmetadata%5D%5Breference%5D=41",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"url":"https://checkout.stripe.com/c/pay/cs_1"}"#,
            ))
            .mount(&server)
            .await;

        let handoff = driver(&server, RequestStrategy::Once)
            .subscribe(&plan(), &reader(Some("cus_existing")), &back())
            .await
            .unwrap();

        assert_eq!(handoff.url, "https://checkout.stripe.com/c/pay/cs_1");
    }

    #[tokio::test]
    async fn buying_outright_is_a_payment_and_carries_the_plan() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .and(body_string_contains("mode=payment"))
            // A one-time purchase gets the promo box too — a lifetime plan
            // sold at a list price is exactly the case that needs it.
            .and(body_string_contains("allow_promotion_codes=true"))
            // The session's own metadata, which is the only place a one-time
            // purchase records what it was for: there is no subscription
            // afterwards to read it back off.
            .and(body_string_contains("metadata%5Bplan%5D=yearly"))
            // `payment_intent_data`, not `subscription_data`. The wrong one
            // does not error — stripe drops it, and the delivery arrives with
            // no way to tell whose payment it was.
            .and(body_string_contains(
                "payment_intent_data%5Bmetadata%5D%5Breference%5D=41",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"url":"https://checkout.stripe.com/c/pay/cs_2"}"#,
            ))
            .mount(&server)
            .await;

        let handoff = driver(&server, RequestStrategy::Once)
            .purchase(&plan(), &reader(Some("cus_existing")), &back())
            .await
            .unwrap();

        assert_eq!(handoff.url, "https://checkout.stripe.com/c/pay/cs_2");
    }

    #[tokio::test]
    async fn subscribing_puts_its_metadata_where_a_subscription_reads_it() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .and(body_string_contains("mode=subscription"))
            .and(body_string_contains(
                "subscription_data%5Bmetadata%5D%5Breference%5D=41",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"url":"https://checkout.stripe.com/c/pay/cs_1"}"#,
            ))
            .mount(&server)
            .await;

        assert!(
            driver(&server, RequestStrategy::Once)
                .subscribe(&plan(), &reader(Some("cus_existing")), &back())
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn a_reader_with_no_customer_yet_gets_one_before_checkout() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/customers"))
            .and(body_string_contains("email=reader%40example.com"))
            .and(body_string_contains("metadata%5Breference%5D=41"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"id":"cus_new"}"#),
            )
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .and(body_string_contains("customer=cus_new"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"url":"https://checkout.stripe.com/c/pay/cs_1"}"#,
            ))
            .mount(&server)
            .await;

        assert!(
            driver(&server, RequestStrategy::Once)
                .subscribe(&plan(), &reader(None), &back())
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn a_reader_who_already_has_a_customer_does_not_get_a_second_one() {
        // Two customers for one account is a payment history split in half.
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"url":"https://checkout.stripe.com/c/pay/cs_1"}"#,
            ))
            .mount(&server)
            .await;

        driver(&server, RequestStrategy::Once)
            .subscribe(&plan(), &reader(Some("cus_existing")), &back())
            .await
            .unwrap();

        let sent = server.received_requests().await.unwrap();
        assert_eq!(sent.len(), 1);
        assert!(!sent.first().unwrap().url.path().contains("customers"));
    }

    #[tokio::test]
    async fn managing_opens_a_portal_for_the_customer_and_says_where_to_return()
    {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/billing_portal/sessions"))
            .and(body_string_contains("customer=cus_existing"))
            .and(body_string_contains(
                "return_url=https%3A%2F%2Fexample.com%2Fsettings%2Fbilling",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"url":"https://billing.stripe.com/p/session/bps_1"}"#,
            ))
            .mount(&server)
            .await;

        let handoff = driver(&server, RequestStrategy::Once)
            .manage("cus_existing", "https://example.com/settings/billing")
            .await
            .unwrap();

        assert_eq!(handoff.url, "https://billing.stripe.com/p/session/bps_1");
    }

    #[tokio::test]
    async fn cancelling_at_period_end_stops_renewal_without_deleting_anything()
    {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/subscriptions/sub_1"))
            .and(body_string_contains("cancel_at_period_end=true"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(SUBSCRIPTION),
            )
            .mount(&server)
            .await;

        let subscription = driver(&server, RequestStrategy::Once)
            .cancel("sub_1", Cancel::AtPeriodEnd)
            .await
            .unwrap();

        assert_eq!(subscription.status, Status::Active);
        assert_eq!(subscription.account.as_deref(), Some("41"));
    }

    #[tokio::test]
    async fn cancelling_now_deletes_it() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/v1/subscriptions/sub_1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(SUBSCRIPTION),
            )
            .mount(&server)
            .await;

        assert!(
            driver(&server, RequestStrategy::Once)
                .cancel("sub_1", Cancel::Now)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn cancelling_at_a_date_hands_stripe_the_moment_to_end_it() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/subscriptions/sub_1"))
            // Unix seconds, which is what `cancel_at` takes. Asserted on the
            // wire because a date sent in any other shape is accepted by
            // nothing and would fail only against the real api.
            .and(body_string_contains("cancel_at=1793577600"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(SUBSCRIPTION),
            )
            .mount(&server)
            .await;

        let when = chrono::DateTime::from_timestamp(1_793_577_600, 0).unwrap();

        assert!(
            driver(&server, RequestStrategy::Once)
                .cancel("sub_1", Cancel::At(when))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn resuming_clears_the_pending_cancellation() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/subscriptions/sub_1"))
            .and(body_string_contains("cancel_at_period_end=false"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(SUBSCRIPTION),
            )
            .mount(&server)
            .await;

        assert!(
            driver(&server, RequestStrategy::Once)
                .resume("sub_1")
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn swapping_names_the_item_and_moves_the_stored_plan_with_the_price()
    {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/subscriptions/sub_1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(SUBSCRIPTION),
            )
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/subscriptions/sub_1"))
            // The item id can only come from the read above — Stripe changes a
            // price by naming the item that holds it.
            .and(body_string_contains("items%5B0%5D%5Bid%5D=si_1"))
            .and(body_string_contains(
                "items%5B0%5D%5Bprice%5D=price_monthly",
            ))
            .and(body_string_contains("proration_behavior=none"))
            // Without this the reader is on the new price and every later read
            // still reports the old plan.
            .and(body_string_contains("metadata%5Bplan%5D=monthly"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(SUBSCRIPTION),
            )
            .mount(&server)
            .await;

        let monthly = Plan {
            id: "monthly".into(),
            price: "price_monthly".to_owned(),
            interval: crate::plan::Interval::Month,
            money: None,
            books: Vec::new(),
            everything: false,
            ppp: Vec::new(),
        };

        assert!(
            driver(&server, RequestStrategy::Once)
                .swap("sub_1", &monthly)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn a_retried_request_carries_one_key_across_every_attempt() {
        // The failure this guards: a key minted per attempt turns a retry loop
        // into one subscription per timeout.
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/subscriptions/sub_1"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/subscriptions/sub_1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(SUBSCRIPTION),
            )
            .mount(&server)
            .await;

        driver(&server, RequestStrategy::Retry(3))
            .resume("sub_1")
            .await
            .unwrap();

        let sent = server.received_requests().await.unwrap();
        assert_eq!(sent.len(), 3);

        let mut keys: Vec<_> = sent
            .iter()
            .map(|request| {
                request
                    .headers
                    .get("idempotency-key")
                    .expect("every attempt must carry one")
                    .to_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();

        keys.dedup();
        assert_eq!(keys.len(), 1, "the three attempts used {keys:?}");
    }

    #[tokio::test]
    async fn a_declined_card_is_an_answer_and_is_not_asked_again() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/subscriptions/sub_1"))
            .respond_with(ResponseTemplate::new(402).set_body_string(
                r#"{"error":{"type":"card_error","code":"card_declined","message":"Your card was declined."}}"#,
            ))
            .mount(&server)
            .await;

        let failure = driver(&server, RequestStrategy::Retry(3))
            .resume("sub_1")
            .await
            .unwrap_err();

        assert!(matches!(
            failure,
            Error::Refused { status: 402, ref code, .. } if code == "card_declined"
        ));
        assert!(!failure.retryable());
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn every_request_pins_the_api_version() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/subscriptions/sub_1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(SUBSCRIPTION),
            )
            .mount(&server)
            .await;

        driver(&server, RequestStrategy::Once)
            .subscription("sub_1")
            .await
            .unwrap();

        let sent = server.received_requests().await.unwrap();
        assert_eq!(
            sent.first()
                .unwrap()
                .headers
                .get("stripe-version")
                .unwrap()
                .to_str()
                .unwrap(),
            client::API_VERSION
        );
    }

    #[test]
    fn the_driver_registers_under_its_own_name() {
        let registration =
            Stripe::register(config(RequestStrategy::ExponentialBackoff(3)))
                .unwrap();

        assert_eq!(registration.name(), NAME);
    }

    #[test]
    fn neither_secret_survives_a_debug_dump() {
        // Guaranteed by `SecretString` rather than by a hand-written `Debug`,
        // which is the point of using it: this holds for every struct the key
        // is ever put inside, including ones written later.
        let real = || StripeConfig {
            secret_key: "sk_live_actual_secret".into(),
            webhook_secret: "whsec_actual_secret".into(),
            ..config(RequestStrategy::Once)
        };

        let settings = real();
        let driver = Stripe::new(real()).unwrap();

        for dumped in [format!("{settings:?}"), format!("{driver:?}")] {
            assert!(!dumped.contains("sk_live_actual_secret"), "{dumped}");
            assert!(!dumped.contains("whsec_actual_secret"), "{dumped}");
        }
    }
}
