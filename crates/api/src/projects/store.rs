//! Every project query, and the only place here that talks to postgres.
//!
//! A whole `query…bind` call lives here, never a SQL string handed elsewhere to
//! be filled in: `$1` and the value that fills it stay on screen together.
//!
//! Nothing here knows about HTTP. These return rows or `sqlx::Error`, and
//! `super::handler` decides what a caller is told.
//!
//! **Every statement binds the reader's id.** There is no parameter anywhere
//! below that could name somebody else's attempts, and no code path that reads
//! a `user_id` off the request body.

use std::collections::HashMap;

use chrono::NaiveDateTime;
use sqlx::postgres::PgPool;
use uuid::Uuid;

use super::progress::{Outcome, Progress};

/// Which run through a project the reader is on.
///
/// One more than the number of times they have restarted it, so a reader who
/// has never restarted is on run 1. **Written by the api, never by the
/// client** — a client that picks its own run number can rewrite history.
///
/// # Errors
///
/// The count.
pub(crate) async fn current_run(
    db: &PgPool,
    user_id: i64,
    project_id: Uuid,
) -> Result<i16, sqlx::Error> {
    let (restarts,) = sqlx::query_as::<_, (i64,)>(
        "SELECT count(*) FROM project_restarts
          WHERE user_id = $1 AND project_id = $2",
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_one(db)
    .await?;

    Ok(run_from(restarts))
}

/// Every task of one project that this reader has touched, keyed by task id.
///
/// A task with no attempts is simply absent — the caller fills in
/// [`Progress::default`], which is "challenge awaits". Returning a row of
/// zeroes for all 162 tasks of a project nobody has started would be the same
/// answer at more cost.
///
/// Two statements rather than one: attempts and hint unlocks are different
/// tables joined to different things, and a single query would have to
/// `LEFT JOIN` one grouped set onto another to avoid multiplying rows — which
/// is where a sum quietly doubles.
///
/// # Errors
///
/// Either query.
pub(crate) async fn progress(
    db: &PgPool,
    user_id: i64,
    project_id: Uuid,
    run: i16,
) -> Result<HashMap<Uuid, Progress>, sqlx::Error> {
    // `FILTER (WHERE run = $3)` on the per-run columns and no filter on
    // `best_points`, which is the whole scope rule from `progress.rs` written
    // in SQL: a restart resets what a task looks like and never takes back
    // points the reader banked.
    //
    // The newest outcome comes out of an ordered `array_agg` rather than a
    // correlated subquery, so the whole thing stays one grouped pass over the
    // index `task_attempts (user_id, task_id, created_at DESC)`. `id` breaks
    // the tie, because two attempts in the same second are ordinary.
    // `minutes` is worked out here rather than in rust, because `started_at`
    // was written by this server's clock and so this server's clock is the one
    // that should be saying how old it is. `greatest(..., 0)` keeps a row
    // stamped in the future from handing out the top scoring tier forever.
    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            i64,
            Option<NaiveDateTime>,
            Option<NaiveDateTime>,
            bool,
            i64,
            Option<i16>,
            i32,
        ),
    >(
        "SELECT a.task_id,
                count(*) FILTER (WHERE a.run = $3)                    AS attempts,
                min(a.created_at) FILTER (WHERE a.run = $3)           AS started_at,
                min(a.created_at)
                    FILTER (WHERE a.run = $3 AND a.outcome = $4)      AS completed_at,
                coalesce(
                    bool_or(a.run = $3 AND a.outcome = $4), false)    AS passed_this_run,
                coalesce(greatest(floor(extract(epoch FROM
                    (now() AT TIME ZONE 'utc')
                      - min(a.created_at) FILTER (WHERE a.run = $3)
                ) / 60), 0), 0)::bigint                               AS minutes,
                (array_agg(a.outcome ORDER BY a.created_at DESC, a.id DESC)
                    FILTER (WHERE a.run = $3))[1]                     AS latest,
                coalesce(
                    max(a.points) FILTER (WHERE a.outcome = $4), 0)   AS best_points
           FROM task_attempts a
           JOIN tasks t ON t.id = a.task_id
          WHERE a.user_id = $1
            AND t.project_id = $2
          GROUP BY a.task_id",
    )
    .bind(user_id)
    .bind(project_id)
    .bind(run)
    .bind(Outcome::Passed.as_db())
    .fetch_all(db)
    .await?;

    let mut progress: HashMap<Uuid, Progress> = rows
        .into_iter()
        .map(
            |(
                task_id,
                attempts,
                started_at,
                completed_at,
                passed_this_run,
                minutes,
                latest,
                best_points,
            )| {
                (
                    task_id,
                    Progress {
                        attempts,
                        started_at,
                        completed_at,
                        passed_this_run,
                        minutes,
                        // A number outside the CHECK constraint means the
                        // database and this enum have drifted. Read as "no
                        // latest attempt" rather than guessed at — see
                        // `Outcome::from_db`.
                        latest: latest.and_then(Outcome::from_db),
                        best_points,
                        hints_deducted: 0,
                    },
                )
            },
        )
        .collect();

    // A reader can unlock a hint on a task they have never submitted against,
    // so this is folded in rather than joined onto the rows above — the entry
    // may not be there yet.
    for (task_id, deducted) in unlocked(db, user_id, project_id).await? {
        progress.entry(task_id).or_default().hints_deducted = deducted;
    }

    Ok(progress)
}

