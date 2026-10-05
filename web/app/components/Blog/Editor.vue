<script setup lang="ts">
/**
 * The write and edit form, which are the same form.
 *
 * Full bleed: this is *not* wrapped in the page's `max-w-7xl` container, so
 * the art column reaches the browser edge. The pages pass their heading in
 * through the `header` slot and the right column carries the padding that
 * would otherwise have been on the page. `<main>` in the layout is
 * unconstrained, which is what makes that possible.
 *
 * **The template root is the `<form>` and nothing else — not even a comment.**
 * Vue counts a root-level comment as a node, which makes this a fragment
 * component, which in turn makes every *page* that renders it multi-root.
 * Nuxt then warns `NUXT_E4004` and route navigation breaks. Notes about the
 * markup go here, or inside the root element.
 *
 * The body is `BlogRichText` — a tiptap editor that reads and writes markdown,
 * so what this emits is still the markdown string rust stores. See that
 * component for why the storage format did not move to html.
 *
 * The editor renders what its own user is typing, in their own browser, which
 * is the one case where showing rich text is not a cross-site scripting
 * question. Nobody else's article is ever rendered here — that happens on the
 * published page, through `SafeMarkdown` on the server.
 */

const props = defineProps<{
  title: string
  subtitle: string
  topics: string[]
  body: string
  /** What the button says: 'publish' or 'save'. */
  action: string
  /** The line across the bottom of the art panel. */
  caption: string
  pending: boolean
  /** For the form as a whole — a refusal no single field owns. */
  error: string | null
  /** The api's refusals, by the field name it was sent under. */
  fieldErrors: Record<string, string>
}>()

/** Characters, matching `articles::payload::MAX_BODY`, which enforces it. */
const MAX_BODY = 50_000
/** The counter only appears near the limit; nobody needs it at paragraph two. */
const SHOW_COUNT_FROM = 45_000

const emit = defineEmits<{
  (e: 'update:title' | 'update:subtitle' | 'update:body', value: string): void
  (e: 'update:topics', value: string[]): void
  (e: 'submit'): void
}>()

const canSubmit = computed(() =>
  !props.pending
  && props.title.trim() !== ''
  && props.subtitle.trim() !== ''
  && props.body.trim() !== ''
  && props.topics.length > 0,
)

/**
 * The title takes focus on load, so writing starts with a keystroke rather
 * than a click.
 *
 * A ref and `onMounted` rather than the `autofocus` attribute: these pages are
 * `ssr: false`, so the input is created by the client after the document is
 * already interactive, and browsers only honour `autofocus` for elements
 * present in the parsed document. The attribute would work in development and
 * quietly do nothing where it matters.
 */
const titleField = ref<HTMLInputElement | null>(null)

onMounted(() => titleField.value?.focus())

const router = useRouter()
const { reader } = useReader()

/**
 * Back to wherever the author came from.
 *
 * `history.state.back` rather than `window.history.length`: the length counts
 * the whole tab's history, so it is greater than one on a direct visit that
 * followed anything at all, and `router.back()` would then walk off this site.
 * `state.back` is null exactly when this entry has nothing before it, which is
 * the question being asked — and the author's own listing is the honest answer
 * then, since that is where the article ends up either way.
 */
function cancel(): void {
  if (window.history.state?.back) {
    router.back()
    return
  }

  void navigateTo(ownWritingUrl(reader.value?.username))
}
</script>

