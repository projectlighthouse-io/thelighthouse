//! Registering the providers once at boot, and looking one up per request.
//!
//! Credentials are read from configuration in one place during startup, and a
//! handler asks for a driver by name — usually the name in a route parameter,
//! so an endpoint never sees an api key and an unknown provider is decided
//! once here rather than in every handler.
//!
//! ```text
//! // boot, after configuration and before the router
//! let billing = billing::providers([
//!     Stripe::register(config)?,
//! ])?;
//!
//! // per request
//! let driver = billing.driver(name).ok_or(NoSuchProvider)?;
//! ```

use std::sync::Arc;

use crate::{error::Error, gateway::Gateway};

/// One driver, under the name it answers to.
///
/// Built by [`Registration::new`], or by a shipped driver's own constructor,
/// and only useful as an argument to [`providers`].
///
/// The driver is behind an `Arc<dyn Gateway>` rather than a generic parameter,
/// and that is the deliberate trade. A generic registry would dispatch
/// statically, but it could hold only one implementation — and a registry of
/// one provider is a field, not a registry. A closed enum over the drivers
/// this crate ships would also dispatch statically, and would make adding a
/// provider a change to *this* crate, which is precisely what the trait exists
/// to avoid. What the indirection costs is one boxed future per call, next to
/// a network round trip to a payment provider.
#[derive(Clone, Debug)]
pub struct Registration {
    name: String,
    driver: Arc<dyn Gateway>,
}

impl Registration {
    /// Name a driver and make it registrable.
    ///
    /// The name is what [`Providers::driver`] is called with, and it ends up
    /// in urls and in stored rows recording which provider took the money.
    /// Something short, lowercase and stable: `stripe`, not `Stripe v2`.
    pub fn new(
        name: impl Into<String>,
        driver: impl Gateway + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            driver: Arc::new(driver),
        }
    }

    /// The name this driver answers to.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Every provider this process can take money through.
///
/// Cheap to clone — one `Arc` per driver, and the drivers themselves share a
/// connection pool — so it can go straight into request state.
#[derive(Clone, Debug)]
pub struct Providers {
    drivers: Vec<Registration>,
}

/// Build the registry. Call once, at boot.
///
/// Doing it here rather than per request means a bad credential is a startup
/// failure with a name attached, rather than a 500 the first time somebody
/// tries to pay with that provider.
///
/// # Errors
///
/// [`Error::Duplicate`] if a name is registered twice. The second registration
/// would otherwise be silently ignored and read, much later, as the first one
/// holding the wrong credentials.
pub fn providers(
    list: impl IntoIterator<Item = Registration>,
) -> Result<Providers, Error> {
    let mut drivers: Vec<Registration> = Vec::new();

    for registration in list {
        if drivers
            .iter()
            .any(|driver| driver.name == registration.name)
        {
            return Err(Error::Duplicate {
                provider: registration.name,
            });
        }

        drivers.push(registration);
    }

    Ok(Providers { drivers })
}

impl Providers {
    /// The driver registered under this name, if there is one.
    ///
    /// `None` is a 404 rather than a bad request: an unknown provider is not a
    /// malformed ask, it is a url that does not exist.
    #[must_use]
    pub fn driver(&self, name: &str) -> Option<&dyn Gateway> {
        self.drivers
            .iter()
            .find(|registration| registration.name == name)
            .map(|registration| registration.driver.as_ref())
    }

