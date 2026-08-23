//! Choosing a username nobody else has.
//!
//! A port of `GenerateUniqueUsername`: slugify the name, then walk `.1`, `.2` …
//! until one is free. The walk races under concurrency — `users_username_unique`
//! is the guard that actually holds, and `super::store` retries when it fires.

use sqlx::postgres::PgPool;

/// How far the `.1`, `.2` … search goes before it stops asking the database.
pub(crate) const SUFFIX_LIMIT: u32 = 100;

/// The first free username derived from `source`, as the laravel action picks
/// it: the slug, then `slug.1`, `slug.2` …
///
/// Past [`SUFFIX_LIMIT`] it stops counting and appends randomness instead. The
/// loop is one query per attempt, so an unbounded one is a way to hang a
/// sign-in on a popular name.
pub(crate) async fn choose(
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
}
