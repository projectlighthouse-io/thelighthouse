//! What an author sends, and whether it is acceptable.
//!
//! Decided without touching the database, as `notes::payload` is.

use serde::Deserialize;

use super::refusal::Refusal;

/// How many topics one article may carry.
///
/// Not a limit the database enforces — the join table would take any number.
/// It is editorial: an article tagged with half the eight topics has not said
/// anything about itself, and the listing filter stops meaning much.
pub(crate) const MAX_TOPICS: usize = 3;

/// Every topic an article may be filed under.
///
/// A constant, not a table and not a database enum. It is short, it changes
/// about once a year, and every alternative makes adding "wasm" a migration
/// and a deploy instead of a line here. The wire values are lowercase and
/// stable — they are in urls (`/blog?category=rust`) and a client branches on
/// them, so they are as much a contract as a field name. What a topic is
/// *called* on the page is the frontend's, keyed by these.
pub(crate) const CATEGORIES: [&str; 8] = [
    "go",
    "rust",
    "containers",
    "systems",
    "docker",
    "kubernetes",
    "networking",
    "dsa",
];

/// Characters, matching `articles.title` — which is `varchar(200)`, and
/// postgres counts that in characters, so the two limits are the same number
/// and mean the same thing.
pub(crate) const MAX_TITLE: usize = 200;

/// Characters, matching `articles.subtitle`, which is also `varchar(200)`.
///
/// The line a reader decides from on the listing. Required — an article with
/// no subtitle falls back to a machine-made summary of how it opens, which is
/// what the column exists to replace.
pub(crate) const MAX_SUBTITLE: usize = 200;

/// Characters. The column is `text` and would take far more; this is
/// editorial. Long enough for anything anybody sits down and writes, short
/// enough that a paste of something else entirely is refused rather than
/// stored.
pub(crate) const MAX_BODY: usize = 50_000;

/// How much of the title survives into the slug, before the suffix.
///
/// The column takes 255 and the suffix is seven more; this is well short of
/// both, because the rest of a very long title adds nothing to a url anybody
/// reads or shares.
const SLUG_TITLE: usize = 60;

/// Hex characters of randomness on the end of every slug.
///
/// Not for secrecy — the slug is public. It is what lets two people both title
/// something "Getting started" without a retry loop around the insert, and
/// twenty-four bits is enough for that at this scale: the unique index is
/// still the thing that guarantees it, and a collision is a refused write
/// rather than a wrong one.
const SLUG_SUFFIX: usize = 6;

/// What an author may send when publishing.
///
/// **No `user_id` and no `slug`.** The author comes from the session and the
/// slug is minted here, so there is no field for a caller to file an article
/// under somebody else's name or to claim a url that is already somebody's.
#[derive(Debug, Deserialize)]
///
/// Every field defaults to empty, so a missing key is refused by that field's
/// own check rather than as a body that could not be read.
pub(crate) struct NewArticleRequest {
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) subtitle: String,
    /// What the article is about — one at least, [`MAX_TOPICS`] at most.
    #[serde(default)]
    pub(crate) topics: Vec<String>,
    /// Markdown, as typed.
    #[serde(default)]
    pub(crate) body: String,
}

/// What an edit may touch. The same three fields, and deliberately not the
/// slug: a url that changed when its title did would break every link to it.
#[derive(Debug, Deserialize)]
pub(crate) struct EditArticleRequest {
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) subtitle: String,
    #[serde(default)]
    pub(crate) topics: Vec<String>,
    #[serde(default)]
    pub(crate) body: String,
}

/// Narrows the listing to one topic: `?topic=rust`.
///
/// One topic, not a list. "Articles about rust" is the question a reader asks
/// from a tag; intersecting several is a search feature, and there is no
/// search page to hang it off yet.
///
/// Beside [`crate::request::ListQuery`] rather than inside it — paging and
/// search belong to every listing, and this belongs to one.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct TopicFilter {
    pub(crate) topic: Option<String>,
}

impl TopicFilter {
    /// The topic asked for, or `None` for all of them.
    ///
    /// **A topic that is not one of ours is refused, not ignored.** Dropping
    /// it would answer `/blog?category=rus` with the whole blog, which looks
    /// like working software right up until somebody wonders why their filter
    /// does nothing.
    pub(crate) fn topic(&self) -> Result<Option<&str>, Refusal> {
        let Some(topic) = self.topic.as_deref().map(str::trim) else {
            return Ok(None);
        };

        // An empty value is the same as not asking: `?topic=` is what an "all
        // topics" option in a `<select>` submits.
        if topic.is_empty() {
            return Ok(None);
        }

        validate_topic(topic).map(Some)
    }
}

