<script setup lang="ts">
/**
 * One saved note, shown against the passage it was taken on.
 *
 * Opened by clicking the highlight rather than by hovering it: a reader's
 * pointer crosses a highlight on the way to somewhere else all the time, and a
 * panel that opens on the way past is a panel in the way.
 */

import type { Note } from '@/composables/UseNotes'

defineProps<{
  note: Note
  x: number
  y: number
  removing: boolean
  /** Why the last delete failed, if it did. */
  error: string | null
}>()

const emit = defineEmits<{
  remove: []
  close: []
}>()
</script>

<template>
  <div
    class="reader-notepop"
    :style="{ left: `${x}px`, top: `${y}px` }"
    role="dialog"
    aria-label="Your note"
    @mousedown.stop
    @mouseup.stop
  >
    <p class="reader-notepop__body">
      {{ note.noteContent }}
    </p>

    <div class="reader-notepop__foot">
      <span v-if="!note.isPublic" class="lh-mono lh-faint">
        private
      </span>
      <span v-else />

      <div class="reader-notepop__actions">
        <button
          type="button"
          class="reader-notepop__action"
          :disabled="removing"
          @click="emit('remove')"
        >
          {{ removing ? 'deleting…' : 'delete' }}
        </button>
        <button
          type="button"
          class="reader-notepop__action"
          @click="emit('close')"
        >
          close
        </button>
      </div>
    </div>

    <p v-if="error" class="reader-notepop__error" role="alert">{{ error }}</p>
  </div>
</template>
