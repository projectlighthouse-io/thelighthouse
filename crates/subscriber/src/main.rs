//! `lighthouse-subscriber` — what one reader has bought.
//!
//! ```text
//!   lighthouse-subscriber --email someone@example.com
//!   lighthouse-subscriber --id 1
//! ```
//!
//! Answers json on stdout: the user, the subscription they currently hold, and
//! the books they have been granted outright.
//!
//! A separate binary for the reason `lighthouse-content` and
//! `lighthouse-migrate` are: reading is harmless, but this is where revoking
//! will live, and writing to the database is a decision rather than a side
//! effect of starting a server.
//!
//! # Why it does not ask the api
//!
//! `/api/billing/access` answers a similar question, but only ever about
//! *whoever is holding the session cookie*. There is no way to ask it about
//! somebody else, which is exactly what is wanted when a reader writes in. So
//! this reads the rows directly and reports them as they are — no entitlement
//! rules applied, because the question it answers is "what is actually
//! recorded", not "what would the api conclude".

use std::process::ExitCode;

use serde::Serialize;
use sqlx::{FromRow, postgres::PgPool};

const USAGE: &str =
    "usage: lighthouse-subscriber (--email <email> | --id <id>)";

/// Paid up. The only status that grants anything — the api's rule, named here
/// so the json can say `active` rather than `0`.
const ACTIVE: i16 = 0;
const GRACE: i16 = 1;
const ENDED: i16 = 2;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// How the reader was named on the command line.
#[derive(Debug)]
enum Who {
    Email(String),
    Id(i64),
}

/// Parsed before anything is opened, as the other binaries do: a typo in the
/// flags should say so without a connection having been made.
fn who(args: &[String]) -> Result<Who, String> {
    match args {
        [flag, value] if flag == "--email" => Ok(Who::Email(value.clone())),
        [flag, value] if flag == "--id" => value
            .parse()
            .map(Who::Id)
            .map_err(|_| format!("{value} is not a user id")),
        _ => Err(USAGE.to_owned()),
    }
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let who = who(&args)?;

    // Same rule as everywhere else in this workspace: a missing .env is fine —
    // the runtime image ships without one — and an unreadable one is not.
    if let Err(error) = dotenvy::dotenv()
        && !error.not_found()
    {
        return Err(format!("cannot read .env: {error}"));
    }

    let url = std::env::var("DATABASE_URL").map_err(|_| {
        "DATABASE_URL is not set in .env or the environment".to_owned()
    })?;

    // One connection: this reads a handful of rows and should not hold more of
    // the database open than it is using.
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|error| {
            format!("cannot connect to {}: {error}", redacted(&url))
        })?;

    let report = look(&db, &who).await?;

    // Pretty, because a person reads this. Piping it to `jq` still works.
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("cannot render the answer: {error}"))?;

    println!("{json}");

    Ok(())
}

/// Everything recorded about one reader's buying.
#[derive(Debug, Serialize)]
struct Report {
    user: User,
    /// The membership that is not over, if there is one. At most one exists —
    /// a partial unique index enforces it — so this is an option, not a list.
    subscription: Option<Subscription>,
    /// Memberships that have ended, newest first. History rather than access:
    /// a reader who cancelled and resubscribed has both, and which one ended
    /// when is usually the question.
    ended: Vec<Subscription>,
    /// Books granted outright, which a subscription does not leave behind.
    /// Empty here and a live subscription above is the ordinary shape, and the
    /// one that confuses people.
    entitlements: Vec<Entitlement>,
}

