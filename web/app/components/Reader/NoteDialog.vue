<script setup lang="ts">
/**
 * Writing a note against the selected passage.
 *
 * A dialog rather than an inline field: a note is a sentence or two the reader
 * stops to write, and a textarea appearing between two paragraphs pushes the
 * prose they are annotating off the screen.
 */

/**
 * Characters, matching `notes::payload::MAX_NOTE` in the api. The counter is a
 * courtesy — the api enforces it, in characters rather than bytes, and refuses
 * anything longer whatever this says.
 */
const MAX_NOTE = 500

/** How much of the passage the header shows before it elides the middle. */
const PREVIEW = 120

const props = defineProps<{
  passage: string
  saving: boolean
  /** The api's own words for a refused write, shown as it sent them. */
  error: string | null
}>()

const emit = defineEmits<{
  save: [content: string, isPublic: boolean]
  cancel: []
}>()

const content = ref<string>('')
const isPrivate = ref<boolean>(false)
const field = ref<HTMLTextAreaElement | null>(null)

/**
 * Both ends of a long passage rather than the first half.
 *
 * A reader recognises what they selected by where it started *and* stopped;
 * a plain truncation shows them a sentence that trails into nothing.
 */
const preview = computed<string>(() => {
  if (props.passage.length <= PREVIEW) return props.passage

  const half = Math.floor(PREVIEW / 2)

  return `${props.passage.slice(0, half)} … ${props.passage.slice(-half)}`
})

const submittable = computed<boolean>(
  () => !props.saving && content.value.trim().length > 0,
)

const save = (): void => {
  if (!submittable.value) return

  emit('save', content.value.trim(), !isPrivate.value)
}

onMounted(() => field.value?.focus())

// Cmd/Ctrl-Enter saves, Escape closes. Both are what a reader who writes a lot
// of these will reach for without being told.
const onKeydown = (event: KeyboardEvent): void => {
  if (event.key === 'Escape') emit('cancel')
  if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) save()
}
</script>

<template>
  <div
    class="reader-notedialog"
    role="dialog"
    aria-modal="true"
    aria-label="Write a note"
    @keydown="onKeydown"
  >
    <!-- The backdrop closes it. Nothing here is destructive, so a stray click
         costs an unsaved sentence and never a saved one. -->
    <div class="reader-notedialog__veil" @click="emit('cancel')" />

    <div class="reader-notedialog__card border-pencil-light bg-panel">
      <div class="font-mono text-xs tracking-wide text-teal-deep">
        <span class="text-faint">/ </span>your note
      </div>

      <blockquote class="reader-notedialog__passage">
        {{ preview }}
      </blockquote>

      <textarea
        ref="field"
        v-model="content"
        class="reader-notedialog__field"
        rows="5"
        :maxlength="MAX_NOTE"
        placeholder="what you want to remember about this…"
      />

      <div class="mt-2 flex items-center justify-between">
        <label class="flex cursor-pointer items-center gap-2 text-xs text-quiet">
          <input v-model="isPrivate" type="checkbox" class="cursor-pointer">
          keep this one private
        </label>

        <span class="font-mono text-[11px] text-faint">
          {{ content.length }}/{{ MAX_NOTE }}
        </span>
      </div>

      <p v-if="error" class="reader-notedialog__error">
        {{ error }}
      </p>

      <div class="mt-5 flex items-center justify-end gap-3">
        <button
          type="button"
          class="cursor-pointer rounded-lg border border-stroke bg-panel px-4 py-2 text-sm text-ink transition hover:bg-paper-warm"
          @click="emit('cancel')"
        >
          cancel
        </button>
        <button
          type="button"
          class="cursor-pointer rounded-lg bg-ink px-4 py-2 text-sm font-semibold text-on-ink transition hover:bg-ink-hover disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="!submittable"
          @click="save"
        >
          {{ saving ? 'saving…' : 'save note' }}
        </button>
      </div>
    </div>
  </div>
</template>
