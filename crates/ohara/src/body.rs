//! A lesson's markdown: where the paywall cuts it, and rendering both halves.

use comrak::{Options, markdown_to_html};

/// Where a lesson stops being free.
///
/// An html comment, so it renders as nothing if anything ever passes the raw
/// markdown through a renderer that does not know about it — a visible
/// `[paywall]` in a published lesson is worse than a missed split.
///
/// Matched on its own line and trimmed, so trailing whitespace in an editor
/// does not silently stop it being a marker.
pub const PAYWALL: &str = "<!-- paywall -->";

/// Whether a whole lesson is paid, stated in `lesson.yaml`.
///
/// Two answers, because the marker already covers the third. A lesson is paid
/// as a whole or it is not, and *which parts* of a lesson that is not are
/// withheld is the marker's job — including "all of it", which is a marker on
/// the first line.
///
/// So this exists for one thing the markdown cannot say without being edited:
/// pricing a finished lesson is a yaml change, not a change to the writing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Access {
    /// Not paywalled as a whole. The marker decides what, if anything, is
    /// withheld: no marker is a wholly free lesson, a marker mid-lesson splits
    /// it, and a marker on the first line withholds all of it.
    ///
    /// The default, and what every lesson written before this field existed
    /// already meant — so adding it changed nothing.
    #[default]
    Free,
    /// Paid in full, with no marker needed. The reader gets the metadata and
    /// none of the prose.
    Paid,
}

/// A lesson split at the paywall.
///
/// Three shapes, and the marker's position is what picks between them:
///
/// | marker      | free            | paid          |
/// |-------------|-----------------|---------------|
/// | absent      | the whole thing | none          |
/// | mid-lesson  | above it        | below it      |
/// | first line  | none            | the whole thing |
///
/// The lesson endpoint returns `free` to everyone and `paid` only to a reader
/// who is entitled, so `paid` must never reach a response that a cache may hold
/// — see `docs/rebuild.md`.
#[derive(Debug, PartialEq, Eq)]
pub struct Body {
    pub free: String,
    pub paid: Option<String>,
}

impl Body {
    /// The body a reader may be served, under this lesson's access.
    ///
    /// `Free` and `Paid` ignore the marker rather than erroring on it: the
    /// yaml is the more explicit statement of the two, so it wins, and a
    /// leftover marker in a lesson someone priced cannot quietly re-open half
    /// of it.
    #[must_use]
    pub fn under(markdown: &str, access: Access) -> Self {
        match access {
            Access::Free => Self::split(markdown),
            // Empty free half, exactly as a marker on the first line gives.
            // The lesson still exists — title, description and neighbours all
            // come from the yaml — but none of its prose is served.
            Access::Paid => Self {
                free: String::new(),
                paid: Some(markdown.trim().to_owned()),
            },
        }
    }

    /// Splits raw markdown at the marker. Nothing is rendered here.
    #[must_use]
    pub fn split(markdown: &str) -> Self {
        let Some((free, paid)) = cut(markdown) else {
            return Self {
                free: markdown.trim().to_owned(),
                paid: None,
            };
        };

        Self {
            free: free.trim().to_owned(),
            paid: Some(paid.trim().to_owned()),
        }
    }

    /// Whether a reader who is not entitled is missing anything.
    ///
    /// A marker with nothing after it is not a paywall, so an editor who leaves
    /// one at the end of a draft does not accidentally show a "read the rest"
    /// call to action that leads to no rest.
    #[must_use]
    pub fn has_paid_part(&self) -> bool {
        self.paid.as_ref().is_some_and(|paid| !paid.is_empty())
    }
}

/// The marker's own line, split away from both halves.
fn cut(markdown: &str) -> Option<(&str, &str)> {
    let at = markdown
        .lines()
        .scan(0_usize, |offset, line| {
            let start = *offset;
            *offset += line.len() + 1;
            Some((start, line))
        })
        .find(|(_, line)| line.trim() == PAYWALL)?;

    let (start, line) = at;

    Some((
        &markdown[..start],
        &markdown[(start + line.len()).min(markdown.len())..],
    ))
}

