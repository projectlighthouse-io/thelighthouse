/**
 * Anchoring a passage to a place in the rendered lesson.
 *
 * **A character offset pair over the container's text, and nothing else.** The
 * laravel reader also stored an xpath for each end and never read it back;
 * `20260822220000_drop_note_xpath_columns.sql` explains why it was worse than
 * useless — its generator counted element siblings while writing `text()[n]`,
 * so the path did not identify the node it was built from. Both columns are
 * gone from `notes` and `lesson_bookmarks`, and nothing here recreates them.
 *
 * What makes offsets work is that the api renders the body: the same markdown
 * produces the same html produces the same text, so offset 4,120 is the same
 * words on every device. What breaks them is an edit to the prose above a
 * highlight, which is a real limitation and a known one — see `docs/rebuild.md`
 * phase 6 on the drift columns that were deliberately not ported.
 */

/** Where a passage sits: character offsets over the container's text. */
export interface Anchor {
  text: string
  start: number
  end: number
}

/** One text node, and the slice of it a range covers. */
interface Slice {
  node: Text
  from: number
  to: number
}

/**
 * The text nodes a `[start, end)` offset range touches, and where it cuts them.
 *
 * Collected in one pass *before* anything is wrapped: [`paint`] mutates the
 * tree, and a walker that is still walking would visit the marks it just made.
 */
function slices(container: Element, start: number, end: number): Slice[] {
  const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT)
  const found: Slice[] = []
  let at = 0

  while (walker.nextNode()) {
    const node = walker.currentNode as Text
    const length = node.data.length
    const from = Math.max(start - at, 0)
    const to = Math.min(end - at, length)

    if (from < to) found.push({ node, from, to })

    at += length
    if (at >= end) break
  }

  return found
}

/**
 * The offsets a live selection covers, or `null` when it is not in the lesson.
 *
 * Measured by cloning the range and stretching it back to the start of the
 * container, so the number is "characters of lesson text before this", which is
 * exactly what the api stores. `Range.toString()` counts what the reader sees.
 */
export function anchorOf(container: Element, range: Range): Anchor | null {
  const raw = range.toString()
  const text = raw.trim()
  if (!text) return null
  if (!container.contains(range.commonAncestorContainer)) return null

  const before = range.cloneRange()
  before.selectNodeContents(container)
  before.setEnd(range.startContainer, range.startOffset)

  // Past whatever the trim took off the front. A double-click usually takes the
  // space after the word too, and storing the untrimmed start against the
  // trimmed text would draw every such highlight one character early.
  const start = before.toString().length + raw.indexOf(text)

  return { text, start, end: start + text.length }
}

/**
 * Wraps a passage in `<mark class="…">`, one element per text node it spans.
 *
 * **Per text node, deliberately.** The obvious implementation is
 * `range.surroundContents(mark)`, which throws whenever a selection crosses an
 * element boundary — so the laravel reader falls back to
 * `extractContents()` + `insertNode()`, and that lifts the selected content out
 * of the paragraphs it was in and drops it back as one flat run. A highlight
 * spanning two paragraphs rewrites the lesson's markup to draw itself.
 *
 * Wrapping each text node separately never crosses a boundary, so it never
 * throws and never moves anything. Two `<mark>`s that look like one highlight
 * are the correct rendering of a passage that really does span two blocks.
 *
 * Returns the marks, for the caller to hang listeners on and to unwrap later.
 * Empty when the offsets resolve to nothing — a stale anchor into prose that
 * has since changed, which is a highlight that quietly does not appear rather
 * than an error.
 */
export function paint(
  container: Element,
  anchor: { start: number, end: number },
  className: string,
): HTMLElement[] {
  const marks: HTMLElement[] = []

  for (const { node, from, to } of slices(container, anchor.start, anchor.end)) {
    const range = document.createRange()
    range.setStart(node, from)
    range.setEnd(node, to)

    const mark = document.createElement('mark')
    mark.className = className
    // Inside one text node, so this cannot throw the way the cross-element
    // case would.
    range.surroundContents(mark)

    marks.push(mark)
  }

  return marks
}

/**
 * Takes a highlight back off, leaving the text where it was.
 *
 * `normalize()` because unwrapping leaves the text split into three neighbours
 * where the mark used to be, and the next `paint` counts nodes as it walks —
 * it would still measure the same total, but a tree that accumulates fragments
 * every time a highlight is toggled is a tree that gets slower for no reason.
 */
export function unpaint(marks: HTMLElement[]): void {
  for (const mark of marks) {
    const parent = mark.parentNode
    if (!parent) continue

    while (mark.firstChild) parent.insertBefore(mark.firstChild, mark)
    parent.removeChild(mark)
    parent.normalize()
  }
}