/// `?author=` — one author's articles rather than everybody's.
///
/// Beside [`TopicFilter`] rather than inside it: a listing narrows by either,
/// both or neither, and one struct with two `Option`s would make "which of
/// these did the caller send" a question with four answers.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct AuthorFilter {
    pub(crate) author: Option<String>,
}

impl AuthorFilter {
    /// The username asked for, or `None` for everybody's.
    ///
    /// **A username nobody has is not refused.** Topics are a fixed list, so
    /// `?topic=rus` is a typo the api can name; usernames are not, and the
    /// only thing refusing one would prove is which usernames exist.
    pub(crate) fn author(&self) -> Option<&str> {
        self.author
            .as_deref()
            .map(str::trim)
            .filter(|author| !author.is_empty())
    }
}

/// A checked article, borrowing the request it came from.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ValidArticle<'a> {
    pub(crate) title: &'a str,
    pub(crate) subtitle: &'a str,
    /// Deduplicated, and each one spelled as [`CATEGORIES`] spells it.
    /// `'static` because they are borrowed from that constant rather than
    /// from the request — see [`validate_topics`].
    pub(crate) topics: Vec<&'static str>,
    pub(crate) body: &'a str,
}

/// What an author sent, trimmed and checked.
///
/// Whitespace-only is refused rather than accepted as "present", and both
/// limits are in characters rather than bytes — a 200-character Bengali title
/// is 200, not the 600 bytes it stores as.
pub(crate) fn validate_article<'a>(
    title: &'a str,
    subtitle: &'a str,
    topics: &[String],
    body: &'a str,
) -> Result<ValidArticle<'a>, Vec<Refusal>> {
    let title = title.trim();
    let subtitle = subtitle.trim();
    let body = body.trim();

    // Every field checked, not only up to the first failure, so the editor can
    // mark all of them at once.
    let mut refused = Vec::new();

    if title.is_empty() {
        refused.push(Refusal::EmptyTitle);
    } else if title.chars().count() > MAX_TITLE {
        refused.push(Refusal::TitleTooLong);
    }

    if subtitle.is_empty() {
        refused.push(Refusal::EmptySubtitle);
    } else if subtitle.chars().count() > MAX_SUBTITLE {
        refused.push(Refusal::SubtitleTooLong);
    }

    let topics = validate_topics(topics)
        .map_err(|cause| refused.push(cause))
        .unwrap_or_default();

    if body.is_empty() {
        refused.push(Refusal::EmptyBody);
    } else if body.chars().count() > MAX_BODY {
        refused.push(Refusal::BodyTooLong);
    }

    if !refused.is_empty() {
        return Err(refused);
    }

    Ok(ValidArticle {
        title,
        subtitle,
        topics,
        body,
    })
}

/// The topics an author sent, checked and deduplicated.
///
/// Order is the author's, minus repeats — the first spelling of a topic wins
/// and later ones are dropped rather than refused, because `["go", "Go"]` is a
/// double click rather than a mistake worth a 400. The count is checked
/// *after* deduplicating, so five clicks on the same chip is not "too many".
fn validate_topics(topics: &[String]) -> Result<Vec<&'static str>, Refusal> {
    let mut checked: Vec<&'static str> = Vec::new();

    for topic in topics {
        let known = validate_topic(topic)?;

        if !checked.contains(&known) {
            checked.push(known);
        }
    }

    if checked.is_empty() {
        return Err(Refusal::NoTopics);
    }

    if checked.len() > MAX_TOPICS {
        return Err(Refusal::TooManyTopics);
    }

    Ok(checked)
}

/// One of [`CATEGORIES`], as the constant spells it.
///
/// The *borrowed* match is returned rather than what was sent, so `"Rust"` is
/// stored as `rust` — a column holding two spellings of one topic is a filter
/// that silently misses half the rows.
fn validate_topic(topic: &str) -> Result<&'static str, Refusal> {
    let topic = topic.trim();

    CATEGORIES
        .into_iter()
        .find(|known| known.eq_ignore_ascii_case(topic))
        .ok_or(Refusal::UnknownCategory)
}

