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

// ------------------------------------------------------------------ profile

/// The caps, which are the columns' where the column has one.
///
/// `tagline` is `varchar(160)`, `location` is `varchar(120)` and `bio` is
/// `text`; the bio's limit is the laravel form's, kept because readers have
/// written against it. Checking here rather than letting postgres refuse means
/// a message that names the field instead of a constraint.
pub(crate) const MAX_TAGLINE: usize = 160;
pub(crate) const MAX_LOCATION: usize = 120;
pub(crate) const MAX_BIO: usize = 1_000;
pub(crate) const MAX_SHORT: usize = 255;

/// What the public profile page renders.
///
/// `username` and `github_username` are here and are not editable: one is
/// generated at sign-up, the other arrives from GitHub. Sending them keeps the
/// page from having to ask somewhere else for the two fields it displays
/// beside the ones it edits.
#[derive(Debug, Serialize)]
pub(crate) struct ProfileView {
    pub(crate) username: Option<String>,
    pub(crate) github_username: Option<String>,
    pub(crate) tagline: Option<String>,
    pub(crate) bio: Option<String>,
    pub(crate) company: Option<String>,
    pub(crate) education: Option<String>,
    pub(crate) location: Option<String>,
    pub(crate) linkedin_url: Option<String>,
    pub(crate) x_url: Option<String>,
    pub(crate) website_url: Option<String>,
}

impl From<crate::users::Profile> for ProfileView {
    fn from(row: crate::users::Profile) -> Self {
        Self {
            username: row.username,
            github_username: row.github_username,
            tagline: row.tagline,
            bio: row.bio,
            company: row.company,
            education: row.education,
            location: row.location,
            linkedin_url: row.linkedin_url,
            x_url: row.x_url,
            website_url: row.website_url,
        }
    }
}

/// The body of `PATCH /api/settings/profile`.
///
/// Every field optional and every field written — see `users::update_profile`
/// for why a partial update is not on offer. An absent key clears the column,
/// which is what the form does when a reader empties an input.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct EditProfile {
    #[serde(default)]
    pub(crate) tagline: Option<String>,
    #[serde(default)]
    pub(crate) bio: Option<String>,
    #[serde(default)]
    pub(crate) company: Option<String>,
    #[serde(default)]
    pub(crate) education: Option<String>,
    #[serde(default)]
    pub(crate) location: Option<String>,
    #[serde(default)]
    pub(crate) linkedin_url: Option<String>,
    #[serde(default)]
    pub(crate) x_url: Option<String>,
    #[serde(default)]
    pub(crate) website_url: Option<String>,
}