/// What hints have cost this reader, per task of one project.
async fn unlocked(
    db: &PgPool,
    user_id: i64,
    project_id: Uuid,
) -> Result<Vec<(Uuid, i32)>, sqlx::Error> {
    // Joined through `task_hints` because `user_unlocked_hints.task_id` was
    // dropped: a hint belongs to exactly one task and ohara says which, so a
    // second column here could disagree with the first.
    sqlx::query_as::<_, (Uuid, i32)>(
        "SELECT h.task_id, coalesce(sum(u.points_deducted), 0)::int
           FROM user_unlocked_hints u
           JOIN task_hints h ON h.id = u.task_hint_id
           JOIN tasks t ON t.id = h.task_id
          WHERE u.user_id = $1
            AND t.project_id = $2
          GROUP BY h.task_id",
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_all(db)
    .await
}

/// The hints of one task that this reader has already paid for.
///
/// # Errors
///
/// The query.
pub(crate) async fn unlocked_hints(
    db: &PgPool,
    user_id: i64,
    task_id: Uuid,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (Uuid,)>(
        "SELECT u.task_hint_id
           FROM user_unlocked_hints u
           JOIN task_hints h ON h.id = u.task_hint_id
          WHERE u.user_id = $1 AND h.task_id = $2",
    )
    .bind(user_id)
    .bind(task_id)
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// One attempt, appended.
///
/// Never an update and never a delete: the log is what every other answer here
/// is derived from, so a row that could be rewritten is a history that could
/// be rewritten. `run` and `points` are both arguments the *caller worked out*
/// rather than anything the client sent.
///
/// # Errors
///
/// The insert. A foreign key failing means the task or the reader went away
/// between the check and the write, which the caller can say something useful
/// about.
pub(crate) async fn record(
    db: &PgPool,
    user_id: i64,
    task_id: Uuid,
    run: i16,
    outcome: Outcome,
    points: i32,
    context: Option<&str>,
) -> Result<(i64, NaiveDateTime), sqlx::Error> {
    sqlx::query_as::<_, (i64, NaiveDateTime)>(
        "INSERT INTO task_attempts
             (user_id, task_id, run, outcome, points, context)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, created_at",
    )
    .bind(user_id)
    .bind(task_id)
    .bind(run)
    .bind(outcome.as_db())
    .bind(points)
    .bind(context)
    .fetch_one(db)
    .await
}

/// Opens a new run of a project, and says which one it is.
///
/// **Two statements in one transaction, not one statement.** The obvious
/// version puts the `INSERT` in a CTE and counts the rows beside it — and it
/// answers `1` after the first restart, forever. A data-modifying CTE's rows
/// are not visible to the rest of the statement that contains it: every branch
/// reads the snapshot taken *before* the statement ran, so the count never
/// includes the row being inserted. In a transaction the second statement sees
/// the first, which is the behaviour the arithmetic assumes.
///
/// The transaction also makes the pair atomic, so a restart that inserted and
/// then failed to report cannot leave the reader on a run they were never told
/// about.
///
/// # Errors
///
/// Either statement, or the commit.
pub(crate) async fn restart(
    db: &PgPool,
    user_id: i64,
    project_id: Uuid,
) -> Result<(i16, NaiveDateTime), sqlx::Error> {
    let mut tx = db.begin().await?;

    let (at,) = sqlx::query_as::<_, (NaiveDateTime,)>(
        "INSERT INTO project_restarts (user_id, project_id)
         VALUES ($1, $2)
         RETURNING restarted_at",
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_one(&mut *tx)
    .await?;

    let (restarts,) = sqlx::query_as::<_, (i64,)>(
        "SELECT count(*) FROM project_restarts
          WHERE user_id = $1 AND project_id = $2",
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((run_from(restarts), at))
}

/// The run a restart count puts a reader on: one more than the restarts.
///
/// A reader who has never restarted is on run 1. Shared by [`current_run`] and
/// [`restart`] so the two cannot drift into different arithmetic, which is
/// exactly the bug that made a restart report run 1 twice.
///
/// Clamped because the column is a `smallint`. A reader who restarts thirty-two
/// thousand times has found something more interesting than the project, and
/// clamping keeps their next submission writable rather than failing it.
fn run_from(restarts: i64) -> i16 {
    i16::try_from(restarts.saturating_add(1)).unwrap_or(i16::MAX)
}

/// Records that a reader has unlocked a hint, at the price it cost them today.
///
/// `false` when they already held it, which is not an error — luxctl retries,
/// and a second unlock must not be a second deduction.
///
/// **The exclusion is the database's, not this function's.** A read-then-write
/// guard in rust would let two concurrent unlocks of the same hint both see
/// "not unlocked" and both insert; the laravel action was written that way and
/// only survived it because `user_unlocked_hints_user_id_task_hint_id_unique`
/// caught the second insert — as a 500, which is the right outcome reached the
/// wrong way. That index is in the baseline, so `ON CONFLICT` has something to
/// infer from and a racing second unlock is the ordinary "already unlocked"
/// answer rather than an error.
///
/// **The deduction is stored, not looked up later.** Re-pricing a hint in
/// ohara must not change what somebody was charged last week.
///
/// # Errors
///
/// The insert.
pub(crate) async fn unlock_hint(
    db: &PgPool,
    user_id: i64,
    hint_id: Uuid,
    deduction: i32,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "INSERT INTO user_unlocked_hints
             (user_id, task_hint_id, points_deducted, unlocked_at)
         VALUES ($1, $2, $3, (now() AT TIME ZONE 'utc'))
         ON CONFLICT (user_id, task_hint_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(hint_id)
    .bind(deduction)
    .execute(db)
    .await?;

    Ok(done.rows_affected() > 0)
}

/// A reader's totals across everything they have ever attempted.
///
/// The three numbers `GET /user` reports. Counted over the whole log rather
/// than the current run: this is a career total, not a project's progress.
///
/// `tasks_completed` counts *distinct tasks* that have ever passed. The
/// laravel query counted passing rows, so a reader who re-ran a task they had
/// already completed watched the number climb; that was a bug in a headline
/// figure, and it is not reproduced.
///
/// `total_xp` sums the points on passing attempts and then takes off what the
/// reader has spent on hints, so the headline equals the per-task numbers
/// added up. A re-pass earns zero — see [`record`]'s caller — so the sum
/// cannot double-count a task passed in two runs. The laravel figure left
/// hints out and therefore disagreed with every project page.
///
/// # Errors
///
/// Either query.
pub(crate) async fn totals(
    db: &PgPool,
    user_id: i64,
) -> Result<(i64, i64, i64), sqlx::Error> {
    let (attempted, completed, earned) =
        sqlx::query_as::<_, (i64, i64, Option<i64>)>(
            "SELECT count(DISTINCT t.project_id)                  AS attempted,
                    count(DISTINCT a.task_id) FILTER (WHERE a.outcome = $2)
                                                                  AS completed,
                    sum(a.points) FILTER (WHERE a.outcome = $2)   AS earned
               FROM task_attempts a
               JOIN tasks t ON t.id = a.task_id
              WHERE a.user_id = $1",
        )
        .bind(user_id)
        .bind(Outcome::Passed.as_db())
        .fetch_one(db)
        .await?;

    // Its own statement, not a join: joining the unlocks onto the attempts
    // would multiply one against the other and inflate both sums.
    let (spent,) = sqlx::query_as::<_, (Option<i64>,)>(
        "SELECT sum(points_deducted) FROM user_unlocked_hints
          WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(db)
    .await?;

    Ok((
        attempted,
        completed,
        earned
            .unwrap_or_default()
            .saturating_sub(spent.unwrap_or_default())
            .max(0),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reader_who_has_never_restarted_is_on_the_first_run() {
        assert_eq!(run_from(0), 1);
        assert_eq!(run_from(1), 2);
        assert_eq!(run_from(9), 10);
    }

    #[test]
    fn an_absurd_number_of_restarts_still_writes() {
        // The column is a `smallint`, so this clamps rather than failing the
        // reader's next submission on a value the CHECK would refuse.
        assert_eq!(run_from(i64::from(i16::MAX)), i16::MAX);
        assert_eq!(run_from(i64::MAX), i16::MAX);
    }
}
