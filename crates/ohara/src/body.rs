//! A lesson's markdown: which parts of it are paid, and rendering both halves.

use std::sync::OnceLock;

use comrak::{
    Anchorizer, Options, markdown_to_html_with_plugins, options::Plugins,
    plugins::syntect::SyntectAdapter,
};

/// Where a paid region opens and closes.
///
/// A pair, not a single cut. One marker could only ever mean "everything below
/// this", which forces the paid part to be the tail of a lesson; a region can
/// sit in the middle and let the prose resume after it — a worked example
/// withheld while the paragraphs that follow stay free.
///
/// Tag-shaped so an editor that renders the markdown shows nothing: an unknown
/// html tag disappears, whereas a bare `[paid]` would be visible in a published
/// lesson, which is worse than a missed split. Both lines are stripped before
/// anything is rendered, so neither reaches the page even when unbalanced.
///
/// Matched on its own line and trimmed, so trailing whitespace in an editor
/// does not silently stop it being a marker.
pub const PAID_OPEN: &str = "<paid>";
pub const PAID_CLOSE: &str = "</paid>";

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

/// A lesson with its paid regions lifted out.
///
/// | markdown | free | paid |
/// |---|---|---|
/// | no markers | the whole thing | none |
/// | a region in the middle | everything outside it | the region |
/// | a region spanning the file | none | the whole thing |
/// | several regions | everything outside them | all of them |
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

    /// Lifts every paid region out, leaving the rest as the free body.
    ///
    /// Regions may appear more than once and need not reach the end of the
    /// file. Each half keeps its own line order, so a lesson withholding two
    /// examples reads as one continuous free body with the examples gone,
    /// rather than as everything-after-the-first-one.
    ///
    /// **An unclosed region runs to the end of the file.** That is the safe
    /// direction: a missing `</paid>` withholds too much, which whoever wrote
    /// it sees immediately, rather than serving prose that was meant to be
    /// paid. A close with nothing open is dropped — there is no region for it
    /// to end — and dropped rather than kept so a stray tag cannot reach the
    /// rendered page.
    #[must_use]
    pub fn split(markdown: &str) -> Self {
        let mut free = Vec::new();
        let mut paid = Vec::new();
        let mut inside = false;

        for line in markdown.lines() {
            match line.trim() {
                PAID_OPEN => {
                    inside = true;
                    continue;
                }
                PAID_CLOSE => {
                    inside = false;
                    continue;
                }
                _ => {}
            }

            if inside {
                paid.push(line);
            } else {
                free.push(line);
            }
        }

        Self {
            free: free.join("\n").trim().to_owned(),
            paid: Some(paid.join("\n").trim().to_owned())
                .filter(|body| !body.is_empty()),
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

/// The syntax and theme sets, loaded once.
///
/// Building a `SyntectAdapter` parses every bundled syntax definition, which is
/// tens of milliseconds — fine once, absurd per lesson. Held in a `OnceLock` so
/// the first rendered lesson pays for it and the rest do not.
static HIGHLIGHTER: OnceLock<SyntectAdapter> = OnceLock::new();

/// The theme the code is coloured with.
///
/// A dark one, because the block it sits in is dark in both site themes — see
/// `.reader-prose pre` in `reader.css`. A light theme's tokens on that panel
/// would be unreadable, and the panel is the part that is not negotiable: code
/// reads as the machine's voice rather than the page's.
const THEME: &str = "base16-ocean.dark";

/// Markdown to html, GitHub flavoured, with code coloured.
///
/// The options match the laravel app's `GithubFlavoredMarkdownExtension`, so a
/// lesson written for that renderer produces the same html here. Raw html in
/// the source is *not* enabled: the content repo is trusted, but a renderer
/// that passes html through is one script tag away from being the reason a
/// paywalled page leaks.
///
/// **Highlighting happens here, not in the browser.** The free half of a lesson
/// is edge-cached and identical for everyone, so colouring it once at render
/// time is work the cache keeps; shipping a highlighter to every reader would
/// be the same work repeated per visit, on the slowest machine in the chain,
/// and after the text had already been painted once uncoloured.
///
/// A fence with no language, or one syntect does not know, is left alone rather
/// than guessed at — it comes back as plain text in the same panel.
#[must_use]
pub fn render(markdown: &str) -> String {
    let mut options = Options::default();

    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.footnotes = true;
    // Without this every heading renders as a bare `<h2>`, and the contents
    // list — which links to `#some-heading` — points at nothing. The empty
    // prefix means the id is the anchor itself; comrak also drops a deep-link
    // <a class="anchor"> inside each heading, which `prose.css` can style or
    // leave invisible.
    options.extension.header_id_prefix = Some(String::new());

    let highlighter =
        HIGHLIGHTER.get_or_init(|| SyntectAdapter::new(Some(THEME)));

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(highlighter);

    markdown_to_html_with_plugins(markdown, &options, &plugins)
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
    // Comrak's own, so these ids are the ids it writes into the html. A second
    // implementation here would agree until the first heading with a character
    // the two treat differently — and the symptom of that is a contents entry
    // that silently scrolls nowhere. One anchorizer for the whole document,
    // because it is what makes a repeated heading `-1` rather than a duplicate.
    let mut anchorizer = Anchorizer::new();
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
            id: anchorizer.anchorize(text.trim()),
            text: text.trim().to_owned(),
        })
        .collect()
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

    const MARKED: &str =
        "Free part.\n\n<paid>\nPaid part.\n</paid>\n\nFree again.";

    #[test]
    fn a_region_is_lifted_out_and_the_prose_resumes_after_it() {
        // The reason for a pair rather than one cut: what follows the region
        // is free again, so a withheld example need not be the tail.
        let body = Body::split(MARKED);

        assert_eq!(body.free, "Free part.\n\n\nFree again.");
        assert_eq!(body.paid.as_deref(), Some("Paid part."));
    }

    #[test]
    fn several_regions_all_come_out() {
        let body =
            Body::split("A\n<paid>\none\n</paid>\nB\n<paid>\ntwo\n</paid>\nC");

        assert_eq!(body.free, "A\nB\nC");
        assert_eq!(body.paid.as_deref(), Some("one\ntwo"));
    }

    #[test]
    fn no_markers_means_nothing_is_withheld() {
        let body = Body::split("Just prose.");

        assert_eq!(body.free, "Just prose.");
        assert_eq!(body.paid, None);
        assert!(!body.has_paid_part());
    }

    #[test]
    fn a_region_spanning_the_file_withholds_all_of_it() {
        let body = Body::split("<paid>\nAll of it.\n</paid>");

        assert!(body.free.is_empty());
        assert_eq!(body.paid.as_deref(), Some("All of it."));
    }

    #[test]
    fn an_unclosed_region_withholds_the_rest_rather_than_serving_it() {
        // Fails closed. Too much withheld is seen by whoever wrote it; too
        // little is prose given away that somebody was meant to pay for.
        let body = Body::split("Free.\n<paid>\nMeant to be paid.");

        assert_eq!(body.free, "Free.");
        assert_eq!(body.paid.as_deref(), Some("Meant to be paid."));
    }

    #[test]
    fn a_stray_close_is_dropped_rather_than_rendered() {
        let body = Body::split("Free.\n</paid>\nStill free.");

        assert_eq!(body.free, "Free.\nStill free.");
        assert_eq!(body.paid, None);
        // It must not survive into the page as a raw tag.
        assert!(!body.free.contains("paid"));
    }

    #[test]
    fn the_markers_are_matched_trimmed_and_alone_on_their_line() {
        // Indented by an editor: still a marker.
        assert_eq!(Body::split("a\n  <paid>  \nb").paid.as_deref(), Some("b"));

        // Mentioned inside a sentence: not a marker, and the prose survives.
        let prose = Body::split("Write <paid> to open a region.");
        assert_eq!(prose.free, "Write <paid> to open a region.");
        assert_eq!(prose.paid, None);
    }

    #[test]
    fn free_lets_the_markers_decide_and_paid_overrides_them() {
        assert_eq!(Body::under(MARKED, Access::Free), Body::split(MARKED));

        let paid = Body::under(MARKED, Access::Paid);
        assert!(paid.free.is_empty());
        assert_eq!(paid.paid.as_deref(), Some(MARKED.trim()));
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

        assert!(serde_norway::from_str::<Access>("Paid").is_err());
        assert!(serde_norway::from_str::<Access>("partial").is_err());
    }

    #[test]
    fn absent_is_free_so_nothing_already_written_changed() {
        assert_eq!(Access::default(), Access::Free);
    }

    #[test]
    fn an_open_region_with_nothing_in_it_is_not_a_paywall() {
        // An editor left the tags at the end of a draft. Showing a "read the
        // rest" call to action here would lead to no rest.
        let body = Body::split("Everything.\n\n<paid>\n</paid>\n");

        assert_eq!(body.free, "Everything.");
        assert!(!body.has_paid_part());
    }

    #[test]
    fn neither_marker_survives_into_either_half() {
        let body = Body::split("Free.\n\n<paid>\nPaid.\n</paid>\n");

        for half in [&body.free, body.paid.as_ref().unwrap()] {
            assert!(!half.contains(PAID_OPEN), "{half}");
            assert!(!half.contains(PAID_CLOSE), "{half}");
        }
    }

    #[test]
    fn every_contents_entry_points_at_a_heading_that_exists() {
        // The bug this exists for: the contents list linked to `#some-heading`
        // while `render` emitted bare `<h2>` with no id, so every click did
        // nothing. Both sides derive the id from comrak's anchorizer now, and
        // this is what proves they still agree — including the `-1` suffix a
        // repeated heading gets, which is where two implementations drift
        // first.
        let markdown = "## Ticks aren\'t in\n\ntext\n\n## C & Assembly\n\n                        text\n\n## Ticks aren\'t in\n\ntext\n";

        let html = render(markdown);

        for heading in headings(markdown) {
            assert!(
                html.contains(&format!("id=\"{}\"", heading.id)),
                "no heading in the html has id {:?}\n{html}",
                heading.id
            );
        }
    }

    #[test]
    fn a_repeated_heading_gets_its_own_anchor() {
        let markdown = "## Setup\n\na\n\n## Setup\n\nb\n";
        let ids: Vec<String> =
            headings(markdown).into_iter().map(|h| h.id).collect();

        assert_eq!(ids, ["setup", "setup-1"]);
    }

    #[test]
    fn a_fenced_language_comes_back_coloured() {
        let html = render("```rust\nfn main() {}\n```\n");

        // More than one colour is the whole point: a keyword, a name and the
        // punctuation around them are not the same token. Without the plugin
        // this would be a bare <code class="language-rust"> and no colour.
        assert!(colours(&html).len() > 1, "{html}");
        assert!(html.contains("main"), "{html}");
    }

    /// Every distinct token colour syntect used.
    ///
    /// Matched with the opening quote, so the `background-color` syntect puts
    /// on the `<pre>` is not counted as a token colour — it is one either way,
    /// and counting it would make every block look highlighted.
    fn colours(html: &str) -> std::collections::BTreeSet<String> {
        html.split("\"color:#")
            .skip(1)
            .filter_map(|rest| rest.get(..6).map(str::to_owned))
            .collect()
    }

    #[test]
    fn an_unlabelled_fence_is_not_coloured_as_if_it_were_source() {
        // Most fences in the corpus carry no language, and they are output and
        // ascii diagrams as often as code. Syntect still wraps them, but in one
        // flat foreground colour — nothing is picked out as a keyword or a
        // string, which is the part that would be a lie.
        let html = render("```\n+---+\n| a |\n+---+\n```\n");

        assert!(html.contains("+---+"), "{html}");
        assert_eq!(colours(&html).len(), 1, "{html}");
    }

    #[test]
    fn a_language_it_does_not_know_is_not_guessed_at() {
        let html = render("```notalanguage\nfn main() {}\n```\n");

        assert_eq!(colours(&html).len(), 1, "{html}");
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