/// Trimmed, with empty meaning absent.
///
/// A column holding `""` renders as an empty line rather than as nothing, and
/// is a different value from `NULL` for anything that later asks "has this
/// reader written a bio". One representation of "no answer", chosen here.
fn tidy(value: Option<&String>) -> Option<String> {
    let trimmed = value?.trim();

    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// Whether this is a link a page can safely render as an `href`.
///
/// **Not a url parser, and deliberately narrower than laravel's `url` rule.**
/// The value comes back out into an anchor, and `javascript:...` is a
/// perfectly valid url — so the scheme is checked against a list of two rather
/// than parsed. A host has to follow it, and whitespace is refused outright
/// because a link containing any is not one.
fn is_web_url(value: &str) -> bool {
    if value.len() > MAX_SHORT || value.chars().any(char::is_whitespace) {
        return false;
    }

    let Some(("https" | "http", host)) = value.split_once("://") else {
        return false;
    };

    // Something before the first slash, and a dot in it: `https://` and
    // `https://localhost` are not profile links anybody meant to save.
    let host = host.split('/').next().unwrap_or_default();

    host.len() > 1 && host.contains('.')
}

impl EditProfile {
    /// The fields as they will be written, or the first thing wrong with them.
    ///
    /// # Errors
    ///
    /// A field over its cap, or a link that is not an http(s) one.
    pub(crate) fn checked(
        &self,
    ) -> Result<crate::users::Profile, super::refusal::Refusal> {
        use super::refusal::Refusal;

        let tagline = tidy(self.tagline.as_ref());
        let bio = tidy(self.bio.as_ref());
        let company = tidy(self.company.as_ref());
        let education = tidy(self.education.as_ref());
        let location = tidy(self.location.as_ref());
        let linkedin_url = tidy(self.linkedin_url.as_ref());
        let x_url = tidy(self.x_url.as_ref());
        let website_url = tidy(self.website_url.as_ref());

        // Characters, not bytes: sixty accented letters are sixty characters.
        let over = |value: &Option<String>, cap: usize| {
            value.as_ref().is_some_and(|v| v.chars().count() > cap)
        };

        if over(&tagline, MAX_TAGLINE) {
            return Err(Refusal::TaglineTooLong);
        }
        if over(&bio, MAX_BIO) {
            return Err(Refusal::BioTooLong);
        }
        if over(&company, MAX_SHORT) || over(&education, MAX_SHORT) {
            return Err(Refusal::FieldTooLong);
        }
        if over(&location, MAX_LOCATION) {
            return Err(Refusal::LocationTooLong);
        }

        // Every one of the three is reader-supplied and comes back out of the
        // profile page as an `href`, so all three go through the same check.
        let links = [&linkedin_url, &x_url, &website_url];
        if links
            .iter()
            .any(|link| link.as_deref().is_some_and(|url| !is_web_url(url)))
        {
            return Err(Refusal::NotALink);
        }

        Ok(crate::users::Profile {
            username: None,
            github_username: None,
            tagline,
            bio,
            company,
            education,
            location,
            linkedin_url,
            x_url,
            website_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_must_be_http_and_have_a_host() {
        for good in [
            "https://linkedin.com/in/someone",
            "http://example.co.uk/a/b",
        ] {
            assert!(is_web_url(good), "{good}");
        }

        for bad in [
            // The reason this is not a url parser.
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "vbscript:msgbox",
            // Shapes that are not links.
            "",
            "linkedin.com/in/someone",
            "https://",
            "https://localhost",
            "https://has a space.com",
        ] {
            assert!(!is_web_url(bad), "{bad}");
        }
    }

    #[test]
    fn a_link_longer_than_the_column_is_refused() {
        let long = format!("https://x.com/{}", "a".repeat(MAX_SHORT));

        assert!(!is_web_url(&long));
    }

    #[test]
    fn empty_and_blank_are_one_answer_and_it_is_none() {
        assert_eq!(tidy(None), None);
        assert_eq!(tidy(Some(&String::new())), None);
        assert_eq!(tidy(Some(&"   \n ".to_owned())), None);
        assert_eq!(tidy(Some(&"  hi  ".to_owned())), Some("hi".to_owned()));
    }

    #[test]
    fn a_field_over_its_cap_is_refused_by_name() {
        let too_long = |field: fn(String) -> EditProfile, n: usize| {
            field("x".repeat(n)).checked().unwrap_err()
        };

        assert_eq!(
            too_long(
                |v| EditProfile {
                    tagline: Some(v),
                    ..EditProfile::default()
                },
                MAX_TAGLINE + 1
            ),
            super::super::refusal::Refusal::TaglineTooLong
        );
        assert_eq!(
            too_long(
                |v| EditProfile {
                    bio: Some(v),
                    ..EditProfile::default()
                },
                MAX_BIO + 1
            ),
            super::super::refusal::Refusal::BioTooLong
        );
    }

    #[test]
    fn a_profile_at_its_caps_exactly_is_accepted() {
        let edit = EditProfile {
            tagline: Some("t".repeat(MAX_TAGLINE)),
            bio: Some("b".repeat(MAX_BIO)),
            company: Some("c".repeat(MAX_SHORT)),
            education: Some("e".repeat(MAX_SHORT)),
            location: Some("l".repeat(MAX_LOCATION)),
            linkedin_url: None,
            x_url: None,
            website_url: None,
        };

        assert!(edit.checked().is_ok());
    }

    #[test]
    fn location_has_its_own_cap_and_its_own_refusal() {
        // `varchar(120)`, not the 255 the other short fields share, so the
        // reader is told the number their column actually holds.
        let edit = EditProfile {
            location: Some("l".repeat(MAX_LOCATION + 1)),
            ..EditProfile::default()
        };

        assert_eq!(
            edit.checked().unwrap_err(),
            super::super::refusal::Refusal::LocationTooLong
        );
    }

    #[test]
    fn every_link_field_is_checked_and_not_only_linkedin() {
        let fields: [fn(String) -> EditProfile; 3] = [
            |v| EditProfile {
                linkedin_url: Some(v),
                ..EditProfile::default()
            },
            |v| EditProfile {
                x_url: Some(v),
                ..EditProfile::default()
            },
            |v| EditProfile {
                website_url: Some(v),
                ..EditProfile::default()
            },
        ];

        for field in fields {
            assert_eq!(
                field("javascript:alert(1)".to_owned())
                    .checked()
                    .unwrap_err(),
                super::super::refusal::Refusal::NotALink
            );
            assert!(
                field("https://example.com/a".to_owned()).checked().is_ok()
            );
        }
    }

    #[test]
    fn nothing_a_reader_sends_can_set_their_username() {
        // The one field on the row that other things may point at. It is not
        // on `EditProfile` at all, so this is a statement about the type
        // rather than about a branch somebody could remove.
        let written = EditProfile {
            tagline: Some("hi".to_owned()),
            ..EditProfile::default()
        }
        .checked()
        .unwrap();

        assert!(written.username.is_none());
        assert!(written.github_username.is_none());
    }
}

/// Whether this reader is on the newsletter.
///
/// One boolean, and deliberately one. The page used to offer a checkbox per
/// kind of email — new books, platform changes — against no column and no
/// sender that read them; there is one list at Kit and one thing to say about
/// it, which is whether a reader is on it.
#[derive(Debug, Serialize)]
pub(crate) struct NewsletterView {
    pub(crate) subscribed: bool,
}

/// The toggle, on its way in.
#[derive(Debug, Deserialize)]
pub(crate) struct EditNewsletter {
    pub(crate) subscribed: bool,
}
