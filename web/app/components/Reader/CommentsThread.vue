<script setup lang="ts">
/**
 * Notes & Highlights: the thread under a lesson.
 *
 * A port of the laravel reader's `CommentsThread` and `NoteComment`, kept
 * visually the same on purpose — readers coming across from the old site should
 * not have to learn this again. A compose box on top, then a flat list of
 * avatar-and-body rows, each with the passage quoted underneath the note rather
 * than above it: the note is what the reader wrote, the quote is context for it.
 *
 * **Only this reader's notes.** The api scopes the listing to the session, so
 * every row here is the reader's own — which is why the avatar and name are
 * taken from the session rather than sent per note. A public note's thread,
 * with other people in it and replies under it, is `docs/rebuild.md`'s slice 3
 * and is what `is_public` and `parent_id` are being written for.
 */

import type { Note } from '@/composables/UseNotes'

/** Matches `notes::payload::MAX_NOTE` in the api, which enforces it. */
const MAX_NOTE = 500

const props = defineProps<{
  notes: Note[]
  loading: boolean
  signedIn: boolean
  submitting: boolean
  submitError: string | null
  editingId: number | null
  savingId: number | null
}>()

const emit = defineEmits<{
  submit: [content: string]
  jump: [note: Note]
  edit: [note: Note]
  save: [note: Note, content: string]
  cancelEdit: []
  remove: [note: Note]
  signIn: []
}>()

const { reader, initials } = useReader()

const draft = ref<string>('')
const edited = ref<string>('')

const post = (): void => {
  const content = draft.value.trim()
  if (!content || props.submitting) return

  emit('submit', content)
  draft.value = ''
}

const startEdit = (note: Note): void => {
  edited.value = note.noteContent ?? ''
  emit('edit', note)
}

/**
 * How long ago, in the units somebody actually asks in.
 *
 * Minutes for the last hour, hours for the last day, days for the last week,
 * and a date beyond that — past a week "9d ago" stops being easier to read
 * than "Mar 4".
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
        @keydown.meta.enter="post"
        @keydown.ctrl.enter="post"
      />
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
      <p v-if="submitError" class="reader-comments__error">{{ submitError }}</p>
    </div>

    <div v-if="loading" class="reader-comments__loading">Loading notes…</div>

    <div v-else-if="!notes.length" class="reader-comments__empty">
      Nothing yet. Select a passage as you read, or write a note above.
    </div>

    <div v-else class="reader-comments__list">
      <article v-for="note in notes" :key="note.id" class="note-comment">
        <img
          v-if="reader?.avatar"
          :src="reader.avatar"
          :alt="reader?.name ?? ''"
          class="note-comment__avatar"
        >
        <div v-else class="note-comment__avatar note-comment__avatar--initials">
          {{ initials }}
        </div>

        <div class="note-comment__main">
          <header class="note-comment__head">
            <span class="note-comment__who">{{ reader?.name ?? 'You' }}</span>

            <span v-if="!note.isPublic" class="note-comment__lock">private</span>

            <span class="note-comment__dot">·</span>
            <span class="note-comment__when">{{ since(note.createdAt) }}</span>

            <div class="note-comment__acts">
              <button
                type="button"
                class="note-comment__act"
                @click="startEdit(note)"
              >
                edit
              </button>
              <button
                type="button"
                class="note-comment__act note-comment__act--danger"
                :disabled="savingId === note.id"
                @click="emit('remove', note)"
              >
                delete
              </button>
            </div>
          </header>

          <div v-if="editingId === note.id" class="note-comment__edit">
            <textarea
              v-model="edited"
              rows="3"
              :maxlength="MAX_NOTE"
              class="note-comment__field"
              :disabled="savingId === note.id"
            />
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
                  :disabled="!edited.trim() || savingId === note.id"
                  @click="emit('save', note, edited.trim())"
                >
                  {{ savingId === note.id ? 'Saving…' : 'Save' }}
                </button>
              </div>
            </div>
          </div>

          <!-- Plain text, not markdown. The laravel reader renders `**bold**`
               through marked and then DOMPurify; this app has marked but no
               sanitiser, and user-authored html into `v-html` without one is
               an xss hole. Text until dompurify is added. -->
          <p v-else-if="note.noteContent" class="note-comment__body">
            {{ note.noteContent }}
          </p>

          <!-- Under the note, not over it: the note is what the reader wrote,
               and the passage is the context for it. Clickable because there is
               a highlight up the page it belongs to. -->
          <button
            v-if="note.selectedText"
            type="button"
            class="note-comment__quote"
            @click="emit('jump', note)"
          >
            “{{ note.selectedText }}”
          </button>
        </div>
      </article>
    </div>
  </section>
</template>
