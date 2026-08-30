<script setup lang="ts">
/**
 * A book's images, cycling, with a lightbox behind a click.
 *
 * Ported from the laravel book page. The cover first and then figures from
 * inside — it is the book showing what is in it, which is a different job from
 * `thumbnailUrl`, whose one image goes on cards and into link previews.
 */

/** How long each image holds. The laravel original's interval, unchanged. */
const CYCLE_MS = 4000

const props = defineProps<{
  images: string[]
  title: string
}>()

const at = ref<number>(0)
const zoomed = ref<number | null>(null)

let timer: ReturnType<typeof setInterval> | null = null

const stop = (): void => {
  if (timer) clearInterval(timer)
  timer = null
}

/**
 * Only when there is more than one, and never while the lightbox is open —
 * an image that changes under the reader who just clicked it is not a feature.
 */
const start = (): void => {
  stop()
  if (props.images.length < 2) return

  timer = setInterval(() => {
    at.value = (at.value + 1) % props.images.length
  }, CYCLE_MS)
}

const show = (index: number): void => {
  at.value = index
  start()
}

/** the arrows either side of the dots. wraps in both directions. */
const step = (by: number): void => {
  const count = props.images.length
  show((at.value + by + count) % count)
}

const zoom = (index: number): void => {
  zoomed.value = index
  stop()
}

const close = (): void => {
  zoomed.value = null
  start()
}

const onKeydown = (event: KeyboardEvent): void => {
  if (zoomed.value !== null && event.key === 'Escape') close()
}

onMounted(() => {
  start()
  document.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  stop()
  document.removeEventListener('keydown', onKeydown)
})

// A different book has different images, and the old index may not exist in
// the new set.
watch(() => props.images, () => {
  at.value = 0
  zoomed.value = null
  start()
})
</script>

<template>
  <!-- Shown at every width, not hidden below lg: the hero collapses to one
       column on a narrow screen and the images sit under the title there,
       which is a stack, not a reason to drop them. -->
  <div v-if="images.length" class="flex max-w-[300px] flex-col gap-2.5">
    <div
      class="relative aspect-[4/3] w-full cursor-zoom-in overflow-hidden rounded-lg"
      @click="zoom(at)"
    >
      <Transition name="cover-fade" mode="out-in">
        <img
          :key="at"
          :src="images[at]"
          :alt="title"
          class="absolute inset-0 h-full w-full rounded-lg object-contain"
        >
      </Transition>
    </div>

    <div v-if="images.length > 1" class="flex items-center justify-center gap-3.5">
      <button
        type="button"
        class="text-read-faint transition-colors hover:text-teal-deep"
        aria-label="Previous image"
        @click="step(-1)"
      >
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path
            d="M10 3.5L5.5 8l4.5 4.5"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>

      <div class="flex gap-1.5">
        <button
          v-for="(image, i) in images"
          :key="image"
          type="button"
          class="size-[5px] rounded-full transition-colors"
          :class="i === at ? 'bg-teal' : 'bg-read-line'"
          :aria-label="`Show image ${i + 1}`"
          @click="show(i)"
        />
      </div>

      <button
        type="button"
        class="text-read-faint transition-colors hover:text-teal-deep"
        aria-label="Next image"
        @click="step(1)"
      >
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path
            d="M6 3.5L10.5 8 6 12.5"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    </div>

    <Teleport to="body">
      <Transition name="cover-fade">
        <div
          v-if="zoomed !== null"
          class="fixed inset-0 z-[100] flex items-center justify-center bg-black/80 p-4 backdrop-blur-sm"
          @click.self="close"
        >
          <button
            type="button"
            class="absolute top-4 right-4 rounded-full bg-white/10 p-2 text-white transition hover:bg-white/20"
            aria-label="Close"
            @click="close"
          >
            <svg
              class="size-6"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
            >
              <path d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
          <img
            :src="images[zoomed]"
            :alt="title"
            class="max-h-[90vh] max-w-[90vw] rounded-lg object-contain"
          >
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.cover-fade-enter-active,
.cover-fade-leave-active {
    transition: opacity 500ms ease;
}

.cover-fade-enter-from,
.cover-fade-leave-to {
    opacity: 0;
}
</style>
