/**
 * The markdown round trip through tiptap.
 *
 * `BlogRichText` stores markdown, not html — see the component for why. That
 * only holds if what an author writes survives the trip out to the editor and
 * back, and this is the test that says it does. A serialiser that quietly
 * dropped a code fence or turned `-` bullets into `*` would corrupt every
 * article that passed through the editor, silently, one save at a time.
 *
 * @vitest-environment happy-dom
 */

import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import { Markdown } from 'tiptap-markdown'
import { beforeEach, describe, expect, it } from 'vitest'

import { renderArticle } from '../../../server/utils/SafeMarkdown'

/**
 * No *tag* in the output may carry an event handler.
 *
 * Deliberately not a search for `onerror` in the string: escaped text like
 * `&lt;img src=x onerror="steal()"&gt;` contains those characters and is inert
 * — it is what a reader sees, not markup a browser acts on. Matching the
 * substring would fail that and teach whoever hit it to weaken the check.
 */
function assertNoHandlers(html: string): void {
  for (const [tag] of html.matchAll(/<[a-z][^>]*>/gi)) {
    expect(tag, `${tag} carries an event handler`).not.toMatch(/\son[a-z]+\s*=/i)
  }
}

/** The same configuration the component builds, minus the Vue wrapper. */
function editorWith(content: string): Editor {
  return new Editor({
    content,
    extensions: [
      StarterKit,
      Markdown.configure({ html: false, breaks: true, transformPastedText: true }),
    ],
  })
}

/** Markdown in, markdown out. */
function roundTrip(markdown: string): string {
  const editor = editorWith(markdown)
  const out = editor.storage.markdown.getMarkdown()
  editor.destroy()

  return out
}

describe('markdown survives the editor', () => {
  it('keeps headings, emphasis and inline code', () => {
    const out = roundTrip('# Heading\n\nSome **bold** and *italic* and `code`.')

    expect(out).toContain('# Heading')
    expect(out).toContain('**bold**')
    expect(out).toContain('`code`')
  })

  it('keeps a fenced code block and its language', () => {
    const out = roundTrip('```rust\nfn main() {}\n```')

    expect(out).toContain('```rust')
    expect(out).toContain('fn main() {}')
  })

  it('keeps lists and quotes', () => {
    const out = roundTrip('- one\n- two\n\n> a quote')

    expect(out).toContain('one')
    expect(out).toContain('two')
    expect(out).toContain('> a quote')
  })

  it('keeps links', () => {
    expect(roundTrip('[docs](https://example.com)')).toContain('https://example.com')
  })

  it('is stable — a second trip changes nothing the first did not', () => {
    // The property that matters for an article edited more than once: if the
    // output is not a fixed point, every save rewrites the body a little and
    // the drift is only noticed much later.
    const source = '# Title\n\nA paragraph with **bold** text.\n\n- one\n- two\n\n```go\nfmt.Println()\n```'

    const once = roundTrip(source)
    const twice = roundTrip(once)

    expect(twice).toBe(once)
  })
})

describe('the editor is not a security boundary, and does not pretend to be', () => {
  const PAYLOAD = '<script>alert(1)</script>\n\n<img src=x onerror="steal()">\n\nplain text'

  let out: string

  beforeEach(() => {
    out = roundTrip(PAYLOAD)
  })

  it('escapes raw html into text rather than keeping it as markup', () => {
    // `html: false` does not delete it — it entity-escapes it, so what gets
    // stored is the *text* `<script>`, not a tag. Both are fine; this test
    // records which one actually happens so nobody reads the option name and
    // assumes the other.
    expect(out).toContain('&lt;script&gt;')
    expect(out).not.toMatch(/<script>/)
    expect(out).not.toMatch(/<img[^>]*onerror/)
  })

  it('keeps the prose around it', () => {
    expect(out).toContain('plain text')
  })

  it('produces markdown that is still inert once the server renders it', () => {
    // The property that actually matters, checked across the two modules that
    // have to agree: whatever the editor decides to store, `SafeMarkdown` is
    // what turns it into html, and that output must not be able to run.
    const html = renderArticle(out)

    expect(html.toLowerCase()).not.toContain('<script')
    assertNoHandlers(html)
    expect(html).toContain('plain text')
  })

  it('is inert even for a body that never went through the editor', () => {
    // Nobody has to use this component. A hand-rolled `POST /api/articles`
    // stores exactly what it sends, so the server has to hold on the raw
    // payload too — not just on what tiptap chose to write down.
    const html = renderArticle(PAYLOAD)

    expect(html.toLowerCase()).not.toContain('<script')
    assertNoHandlers(html)
  })
})
