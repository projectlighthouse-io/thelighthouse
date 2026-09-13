//! `lighthouse-subscriber` — what one reader has bought.
//!
//! ```text
//!   lighthouse-subscriber --email someone@example.com
//!   lighthouse-subscriber --id 1
//!
//!   lighthouse-subscriber revoke --email someone@example.com --now
//!   lighthouse-subscriber revoke --id 1 --date 2026-12-31T23:59:59Z
//!   lighthouse-subscriber revoke --id 1 --now --dry-run
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

/// What was asked for.
#[derive(Debug)]
enum Command {
    /// Report and change nothing.
    Look(Who),
    /// End the membership, at a moment.
    Revoke {
        who: Who,
        /// When it ends. `--now` is this, filled in with the current time —
        /// there is one notion of "when does it end" and one path that
        /// handles it.
        at: chrono::DateTime<chrono::Utc>,
        /// Say what would happen and touch nothing.
        dry_run: bool,
    },
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

/// The whole command line.
///
/// `now` is passed in rather than read here so the parse is testable: `--now`
/// resolves to a moment, and a test that cannot choose the moment cannot check
/// that it did.
fn parse(
    args: &[String],
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Command, String> {
    let Some((first, rest)) = args.split_first() else {
        return Err(USAGE.to_owned());
    };

    if first != "revoke" {
        return who(args).map(Command::Look);
    }

    let mut named: Vec<String> = Vec::new();
    let mut at: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut dry_run = false;
    let mut rest = rest.iter();

    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--now" => {
                if at.is_some() {
                    return Err(
                        "--now and --date say the same thing twice".to_owned()
                    );
                }

                at = Some(now);
            }
            "--date" => {
                if at.is_some() {
                    return Err(
                        "--now and --date say the same thing twice".to_owned()
                    );
                }

                let value = rest.next().ok_or("--date needs a moment")?;
                at = Some(moment(value)?);
            }
            "--dry-run" => dry_run = true,
            _ => named.push(arg.clone()),
        }
    }

    let who = who(&named)?;
    let at = at.ok_or("revoke needs --date <rfc3339> or --now")?;

    Ok(Command::Revoke { who, at, dry_run })
}

/// An rfc3339 moment, which is the only shape accepted.
///
/// A bare date would have to invent a time and a zone, and both choices are
/// wrong for somebody: "the 31st" means one instant in Dhaka and another in
/// London, and the difference is a day of access. Making the caller say it
/// means the answer is never guessed.
fn moment(value: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|when| when.with_timezone(&chrono::Utc))
        .map_err(|_| {
            format!(
                "{value} is not an rfc3339 moment — try 2026-12-31T23:59:59Z"
            )
        })
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let now = chrono::Utc::now();
    let command = parse(&args, now)?;

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

    let report = match &command {
        Command::Look(who) => look(&db, who).await?,
        Command::Revoke { who, at, dry_run } => {
            revoke(&db, who, *at, *dry_run, now).await?;

            // The state afterwards, read back rather than assembled from what
            // was just written. A revoke that reported its own intentions
            // would agree with itself whatever the database did.
            look(&db, who).await?
        }
    };

    // Pretty, because a person reads this. Piping it to `jq` still works.
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("cannot render the answer: {error}"))?;

    println!("{json}");

    Ok(())
}

/// End a membership, at a moment.
///
/// **Stripe first, then the row.** If stripe refuses, nothing here has changed
/// and the two still agree. The other order leaves a row saying cancelled and a
/// subscription that goes on charging — which is the version somebody finds out
/// about from a bank statement.
///
/// **The row is never deleted.** `ended` is a status the schema has for exactly
/// this, and history is what makes "why did this reader lose access" answerable
/// later.
async fn revoke(
    db: &PgPool,
    who: &Who,
    at: chrono::DateTime<chrono::Utc>,
    dry_run: bool,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), String> {
    let report = look(db, who).await?;

    let membership = report.subscription.as_ref().ok_or_else(|| {
        format!("{} holds no membership to revoke", report.user.email)
    })?;

    // Now or in the past ends it here; a future moment is handed to stripe to
    // end itself. One comparison decides, so `--now` really is `--date` with
    // the current time in it.
    let immediate = at <= now;

    if dry_run {
        println!(
            "would {} membership {} ({}) for {}{}",
            if immediate {
                "end"
            } else {
                "schedule the end of"
            },
            membership.id,
            membership.plan,
            report.user.email,
            if immediate {
                String::new()
            } else {
                format!(" at {}", at.to_rfc3339())
            }
        );

        return Ok(());
    }

    // Only when the provider is one. A manual scholarship row has no reference
    // and nothing to tell anybody about — ending it is this row and no more.
    if let Some(reference) = membership.provider_ref.as_deref() {
        at_provider(membership, reference, at, immediate).await?;
    }

    let naive = at.naive_utc();

    sqlx::query(
        "UPDATE memberships \
         SET status = CASE WHEN $2 THEN $3 ELSE status END, \
             cancel_at = $4, \
             ended_at = CASE WHEN $2 THEN $4 ELSE ended_at END \
         WHERE id = $1",
    )
    .bind(membership.id)
    .bind(immediate)
    .bind(ENDED)
    .bind(naive)
    .execute(db)
    .await
    .map_err(|error| format!("cannot record the revoke: {error}"))?;

    Ok(())
}

