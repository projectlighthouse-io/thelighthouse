//! The reader behind a session.

mod store;
mod username;

pub(crate) use store::{
    Error, Profile, find, find_or_create, profile, update_profile,
};

/// What the frontend renders chrome from, and what a session points at.
#[derive(Clone, Debug)]
pub(crate) struct User {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) avatar: Option<String>,
}

/// A tuple in the column order every query below selects.
pub(crate) type UserRow = (i64, String, String, Option<String>);

pub(crate) const COLUMNS: &str = "id, name, email, avatar_url";

impl From<UserRow> for User {
    fn from((id, name, email, avatar): UserRow) -> Self {
        Self {
            id,
            name,
            email,
            avatar,
        }
    }
}
