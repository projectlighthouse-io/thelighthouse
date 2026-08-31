//! What the settings page sees, and what it sends.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::tokens::store::Record;

/// What `name` may hold. The column is `text`; the cap is what the page can
/// render without the row wrapping into something unreadable.
pub(crate) const MAX_NAME: usize = 60;

/// One token on the settings page.
///
/// **The secret is not here and cannot be.** Only its hash was ever stored, so
/// there is nothing to put in this struct even if somebody wanted to — which
/// is the property that makes "shown once" true rather than merely intended.
#[derive(Debug, Serialize)]
pub(crate) struct TokenView {
    pub(crate) id: i64,
    pub(crate) name: String,
    /// `None` for a token that has never been used, which the page shows as
    /// "never" rather than as a blank — a token nobody has used yet is a
    /// finding, not a missing value.
    #[serde(serialize_with = "crate::response::as_utc")]
    pub(crate) last_used_at: Option<NaiveDateTime>,
    #[serde(serialize_with = "crate::response::as_utc")]
    pub(crate) created_at: Option<NaiveDateTime>,
}

impl From<Record> for TokenView {
    fn from(record: Record) -> Self {
        Self {
            id: record.id,
            name: record.name,
            last_used_at: record.last_used_at,
            created_at: record.created_at,
        }
    }
}

/// What comes back from minting one: the row, and the one sight of the secret.
#[derive(Debug, Serialize)]
pub(crate) struct MintedView {
    #[serde(flatten)]
    pub(crate) token: TokenView,
    /// `{id}|{secret}`, assembled here because that is the string a reader
    /// pastes into `lux auth` — handing back the halves separately would leave
    /// every client to join them, and one of them to join them wrongly.
    pub(crate) token_string: String,
}

/// The body of `POST /api/settings/tokens`.
#[derive(Debug, Deserialize)]
pub(crate) struct NewToken {
    pub(crate) name: String,
}

impl NewToken {
    /// The name, trimmed. Empty and over-long are the caller's problem and are
    /// refused by name rather than silently fixed: a token called `""` or one
    /// truncated mid-word is not what anybody asked for.
    pub(crate) fn name(&self) -> &str {
        self.name.trim()
    }
}