<template>
  <form
    class="lg:grid lg:h-[calc(100vh-4rem)] lg:grid-cols-[minmax(0,1fr)_minmax(0,2fr)]"
    @submit.prevent="emit('submit')"
  >
    <ArtPanel :caption="caption" />

    <!-- A fixed-height column from `lg` up, so the action bar sits on the
         viewport's bottom edge and the article scrolls *inside* the card
         rather than moving the page. Below `lg` this is ordinary document
         flow — a scroll box inside a scrolling page is the worst of both. -->
    <div class="px-4 py-2 sm:px-6 lg:flex lg:h-full lg:min-h-0 lg:flex-col lg:px-10">
      <!-- Widened with the column. `max-w-2xl` was chosen when this was half
           the screen; at two thirds it would leave the extra width as empty
           gutter rather than as writing room, which is the point of the split.
           Still capped — an editor that runs the full width of a wide monitor
           gives lines nobody wants to read back.

           `bg-page` on this box and not on the column: the writing surface is
           what wants to be a clean sheet, and letting the column carry it
           would paint the whole right half and lose the layout's dotted
           texture at the edges. Not a literal `bg-white` either — the token is
           `#ffffff` in the light theme and `#121110` in the dark one, so this
           does not become a glaring slab when the theme flips. -->
      <!-- No `overflow-hidden` here, deliberately: it would make this box a
           scroll container, and the action bar below stops being sticky the
           moment an ancestor clips overflow. The bar carries its own
           `rounded-b-xl` instead, which is what clipping would have bought. -->
      <div class="mx-auto w-full max-w-4xl rounded-xl bg-page px-6 pt-2 sm:px-10 sm:pt-4 lg:flex lg:min-h-0 lg:flex-1 lg:flex-col">
        <!-- The scrolling half. `min-h-0` is what makes it work: a flex child
             defaults to `min-height: auto`, so it refuses to shrink below its
             content and the overflow lands on the page instead of here.
             The negative margins and matching padding keep the scrollbar at
             the card's edge rather than inset by the padding. -->
        <div class="lg:-mx-10 lg:min-h-0 lg:flex-1 lg:overflow-y-auto lg:px-10">
        <!-- Spacing hangs off the optional blocks rather than off the first
             field, so a page that passes no header does not open with a
             margin under nothing. -->
        <div v-if="$slots.header" class="mb-10">
          <slot name="header" />
        </div>

        <p
          v-if="error"
          class="dek-face mt-6 mb-2 rounded-md border border-bad/30 bg-bad/10 px-4 py-3 text-bad"
          role="alert"
        >
          {{ error }}
        </p>

        <!-- The title is typed at the size it will be read at: `masthead-title`
             is the same class `/books/{slug}` puts on its `<h1>`, so this is
             the book page's face and size rather than an approximation of it
             that drifts the next time that page is restyled.
             No border, no background and no padding — the chrome is what makes
             a field look like a form control, and this should read as the
             heading it becomes. The `<span>` label is dropped for the same
             reason; the placeholder says what it is. -->
        <label class="block">
          <span class="sr-only">title</span>
          <input
            ref="titleField"
            :value="title"
            type="text"
            maxlength="200"
            required
            placeholder="Title"
            class="masthead-title w-full border-0 bg-transparent p-0 outline-none placeholder:text-faint"
            :aria-invalid="!!fieldErrors.title"
            @input="emit('update:title', ($event.target as HTMLInputElement).value)"
          >
        </label>
        <p v-if="fieldErrors.title" class="mt-2 text-sm text-bad" role="alert">{{ fieldErrors.title }}</p>

        <!-- Under the title and set in the dek face, so the pair reads as a
             masthead — the same relationship `/books/{slug}` has between its
             title and its dek. Required, and `maxlength` matches the column
             and rust: the field stops accepting at 200 rather than letting
             somebody write 400 and refusing the save. -->
        <label class="mt-3 block">
          <span class="sr-only">subtitle</span>
          <input
            :value="subtitle"
            type="text"
            maxlength="200"
            required
            placeholder="Add a subtitle"
            class="dek-face w-full border-0 bg-transparent p-0 text-quiet outline-none placeholder:text-faint"
            :aria-invalid="!!fieldErrors.subtitle"
            @input="emit('update:subtitle', ($event.target as HTMLInputElement).value)"
          >
        </label>
        <p v-if="fieldErrors.subtitle" class="mt-2 text-sm text-bad" role="alert">{{ fieldErrors.subtitle }}</p>

        <!-- Chips, because an article can be about Go *and* Docker. See
             `BlogTopics`; the wire shape is `topics: string[]`. -->
        <div class="mt-6">
          <BlogTopics
            :model-value="topics"
            @update:model-value="emit('update:topics', $event)"
          />
          <p v-if="fieldErrors.topics" class="mt-2 text-sm text-bad" role="alert">{{ fieldErrors.topics }}</p>
        </div>

        <div class="mt-8">
          <span class="sr-only">body</span>
          <BlogRichText
            :model-value="body"
            @update:model-value="emit('update:body', $event)"
          />
          <p v-if="body.length >= SHOW_COUNT_FROM" class="mt-2 font-mono text-xs text-quiet">
            {{ body.length.toLocaleString() }} / {{ MAX_BODY.toLocaleString() }} characters
          </p>
          <p v-if="fieldErrors.body" class="mt-2 text-sm text-bad" role="alert">{{ fieldErrors.body }}</p>
        </div>

        </div>

        <!-- Outside the scroll box, so it is pinned by the layout rather than
             by `position` — no sticky, no fixed, and nothing that needs to
             know the art column's width. `shrink-0` keeps it its own height
             when the content above wants everything. -->
        <div
          class="-mx-6 mt-8 flex items-center justify-end gap-5 rounded-b-xl bg-page px-6 py-4 sm:-mx-10 sm:px-10 lg:mt-0 lg:shrink-0"
        >
          <button
            type="button"
            class="dek-face cursor-pointer rounded-md border-0 bg-transparent px-4 py-2 text-faint transition hover:bg-paper-warm hover:text-ink"
            @click="cancel"
          >
            cancel
          </button>

          <!-- Disabled is muted, not faded out. `opacity-40` on a solid button
               leaves something that reads as broken rather than as "not yet" —
               and this is the state an author sees for as long as the article
               is unfinished, which is most of the time they are on the page.
               A filled grey button at full opacity still says "this is the
               thing you press", and `title` says what is missing. -->
          <button
            type="submit"
            :disabled="!canSubmit"
            :title="canSubmit ? undefined : 'add a title, a subtitle, a topic and some words first'"
            class="dek-face cursor-pointer rounded-md border-0 px-6 py-2 font-medium transition disabled:cursor-not-allowed"
            :class="canSubmit
              ? 'bg-ink text-on-ink hover:opacity-85'
              : 'bg-paper-edge text-faint'"
          >
            {{ pending ? 'saving…' : action }}
          </button>
        </div>
      </div>
    </div>
  </form>
</template>
