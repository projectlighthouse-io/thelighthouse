//! Personal access tokens: how luxctl says who it is.
//!
//! ```text
//!   mod.rs    this map, and what a resolved token is
//!   token.rs  the string a reader pastes, taken apart and hashed
//!   store.rs  the lookup, and the only thing here that talks to postgres
//! ```
//!
//! **The shape is Sanctum's, on purpose.** A token reads `{id}|{secret}`; the
//! row is found by the id and the sha256 of the secret is compared against
//! `personal_access_tokens.token`. Every token a reader has already configured
//! keeps working the day the rebuild takes over, and the `/settings/tokens`
//! page keeps meaning what it says. Minting a cleaner row of our own would
//! have signed out everybody who ever ran `lux login`, to no one's benefit.
//!
//! **The id half is a lookup key, not a credential.** It is in the string a
//! reader can read off their screen, and knowing it proves nothing — the
//! secret half is the whole of the proof, it is never stored, and what is
//! stored is a hash compared in constant time.
//!
//! **A cookie is not a token and a token is not a cookie.** The two carry the
//! same reader through different doors: the browser gets `session`, luxctl
//! gets this, and neither is ever accepted where the other belongs. That is
//! also why nothing here mints a CSRF token — a bearer credential is not sent
//! automatically by anything, so there is no cross-site request to forge.
//!
//! **Abilities are not read.** Sanctum's `abilities` column exists and every
//! token this platform ever issued holds `["*"]`. Reading it would be
//! implementing a permission model that has no second value in it yet; when
//! there is one, this is where it goes.

pub(crate) mod store;
mod token;

pub(crate) use token::Presented;

/// A token that resolved to a reader.
///
/// Put in request extensions by `middleware::bearer`, so a handler behind that
/// layer takes it as an extractor and can be sure of it.
///
/// The reader's id and nothing else. *Which* of their tokens it was is the
/// kind of thing a revoked-credential incident wants, and it is not here
/// because nothing writes it down yet — a field carried on the chance somebody
/// logs it later is a field nobody logs.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Holder {
    pub(crate) user_id: i64,
}
