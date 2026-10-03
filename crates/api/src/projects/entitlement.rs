//! Which of a project's tasks a reader may work on.
//!
//! **A project is not sold on its own; paying for anything opens all of them.**
//! A reader with a live membership — a subscription in good standing, or a
//! track bought outright — or a live purchase of any book has every project's
//! paid tasks. Everyone else, signed in or not, has the free ones: `is_free` in
//! a task's yaml is how a project gives its first few away.
//!
//! There is no `projects.price` and no per-project link to a book. Which plan
//! or book was bought does not matter here, only that something was.

use sqlx::postgres::PgPool;

use ohara::catalog::TaskEntry;

use crate::books::entitlement::Access;

/// Whether this reader has paid for anything that is still live.
///
/// `Access::Full` if so, `Access::FreeOnly` otherwise — and the free tasks are
/// still theirs, which is what makes the name right.
///
/// A membership in grace — a payment failed and is being retried — does not
/// count, the same rule books follow (`Membership::grants_access`).
///
/// # Errors
///
/// The queries. Deliberately not swallowed: a database that cannot be reached
/// means *we do not know*, and answering "not entitled" would lock a paying
/// reader out of work they are in the middle of.
pub(crate) async fn for_reader(
    db: &PgPool,
    reader: Option<i64>,
) -> Result<Access, sqlx::Error> {
    let Some(user_id) = reader else {
        return Ok(Access::FreeOnly);
    };

    if crate::payments::live(db, user_id)
        .await?
        .is_some_and(|membership| membership.grants_access())
    {
        return Ok(Access::Full);
    }

    let bought: Option<(i32,)> = sqlx::query_as(
        "SELECT 1 FROM entitlements \
         WHERE user_id = $1 \
         AND (expires_at IS NULL OR expires_at > now()) \
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;

    Ok(if bought.is_some() {
        Access::Full
    } else {
        Access::FreeOnly
    })
}

/// Whether this particular task is behind that paywall.
///
/// `is_free` in the task's yaml opens it to everybody, which is how a project
/// gives its first few tasks away. Everything else needs [`Access::Full`].
#[must_use]
pub(crate) fn is_paid(task: &TaskEntry, held: Access) -> bool {
    !task.task.is_free && held == Access::FreeOnly
}

#[cfg(test)]
mod tests {
    use super::*;
    use ohara::{Drafts, catalog::Snapshot, fixture};

    #[test]
    fn a_free_task_is_open_to_a_reader_who_has_bought_nothing() {
        let snapshot =
            Snapshot::load(&fixture::content(), Drafts::Hidden).unwrap();
        let project = snapshot.project("fixture-project").unwrap();
        let free = project.task("listen-on-port").unwrap();
        let paid = project.task("say-something").unwrap();

        assert!(!is_paid(free, Access::FreeOnly));
        assert!(is_paid(paid, Access::FreeOnly));
        // ...and entitlement opens the other one without changing the first.
        assert!(!is_paid(free, Access::Full));
        assert!(!is_paid(paid, Access::Full));
    }

    #[tokio::test]
    async fn a_stranger_has_only_the_free_tasks() {
        // Lazy and never connected: a stranger is answered before any query.
        // If that stops being true this fails loudly rather than passing
        // against a stub.
        let db =
            sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
                .unwrap();

        assert_eq!(for_reader(&db, None).await.unwrap(), Access::FreeOnly);
    }
}