#[derive(Debug, FromRow, Serialize)]
struct User {
    id: i64,
    email: String,
    /// Stripe's customer id, inherited from the laravel schema. Null until the
    /// reader has been sent to checkout at least once.
    stripe_id: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct Subscription {
    id: i64,
    /// Our name for what was bought, never the provider's price handle. The
    /// track is everything before the last underscore.
    plan: String,
    /// The column, which is a number. Read but never reported: `0` is not an
    /// answer anybody can act on.
    #[sqlx(rename = "status")]
    #[serde(skip)]
    code: i16,
    /// The same thing in words, filled in after the read.
    #[sqlx(skip)]
    status: &'static str,
    /// Whether this row grants what it was bought for, right now.
    ///
    /// The api's rule, reproduced rather than imported — `Membership` is
    /// `pub(crate)` to the api and this crate does not link it. The tests
    /// beside `grants_access_at` are the ones that pin the rule down; this is
    /// a report of it, and a disagreement between the two is itself worth
    /// seeing in the output.
    #[sqlx(skip)]
    grants_access: bool,
    /// Why, in words. A `false` with no reason is the thing that sends
    /// somebody back to the database by hand.
    #[sqlx(skip)]
    because: &'static str,
    /// Which provider took the money. `manual` for a membership nobody paid
    /// for, which is a row here and never a payment.
    provider: String,
    /// The provider's own id for it, and what a revoke has to name at stripe.
    provider_ref: Option<String>,
    started_at: Option<chrono::NaiveDateTime>,
    /// The end of the period already paid for. What a reader keeps after
    /// cancelling, which is why cancelling is not ending.
    period_ends_at: Option<chrono::NaiveDateTime>,
    /// When a pending cancellation takes effect. Set means cancelled but not
    /// yet over.
    cancel_at: Option<chrono::NaiveDateTime>,
    ended_at: Option<chrono::NaiveDateTime>,
    /// Bought outright: no end, and none expected. The only thing that makes a
    /// null `period_ends_at` mean forever rather than "we were not told".
    lifetime: bool,
}

#[derive(Debug, FromRow, Serialize)]
struct Entitlement {
    book_id: uuid::Uuid,
    source: String,
    granted_at: Option<chrono::NaiveDateTime>,
    expires_at: Option<chrono::NaiveDateTime>,
}

/// The api's names for the status column.
const fn name_of(status: i16) -> &'static str {
    match status {
        ACTIVE => "active",
        GRACE => "grace",
        ENDED => "ended",
        _ => "unknown",
    }
}

/// Whether a row grants access, and the reason to print beside it.
///
/// Mirrors `Membership::grants_access_at` in the api: status first, then
/// `lifetime`, then the end date. Null `period_ends_at` grants nothing — it
/// means "we were not told", not "forever".
fn verdict(
    membership: &Subscription,
    now: chrono::NaiveDateTime,
) -> (bool, &'static str) {
    if membership.code != ACTIVE {
        return (false, "the status is not active");
    }

    if membership.lifetime {
        return (true, "bought outright, so it does not end");
    }

    match membership.period_ends_at {
        Some(ends) if ends > now => (true, "the paid period has not ended"),
        Some(_) => (false, "the paid period has ended"),
        None => (false, "there is no end date and it is not marked lifetime"),
    }
}

const COLUMNS: &str = "
    id,
    plan,
    status,
    provider,
    provider_ref,
    started_at,
    period_ends_at,
    cancel_at,
    ended_at,
    lifetime
";

async fn look(db: &PgPool, who: &Who) -> Result<Report, String> {
    let found: Option<User> = match who {
        Who::Email(email) => sqlx::query_as(
            "SELECT id, email, stripe_id FROM users WHERE email = $1",
        )
        .bind(email),
        Who::Id(id) => sqlx::query_as(
            "SELECT id, email, stripe_id FROM users WHERE id = $1",
        )
        .bind(id),
    }
    .fetch_optional(db)
    .await
    .map_err(|error| format!("cannot read the user: {error}"))?;

    // Named, so "no such reader" cannot be mistaken for "reader with nothing".
    let user = found.ok_or_else(|| match who {
        Who::Email(email) => format!("no user with the email {email}"),
        Who::Id(id) => format!("no user with the id {id}"),
    })?;

    let mut memberships: Vec<Subscription> = sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM memberships \
         WHERE user_id = $1 \
         ORDER BY started_at DESC NULLS LAST, id DESC"
    ))
    .bind(user.id)
    .fetch_all(db)
    .await
    .map_err(|error| format!("cannot read memberships: {error}"))?;

    let now = chrono::Utc::now().naive_utc();

    for membership in &mut memberships {
        membership.status = name_of(membership.code);

        let (grants, because) = verdict(membership, now);
        membership.grants_access = grants;
        membership.because = because;
    }

    // Split rather than queried twice: one pass over a handful of rows, and
    // the two halves cannot disagree about what "ended" means.
    let (ended, live): (Vec<_>, Vec<_>) =
        memberships.into_iter().partition(|m| m.code == ENDED);

    let entitlements: Vec<Entitlement> = sqlx::query_as(
        "SELECT book_id, source, granted_at, expires_at FROM entitlements \
         WHERE user_id = $1 ORDER BY granted_at DESC NULLS LAST",
    )
    .bind(user.id)
    .fetch_all(db)
    .await
    .map_err(|error| format!("cannot read entitlements: {error}"))?;

    Ok(Report {
        user,
        subscription: live.into_iter().next(),
        ended,
        entitlements,
    })
}

