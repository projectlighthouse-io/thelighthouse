<script setup lang="ts">
/**
 * Notes & Highlights: the thread under a lesson.
 *
 * A port of the laravel reader's `CommentsThread` and `NoteComment`, kept
 * visually the same on purpose — readers coming across from the old site should
 * not have to learn this again. A compose box on top, then the rows.
 *
 * **The public thread, for everyone.** Every reader's shared notes, newest
 * first, twenty at a time, each with its own author and its replies indented
 * under it. Signed out, it is read-only.
 *
 * **Then the reader's private notes, to the reader.** The public thread cannot
 * hold them — the api answers it identically to everybody — so they come from
 * the reader's own listing and sit in a group of their own, marked private, so
 * nobody mistakes what they wrote for themselves for something others can see.
 */

import type { Note } from '@/composables/UseNotes'
import type { CommentEntry, LessonComment } from '@/types/Comments'

/** Matches `notes::payload::MAX_NOTE` in the api, which enforces it. */
const MAX_NOTE = 500

const props = defineProps<{
  comments: LessonComment[]
  /** Top-level comments in the thread, not counting replies. */
  total: number
  /** Whether the first page has answered, either way. */
  loaded: boolean
  /** A page is in flight. */
  loading: boolean
  /** The last request failed. */
  failed: boolean
  hasMore: boolean
  /** The reader's private notes on this lesson. Empty when signed out. */
  privateNotes: Note[]
  /** `Reader.sub`, but only once hydrated — `null` on the server and signed
   *  out, so the server render and the first client render agree. */
  readerId: string | null
  /** Notes with a highlight painted up the page. */
  jumpable: number[]
  signedIn: boolean
  submitting: boolean
  /** The api's refusal of the compose box's text, shown under it. */
  submitFieldError: string | null
  submitError: string | null
  editingId: number | null
  savingId: number | null
  /** Why the note being edited was not saved. */
  editError: string | null
  /** Why a delete failed, and which note it was for. */
  removeError: { id: number, message: string } | null
}>()

const emit = defineEmits<{
  submit: [content: string]
  jump: [id: number]
  edit: [id: number]
  save: [id: number, content: string]
  cancelEdit: []
  remove: [id: number]
  signIn: []
  more: []
  retry: []
}>()

const { reader } = useReader()

const draft = ref<string>('')

const post = (): void => {
  const content = draft.value.trim()
  if (!content || props.submitting) return

  emit('submit', content)
}

// The draft goes only once the api has taken it. Clearing on submit threw away
// what the reader typed whenever the post was refused.
watch(() => props.submitting, (now, was) => {
  if (was && !now && !props.submitError && !props.submitFieldError) draft.value = ''
})

const mine = (entry: CommentEntry): boolean =>
  props.readerId !== null && String(entry.author.id) === props.readerId

const canJump = computed<Set<number>>(() => new Set(props.jumpable))

/** The private notes in the row's shape, authored by the session's reader. */
const privateEntries = computed<CommentEntry[]>(() => {
  if (props.readerId === null) return []

  return props.privateNotes.map(note => ({
    id: note.id,
    selectedText: note.selectedText,
    body: note.noteContent,
    startOffset: note.startOffset,
    endOffset: note.endOffset,
    createdAt: note.createdAt,
    author: {
      id: Number(props.readerId),
      name: reader.value?.name ?? 'You',
      username: reader.value?.username ?? null,
      avatarUrl: reader.value?.avatar ?? null,
    },
  }))
})

const countLabel = computed<string>(
  () => `${props.total} ${props.total === 1 ? 'comment' : 'comments'}`,
)

const rowError = (id: number): string | null =>
  props.removeError?.id === id ? props.removeError.message : null
</script>

