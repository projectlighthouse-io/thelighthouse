<script setup lang="ts">
/**
 * The article body, edited as rich text and stored as markdown.
 *
 * # Why markdown is still the storage format
 *
 * TipTap's native format is html, and storing that would have been the obvious
 * thing. It is deliberately not what happens: `tiptap-markdown` serialises to
 * markdown on the way out and parses it on the way in, so the value this
 * component emits is the same kind of string the `<textarea>` used to emit.
 *
 * That keeps three things unchanged that would otherwise all have moved
 * together — rust's `articles.body` and everything its migration says about
 * it, `SafeMarkdown`, and the published page. It also keeps the *stored* value
 * plain text rather than markup, so the database holds something inert and
 * diffable, and swapping the editor again later is a frontend change rather
 * than a data migration.
 *
 * # This does not make anything safe
 *
 * TipTap constrains what an author can *type* here, and that is a nicety, not
 * a boundary — anyone can skip this component and `POST /api/articles`
 * directly with any body they like. The only thing standing between a reader's
 * markdown and a rendered page is `SafeMarkdown.renderArticle` on the server,
 * which is unchanged by this and still the whole security story.
 *
 * Correspondingly, **nothing here renders untrusted content**. The editor
 * shows what its own user is typing, in their own browser, which is the one
 * case where that is not a cross-site scripting question.
 *
 * # The menu
 *
 * There is no toolbar. Formatting appears as a bubble over the selection, the
 * way Notion does it — permanent chrome above a writing surface makes the page
 * read as a form, which is the opposite of what the rest of this editor is
 * doing. Markdown shortcuts (`## `, `- `, ```` ``` ````) still work while
 * typing, so the menu is for reformatting rather than the only way in.
 *
 * # Client only
 *
 * ProseMirror needs a DOM, so this must not render during SSR. The pages that
 * use it are already `ssr: false` in `nuxt.config`; the `<ClientOnly>` here is
 * belt and braces for anywhere else it gets dropped in.
 */
import { Editor, EditorContent } from '@tiptap/vue-3'
import { BubbleMenu } from '@tiptap/vue-3/menus'
import { Placeholder } from '@tiptap/extensions'
import StarterKit from '@tiptap/starter-kit'
import { Markdown } from 'tiptap-markdown'

const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ (e: 'update:modelValue', value: string): void }>()

const editor = shallowRef<Editor | undefined>()

/** The marks the bubble menu toggles, and what `StarterKit` hangs them on. */
const marks = [
  { name: 'bold', label: 'B', title: 'bold', class: 'font-bold' },
  { name: 'italic', label: 'I', title: 'italic', class: 'italic' },
  { name: 'strike', label: 'S', title: 'strikethrough', class: 'line-through' },
  { name: 'code', label: '`', title: 'inline code', class: 'font-mono' },
] as const

/** The block shapes it can turn the selection into. */
const blocks = [
  { label: 'H1', title: 'heading 1', is: { name: 'heading', attrs: { level: 1 } } },
  { label: 'H2', title: 'heading 2', is: { name: 'heading', attrs: { level: 2 } } },
  { label: 'H3', title: 'heading 3', is: { name: 'heading', attrs: { level: 3 } } },
  { label: '•', title: 'bullet list', is: { name: 'bulletList', attrs: {} } },
  { label: '1.', title: 'numbered list', is: { name: 'orderedList', attrs: {} } },
  { label: '>', title: 'quote', is: { name: 'blockquote', attrs: {} } },
  { label: '{}', title: 'code block', is: { name: 'codeBlock', attrs: {} } },
] as const

/**
 * One chain per block shape, rather than a `toggleNode` that takes a name:
 * tiptap's commands are separate functions and there is no generic form, so
 * the mapping has to exist somewhere. Here is better than seven buttons in the
 * template each with their own handler.
 */
function toggleBlock(label: string): void {
  const chain = editor.value?.chain().focus()
  if (!chain) return

  switch (label) {
    case 'H1': chain.toggleHeading({ level: 1 }).run(); break
    case 'H2': chain.toggleHeading({ level: 2 }).run(); break
    case 'H3': chain.toggleHeading({ level: 3 }).run(); break
    case '•': chain.toggleBulletList().run(); break
    case '1.': chain.toggleOrderedList().run(); break
    case '>': chain.toggleBlockquote().run(); break
    case '{}': chain.toggleCodeBlock().run(); break
    default: break
  }
}

