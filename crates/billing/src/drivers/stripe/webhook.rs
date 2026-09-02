//! Verifying a delivery, and reading what it says.
//!
//! Stripe signs `"{timestamp}.{body}"` with the endpoint's signing secret and
//! sends the result as `Stripe-Signature`:
//!
//! ```text
//! t=1492774577,v1=5257a869e7ecebeda32affa62cdca3fa51cad7e77a0e56ff536d0ce8e10…
//! ```
//!
//! There may be several `v1` values during a secret rotation, and other
//! schemes (`v0`) that are not ours to check. Any one `v1` matching is a valid
//! delivery.

use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sha2::Sha256;
use subtle::ConstantTimeEq;

use super::wire;
use crate::{error::Error, event::Event};

type HmacSha256 = Hmac<Sha256>;

/// How far out of step a delivery's timestamp may be, in seconds.
///
/// Stripe's own default, and the same window the rest of this platform uses.
/// It is what stops a captured delivery being worth replaying tomorrow: the
/// signature stays valid forever, so the timestamp is the only thing bounding
/// it.
const MAX_SKEW: i64 = 300;

/// Check the signature, then say what the delivery means.
pub(crate) fn settle(
    secret: &SecretString,
    body: &[u8],
    signature: &str,
) -> Result<Event, Error> {
    verify(secret, body, signature, now()?)?;
    read(body)
}

fn now() -> Result<i64, Error> {
    let elapsed =
        SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
            Error::Malformed {
                what: "the system clock",
                cause: "it is set before the unix epoch".to_owned(),
            }
        })?;

    i64::try_from(elapsed.as_secs()).map_err(|_| Error::Malformed {
        what: "the system clock",
        cause: "it is too far in the future to represent".to_owned(),
    })
}

/// Whether this body was signed with this secret, recently.
fn verify(
    secret: &SecretString,
    body: &[u8],
    signature: &str,
    now: i64,
) -> Result<(), Error> {
    let (timestamp, candidates) = parse(signature)?;

    if (now - timestamp).abs() > MAX_SKEW {
        return Err(Error::Signature);
    }

    let mut mac = HmacSha256::new_from_slice(secret.expose_secret().as_bytes())
        .map_err(|_| Error::Signature)?;
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(body);
    let expected = mac.finalize().into_bytes();

    let matched = candidates.into_iter().any(|candidate| {
        decode(candidate)
            .is_some_and(|digest| digest.ct_eq(expected.as_slice()).into())
    });

    if matched {
        Ok(())
    } else {
        Err(Error::Signature)
    }
}

/// Pull the timestamp and every `v1` digest out of the header.
fn parse(signature: &str) -> Result<(i64, Vec<&str>), Error> {
    let mut timestamp = None;
    let mut candidates = Vec::new();

    for part in signature.split(',') {
        match part.trim().split_once('=') {
            Some(("t", value)) => timestamp = value.parse::<i64>().ok(),
            // More than one during a secret rotation, when Stripe signs with
            // the old secret and the new one at once.
            Some(("v1", value)) => candidates.push(value),
            _ => {}
        }
    }

    match timestamp {
        Some(timestamp) if !candidates.is_empty() => {
            Ok((timestamp, candidates))
        }
        _ => Err(Error::Signature),
    }
}

/// Hex to bytes. A digest that is not hex simply does not match.
fn decode(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }

    hex.as_bytes()
        .chunks(2)
        .map(|pair| {
            std::str::from_utf8(pair)
                .ok()
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
        })
        .collect()
}

