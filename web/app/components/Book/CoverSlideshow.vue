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
  <div v-if="images.length" class="hidden flex-col items-center gap-3 lg:flex">
    <div
      class="relative aspect-[16/10] w-full cursor-zoom-in overflow-hidden rounded-xl"
      @click="zoom(at)"
    >
      <Transition name="cover-fade" mode="out-in">
        <img
          :key="at"
          :src="images[at]"
          :alt="title"
          class="absolute inset-0 h-full w-full rounded-xl object-contain"
        >
      </Transition>
    </div>

    <div v-if="images.length > 1" class="flex gap-1.5">
      <button
        v-for="(image, i) in images"
        :key="image"
        type="button"
        class="size-2 rounded-full transition-colors"
        :class="i === at ? 'bg-ink' : 'bg-ink/20'"
        :aria-label="`Show image ${i + 1}`"
        @click="show(i)"
      />
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
