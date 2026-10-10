//! The one place a reader's Stripe customer is minted.
//!
//! Called twice: in the background after every sign-in, so the id is usually
//! there before anybody pays, and inline at checkout, so it is certainly there
//! before the reader is sent to Stripe. The checkout call is the guarantee; the
//! sign-in call only saves that one a round trip.
//!
//! The id is stored the moment it is minted, not when a checkout completes.
//! Waiting for the webhook is what used to give one reader several customers:
//! every abandoned checkout minted one that nothing remembered.

use billing::{Customer, drivers::stripe};

use crate::api::AppState;

#[derive(Debug)]
pub(crate) enum Error {
    /// Stripe is not configured on this deployment.
    NoStripe,
    /// The session names a user that no longer exists.
    NoReader,
    Database(sqlx::Error),
    Provider(billing::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoStripe => write!(f, "stripe is not configured"),
            Self::NoReader => write!(f, "the reader does not exist"),
            Self::Database(error) => write!(f, "{error}"),
            // Shows the provider's code, never its message or our request.
            Self::Provider(error) => write!(f, "{error}"),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

/// The reader's Stripe customer id, minted and stored if they have none.
///
/// The row is locked for the whole of it, so a sign-in and a checkout racing
/// for the same reader mint one customer between them: the second waits, then
/// reads the id the first stored.
///
/// ponytail: holds a pooled connection across the Stripe call (seconds at
/// worst, with retries). Fine at this traffic; an advisory lock outside a
/// transaction would free the connection if the pool ever runs dry.
pub(crate) async fn ensure(
    state: &AppState,
    user_id: i64,
) -> Result<String, Error> {
    let driver = state
        .billing
        .providers
        .driver(stripe::NAME)
        .ok_or(Error::NoStripe)?;

    let mut tx = state.db.begin().await?;

    let row: Option<(Option<String>, String, String)> = sqlx::query_as(
        "SELECT stripe_id, email, name FROM users WHERE id = $1 FOR UPDATE",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some((existing, email, name)) = row else {
        return Err(Error::NoReader);
    };

    if let Some(existing) = existing {
        return Ok(existing);
    }

    let reference = user_id.to_string();
    let minted = driver
        .enroll(&Customer {
            reference: &reference,
            email: &email,
            name: Some(&name),
            existing: None,
        })
        .await
        .map_err(Error::Provider)?;

    sqlx::query("UPDATE users SET stripe_id = $2 WHERE id = $1")
        .bind(user_id)
        .bind(&minted)
        .execute(&mut *tx)
        .await?;

    if let Err(error) = tx.commit().await {
        // Minted at Stripe, lost here: the next call mints another.
        tracing::error!(user_id, %error, "minted a stripe customer but failed to store it");
        return Err(error.into());
    }

    tracing::info!(user_id, "stored a new stripe customer");

    Ok(minted)
}

/// [`ensure`], off the request path. For sign-in, which must never wait on or
/// fail because of Stripe.
pub(crate) fn ensure_in_background(state: &AppState, user_id: i64) {
    let state = state.clone();

    tokio::spawn(async move {
        match ensure(&state, user_id).await {
            Ok(_) | Err(Error::NoStripe) => {}
            Err(error) => {
                tracing::warn!(user_id, %error, "failed to ensure a stripe customer at sign-in");
            }
        }
    });
}
