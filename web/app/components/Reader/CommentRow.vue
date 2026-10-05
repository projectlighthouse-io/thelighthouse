<script setup lang="ts">
/**
 * One row of the thread under a lesson: avatar, who, when, the body, and the
 * passage it was taken against. A root and a reply draw the same; the thread
 * indents the replies, not this.
 *
 * The author is the comment's own, not the session's — the thread has other
 * people in it. Edit and delete appear only on `mine`, which the thread works
 * out after hydration; the api refuses anyone else's write regardless.
 */

import type { CommentEntry } from '@/types/Comments'

/** Matches `notes::payload::MAX_NOTE` in the api, which enforces it. */
const MAX_NOTE = 500

const props = defineProps<{
  entry: CommentEntry
  mine: boolean
  /** A private note, shown to its owner only. */
  isPrivate?: boolean
  /** Whether the passage has a highlight up the page to jump to. Only the
   *  reader's own notes are painted, so somebody else's quote is just text. */
  jumpable: boolean
  editing: boolean
  saving: boolean
  editError: string | null
  removeError: string | null
}>()

const emit = defineEmits<{
  jump: [id: number]
  edit: [id: number]
  save: [id: number, content: string]
  cancelEdit: []
  remove: [id: number]
}>()

const edited = ref<string>('')

// Seeded when the field opens rather than when edit is clicked, so a refused
// save that re-renders the row keeps what was typed.
watch(() => props.editing, (now) => {
  if (now) edited.value = props.entry.body ?? ''
}, { immediate: true })

const name = computed<string>(() => props.entry.author.name ?? 'A reader')

/** First letters of the first two words, for an author with no avatar. */
const initials = computed<string>(() =>
  name.value
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map(word => word.charAt(0).toUpperCase())
    .join(''),
)

/**
 * How long ago, in the units somebody actually asks in — minutes, hours, days,
 * then a date, because past a week "9d ago" is no easier to read than "Mar 4".
 *
 * Only ever rendered in the browser: the thread is fetched after mount, so
 * there is no server copy of this to disagree with.
 */
const since = (iso: string | null): string => {
  if (!iso) return ''

  const seconds = Math.floor((Date.now() - new Date(iso).getTime()) / 1000)

  if (seconds < 60) return 'just now'
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`
  if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`

  return new Date(iso).toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
  })
}

/** The exact time, on hover, for a "3d ago" somebody wants to pin down. */
const exactly = (iso: string | null): string | undefined =>
  iso ? new Date(iso).toLocaleString() : undefined
</script>

<template>
  <article class="note-comment">
    <img
      v-if="entry.author.avatarUrl"
      :src="entry.author.avatarUrl"
      :alt="name"
      class="note-comment__avatar"
      loading="lazy"
      referrerpolicy="no-referrer"
    >
    <div v-else class="note-comment__avatar note-comment__avatar--initials" aria-hidden="true">
      {{ initials }}
    </div>

    <div class="note-comment__main">
      <header class="note-comment__head">
        <span class="note-comment__who">{{ name }}</span>
        <span v-if="entry.author.username" class="note-comment__when">@{{ entry.author.username }}</span>

        <span v-if="isPrivate" class="note-comment__lock">private</span>

        <template v-if="entry.createdAt">
          <span class="note-comment__dot">·</span>
          <time
            class="note-comment__when"
            :datetime="entry.createdAt"
            :title="exactly(entry.createdAt)"
          >{{ since(entry.createdAt) }}</time>
        </template>

        <div v-if="mine" class="note-comment__acts">
          <button
            type="button"
            class="note-comment__act"
            @click="emit('edit', entry.id)"
          >
            edit
          </button>
          <button
            type="button"
            class="note-comment__act note-comment__act--danger"
            :disabled="saving"
            @click="emit('remove', entry.id)"
          >
            delete
          </button>
        </div>
      </header>

      <p v-if="removeError" class="reader-comments__error" role="alert">
        {{ removeError }}
      </p>

      <div v-if="editing" class="note-comment__edit">
        <textarea
          v-model="edited"
          rows="3"
          :maxlength="MAX_NOTE"
          class="note-comment__field"
          :disabled="saving"
          :aria-invalid="!!editError"
        />
        <p v-if="editError" class="reader-comments__error" role="alert">{{ editError }}</p>
        <div class="note-comment__editrow">
          <span class="reader-comments__counter">
            {{ edited.length }}/{{ MAX_NOTE }}
          </span>
          <div class="note-comment__editacts">
            <button
              type="button"
              class="note-comment__cancel"
              @click="emit('cancelEdit')"
            >
              Cancel
            </button>
            <button
              type="button"
              class="note-comment__save"
              :disabled="!edited.trim() || saving"
              @click="emit('save', entry.id, edited.trim())"
            >
              {{ saving ? 'Saving…' : 'Save' }}
            </button>
          </div>
        </div>
      </div>

      <!-- Plain text, never `v-html`: this is anybody's writing, and there is
           no sanitiser in this app to put between it and the page. -->
      <p v-else-if="entry.body" class="note-comment__body">{{ entry.body }}</p>

      <!-- Under the note, not over it: the note is what was written, the
           passage is the context for it. A button only when there is a
           highlight up the page to go to. -->
      <button
        v-if="entry.selectedText && jumpable"
        type="button"
        class="note-comment__quote"
        @click="emit('jump', entry.id)"
      >
        “{{ entry.selectedText }}”
      </button>
      <blockquote
        v-else-if="entry.selectedText"
        class="note-comment__quote note-comment__quote--still"
      >
        “{{ entry.selectedText }}”
      </blockquote>
    </div>
  </article>
</template>

<style scoped>
/* A quote with nowhere to jump to: the same box, without the affordance. */
.note-comment__quote--still {
  margin: 0;
  cursor: default;
}

.note-comment__quote--still:hover { background: var(--surface-sunken); }
</style>
