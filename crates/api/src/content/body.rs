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
pub(crate) const PAYWALL: &str = "<!-- paywall -->";

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
pub(crate) struct Body {
    pub(crate) free: String,
    pub(crate) paid: Option<String>,
}

impl Body {
    /// Splits raw markdown at the marker. Nothing is rendered here.
    pub(crate) fn split(markdown: &str) -> Self {
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
    pub(crate) fn has_paid_part(&self) -> bool {
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
pub(crate) fn render(markdown: &str) -> String {
    let mut options = Options::default();

    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.footnotes = true;

    markdown_to_html(markdown, &options)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
