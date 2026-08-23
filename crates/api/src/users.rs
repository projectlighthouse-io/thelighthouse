//! Turning a social profile into a `users` row.
//!
//! A port of the laravel app's `FindOrCreateUserFromSocial`, kept faithful on
//! purpose: both stacks run against the same database during the crossover, and
//! two rules for "is this the same person" is how one reader ends up with two
//! accounts and half their purchases on each.
//!
//! Three branches, in order:
//!
//! 1. a row already carries this provider's id — that is the reader;
//! 2. a row carries this email — attach the provider id to it, which is what
//!    lets one person sign in with either Google or GitHub and land in the same
//!    account;
//! 3. otherwise create one.
//!
//! **`email` is the link key, and the column is `NOT NULL`.** A provider that
//! returns no address cannot complete sign-in — [`Error::NoEmail`] — because the
//! alternative is a fabricated address that silently stops being the same person
//! next time.

use loginwith::{Provider, SocialUser};
use sqlx::postgres::PgPool;

/// How many times an insert may lose the username race before giving up.
///
/// [`unique_username`] checks and then inserts, which is not atomic — two
/// sign-ins picking the same free name in the same instant is a real, if rare,
/// event. `users_username_unique` is the guard that actually holds; this turns
/// hitting it into a retry rather than a 500.
const USERNAME_ATTEMPTS: u8 = 3;

/// How far the `.1`, `.2` … search goes before it stops asking the database.
const SUFFIX_LIMIT: u32 = 100;

/// What the frontend renders chrome from, and what a session points at.
#[derive(Clone, Debug)]
pub(crate) struct User {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) avatar: Option<String>,
}

/// A tuple in the column order every query below selects.
type UserRow = (i64, String, String, Option<String>);

const COLUMNS: &str = "id, name, email, avatar_url";

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

#[derive(Debug)]
pub(crate) enum Error {
    /// The provider shared no email address. GitHub does this when every
    /// address on the account is private and the `user:email` scope was
    /// declined. Not a failure to report as a bug — the reader has to be told
    /// what to do about it.
    NoEmail,
    /// Every attempt at a new row lost the username race. Rare enough that
    /// seeing it means something is wrong with the search, not with the reader.
    UsernameRace,
    Database(sqlx::Error),
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

/// The column holding a provider's own id.
///
/// Interpolated into the SQL below rather than bound, because a column name
/// cannot be a bind parameter. That is safe *here and only here*: the value
/// comes from a two-variant enum and is a `&'static str`, so no reader input
/// can reach it. A `&str` parameter in this position would be an injection.
const fn provider_column(provider: Provider) -> &'static str {
    match provider {
        Provider::Github => "github_id",
        Provider::Google => "google_id",
    }
}