/// Markdown to html, GitHub flavoured.
///
/// The options match the laravel app's `GithubFlavoredMarkdownExtension`, so a
/// lesson written for that renderer produces the same html here. Raw html in
/// the source is *not* enabled: the content repo is trusted, but a renderer
/// that passes html through is one script tag away from being the reason a
/// paywalled page leaks.
#[must_use]
pub fn render(markdown: &str) -> String {
    let mut options = Options::default();

    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.footnotes = true;

    markdown_to_html(markdown, &options)
}

/// A heading in the table of contents, and the anchor it scrolls to.
#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct Heading {
    pub id: String,
    pub text: String,
}

/// The `##` headings, in order.
///
/// Only `##`: a lesson's `#` is its title, which the page already shows, and
/// `###` would make the contents longer than the section list is useful.
///
/// Fenced code is skipped, because `# comment` inside a shell block is not a
/// heading and a contents list full of them is worse than none.
#[must_use]
pub fn headings(markdown: &str) -> Vec<Heading> {
    let mut fenced = false;

    markdown
        .lines()
        .filter(|line| {
            if line.trim_start().starts_with("```") {
                fenced = !fenced;
            }
            !fenced
        })
        .filter_map(|line| line.strip_prefix("## "))
        .map(|text| Heading {
            id: anchor(text),
            text: text.trim().to_owned(),
        })
        .collect()
}

/// A heading's anchor, matching what the frontend generated before rust took
/// this over — lowercase, punctuation dropped, spaces to hyphens.
fn anchor(heading: &str) -> String {
    heading
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace() || *c == '-')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

