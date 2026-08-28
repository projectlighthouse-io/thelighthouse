//! How much of a book a reader may read.

use crate::ohara::catalog::BookEntry;

/// What a reader gets, not whether they pass a test.
///
/// An enum rather than a bool because the caller has to choose a body either
/// way, and `if !may_read(book)` puts that choice behind a negated maybe. A
/// `match` on this says which half is being served at the point it is served,
/// and a third answer later — expired, revoked, region-locked — stops
/// compiling everywhere it matters instead of quietly folding into whichever
/// branch `false` already took.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Access {
    /// Everything, paid half included.
    Full,
    /// The free half, and the flag saying there is more.
    FreeOnly,
}

/// What this reader gets of this book.
///
/// **A free book is [`Access::Full`] for everybody.** That is not a
/// placeholder standing in for a check: a book priced at zero is given away,
/// and there is nothing to buy that would grant more than that.
///
/// **A priced book is [`Access::FreeOnly`] for everybody, so far.** There is
/// no entitlements table and no checkout, so no reader can have bought one;
/// answering `FreeOnly` is the truthful answer today rather than a stub
/// pretending to check something. When purchases land this grows a lookup
/// against the reader and the book's uuid, and nothing above it changes.
pub(crate) fn access(book: &BookEntry) -> Access {
    if book.book.price.is_free() {
        Access::Full
    } else {
        Access::FreeOnly
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ohara::{catalog::Snapshot, fixture};

    #[test]
    fn a_priced_book_gives_up_only_its_free_half_until_it_can_be_bought() {
        let snapshot = Snapshot::load(&fixture::content()).unwrap();
        let book = snapshot.book("fixture-book").unwrap();

        // The fixture book is priced at 2900.
        assert_eq!(access(book), Access::FreeOnly);
    }
}
