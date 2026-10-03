//! The reader behind a session.

mod store;
mod username;

pub(crate) use store::{
    Error, Profile, PublicProfile, find, find_or_create, profile,
    public_profile, update_profile,
};

/// What the frontend renders chrome from, and what a session points at.
#[derive(Clone, Debug)]
pub(crate) struct User {
    pub(crate) id: i64,
    pub(crate) name: String,
    /// How the reader is addressed in a url — `/blog?author=`. Generated at
    /// signup and never editable, see `users::username`. Nullable because the
    /// laravel rows predate the column.
    pub(crate) username: Option<String>,
    pub(crate) email: String,
    pub(crate) avatar: Option<String>,
}

/// A tuple in the column order every query below selects.
pub(crate) type UserRow = (i64, String, Option<String>, String, Option<String>);

pub(crate) const COLUMNS: &str = "id, name, username, email, avatar_url";

impl From<UserRow> for User {
    fn from((id, name, username, email, avatar): UserRow) -> Self {
        Self {
            id,
            name,
            username,
            email,
            avatar,
        }
    }
}
