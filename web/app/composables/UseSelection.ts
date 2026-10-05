/**
 * Selecting a passage of a lesson, and where to float a menu over it.
 *
 * Only the selection. What the menu offers — a note, a bookmark, a prompt to
 * sign in — is the page's decision, and this knows about none of it.
 */

import type { Anchor } from '@/utils/Anchor'

/** Where the menu goes, in viewport coordinates: it is `position: fixed`. */
export interface Spot {
  x: number
  y: number
}

/**
 * The passage under the cursor, once the reader lets go.
 *
 * `container` is the rendered lesson body and `exclude` is anything inside it a
 * selection should not offer to annotate — a terminal transcript a reader is
 * copying a command out of, say. Both are selectors rather than elements
 * because the body is `v-html` and its nodes are replaced wholesale when the
 * route changes.
 */
export function useSelection(
  container = '[data-lesson-content]',
  exclude = '[data-interactive-block]',
) {
  const anchor = ref<Anchor | null>(null)
  const spot = ref<Spot>({ x: 0, y: 0 })
  const open = ref<boolean>(false)

  const dismiss = (): void => {
    open.value = false
  }

  /** Drops the selection as well as the menu — after saving, or on cancel. */
  const clear = (): void => {
    anchor.value = null
    open.value = false
    window.getSelection()?.removeAllRanges()
  }

  const capture = (at: Spot): void => {
    const selection = window.getSelection()
    if (!selection || selection.rangeCount === 0) return dismiss()

    const range = selection.getRangeAt(0)
    const body = document.querySelector(container)
    if (!body) return dismiss()

    // A transcript, a code block with its own copy button: inside the lesson,
    // but not prose to annotate. `closest` from the common ancestor covers a
    // selection that starts and ends inside one, which is the only shape that
    // reaches here.
    const within = range.commonAncestorContainer
    const element
      = within.nodeType === Node.ELEMENT_NODE
        ? (within as Element)
        : within.parentElement
    if (exclude && element?.closest(exclude)) return dismiss()

    const found = anchorOf(body, range)
    if (!found) return dismiss()

    anchor.value = found
    spot.value = at
    open.value = true
  }

  // On mouseup rather than on `selectionchange`: the latter fires on every
  // character as the pointer drags, so the menu would chase the cursor across
  // the paragraph and settle only when the reader stopped.
  const onMouseUp = (event: MouseEvent): void => {
    // A tick after the event, because the selection is not yet collapsed at
    // mouseup time when the reader is clicking to dismiss one.
    setTimeout(() => capture({ x: event.clientX, y: event.clientY }), 0)
  }

  // A new selection begins by discarding the old one, and the menu belongs to
  // the old one.
  const onMouseDown = (): void => dismiss()

  /**
   * The same moment on a touchscreen: the finger comes off a selection handle.
   *
   * Positioned from the selection's own rectangle rather than from the touch,
   * which is under the reader's finger and often past the edge of the screen.
   * Not `selectionchange` — that fires on every character as the handle drags,
   * so the menu would chase it across the paragraph.
   */
  const onTouchEnd = (): void => {
    setTimeout(() => {
      const selection = window.getSelection()
      if (!selection || selection.isCollapsed || selection.rangeCount === 0) {
        return dismiss()
      }

      const rect = selection.getRangeAt(0).getBoundingClientRect()
      capture({ x: rect.left + rect.width / 2, y: rect.bottom })
    }, 0)
  }

  onMounted(() => {
    document.addEventListener('mouseup', onMouseUp)
    document.addEventListener('mousedown', onMouseDown)
    document.addEventListener('touchend', onTouchEnd, { passive: true })
  })

  onBeforeUnmount(() => {
    document.removeEventListener('mouseup', onMouseUp)
    document.removeEventListener('mousedown', onMouseDown)
    document.removeEventListener('touchend', onTouchEnd)
  })

  return { anchor, spot, open, capture, dismiss, clear }
}