    /// Every registered name, in the order they were registered.
    ///
    /// For a startup log line, and for an endpoint that offers the reader a
    /// choice of how to pay.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.drivers.iter().map(Registration::name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        checkout::{Bought, Customer, Handoff, Returns},
        event::Event,
        plan::Plan,
        subscription::{Cancel, Subscription},
    };

    /// A provider defined outside this crate's `drivers` directory, which is
    /// the whole claim the registry makes: it takes drivers nobody here wrote.
    /// If this stops compiling, the registry has closed.
    #[derive(Debug)]
    struct Elsewhere {
        answer: &'static str,
    }

    #[async_trait::async_trait]
    impl Gateway for Elsewhere {
        async fn subscribe(
            &self,
            _to: &Plan,
            _who: &Customer<'_>,
            _back: &Returns,
        ) -> Result<Handoff, Error> {
            Ok(Handoff {
                url: self.answer.to_owned(),
            })
        }

        async fn purchase(
            &self,
            _what: &Plan,
            _who: &Customer<'_>,
            _back: &Returns,
        ) -> Result<Handoff, Error> {
            Ok(Handoff {
                url: self.answer.to_owned(),
            })
        }

        async fn cancel(
            &self,
            _subscription: &str,
            _when: Cancel,
        ) -> Result<Subscription, Error> {
            Ok(canned())
        }

        async fn resume(
            &self,
            _subscription: &str,
        ) -> Result<Subscription, Error> {
            Ok(canned())
        }

        async fn swap(
            &self,
            _subscription: &str,
            _to: &Plan,
        ) -> Result<Subscription, Error> {
            Ok(canned())
        }

        async fn bought(&self, _session: &str) -> Result<Bought, Error> {
            Ok(Bought {
                plan: None,
                reference: None,
                paid: false,
                subscription: None,
            })
        }

        fn signature_header(&self) -> &'static str {
            "x-elsewhere-signature"
        }

        fn settle(
            &self,
            _body: &[u8],
            _signature: &str,
        ) -> Result<Event, Error> {
            Ok(Event::Ignored)
        }
    }

    /// The three verbs this test does not exercise still have to return
    /// something; what they return is not the point being made.
    fn canned() -> Subscription {
        Subscription {
            reference: "sub_stub".to_owned(),
            account: None,
            plan: None,
            status: crate::subscription::Status::Canceled,
            period_ends_at: None,
            cancel_at: None,
        }
    }

    fn plan() -> Plan {
        Plan {
            id: "voyage_yearly".into(),
            price: "handle".to_owned(),
            interval: crate::plan::Interval::Year,
            money: None,
        }
    }

    #[tokio::test]
    async fn a_driver_written_elsewhere_registers_and_answers() {
        let registry = providers([Registration::new(
            "elsewhere",
            Elsewhere { answer: "paid" },
        )])
        .unwrap();

        let driver = registry.driver("elsewhere").unwrap();
        let handoff = driver
            .subscribe(
                &plan(),
                &Customer {
                    reference: "1",
                    email: "reader@example.com",
                    existing: None,
                },
                &Returns {
                    success: "https://example.com/paid".to_owned(),
                    cancel: "https://example.com/pricing".to_owned(),
                },
            )
            .await
            .unwrap();

        assert_eq!(handoff.url, "paid");
    }

    #[test]
    fn a_name_registered_twice_is_a_startup_failure() {
        let error = providers([
            Registration::new("elsewhere", Elsewhere { answer: "first" }),
            Registration::new("elsewhere", Elsewhere { answer: "second" }),
        ])
        .unwrap_err();

        assert!(matches!(
            error,
            Error::Duplicate { provider } if provider == "elsewhere"
        ));
    }

    #[test]
    fn an_unknown_name_is_nothing_rather_than_a_wrong_driver() {
        let registry = providers([Registration::new(
            "elsewhere",
            Elsewhere { answer: "" },
        )])
        .unwrap();

        assert!(registry.driver("stripe").is_none());
    }

    #[test]
    fn two_providers_both_answer_to_their_own_names() {
        let registry = providers([
            Registration::new("one", Elsewhere { answer: "first" }),
            Registration::new("two", Elsewhere { answer: "second" }),
        ])
        .unwrap();

        assert_eq!(registry.names().collect::<Vec<_>>(), ["one", "two"]);
        assert!(registry.driver("one").is_some());
        assert!(registry.driver("two").is_some());
    }
}