/// A connection string with the password taken out, for an error message.
fn redacted(url: &str) -> String {
    match (url.find("://"), url.rfind('@')) {
        (Some(scheme), Some(at)) if at > scheme => {
            format!("{}://***@{}", &url[..scheme], &url[at + 1..])
        }
        _ => url.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(of: &[&str]) -> Vec<String> {
        of.iter().map(|a| (*a).to_owned()).collect()
    }

    #[test]
    fn a_reader_is_named_by_email_or_by_id() {
        assert!(matches!(
            who(&args(&["--email", "a@b.c"])),
            Ok(Who::Email(email)) if email == "a@b.c"
        ));
        assert!(matches!(who(&args(&["--id", "7"])), Ok(Who::Id(7))));
    }

    #[test]
    fn an_id_that_is_not_a_number_says_so_rather_than_meaning_zero() {
        let refused = who(&args(&["--id", "one"])).unwrap_err();

        assert!(refused.contains("one"), "{refused}");
    }

    #[test]
    fn naming_nobody_or_everybody_is_refused() {
        for wrong in [
            vec![],
            args(&["--email"]),
            args(&["--email", "a", "--id", "1"]),
        ] {
            assert!(who(&wrong).is_err(), "{wrong:?} was accepted");
        }
    }

    #[test]
    fn a_status_reads_as_a_word() {
        assert_eq!(name_of(ACTIVE), "active");
        assert_eq!(name_of(GRACE), "grace");
        assert_eq!(name_of(ENDED), "ended");
        // A status the schema does not define yet must not read as one it does.
        assert_eq!(name_of(9), "unknown");
    }

    fn row(
        code: i16,
        period_ends_at: Option<&str>,
        lifetime: bool,
    ) -> Subscription {
        Subscription {
            id: 1,
            plan: "go_yearly".to_owned(),
            code,
            status: "",
            grants_access: false,
            because: "",
            provider: "stripe".to_owned(),
            provider_ref: None,
            started_at: None,
            period_ends_at: period_ends_at.map(when),
            cancel_at: None,
            ended_at: None,
            lifetime,
        }
    }

    fn when(text: &str) -> chrono::NaiveDateTime {
        chrono::NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S")
            .expect("a test timestamp")
    }

    const NOW: &str = "2026-09-13 12:00:00";

    #[test]
    fn the_verdict_matches_the_rule_the_api_applies() {
        let now = when(NOW);

        assert!(
            verdict(&row(ACTIVE, Some("2027-01-01 00:00:00"), false), now).0
        );
        assert!(verdict(&row(ACTIVE, None, true), now).0);

        // Lapsed, undated, and not active. None of these grant.
        assert!(
            !verdict(&row(ACTIVE, Some("2026-01-01 00:00:00"), false), now).0
        );
        assert!(!verdict(&row(ACTIVE, None, false), now).0);
        assert!(
            !verdict(&row(GRACE, Some("2027-01-01 00:00:00"), false), now).0
        );
        assert!(!verdict(&row(ENDED, None, true), now).0);
    }

    #[test]
    fn every_verdict_says_why() {
        let now = when(NOW);

        for held in [
            row(ACTIVE, Some("2027-01-01 00:00:00"), false),
            row(ACTIVE, None, true),
            row(ACTIVE, Some("2026-01-01 00:00:00"), false),
            row(ACTIVE, None, false),
            row(GRACE, None, false),
        ] {
            assert!(
                !verdict(&held, now).1.is_empty(),
                "{held:?} gave no reason"
            );
        }
    }

    #[test]
    fn a_connection_string_loses_its_password() {
        let hidden =
            redacted("postgres://user:secret@127.0.0.1:5433/lighthouse");

        assert!(!hidden.contains("secret"), "{hidden}");
        assert!(hidden.contains("127.0.0.1:5433/lighthouse"), "{hidden}");
    }
}
