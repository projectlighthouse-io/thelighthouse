//! Which books a plan sells.
//!
//! A track is a bundle of books — Foundation, Go, Rust, All — and it is what
//! is actually for sale. Two plans sell each one: a recurring `_yearly` and a
//! `_lifetime` bought outright.
//!
//! **The books on a track are not configured here, or anywhere else.** They
//! are the `tracks:` map in each book's `book.yaml`, which already exists
//! because a track is a reading order first and a bundle second. A book joins
//! Rust by saying so in the content repo, and nothing about pricing has to be
//! redeployed for it to be included.

use billing::{PlanId, Plans};
use ohara::catalog::Snapshot;

/// The track that means "everything", including books on no track at all.
///
/// Three books — the container one, C, and the interview one — are on no
/// reading order. They are still for sale, and this is what sells them.
pub(crate) const EVERYTHING: &str = "all";

/// The track a plan sells, taken from its name.
///
/// `rust_yearly` and `rust_lifetime` both sell `rust`. A convention rather
/// than a second config table, because the alternative is two places that can
/// disagree about what a plan grants — and the failure when they do is a
/// reader paying for books they do not get, or getting books nobody paid for.
///
/// Everything up to the last `_`, so a track whose own name contains one still
/// works.
pub(crate) fn of(plan: &str) -> &str {
    plan.rsplit_once('_').map_or(plan, |(track, _)| track)
}

/// Whether a track includes this book.
pub(crate) fn covers(track: &str, book: &ohara::book::Book) -> bool {
    track == EVERYTHING || book.tracks.contains_key(track)
}

/// Every book on a track, by the id an entitlement points at.
///
/// A book with no id is skipped rather than failing the purchase: an
/// entitlement names a book by id, so one that has never been given an id
/// cannot be granted. Silently dropping it would be worse, which is why the
/// caller logs the difference between what was bought and what was granted.
pub(crate) fn books(snapshot: &Snapshot, track: &str) -> Vec<uuid::Uuid> {
    snapshot
        .books()
        .filter(|entry| covers(track, &entry.book))
        .filter_map(|entry| entry.book.id)
        .collect()
}

/// Check every configured plan sells a track that exists.
///
/// At boot, so a typo is a named startup failure. The failure it prevents is
/// quiet: `rustt_yearly` parses, sells, charges, and grants nothing, and the
/// first report of it is a reader who paid.
///
/// # Errors
///
/// The plan and the track it names, when no book is on that track.
pub(crate) fn validate(
    plans: &Plans,
    snapshot: &Snapshot,
) -> Result<(), String> {
    for plan in plans.all() {
        let track = of(plan.id.as_str());

        if track == EVERYTHING {
            continue;
        }

        if !snapshot
            .books()
            .any(|entry| entry.book.tracks.contains_key(track))
        {
            return Err(format!(
                "the plan {} sells the track {track:?}, which no book is on",
                plan.id
            ));
        }
    }

    Ok(())
}

/// The track a reader's membership covers, if the plan still names one.
pub(crate) fn of_plan(plan: &PlanId) -> &str {
    of(plan.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> Snapshot {
        Snapshot::load(&ohara::fixture::content(), ohara::Drafts::Hidden)
            .unwrap()
    }

    #[test]
    fn a_plan_names_its_track_before_the_last_underscore() {
        assert_eq!(of("rust_yearly"), "rust");
        assert_eq!(of("rust_lifetime"), "rust");
        assert_eq!(of("foundation_yearly"), "foundation");
        assert_eq!(of("all_lifetime"), "all");
        // A name with no underscore is the track itself, not an error.
        assert_eq!(of("rust"), "rust");
    }

    #[test]
    fn everything_covers_a_book_that_is_on_no_track_at_all() {
        // The three books on no reading order are only ever sold by `all`. If
        // this stops holding they become unbuyable.
        let snapshot = snapshot();
        let orphan =
            snapshot.books().find(|entry| entry.book.tracks.is_empty());

        if let Some(entry) = orphan {
            assert!(covers(EVERYTHING, &entry.book));
            assert!(!covers("rust", &entry.book));
        }
    }

    #[test]
    fn a_plan_selling_a_track_no_book_is_on_fails_the_boot() {
        let plans = Plans::from_yaml(
            "
plans:
  - id: nosuchtrack_yearly
    price: price_x
    interval: year
",
        )
        .unwrap();

        assert!(validate(&plans, &snapshot()).is_err());
    }

    #[test]
    fn everything_is_always_a_valid_track_to_sell() {
        let plans = Plans::from_yaml(
            "
plans:
  - id: all_yearly
    price: price_x
    interval: year
",
        )
        .unwrap();

        assert!(validate(&plans, &snapshot()).is_ok());
    }
}
