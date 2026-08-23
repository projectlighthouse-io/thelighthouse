//! The closed sets that content columns store as integers, and their names.
//!
//! `content` rather than `books` because [`Status`] is shared: a book and a
//! lesson are both drafted and both published, and one enum for both is what
//! stops the two tables drifting into different ideas of what "published"
//! means.
//!
//! The database stores `status` and `tier` as `smallint`. The numbers are the
//! storage; these enums are the meaning, and they are the only place the two
//! are allowed to be connected. A `1` read out of `books.status` becomes
//! [`Status::Published`] here or it does not become anything at all.
//!
//! Kept in step with the CHECK constraints in the migrations that added these
//! columns — `20260822180000_reshape_books_columns.sql` and
//! `20260822190000_reshape_lessons_columns.sql`. Adding a variant means
//! widening the CHECK in a new migration, and widening a CHECK without adding
//! the variant means rows this code refuses to read. The column COMMENTs point
//! back here so whoever finds the table first also finds this file.
//!
//! Why integers at all, when postgres has enum types and text is readable in a
//! `psql` session: an enum type has to be altered by migration to add a value
//! anyway, and text invites a typo that reads as neither of the valid values —
//! a row that is silently missing from every listing rather than one that fails
//! loudly on write.

// Nothing queries `books` yet — the content endpoints are a later phase — so
// every item here is currently unused. The alternative to this allow is holding
// the schema's meaning in someone's head until the first query needs it, which
// is how a smallint column ends up decoded three different ways. Remove the
// allow with the first reader.
#![allow(dead_code)]

use serde::Serialize;

/// Where a piece of content is in its life, from written to readable. Books and
/// lessons both use it.
///
/// Not a boolean. `is_published` was one, and it could not say "this is written
/// but not ready", which is the state most content is in for most of the time
/// it is being worked on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Status {
    /// Written, synced, and not for readers yet. A local run shows drafts so
    /// they can be read while being written; production does not, which is the
    /// whole point of the distinction.
    ///
    /// The default for a new row, because the failure mode of the other default
    /// is publishing something by accident.
    Draft,
    /// Live. Anonymous readers can see it exists; whether they can read all of
    /// it is a separate question, answered by entitlement rather than here.
    Published,
}

/// How hard a book is, which is a promise to the reader about what it assumes
/// they already know.
///
/// Deliberately not a replacement for the old `required_tier`. That column said
/// which subscription unlocked a course. This says nothing about payment, and
/// nothing here should ever be consulted to decide access.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Tier {
    /// Assumes a working knowledge of one language and nothing else.
    Foundation,
    /// Assumes the foundation material, or the equivalent from elsewhere.
    Intermediate,
    /// Assumes comfort with the systems layer — syscalls, memory, concurrency.
    Advanced,
}

impl Status {
    /// The stored representation. Paired with [`Status::from_db`]; changing one
    /// without the other silently remaps every row in the table.
    pub(crate) const fn as_db(self) -> i16 {
        match self {
            Self::Draft => 0,
            Self::Published => 1,
        }
    }

    /// `None` for anything the CHECK constraint should have refused.
    ///
    /// Returning an option rather than defaulting to `Draft`: a value outside
    /// the set means the database and this enum have drifted, and quietly
    /// treating it as unpublished would hide that for as long as nobody
    /// noticed the missing book.
    pub(crate) const fn from_db(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Draft),
            1 => Some(Self::Published),
            _ => None,
        }
    }
}

impl Tier {
    pub(crate) const fn as_db(self) -> i16 {
        match self {
            Self::Foundation => 0,
            Self::Intermediate => 1,
            Self::Advanced => 2,
        }
    }

    /// `None` for anything outside the set — see [`Status::from_db`].
    pub(crate) const fn from_db(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Foundation),
            1 => Some(Self::Intermediate),
            2 => Some(Self::Advanced),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant, so a new one cannot be added without being listed here —
    /// which is the prompt to widen the CHECK constraint in a migration too.
    const STATUSES: [Status; 2] = [Status::Draft, Status::Published];
    const TIERS: [Tier; 3] =
        [Tier::Foundation, Tier::Intermediate, Tier::Advanced];

    #[test]
    fn every_status_survives_the_round_trip() {
        for status in STATUSES {
            assert_eq!(Status::from_db(status.as_db()), Some(status));
        }
    }

    #[test]
    fn every_tier_survives_the_round_trip() {
        for tier in TIERS {
            assert_eq!(Tier::from_db(tier.as_db()), Some(tier));
        }
    }

    /// The numbers are the storage format, so they are part of the schema and
    /// cannot be reordered for tidiness — every row already on disk means what
    /// these say it means.
    #[test]
    fn the_stored_numbers_are_fixed() {
        assert_eq!(Status::Draft.as_db(), 0);
        assert_eq!(Status::Published.as_db(), 1);

        assert_eq!(Tier::Foundation.as_db(), 0);
        assert_eq!(Tier::Intermediate.as_db(), 1);
        assert_eq!(Tier::Advanced.as_db(), 2);
    }

    #[test]
    fn a_value_the_check_constraint_would_refuse_is_not_guessed_at() {
        for out_of_range in [-1, 2, 99] {
            assert_eq!(Status::from_db(out_of_range), None);
        }
        for out_of_range in [-1, 3, 99] {
            assert_eq!(Tier::from_db(out_of_range), None);
        }
    }

    #[test]
    fn json_uses_the_name_not_the_number() {
        // The wire format the frontend reads. Numbers are a storage detail and
        // would make every template carry the same mapping again.
        assert_eq!(
            serde_json::to_string(&Status::Published).unwrap(),
            "\"published\""
        );
        assert_eq!(
            serde_json::to_string(&Tier::Foundation).unwrap(),
            "\"foundation\""
        );
    }
}
