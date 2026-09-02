//! The interface a payment provider is reached through.

use crate::{
    checkout::{Customer, Handoff},
    error::Error,
    event::Event,
    plan::Plan,
    subscription::{Cancel, Subscription},
};

/// One payment provider, in five verbs.
///
/// The vocabulary is deliberately ours and not any provider's. Nothing in a
/// signature here names anything a provider invented — a checkout session, an
/// order, a wallet intent — because the moment one leaks through, the second
/// implementation has to pretend to have the first one's concepts.
///
/// # Implementing one
///
/// This trait is public and the registry takes anything that implements it, so
/// a provider this crate has never heard of is a driver you write and register
/// beside the ones that ship:
///
/// ```text
/// #[async_trait::async_trait]
/// impl Gateway for MyProvider {
///     async fn subscribe(&self, to: &Plan, who: &Customer<'_>) -> Result<Handoff, Error> { … }
///     // …
/// }
///
/// let billing = billing::providers([
///     Registration::new("myprovider", MyProvider::new(key)),
/// ])?;
/// ```
///
/// `Debug` is a supertrait because a registry of these usually ends up inside
/// some larger state struct that derives it. Implement it by hand and redact
/// the credentials — the whole point of the bound is defeated by a derive that
/// prints an api key into a log.
#[async_trait::async_trait]
pub trait Gateway: std::fmt::Debug + Send + Sync {
    /// Start a subscription, and say where to send the browser to pay for it.
    ///
    /// Returns before any money moves. The subscription does not exist yet and
    /// may never exist — a reader who closes the tab leaves nothing behind but
    /// an abandoned session. Treat the [`Handoff`] as an invitation, and let
    /// the webhook be what says it was accepted.
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] if the provider will not open a session — an unknown
    /// price handle, a customer it has deleted.
    async fn subscribe(
        &self,
        to: &Plan,
        who: &Customer<'_>,
    ) -> Result<Handoff, Error>;

    /// Buy something outright, and say where to send the browser to pay.
    ///
    /// The one-time counterpart to [`Gateway::subscribe`]. A separate verb
    /// rather than a flag on that one because the two are different objects at
    /// every provider — one renews and can be cancelled, the other is a single
    /// payment with nothing to manage afterwards — and a boolean would put
    /// that difference in the caller's head instead of in the type.
    ///
    /// What the purchase entitles somebody to is the caller's to decide; this
    /// only takes the money and reports that it was taken.
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] if the provider will not open a session.
    async fn purchase(
        &self,
        what: &Plan,
        who: &Customer<'_>,
    ) -> Result<Handoff, Error>;

    /// Stop a subscription, at the end of the paid period or immediately.
    ///
    /// Returns it as it now stands rather than nothing, because the two
    /// [`Cancel`] variants leave it in genuinely different states and the
    /// caller has a row to update either way.
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] if the provider does not recognise the reference, or
    /// has already ended it.
    async fn cancel(
        &self,
        subscription: &str,
        when: Cancel,
    ) -> Result<Subscription, Error>;

    /// Undo a cancellation that has not taken effect yet.
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] if the period has already ended — that subscription
    /// is over, and starting another one is [`Gateway::subscribe`].
    async fn resume(&self, subscription: &str) -> Result<Subscription, Error>;

    /// Move a subscription to another plan.
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] if the provider will not make the change.
    async fn swap(
        &self,
        subscription: &str,
        to: &Plan,
    ) -> Result<Subscription, Error>;

    /// The request header this provider puts its delivery signature in.
    ///
    /// Here rather than in the caller because it is the provider's choice, and
    /// an endpoint that hardcoded `Stripe-Signature` would be an endpoint that
    /// only ever serves one provider — which is the coupling the rest of this
    /// trait exists to avoid.
    fn signature_header(&self) -> &'static str;

    /// Verify a webhook delivery and say what it means.
    ///
    /// **Not `async`, and that is deliberate.** This is a hash and a parse with
    /// no io in it, and marking it `async` would promise a suspension point
    /// that does not exist while forcing every caller to await a future that is
    /// ready before it is polled.
    ///
    /// Pass the body exactly as it arrived — the raw bytes, before any json
    /// round trip. Signatures cover the bytes, and a body that has been parsed
    /// and re-serialised is a different sequence of them.
    ///
    /// # Errors
    ///
    /// [`Error::Signature`] if it does not verify or arrived too long after it
    /// was signed. [`Error::Malformed`] if it verifies but cannot be read,
    /// which means the provider changed its payload and this driver has not
    /// caught up.
    fn settle(&self, body: &[u8], signature: &str) -> Result<Event, Error>;
}