/// # Errors
///
/// [`Error::NoEmail`] when the provider shared no address, otherwise whatever
/// the database said.
pub(crate) async fn find_or_create(
    pool: &PgPool,
    provider: Provider,
    social: &SocialUser,
) -> Result<User, Error> {
    let email = social.email.as_deref().ok_or(Error::NoEmail)?;
    let column = provider_column(provider);
    let github = provider == Provider::Github;

    // 1. Known provider id. `users_{provider}_id_unique` makes this an index
    //    lookup, and the common case one query.
    let found = sqlx::query_as::<_, UserRow>(&format!(
        "SELECT {COLUMNS} FROM users WHERE {column} = $1"
    ))
    .bind(&social.id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = found {
        return Ok(User::from(row));
    }

    // 2. Known email, unknown provider — the cross-provider link. An UPDATE
    //    that matches nothing returns no row, which is exactly "no account with
    //    that address", so the read and the write are one round trip.
    let mut attach = format!(
        "UPDATE users SET {column} = $2, avatar_url = $3, updated_at = NOW()"
    );
    if github {
        // Only for GitHub, matching the laravel action: Google has no
        // equivalent of `login`, so there is nothing to write.
        attach.push_str(", github_username = $4");
    }
    attach.push_str(" WHERE email = $1 RETURNING ");
    attach.push_str(COLUMNS);

    let mut query = sqlx::query_as::<_, UserRow>(&attach)
        .bind(email)
        .bind(&social.id)
        .bind(social.avatar.as_deref());
    if github {
        query = query.bind(social.nickname.as_deref());
    }

    if let Some(row) = query.fetch_optional(pool).await? {
        return Ok(User::from(row));
    }

    // 3. Nobody yet.
    create(pool, provider, social, email).await
}

/// The reader behind a session id, for the endpoint that answers "who is this".
///
/// # Errors
///
/// Whatever the database said. A missing row is `Ok(None)` — a session pointing
/// at a deleted user is a signed-out reader, not an error.
pub(crate) async fn find(
    pool: &PgPool,
    id: i64,
) -> Result<Option<User>, sqlx::Error> {
    Ok(sqlx::query_as::<_, UserRow>(&format!(
        "SELECT {COLUMNS} FROM users WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?
    .map(User::from))
}

async fn create(
    pool: &PgPool,
    provider: Provider,
    social: &SocialUser,
    email: &str,
) -> Result<User, Error> {
    let column = provider_column(provider);
    let github = provider == Provider::Github;

    // GitHub leaves `name` null far more often than `login`, and a reader with
    // neither still needs something to be called.
    let name = present(social.name.as_deref())
        .or_else(|| present(social.nickname.as_deref()))
        .unwrap_or("User");

    // The GitHub handle is already a username somebody chose; a display name is
    // not. Prefer it when there is one.
    let source = match present(social.nickname.as_deref()) {
        Some(nickname) if github => nickname,
        _ => name,
    };

    let nickname = github.then_some(social.nickname.as_deref()).flatten();

    // `email_verified_at` is set because a provider only hands over an address
    // it verified itself. `password` is left null: there is no password login.
    let insert = format!(
        "INSERT INTO users (name, username, email, {column}, avatar_url, github_username, \
         email_verified_at, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW(), NOW()) RETURNING {COLUMNS}"
    );

    for _ in 0..USERNAME_ATTEMPTS {
        let username = unique_username(pool, source).await?;

        let row = sqlx::query_as::<_, UserRow>(&insert)
            .bind(name)
            .bind(&username)
            .bind(email)
            .bind(&social.id)
            .bind(social.avatar.as_deref())
            .bind(nickname)
            .fetch_one(pool)
            .await;

        match row {
            Ok(row) => return Ok(User::from(row)),
            // Somebody took the name between the check and the insert. Ask
            // again — the winner's row is committed now, so the next search
            // steps past it.
            Err(sqlx::Error::Database(error))
                if error.constraint() == Some("users_username_unique") =>
            {
                tracing::info!(%error, "lost a username race, retrying");
            }
            Err(error) => return Err(error.into()),
        }
    }

    Err(Error::UsernameRace)
}

/// The first free username derived from `source`, as the laravel action picks
/// it: the slug, then `slug.1`, `slug.2` …
///
/// Past [`SUFFIX_LIMIT`] it stops counting and appends randomness instead. The
/// loop is one query per attempt, so an unbounded one is a way to hang a
/// sign-in on a popular name.
async fn unique_username(
    pool: &PgPool,
    source: &str,
) -> Result<String, sqlx::Error> {
    let base = slugify(source);
    let base = if base.is_empty() { "user" } else { &base };

    for counter in 0..SUFFIX_LIMIT {
        let candidate = if counter == 0 {
            base.to_owned()
        } else {
            format!("{base}.{counter}")
        };

        if !exists(pool, &candidate).await? {
            return Ok(candidate);
        }
    }

    // Not checked, deliberately: `users_username_unique` and the insert retry
    // above are the real guard, and another query here would only narrow a
    // window it cannot close.
    let suffix = loginwith::random_state().unwrap_or_default();

    Ok(format!("{base}.{}", suffix.get(..8).unwrap_or("0")))
}

async fn exists(pool: &PgPool, username: &str) -> Result<bool, sqlx::Error> {
    let (found,) = sqlx::query_as::<_, (bool,)>(
        "SELECT EXISTS (SELECT 1 FROM users WHERE username = $1)",
    )
    .bind(username)
    .fetch_one(pool)
    .await?;

    Ok(found)
}

/// `Some` only for a value with something in it. PHP's truthiness treats `""`
/// and `null` alike, and the action being ported relies on that — an empty
/// GitHub `login` has to fall through to the display name, not become an empty
/// username.
fn present(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// A name, as a username.
///
/// Lowercase, drop anything outside `[a-z0-9 .]`, whitespace and underscores to
/// `.`, collapse repeated dots, trim them off the ends. That is
/// `GenerateUniqueUsername::slugify`, with one deliberate difference:
///
/// **Non-ASCII characters are dropped rather than transliterated.** Laravel runs
/// `Str::ascii` first, so `José` becomes `jose` there and `jos` here, and a name
/// written entirely in another script becomes empty and falls back to `user`.
/// Doing better means a transliteration table for every script, which is a
/// dependency bought for a default the reader can change on their profile.
/// ponytail: if a mangled username is ever reported, `deunicode` is the fix and
/// slots in on the line below.
fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());

    for c in name.chars().flat_map(char::to_lowercase) {
        match c {
            'a'..='z' | '0'..='9' | '.' => out.push(c),
            '_' => out.push('.'),
            c if c.is_whitespace() => out.push('.'),
            // Everything else, including every non-ascii character, is removed
            // rather than replaced — a separator here would turn `Ali (Dhaka)`
            // into `ali..dhaka.` and then, after collapsing, into `ali.dhaka`,
            // which reads as a name nobody typed.
            _ => {}
        }
    }

    // Collapse runs of dots, then trim them. One pass, because the two rules
    // interact: `. a . . b .` has to end up `a.b`.
    let mut slug = String::with_capacity(out.len());
    let mut last_was_dot = true;

    for c in out.chars() {
        if c == '.' {
            if !last_was_dot {
                slug.push('.');
            }
            last_was_dot = true;
        } else {
            slug.push(c);
            last_was_dot = false;
        }
    }

    slug.trim_end_matches('.').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_becomes_a_dotted_lowercase_slug() {
        assert_eq!(slugify("Aryan Ahmed"), "aryan.ahmed");
        assert_eq!(slugify("octocat"), "octocat");
        assert_eq!(slugify("Ada  Lovelace"), "ada.lovelace");
        assert_eq!(slugify("first_last"), "first.last");
    }

    #[test]
    fn punctuation_is_removed_rather_than_separated() {
        assert_eq!(slugify("O'Brien"), "obrien");
        assert_eq!(slugify("Ali (Dhaka)"), "ali.dhaka");
        assert_eq!(slugify("dr. who"), "dr.who");
    }

    #[test]
    fn repeated_and_edge_dots_collapse() {
        assert_eq!(slugify("...a...b..."), "a.b");
        assert_eq!(slugify(".leading"), "leading");
        assert_eq!(slugify("trailing."), "trailing");
        assert_eq!(slugify("  spaced  out  "), "spaced.out");
    }

    #[test]
    fn a_name_that_slugifies_to_nothing_is_empty_so_the_caller_falls_back() {
        // `unique_username` turns each of these into "user".
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("!!!"), "");
        assert_eq!(slugify("..."), "");
        assert_eq!(slugify("   "), "");
    }

    #[test]
    fn digits_survive_because_they_are_valid_in_a_username() {
        assert_eq!(slugify("user42"), "user42");
        assert_eq!(slugify("Web 2.0"), "web.2.0");
    }

    #[test]
    fn an_empty_provider_field_reads_as_absent() {
        assert_eq!(present(Some("octocat")), Some("octocat"));
        assert_eq!(present(Some("  padded  ")), Some("padded"));
        assert_eq!(present(Some("")), None);
        assert_eq!(present(Some("   ")), None);
        assert_eq!(present(None), None);
    }

    #[test]
    fn each_provider_has_its_own_column() {
        assert_eq!(provider_column(Provider::Github), "github_id");
        assert_eq!(provider_column(Provider::Google), "google_id");
    }
}