#[derive(Debug, Deserialize)]
struct Delivery {
    #[serde(rename = "type")]
    kind: String,
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    object: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct Invoice {
    customer: Option<String>,
}

/// What a verified delivery means, in this crate's vocabulary.
///
/// Anything not listed is [`Event::Ignored`]. Stripe sends dozens of event
/// types and answering an unhandled one with a failure earns retries of it
/// every hour until the endpoint is disabled.
fn read(body: &[u8]) -> Result<Event, Error> {
    let malformed = |cause: String| Error::Malformed {
        what: "a stripe webhook",
        cause,
    };

    let delivery: Delivery = serde_json::from_slice(body)
        .map_err(|source| malformed(source.to_string()))?;

    let object = delivery.data.object;

    let subscription = |object: serde_json::Value| {
        serde_json::from_value::<wire::Sub>(object)
            .map_err(|source| malformed(source.to_string()))
    };

    match delivery.kind.as_str() {
        "checkout.session.completed" => {
            let session: wire::Session = serde_json::from_value(object)
                .map_err(|source| malformed(source.to_string()))?;

            // Both are set by this driver at checkout. A session missing
            // either was created by something else, and there is no account to
            // attach it to.
            match (session.customer, session.client_reference_id) {
                (Some(customer), Some(reference)) => {
                    Ok(Event::CheckoutCompleted {
                        customer,
                        reference,
                        subscription: session.subscription,
                    })
                }
                _ => Ok(Event::Ignored),
            }
        }
        "customer.subscription.created" => {
            Ok(Event::Started(subscription(object)?.into()))
        }
        "customer.subscription.updated" => {
            Ok(Event::Changed(subscription(object)?.into()))
        }
        "customer.subscription.deleted" => {
            Ok(Event::Ended(subscription(object)?.into()))
        }
        "invoice.payment_failed" => {
            let invoice: Invoice = serde_json::from_value(object)
                .map_err(|source| malformed(source.to_string()))?;

            Ok(invoice.customer.map_or(Event::Ignored, |customer| {
                Event::PaymentFailed { customer }
            }))
        }
        _ => Ok(Event::Ignored),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subscription::Status;

    fn secret(key: &str) -> SecretString {
        key.into()
    }

    fn sign(secret: &str, body: &[u8], timestamp: i64) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(body);

        let digest = mac.finalize().into_bytes();
        let hex = digest.iter().fold(String::new(), |mut hex, byte| {
            use std::fmt::Write as _;
            let _ = write!(hex, "{byte:02x}");
            hex
        });

        format!("t={timestamp},v1={hex}")
    }

    #[test]
    fn a_delivery_signed_with_the_secret_verifies() {
        let body = br#"{"type":"ping","data":{"object":{}}}"#;

        assert!(
            verify(
                &secret("whsec_test"),
                body,
                &sign("whsec_test", body, 1000),
                1000
            )
            .is_ok()
        );
    }

    #[test]
    fn a_delivery_signed_with_another_secret_does_not() {
        let body = br#"{"type":"ping","data":{"object":{}}}"#;
        let forged = sign("whsec_someone_else", body, 1000);

        assert!(matches!(
            verify(&secret("whsec_test"), body, &forged, 1000),
            Err(Error::Signature)
        ));
    }

    #[test]
    fn a_body_changed_after_signing_does_not_verify() {
        let body = br#"{"type":"ping","data":{"object":{}}}"#;
        let signature = sign("whsec_test", body, 1000);
        let tampered = br#"{"type":"pong","data":{"object":{}}}"#;

        assert!(matches!(
            verify(&secret("whsec_test"), tampered, &signature, 1000),
            Err(Error::Signature)
        ));
    }

    #[test]
    fn a_captured_delivery_stops_being_worth_replaying() {
        let body = br#"{"type":"ping","data":{"object":{}}}"#;
        let signature = sign("whsec_test", body, 1000);

        // Inside the window, either side of it.
        assert!(
            verify(&secret("whsec_test"), body, &signature, 1000 + MAX_SKEW)
                .is_ok()
        );
        assert!(
            verify(&secret("whsec_test"), body, &signature, 1000 - MAX_SKEW)
                .is_ok()
        );

        // Outside it. The signature is still perfectly valid, which is the
        // point: only the clock stops this being replayable forever.
        assert!(
            verify(
                &secret("whsec_test"),
                body,
                &signature,
                1000 + MAX_SKEW + 1
            )
            .is_err()
        );
    }

    #[test]
    fn one_matching_digest_is_enough_during_a_rotation() {
        let body = br#"{"type":"ping","data":{"object":{}}}"#;
        let ours = sign("whsec_test", body, 1000);
        let theirs = sign("whsec_old", body, 1000);

        // Stripe signs with both secrets while one is being rotated out, and
        // sends both digests under `v1`.
        let (_, digest) = theirs.split_once("v1=").unwrap();
        let both = format!("{ours},v1={digest}");

        assert!(verify(&secret("whsec_test"), body, &both, 1000).is_ok());
    }

    #[test]
    fn a_header_that_is_not_a_signature_is_refused() {
        let body = b"{}";

        for header in ["", "t=1000", "v1=abcd", "nonsense", "t=soon,v1=abcd"] {
            assert!(
                verify(&secret("whsec_test"), body, header, 1000).is_err(),
                "{header} should not verify"
            );
        }
    }

    #[test]
    fn a_finished_checkout_carries_the_account_it_belongs_to() {
        let body = br#"{
            "type": "checkout.session.completed",
            "data": { "object": {
                "customer": "cus_1",
                "client_reference_id": "41",
                "subscription": "sub_1",
                "url": null
            }}
        }"#;

        assert_eq!(
            read(body).unwrap(),
            Event::CheckoutCompleted {
                customer: "cus_1".to_owned(),
                reference: "41".to_owned(),
                subscription: Some("sub_1".to_owned()),
            }
        );
    }

    #[test]
    fn a_checkout_this_driver_did_not_start_is_ignored() {
        // No client_reference_id: nothing here can say whose it is, and
        // guessing would attach somebody else's payment to an account.
        let body = br#"{
            "type": "checkout.session.completed",
            "data": { "object": { "customer": "cus_1" } }
        }"#;

        assert_eq!(read(body).unwrap(), Event::Ignored);
    }

    #[test]
    fn the_three_subscription_events_become_three_different_things() {
        let object = r#"{
            "id": "sub_1",
            "status": "active",
            "items": { "data": [{ "id": "si_1", "current_period_end": 1682288167 }] },
            "metadata": { "plan": "yearly" }
        }"#;

        let delivery = |kind: &str| {
            read(
                format!(r#"{{"type":"{kind}","data":{{"object":{object}}}}}"#)
                    .as_bytes(),
            )
            .unwrap()
        };

        assert!(matches!(
            delivery("customer.subscription.created"),
            Event::Started(_)
        ));
        assert!(matches!(
            delivery("customer.subscription.updated"),
            Event::Changed(_)
        ));

        let Event::Ended(ended) = delivery("customer.subscription.deleted")
        else {
            panic!("a deletion should end the subscription");
        };
        assert_eq!(ended.reference, "sub_1");
        assert_eq!(ended.status, Status::Active);
    }

    #[test]
    fn a_failed_payment_names_the_customer_and_changes_nothing_else() {
        let body = br#"{
            "type": "invoice.payment_failed",
            "data": { "object": { "customer": "cus_1" } }
        }"#;

        assert_eq!(
            read(body).unwrap(),
            Event::PaymentFailed {
                customer: "cus_1".to_owned()
            }
        );
    }

    #[test]
    fn an_event_this_driver_has_no_opinion_on_is_not_an_error() {
        // Answering 4xx here would make Stripe retry it hourly until the
        // endpoint is disabled for failing too often.
        let body = br#"{"type":"charge.dispute.funds_withdrawn","data":{"object":{}}}"#;

        assert_eq!(read(body).unwrap(), Event::Ignored);
    }
}
