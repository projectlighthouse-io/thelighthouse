<script setup lang="ts">
/**
 * What a reader may do with the passage they just selected.
 *
 * Floats where the selection ended rather than in a fixed corner: the reader is
 * looking at the words they highlighted, and a menu anywhere else asks them to
 * look away and come back.
 *
 * It offers, and decides nothing. Whether there is a session, whether the note
 * saved, what a bookmark replaces — all the page's.
 */

/** Past this a selection is a passage, not a place. See `bookmarkable`. */
const BOOKMARK_LIMIT = 100

const props = defineProps<{
  x: number
  y: number
  /** Characters selected. Decides whether bookmarking is offered at all. */
  length: number
  /** Anonymous readers get one button, and it goes to the sign-in panel. */
  signedIn: boolean
  /** Whether a bookmark already sits on this lesson, so the verb can say so. */
  bookmarked: boolean
}>()

const emit = defineEmits<{
  note: []
  bookmark: []
  signIn: []
}>()

/**
 * A bookmark is a place, so a whole paragraph is not one.
 *
 * The laravel reader draws the same line at the same hundred characters. It is
 * not a storage limit — the column takes five hundred — it is that "where I
 * left off" stops meaning anything once it spans half a screen.
 */
const bookmarkable = computed<boolean>(() => props.length <= BOOKMARK_LIMIT)

/**
 * Kept inside the viewport, and offset from the cursor rather than under it.
 *
 * The menu is `position: fixed`, so these are viewport coordinates and no
 * scroll offset belongs in them. The right-hand clamp is what stops a selection
 * ending near the right margin from opening a menu half off screen.
 */
const placement = computed(() => ({
  left: `${Math.min(props.x + 8, (import.meta.client ? window.innerWidth : 0) - 240)}px`,
  top: `${props.y + 12}px`,
}))
</script>

<template>
  <div
    class="reader-selectmenu"
    :style="placement"
    role="menu"
    @mousedown.stop
    @mouseup.stop
  >
    <template v-if="signedIn">
      <button
        type="button"
        class="reader-selectmenu__item"
        @click="emit('note')"
      >
        write a note
      </button>

      <template v-if="bookmarkable">
        <span class="reader-selectmenu__sep" />
        <button
          type="button"
          class="reader-selectmenu__item"
          @click="emit('bookmark')"
        >
          {{ bookmarked ? 'move bookmark here' : 'bookmark here' }}
        </button>
      </template>
    </template>

    <!-- One button, and it says what it is for. A menu that offers actions and
         then bounces the reader to a sign-in page is a menu that lied. -->
    <button
      v-else
      type="button"
      class="reader-selectmenu__item"
      @click="emit('signIn')"
    >
      sign in to take notes
    </button>
  </div>
</template>