<template>
  <section class="reader-comments">
    <h2 class="reader-comments__title">Notes &amp; Highlights</h2>

    <div v-if="!signedIn" class="reader-comments__signin">
      <button
        type="button"
        class="reader-comments__signin-link"
        @click="emit('signIn')"
      >
        Sign in
      </button>
      to leave a note, or highlight a passage as you read.
    </div>

    <div v-else class="reader-comments__compose">
      <textarea
        v-model="draft"
        :maxlength="MAX_NOTE"
        rows="3"
        class="reader-comments__textarea"
        placeholder="Have a thought? Mark-it-down"
        :disabled="submitting"
        :aria-invalid="!!submitFieldError"
        @keydown.meta.enter="post"
        @keydown.ctrl.enter="post"
      />
      <p v-if="submitFieldError" class="reader-comments__error" role="alert">{{ submitFieldError }}</p>
      <div class="reader-comments__compose-row">
        <span class="reader-comments__counter">
          {{ draft.length }}/{{ MAX_NOTE }}
        </span>
        <button
          type="button"
          class="reader-comments__submit"
          :disabled="!draft.trim() || submitting"
          @click="post"
        >
          {{ submitting ? 'Posting…' : 'Post' }}
        </button>
      </div>
      <p v-if="submitError" class="reader-comments__error" role="alert">{{ submitError }}</p>
    </div>

    <div v-if="privateEntries.length" class="reader-comments__group">
      <p class="reader-comments__label">Your private notes · only you can see these</p>
      <div class="reader-comments__list">
        <ReaderCommentRow
          v-for="entry in privateEntries"
          :key="entry.id"
          :entry="entry"
          :mine="true"
          is-private
          :jumpable="canJump.has(entry.id)"
          :editing="editingId === entry.id"
          :saving="savingId === entry.id"
          :edit-error="editingId === entry.id ? editError : null"
          :remove-error="rowError(entry.id)"
          @jump="emit('jump', $event)"
          @edit="emit('edit', $event)"
          @save="(id, content) => emit('save', id, content)"
          @cancel-edit="emit('cancelEdit')"
          @remove="emit('remove', $event)"
        />
      </div>
    </div>

    <!-- `loaded` is false on the server and on the first client render alike,
         so both draw this and hydration has nothing to disagree about. -->
    <div v-if="!loaded" class="reader-comments__loading">Loading comments…</div>

    <div v-else-if="failed && !comments.length" class="reader-comments__empty" role="alert">
      The comments could not be loaded.
      <button type="button" class="reader-comments__signin-link" @click="emit('retry')">
        Try again
      </button>
    </div>

    <div v-else-if="!comments.length" class="reader-comments__empty">
      No comments yet.
      <template v-if="signedIn">Select a passage as you read, or write one above.</template>
    </div>

    <div v-else class="reader-comments__group">
      <p class="reader-comments__label lh-num">{{ countLabel }}</p>

      <div class="reader-comments__list">
        <div v-for="comment in comments" :key="comment.id" class="reader-comments__thread">
          <ReaderCommentRow
            :entry="comment"
            :mine="mine(comment)"
            :jumpable="canJump.has(comment.id)"
            :editing="editingId === comment.id"
            :saving="savingId === comment.id"
            :edit-error="editingId === comment.id ? editError : null"
            :remove-error="rowError(comment.id)"
            @jump="emit('jump', $event)"
            @edit="emit('edit', $event)"
            @save="(id, content) => emit('save', id, content)"
            @cancel-edit="emit('cancelEdit')"
            @remove="emit('remove', $event)"
          />

          <div v-if="comment.replies.length" class="reader-comments__replies">
            <ReaderCommentRow
              v-for="reply in comment.replies"
              :key="reply.id"
              :entry="reply"
              :mine="mine(reply)"
              :jumpable="canJump.has(reply.id)"
              :editing="editingId === reply.id"
              :saving="savingId === reply.id"
              :edit-error="editingId === reply.id ? editError : null"
              :remove-error="rowError(reply.id)"
              @jump="emit('jump', $event)"
              @edit="emit('edit', $event)"
              @save="(id, content) => emit('save', id, content)"
              @cancel-edit="emit('cancelEdit')"
              @remove="emit('remove', $event)"
            />
          </div>
        </div>
      </div>

      <p v-if="failed" class="reader-comments__error" role="alert">
        Older comments could not be loaded.
      </p>

      <button
        v-if="hasMore"
        type="button"
        class="reader-comments__more"
        :disabled="loading"
        @click="emit('more')"
      >
        {{ loading ? 'Loading…' : failed ? 'Try again' : 'Show older comments' }}
      </button>
    </div>
  </section>
</template>

<style scoped>
/* reader.css owns the rest of this section; these are the pieces the public
   thread added — a label over each group, replies indented under their root,
   and the button that pages. */
.reader-comments__group {
  display: grid;
  gap: var(--space-4);
}

.reader-comments__label {
  margin: 0;
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

.reader-comments__thread {
  display: grid;
  gap: var(--space-4);
}

/* Lined up under the root's text column: the avatar is 32px and the gap 12px. */
.reader-comments__replies {
  display: grid;
  gap: var(--space-4);
  margin-left: 44px;
  padding-left: var(--space-4);
  border-left: 1px solid var(--border-strong);
}

@media (max-width: 480px) {
  .reader-comments__replies { margin-left: var(--space-3); }
}

.reader-comments__more {
  justify-self: start;
  height: 32px;
  padding: 0 14px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-full);
  background: transparent;
  font: 500 14px/1 var(--font-sans);
  color: var(--ink);
  cursor: pointer;
}

.reader-comments__more:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