onMounted(() => {
  editor.value = new Editor({
    content: props.modelValue,
    extensions: [
      // `underline: false` because markdown has no underline and
      // `tiptap-markdown` warns on every serialise when the mark exists but
      // cannot be written — hundreds of times a second while typing. Dropping
      // the extension is the honest fix: the storage format cannot express it,
      // so the editor should not offer it.
      StarterKit.configure({ underline: false }),
      // `html: false` is the point of the whole file: raw html an author
      // pastes is dropped at the door rather than round-tripped into the
      // markdown we store. The server would strip it anyway — this just means
      // it never gets written down in the first place.
      Markdown.configure({ html: false, breaks: true, transformPastedText: true }),
      // Ships inside `@tiptap/extensions`, which `vue-3` already pulls in, so
      // this is a prompt on an empty document rather than a new dependency.
      Placeholder.configure({
        placeholder:
          'markdown shortcuts work as you type — ## for a heading, - for a '
          + 'list, ``` for code. Any raw HTML is removed when the article '
          + 'renders.',
      }),
    ],
    editorProps: {
      attributes: {
        class: 'prose dek-face max-w-none min-h-[32rem] p-0 outline-none',
      },
    },
    onUpdate: ({ editor }) => {
      emit('update:modelValue', editor.storage.markdown.getMarkdown())
    },
  })
})

// The edit page loads an article after mount, so the first real value arrives
// once the editor already exists. Guarded on equality: writing the content
// back on every keystroke would reset the cursor to the top of the document.
watch(
  () => props.modelValue,
  (incoming) => {
    const current = editor.value?.storage.markdown.getMarkdown()

    if (editor.value && incoming !== current) {
      editor.value.commands.setContent(incoming, { emitUpdate: false })
    }
  },
)

onBeforeUnmount(() => editor.value?.destroy())

</script>

<template>
  <ClientOnly>
    <div>
      <!-- Notion-style: nothing is drawn until text is selected, and the menu
           comes to the selection rather than living in a bar at the top. That
           is the whole reason the static toolbar is gone — a writing surface
           with permanent chrome above it reads as a form, and the point of
           this page is that it does not. -->
      <BubbleMenu
        v-if="editor"
        :editor="editor"
        :options="{ placement: 'top', offset: 8 }"
        class="flex items-center gap-0.5 rounded-lg bg-ink p-1 shadow-lg"
      >
        <button
          v-for="mark in marks"
          :key="mark.name"
          type="button"
          :title="mark.title"
          :class="[
            mark.class,
            'cursor-pointer rounded px-2 py-1 text-[13px] leading-none transition',
            editor.isActive(mark.name)
              ? 'bg-on-ink/20 text-on-ink'
              : 'text-on-ink/70 hover:bg-on-ink/10 hover:text-on-ink',
          ]"
          @click="editor.chain().focus().toggleMark(mark.name).run()"
        >
          {{ mark.label }}
        </button>

        <span class="mx-1 h-4 w-px bg-on-ink/25" />

        <button
          v-for="block in blocks"
          :key="block.label"
          type="button"
          :title="block.title"
          :class="[
            'cursor-pointer rounded px-2 py-1 text-[13px] leading-none transition',
            editor.isActive(block.is.name, block.is.attrs)
              ? 'bg-on-ink/20 text-on-ink'
              : 'text-on-ink/70 hover:bg-on-ink/10 hover:text-on-ink',
          ]"
          @click="toggleBlock(block.label)"
        >
          {{ block.label }}
        </button>
      </BubbleMenu>

      <EditorContent :editor="editor" />
    </div>

    <template #fallback>
      <div class="dek-face min-h-[32rem] text-faint">
        loading the editor…
      </div>
    </template>
  </ClientOnly>
</template>

<style>
/* Tiptap marks the empty node and leaves the drawing to css. Not scoped: the
 * element belongs to prosemirror, so a scoped rule's data attribute never
 * lands on it. */
.tiptap p.is-editor-empty:first-child::before {
  content: attr(data-placeholder);
  color: var(--color-faint);
  float: left;
  height: 0;
  pointer-events: none;
}
</style>
