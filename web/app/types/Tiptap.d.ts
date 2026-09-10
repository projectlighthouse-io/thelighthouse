import type { MarkdownStorage } from 'tiptap-markdown'

/**
 * `tiptap-markdown`'s storage, declared where tiptap looks for it.
 *
 * `@tiptap/core` ships `interface Storage {}` empty on purpose: an extension is
 * meant to augment it so `editor.storage.<name>` is typed. `tiptap-markdown`
 * exports the shape as `MarkdownStorage` but never does the augmenting, so
 * every `editor.storage.markdown.getMarkdown()` in the editor was an error.
 *
 * Declared here rather than cast at each call site — three today — because a
 * cast would have to be repeated and would stop describing the extension the
 * moment its storage changed.
 */
declare module '@tiptap/core' {
  interface Storage {
    markdown: MarkdownStorage
  }
}