/// The url an article is addressed by: the title, then randomness.
///
/// Minted once at publish and never again, so retitling breaks no link. The
/// suffix is always there — including when the title slugs to nothing at all,
/// which is what a title written entirely in a script this does not
/// transliterate does — so the result is never empty and never collides with
/// a literal route segment like `mine`.
pub(crate) fn slugify(title: &str, entropy: &str) -> String {
    let mut slug = String::with_capacity(SLUG_TITLE + 1 + SLUG_SUFFIX);
    let mut dash = false;

    for c in title.chars() {
        if slug.len() >= SLUG_TITLE {
            break;
        }

        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            dash = false;
        } else if !slug.is_empty() && !dash {
            // One dash for any run of anything else, and never a leading one.
            slug.push('-');
            dash = true;
        }
    }

    // A trailing dash would make the suffix read as part of the title.
    while slug.ends_with('-') {
        slug.pop();
    }

    if !slug.is_empty() {
        slug.push('-');
    }

    slug.extend(entropy.chars().filter(char::is_ascii).take(SLUG_SUFFIX));

    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    fn topics(list: &[&str]) -> Vec<String> {
        list.iter().map(|t| (*t).to_owned()).collect()
    }

    #[test]
    fn an_article_is_trimmed_and_must_have_something_in_it() {
        let checked = validate_article(
            "  Fork  ",
            "a subtitle",
            &topics(&["go"]),
            "  words  ",
        )
        .unwrap();

        assert_eq!(checked.title, "Fork");
        assert_eq!(checked.body, "words");
        assert_eq!(checked.topics, vec!["go"]);

        assert_eq!(
            validate_article("   ", "a subtitle", &topics(&["go"]), "body"),
            Err(vec![Refusal::EmptyTitle])
        );
        assert_eq!(
            validate_article("title", "a subtitle", &topics(&["go"]), "\n\t"),
            Err(vec![Refusal::EmptyBody])
        );
    }

    #[test]
    fn both_limits_are_measured_in_characters_not_bytes() {
        let go = topics(&["go"]);

        assert!(
            validate_article(&"a".repeat(MAX_TITLE), "a subtitle", &go, "b")
                .is_ok()
        );
        assert_eq!(
            validate_article(
                &"a".repeat(MAX_TITLE + 1),
                "a subtitle",
                &go,
                "b"
            ),
            Err(vec![Refusal::TitleTooLong])
        );

        assert!(
            validate_article("t", "a subtitle", &go, &"b".repeat(MAX_BODY))
                .is_ok()
        );
        assert_eq!(
            validate_article("t", "a subtitle", &go, &"b".repeat(MAX_BODY + 1)),
            Err(vec![Refusal::BodyTooLong])
        );

        // Three bytes each in utf-8, so this passes only if characters count.
        let bengali = "\u{986}".repeat(MAX_TITLE);
        assert!(
            bengali.len() > MAX_TITLE,
            "the fixture stopped being multibyte"
        );
        assert!(validate_article(&bengali, "a subtitle", &go, "b").is_ok());
    }

    /// The whole point of the join table: an article about running go in a
    /// container is about both, and does not have to pick.
    #[test]
    fn an_article_must_have_a_subtitle() {
        assert_eq!(
            validate_article("t", "   ", &topics(&["go"]), "b"),
            Err(vec![Refusal::EmptySubtitle])
        );
        assert_eq!(
            validate_article("t", "", &topics(&["go"]), "b"),
            Err(vec![Refusal::EmptySubtitle])
        );
    }

    #[test]
    fn a_subtitle_is_trimmed_and_capped_in_characters() {
        let checked =
            validate_article("t", "  a line  ", &topics(&["go"]), "b").unwrap();
        assert_eq!(checked.subtitle, "a line");

        assert!(
            validate_article(
                "t",
                &"s".repeat(MAX_SUBTITLE),
                &topics(&["go"]),
                "b"
            )
            .is_ok()
        );
        assert_eq!(
            validate_article(
                "t",
                &"s".repeat(MAX_SUBTITLE + 1),
                &topics(&["go"]),
                "b"
            ),
            Err(vec![Refusal::SubtitleTooLong])
        );

        // Characters, not bytes — the column counts the same way.
        let bengali = "\u{986}".repeat(MAX_SUBTITLE);
        assert!(bengali.len() > MAX_SUBTITLE);
        assert!(validate_article("t", &bengali, &topics(&["go"]), "b").is_ok());
    }

    #[test]
    fn an_article_can_be_about_several_things() {
        let checked = validate_article(
            "Go in a container",
            "a subtitle",
            &topics(&["go", "docker", "containers"]),
            "body",
        )
        .unwrap();

        assert_eq!(checked.topics, vec!["go", "docker", "containers"]);
    }

    #[test]
    fn an_article_must_be_about_something() {
        assert_eq!(
            validate_article("t", "a subtitle", &topics(&[]), "b"),
            Err(vec![Refusal::NoTopics])
        );
    }

    #[test]
    fn a_repeated_topic_is_dropped_rather_than_refused() {
        // A double click, not a mistake worth a 400.
        let checked = validate_article(
            "t",
            "a subtitle",
            &topics(&["go", "Go", " go "]),
            "b",
        )
        .unwrap();

        assert_eq!(checked.topics, vec!["go"]);
    }

    #[test]
    fn the_count_is_checked_after_deduplicating() {
        // Five clicks on one chip is one topic, not "too many".
        let same = topics(&["go", "go", "go", "go", "go"]);
        assert!(validate_article("t", "a subtitle", &same, "b").is_ok());

        let at_limit = topics(&["go", "rust", "docker"]);
        assert!(validate_article("t", "a subtitle", &at_limit, "b").is_ok());

        let many = topics(&["go", "rust", "docker", "systems"]);
        assert_eq!(
            validate_article("t", "a subtitle", &many, "b"),
            Err(vec![Refusal::TooManyTopics])
        );
    }

    #[test]
    fn a_topic_has_to_be_one_of_ours() {
        for known in CATEGORIES {
            assert_eq!(validate_topic(known), Ok(known));
        }

        assert_eq!(validate_topic("php"), Err(Refusal::UnknownCategory));
        assert_eq!(validate_topic(""), Err(Refusal::UnknownCategory));
        // Not a prefix or a substring match.
        assert_eq!(validate_topic("rus"), Err(Refusal::UnknownCategory));
        assert_eq!(
            validate_topic("rust and go"),
            Err(Refusal::UnknownCategory)
        );
    }

    #[test]
    fn one_unknown_topic_refuses_the_whole_write() {
        assert_eq!(
            validate_article("t", "a subtitle", &topics(&["go", "php"]), "b"),
            Err(vec![Refusal::UnknownCategory])
        );
    }

    #[test]
    fn a_topic_is_stored_the_way_the_constant_spells_it() {
        assert_eq!(validate_topic("  RUST "), Ok("rust"));
        assert_eq!(validate_topic("Kubernetes"), Ok("kubernetes"));
    }

    #[test]
    fn every_category_is_a_lowercase_wire_value_and_they_are_distinct() {
        let mut seen = CATEGORIES.to_vec();
        seen.sort_unstable();
        let total = seen.len();
        seen.dedup();

        assert_eq!(seen.len(), total, "two categories share a value");

        for category in CATEGORIES {
            assert!(
                category.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "{category} is not a stable wire value"
            );
            // The column is varchar(32).
            assert!(category.len() <= 32, "{category}");
        }
    }

    #[test]
    fn an_absent_or_empty_filter_asks_for_every_topic() {
        let filter = |topic: Option<&str>| TopicFilter {
            topic: topic.map(str::to_owned),
        };

        assert_eq!(filter(None).topic(), Ok(None));
        assert_eq!(filter(Some("")).topic(), Ok(None));
        assert_eq!(filter(Some("   ")).topic(), Ok(None));
    }

    #[test]
    fn a_filter_for_a_topic_that_is_not_ours_is_refused_not_ignored() {
        let filter = TopicFilter {
            topic: Some("rus".to_owned()),
        };

        assert_eq!(filter.topic(), Err(Refusal::UnknownCategory));
    }

    #[test]
    fn a_slug_is_the_title_lowercased_and_dashed() {
        assert_eq!(
            slugify("Why fork() is strange", "a3f19cdeadbeef"),
            "why-fork-is-strange-a3f19c"
        );
    }

    #[test]
    fn a_slug_has_no_run_of_dashes_and_none_at_either_end() {
        assert_eq!(
            slugify("  --Hello,  World!  ", "abcdef"),
            "hello-world-abcdef"
        );
        assert_eq!(slugify("...", "abcdef"), "abcdef");
        assert_eq!(slugify("", "abcdef"), "abcdef");
    }

    #[test]
    fn a_slug_that_would_be_empty_is_still_a_slug() {
        let slug = slugify("\u{9ac}\u{9be}\u{982}\u{9b2}\u{9be}", "abcdef");

        assert_eq!(slug, "abcdef");
        assert!(!slug.is_empty());
    }

    #[test]
    fn a_slug_never_collides_with_a_literal_route_segment() {
        assert_ne!(slugify("mine", "abcdef"), "mine");
    }

    #[test]
    fn a_very_long_title_is_cut_before_the_suffix() {
        let slug = slugify(&"word ".repeat(100), "abcdef");

        assert!(slug.len() <= SLUG_TITLE + 1 + SLUG_SUFFIX, "{slug}");
        assert!(slug.ends_with("-abcdef"), "{slug}");
    }
}
