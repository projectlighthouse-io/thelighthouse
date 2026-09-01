//! Subscription billing, behind an interface, with the provider swappable.
//!
//! Taking recurring money is the same five moves everywhere: send somebody to
//! a hosted page, stop it, start it again, move it to another plan, and listen
//! for what the provider says happened. Only the wire format differs. This
//! crate is those five moves as a trait — [`Gateway`] — plus the drivers that
//! speak them.
//!
//! ```text
//! // boot: once, where the rest of the configuration is read
//! let billing = billing::providers([
//!     StripeProvider::with(&secret_key, &webhook_secret)
//!         .strategy(RequestStrategy::ExponentialBackoff(3))
//!         .register()?,
//! ])?;
//!
//! // a reader wants to subscribe
//! let driver = billing.driver("stripe").ok_or(NoSuchProvider)?;
//! let handoff = driver.subscribe(plan, &Customer {
//!     reference: &user.id.to_string(),
//!     email: &user.email,
//!     existing: user.customer_id.as_deref(),
//! }).await?;
//! redirect(handoff.url);
//!
//! // and later, on the webhook endpoint
//! match driver.settle(&body, signature)? {
//!     Event::Started(subscription) => { /* write the row */ }
//!     Event::Ignored => { /* 200, and nothing else */ }
//!     // …
//! }
//! ```
//!
//! # What it does not do
//!
//! No database, no http server, no session, no money arithmetic. It does not
//! know who your users are, where you keep them, or what a plan costs — an
//! amount lives at the provider, and which plans exist is configuration read at
//! boot. Given an intent it talks to a provider, and given a webhook it says
//! what happened. Deciding what either means is the caller's, because that
//! decision is the part every application does differently.
//!
//! Also absent, because they can be added when something needs them rather
//! than carried by everyone who does not: invoices, hosted billing portals,
//! trials, saved payment methods, metered usage, quantities, and tax.
//!
//! # Extending it
//!
//! [`providers`] takes anything implementing [`Gateway`], so a provider that
//! does not ship here is a driver you write and register alongside the ones
//! that do. Nothing in this crate needs editing to accept it, which is the
//! reason the registry holds trait objects rather than a closed enum of the
//! providers somebody thought of first.
//!
//! # Layout
//!
//! Every module is private and its public items are re-exported here, so the
//! paths callers write stay flat — `billing::Gateway`, not
//! `billing::gateway::Gateway` — and moving something between modules is not a
//! breaking change.
//!
//! - `gateway` — the trait, and nothing else.
//! - `plan` — what is being sold, and how often it is charged for.
//! - `checkout` — who is paying, and where to send them.
//! - `subscription` — a subscription as the provider currently sees it.
//! - `registry` — registering the drivers at boot, and looking one up.
//! - `request` — how hard to try, and how not to charge twice.
//! - `event` — what a provider told us happened.
//! - `error` — the one error type that crosses the boundary.

mod checkout;
mod error;
mod event;
mod gateway;
mod plan;
mod registry;
mod request;
mod subscription;

pub use checkout::{Customer, Handoff};
pub use error::Error;
pub use event::Event;
pub use gateway::Gateway;
pub use plan::{Interval, Plan, PlanId, Plans};
pub use registry::{Providers, Registration, providers};
pub use request::{Attempts, IdempotencyKey, RequestStrategy};
pub use subscription::{Cancel, Status, Subscription};
