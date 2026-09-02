//! Which of a project's tasks a reader may work on.
//!
//! **A project is not sold; the book beside it is.** ohara's
//! `related_book_slug` names the book that teaches what the project asks for,
//! and holding an entitlement to that book is what opens the project's paid
//! tasks. There is no `projects.price`, no subscription tier, and nothing here
//! consults a price — `books::entitlement` says why price never decides access.
//!
//! **A project with no related book is free.** There is nothing to have bought:
//! no book means no entitlement row could name it, so a paywall there would be
//! one nobody can pass. That is `docs/rebuild_continuation.md`'s rule, and it
//! has a consequence worth stating out loud rather than discovering: today only
//! three of the sixteen projects in ohara set `related_book_slug`, so every
//! task of the other thirteen is open regardless of its `is_free: false`. That
//! is a content decision — add the slug — and not something to patch here by
//! inventing a second gate.

use ohara::catalog::{ProjectEntry, Snapshot, TaskEntry};
use sqlx::postgres::PgPool;

use crate::books::entitlement::{Access, access};

/// Whether this reader has bought their way into this project's paid tasks.
///
/// `Access::Full` for a project with no related book, or one whose book the
/// reader holds. `Access::FreeOnly` otherwise — and the free tasks are still
/// theirs, which is what makes the name right.
///
/// # Errors
///
/// The entitlement query. Deliberately not swallowed: a database that cannot
/// be reached means *we do not know*, and answering "not entitled" would lock
/// a paying reader out of work they are in the middle of.
pub(crate) async fn for_project(
    db: &PgPool,
    snapshot: &Snapshot,
    reader: Option<i64>,
    project: &ProjectEntry,
    excluded: &[String],
) -> Result<Access, sqlx::Error> {
    let Some(slug) = project.project.related_book_slug.as_deref() else {
        return Ok(Access::Full);
    };

    // A slug naming a book that is not in the catalogue: a draft, a typo, or a
    // book not written yet. Not `Full` — a project pointed at a book that does
    // not exist yet is one whose paid tasks are not for sale yet either, and
    // opening them would be the wrong way to be wrong.
    let Some(book) = snapshot.book(slug) else {
        return Ok(Access::FreeOnly);
    };

    access(db, reader, book, excluded).await
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
    use ohara::{Drafts, fixture};

    fn snapshot() -> Snapshot {
        Snapshot::load(&fixture::content(), Drafts::Hidden).unwrap()
    }

    #[test]
    fn a_free_task_is_open_to_a_reader_who_has_bought_nothing() {
        let snapshot = snapshot();
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
    async fn a_project_with_no_related_book_has_nothing_to_sell() {
        let snapshot =
            Snapshot::load(&fixture::drafts(), Drafts::Shown).unwrap();
        let project = snapshot.project("unfinished-project").unwrap();
        // Lazy and never connected: a project with no book is answered before
        // the query would happen. If that stops being true this fails loudly
        // rather than passing against a stub.
        let db =
            sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
                .unwrap();

        assert!(project.project.related_book_slug.is_none());
        assert_eq!(
            for_project(&db, &snapshot, None, project, &[])
                .await
                .unwrap(),
            Access::Full
        );
    }

    #[tokio::test]
    async fn a_related_book_that_is_not_in_the_catalogue_keeps_the_paywall() {
        // A project pointing at a book that is a draft, or misspelled, or not
        // written yet. Opening its paid tasks would be the wrong way to be
        // wrong — and it is decided before any query, so this needs no
        // database either.
        let snapshot =
            Snapshot::load(&fixture::drafts(), Drafts::Shown).unwrap();
        let project = snapshot.project("pointed-at-nothing").unwrap();
        let db =
            sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
                .unwrap();

        assert_eq!(
            project.project.related_book_slug.as_deref(),
            Some("no-such-book")
        );
        assert!(snapshot.book("no-such-book").is_none());
        assert_eq!(
            for_project(&db, &snapshot, Some(1), project, &[])
                .await
                .unwrap(),
            Access::FreeOnly
        );
        assert!(is_paid(
            project.task("a-paid-task").unwrap(),
            Access::FreeOnly
        ));
    }
}