/// Minutes to read, at 220 words a minute, and never zero.
///
/// Counts the markdown rather than the html, so tags are not words. It is an
/// estimate on the page and treated as one.
#[must_use]
pub fn read_minutes(markdown: &str) -> usize {
    let words = markdown.split_whitespace().count();

    (words.div_ceil(220)).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARKED: &str = "Free part.\n\n<!-- paywall -->\n\nPaid part.";

    #[test]
    fn free_lets_the_marker_decide_what_is_withheld() {
        let body = Body::under(MARKED, Access::Free);

        assert_eq!(body.free, "Free part.");
        assert_eq!(body.paid.as_deref(), Some("Paid part."));
    }

    #[test]
    fn free_with_no_marker_withholds_nothing() {
        let body = Body::under("Just prose.", Access::Free);

        assert_eq!(body.free, "Just prose.");
        assert_eq!(body.paid, None);
    }

    #[test]
    fn a_marker_on_the_first_line_is_the_same_as_declaring_it_paid() {
        // Why there is no third state: the markdown can already say this.
        let by_marker =
            Body::under(&format!("{PAYWALL}\nAll of it."), Access::Free);
        let by_yaml = Body::under("All of it.", Access::Paid);

        assert!(by_marker.free.is_empty());
        assert_eq!(by_marker.paid.as_deref(), Some("All of it."));
        assert_eq!(by_marker, by_yaml);
    }

    #[test]
    fn paid_withholds_every_word_of_it() {
        let body = Body::under(MARKED, Access::Paid);

        assert!(body.free.is_empty());
        assert_eq!(body.paid.as_deref(), Some(MARKED.trim()));
        assert!(!body.free.contains("Free part."));
    }

    #[test]
    fn the_yaml_spells_both_states_in_lowercase() {
        assert_eq!(
            serde_norway::from_str::<Access>("free").unwrap(),
            Access::Free
        );
        assert_eq!(
            serde_norway::from_str::<Access>("paid").unwrap(),
            Access::Paid
        );

        // A typo is a parse error naming the file, not a lesson that quietly
        // gives away prose somebody is meant to pay for.
        assert!(serde_norway::from_str::<Access>("Paid").is_err());
        assert!(serde_norway::from_str::<Access>("partial").is_err());
    }

    #[test]
    fn absent_is_free_so_nothing_already_written_changed() {
        assert_eq!(Access::default(), Access::Free);
    }

    #[test]
    fn a_lesson_with_no_marker_is_wholly_free() {
        let body = Body::split("# Title\n\nAll of it.\n");

        assert_eq!(body.free, "# Title\n\nAll of it.");
        assert_eq!(body.paid, None);
        assert!(!body.has_paid_part());
    }

    #[test]
    fn a_marker_mid_lesson_splits_there() {
        let body =
            Body::split("Free part.\n\n<!-- paywall -->\n\nPaid part.\n");

        assert_eq!(body.free, "Free part.");
        assert_eq!(body.paid.as_deref(), Some("Paid part."));
        assert!(body.has_paid_part());
    }

    #[test]
    fn a_marker_on_the_first_line_leaves_nothing_free() {
        let body = Body::split("<!-- paywall -->\n\nAll of it is paid.\n");

        assert!(body.free.is_empty());
        assert_eq!(body.paid.as_deref(), Some("All of it is paid."));
        assert!(body.has_paid_part());
    }

    #[test]
    fn a_marker_with_nothing_after_it_is_not_a_paywall() {
        // An editor left it at the end of a draft. Showing a "read the rest"
        // call to action here would lead to no rest.
        let body = Body::split("Everything.\n\n<!-- paywall -->\n");

        assert_eq!(body.free, "Everything.");
        assert!(!body.has_paid_part());
    }

    #[test]
    fn indentation_around_the_marker_does_not_hide_it() {
        let body = Body::split("Free.\n\n   <!-- paywall -->   \n\nPaid.\n");

        assert_eq!(body.free, "Free.");
        assert_eq!(body.paid.as_deref(), Some("Paid."));
    }

    #[test]
    fn only_the_first_marker_cuts() {
        let body = Body::split("A\n<!-- paywall -->\nB\n<!-- paywall -->\nC\n");

        assert_eq!(body.free, "A");
        // The second marker stays in the paid half rather than splitting again:
        // a lesson has one paywall, and silently dropping prose between two
        // markers would lose it.
        assert!(body.paid.as_deref().unwrap().contains("<!-- paywall -->"));
    }

    #[test]
    fn the_marker_itself_never_survives_into_either_half() {
        let body = Body::split("Free.\n\n<!-- paywall -->\n\nPaid.\n");

        assert!(!body.free.contains(PAYWALL));
        assert!(!body.paid.as_deref().unwrap().contains(PAYWALL));
    }

    #[test]
    fn github_flavoured_markdown_renders_as_the_laravel_app_renders_it() {
        let html = render("| a | b |\n|---|---|\n| 1 | 2 |\n\n~~gone~~\n");

        assert!(html.contains("<table>"), "tables are gfm: {html}");
        assert!(html.contains("<del>"), "strikethrough is gfm: {html}");
    }

    #[test]
    fn raw_html_in_the_source_is_not_passed_through() {
        let html = render("<script>alert(1)</script>\n");

        assert!(!html.contains("<script>"), "{html}");
    }

    #[test]
    fn contents_are_the_h2s_with_anchors() {
        let found =
            headings("# Title\n\n## First Part\n\ntext\n\n## And Then?\n");

        assert_eq!(
            found,
            [
                Heading {
                    id: "first-part".to_owned(),
                    text: "First Part".to_owned()
                },
                Heading {
                    id: "and-then".to_owned(),
                    text: "And Then?".to_owned()
                },
            ]
        );
    }

    #[test]
    fn a_comment_inside_a_code_fence_is_not_a_heading() {
        let found = headings(
            "## Real\n\n```sh\n## not a heading\n```\n\n## Also Real\n",
        );

        assert_eq!(found.len(), 2, "{found:?}");
    }

    #[test]
    fn reading_time_rounds_up_and_is_never_zero() {
        assert_eq!(read_minutes(""), 1);
        assert_eq!(read_minutes("one two three"), 1);
        assert_eq!(read_minutes(&"word ".repeat(220)), 1);
        assert_eq!(read_minutes(&"word ".repeat(221)), 2);
    }
}