/// Tell the provider, using the same driver the api uses.
async fn at_provider(
    membership: &Subscription,
    reference: &str,
    at: chrono::DateTime<chrono::Utc>,
    immediate: bool,
) -> Result<(), String> {
    if membership.provider != "stripe" {
        return Err(format!(
            "membership {} is on {}, which this command cannot end — \
             end it there, then run again once the row is the only thing left",
            membership.id, membership.provider
        ));
    }

    let key = std::env::var("STRIPE_SECRET_KEY").map_err(|_| {
        "STRIPE_SECRET_KEY is not set, and ending a stripe subscription \
         needs it"
            .to_owned()
    })?;

    let stripe = billing::Stripe::new(billing::StripeConfig {
        secret_key: key.into(),
        // Never used: nothing here verifies a webhook. Required by the config,
        // so it is named rather than left looking like an oversight.
        webhook_secret: String::new().into(),
        // Three attempts, backing off, matching the api. A revoke that failed
        // because stripe hiccuped is one somebody has to notice and redo.
        strategy: billing::RequestStrategy::ExponentialBackoff(3),
    })
    .map_err(|error| format!("cannot reach stripe: {error}"))?;

    let when = if immediate {
        billing::Cancel::Now
    } else {
        billing::Cancel::At(at)
    };

    billing::Gateway::cancel(&stripe, reference, when)
        .await
        .map_err(|error| format!("stripe refused: {error}"))?;

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

    fn now() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-09-13T12:00:00Z")
            .expect("a test moment")
            .with_timezone(&chrono::Utc)
    }

    #[test]
    fn no_subcommand_is_a_read() {
        assert!(matches!(
            parse(&args(&["--email", "a@b.c"]), now()),
            Ok(Command::Look(Who::Email(email))) if email == "a@b.c"
        ));
    }

    #[test]
    fn now_is_the_current_moment_rather_than_a_case_of_its_own() {
        // The whole point: one notion of when it ends, and one path handling
        // it. `--now` fills the same field `--date` does.
        let parsed = parse(&args(&["revoke", "--id", "1", "--now"]), now());

        assert!(matches!(
            parsed,
            Ok(Command::Revoke { at, dry_run: false, .. }) if at == now()
        ));
    }

    #[test]
    fn a_date_is_read_as_the_moment_it_names() {
        let parsed = parse(
            &args(&["revoke", "--id", "1", "--date", "2026-12-31T23:59:59Z"]),
            now(),
        );

        assert!(matches!(
            parsed,
            Ok(Command::Revoke { at, .. })
                if at.to_rfc3339() == "2026-12-31T23:59:59+00:00"
        ));
    }

    #[test]
    fn an_offset_is_honoured_rather_than_dropped() {
        // Midnight in Dhaka is not midnight in London, and the difference is a
        // day of access. The zone the caller wrote is the zone that counts.
        let parsed = parse(
            &args(&[
                "revoke",
                "--id",
                "1",
                "--date",
                "2027-01-01T00:00:00+06:00",
            ]),
            now(),
        );

        assert!(matches!(
            parsed,
            Ok(Command::Revoke { at, .. })
                if at.to_rfc3339() == "2026-12-31T18:00:00+00:00"
        ));
    }

    #[test]
    fn revoking_without_saying_when_is_refused() {
        let refused =
            parse(&args(&["revoke", "--id", "1"]), now()).unwrap_err();

        assert!(refused.contains("--date"), "{refused}");
    }

    #[test]
    fn saying_when_twice_is_refused() {
        // Not resolved by taking the last one: two answers means the caller
        // believes something this cannot confirm.
        for wrong in [
            args(&[
                "revoke",
                "--id",
                "1",
                "--now",
                "--date",
                "2027-01-01T00:00:00Z",
            ]),
            args(&[
                "revoke",
                "--id",
                "1",
                "--date",
                "2027-01-01T00:00:00Z",
                "--now",
            ]),
        ] {
            assert!(parse(&wrong, now()).is_err(), "{wrong:?} was accepted");
        }
    }

    #[test]
    fn a_date_that_is_not_a_moment_names_itself() {
        let refused =
            parse(&args(&["revoke", "--id", "1", "--date", "tuesday"]), now())
                .unwrap_err();

        assert!(refused.contains("tuesday"), "{refused}");
    }

    #[test]
    fn a_bare_date_is_refused_rather_than_given_a_time() {
        // It would have to invent both a time and a zone, and every choice is
        // wrong for somebody.
        assert!(
            parse(
                &args(&["revoke", "--id", "1", "--date", "2026-12-31"]),
                now()
            )
            .is_err()
        );
    }

    #[test]
    fn dry_run_is_carried_through_wherever_it_is_written() {
        for order in [
            args(&["revoke", "--dry-run", "--id", "1", "--now"]),
            args(&["revoke", "--id", "1", "--now", "--dry-run"]),
        ] {
            assert!(
                matches!(
                    parse(&order, now()),
                    Ok(Command::Revoke { dry_run: true, .. })
                ),
                "{order:?} lost --dry-run"
            );
        }
    }

    #[test]
    fn revoking_nobody_is_refused() {
        assert!(parse(&args(&["revoke", "--now"]), now()).is_err());
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
