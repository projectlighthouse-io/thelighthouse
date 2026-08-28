<script setup lang="ts">
/**
 * Everything this reader wrote on this lesson, under the lesson.
 *
 * The highlights in the prose say *where* a note is; this says what they all
 * are, in one place, without hunting up the page for yellow. It is the reader's
 * own margin, collected — which is also why it shows nobody else's: the api
 * scopes the listing to the session, and a public note's thread is a different
 * feature that does not exist yet.
 */

import type { Note } from '@/composables/UseNotes'

defineProps<{
  notes: Note[]
  /** The note being edited, if any, so only one field is open at a time. */
  editingId: number | null
  savingId: number | null
}>()

const emit = defineEmits<{
  jump: [note: Note]
  edit: [note: Note]
  save: [note: Note, content: string]
  cancelEdit: []
  remove: [note: Note]
}>()

/** The draft, kept here: the list owns the field, so the page need not. */
const draft = ref<string>('')

const startEdit = (note: Note): void => {
  draft.value = note.noteContent ?? ''
  emit('edit', note)
}

/**
 * The day it was written, and not the time.
 *
 * A note from this morning and one from March are told apart by the date; the
 * minute they were saved at has never answered a question anybody asked.
 */
const on = (iso: string | null): string =>
  iso
    ? new Date(iso).toLocaleDateString(undefined, {
        day: 'numeric',
        month: 'short',
        year: 'numeric',
      })
    : ''
</script>

<template>
  <section v-if="notes.length" class="reader-notes">
    <h2 class="reader-notes__title">
      your notes
      <span class="reader-notes__count">{{ notes.length }}</span>
    </h2>

    <ul class="reader-notes__list">
      <li v-for="note in notes" :key="note.id" class="reader-notes__item">
        <!-- Only when there is somewhere to go. A note written with nothing
             selected has no highlight in the prose to scroll to. -->
        <button
          v-if="note.selectedText"
          type="button"
          class="reader-notes__passage"
          @click="emit('jump', note)"
        >
          {{ note.selectedText }}
        </button>

        <div v-if="editingId === note.id" class="reader-notes__edit">
          <textarea v-model="draft" class="reader-notedialog__field" rows="3" />
          <div class="reader-notes__actions">
            <button
              type="button"
              class="reader-notepop__action"
              @click="emit('cancelEdit')"
            >
              cancel
            </button>
            <button
              type="button"
              class="reader-notepop__action"
              :disabled="savingId === note.id || !draft.trim()"
              @click="emit('save', note, draft.trim())"
            >
              {{ savingId === note.id ? 'saving…' : 'save' }}
            </button>
          </div>
        </div>

        <template v-else>
          <p class="reader-notes__body">{{ note.noteContent }}</p>

          <div class="reader-notes__meta">
            <span class="reader-notes__when">
              {{ on(note.createdAt) }}<template v-if="!note.isPublic"> · private</template>
            </span>

            <div class="reader-notes__actions">
              <button
                type="button"
                class="reader-notepop__action"
                @click="startEdit(note)"
              >
                edit
              </button>
              <button
                type="button"
                class="reader-notepop__action"
                :disabled="savingId === note.id"
                @click="emit('remove', note)"
              >
                delete
              </button>
            </div>
          </div>
        </template>
      </li>
    </ul>
  </section>
</template>
