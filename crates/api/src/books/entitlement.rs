//! Who may read the paid half of a lesson.

use crate::ohara::catalog::BookEntry;

/// Whether a reader may read past the paywall.
///
/// **A free book entitles everybody.** That is not a placeholder — a book
/// priced at zero is given away, and there is nothing to buy that would grant
/// more access than that.
///
/// **A priced book entitles nobody yet.** There is no `purchases` table and no
/// checkout, so no reader can have bought one; answering "no" is the truthful
/// answer today rather than a stub that pretends. When purchases land, this
/// grows a lookup and nothing above it changes.
pub(crate) fn may_read_paid(book: &BookEntry) -> bool {
    book.book.price.is_free()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ohara::{catalog::Snapshot, fixture};

    #[test]
    fn a_priced_book_entitles_nobody_until_there_is_a_way_to_buy_it() {
        let snapshot = Snapshot::load(&fixture::content()).unwrap();
        let book = snapshot.book("fixture-book").unwrap();

        // The fixture book is priced at 2900.
        assert!(!may_read_paid(book));
    }
}
