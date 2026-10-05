<script setup lang="ts">
/**
 * A book's images, cycling inside the cover plate, with a lightbox behind a
 * click.
 *
 * Ported from the laravel book page. The cover first and then figures from
 * inside — it is the book showing what is in it, which is a different job from
 * `thumbnailUrl`, whose one image goes on cards and into link previews.
 *
 * Fills whatever frame it is put in; the arrows and dots sit over the bottom
 * of the image, so the plate keeps its shape.
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

function stop(): void {
  if (timer) clearInterval(timer)
  timer = null
}

/**
 * Only when there is more than one, and never while the lightbox is open —
 * an image that changes under the reader who just clicked it is not a feature.
 */
function start(): void {
  stop()
  if (props.images.length < 2) return

  timer = setInterval(() => {
    at.value = (at.value + 1) % props.images.length
  }, CYCLE_MS)
}

function show(index: number): void {
  at.value = index
  start()
}

/** The arrows. Wraps in both directions. */
function step(by: number): void {
  const count = props.images.length
  show((at.value + by + count) % count)
}

function zoom(): void {
  zoomed.value = at.value
  stop()
}

function close(): void {
  zoomed.value = null
  start()
}

function onKeydown(event: KeyboardEvent): void {
  if (zoomed.value !== null && event.key === 'Escape') close()
}

onMounted(() => {
  // Not for a reader who asked for less motion: the arrows and dots still work.
  if (!globalThis.matchMedia?.('(prefers-reduced-motion: reduce)').matches) start()
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
  <div class="show">
    <button type="button" class="frame" :aria-label="`Enlarge image ${at + 1} of ${images.length}`" @click="zoom">
      <Transition name="fade">
        <img :key="at" :src="images[at]" :alt="title" fetchpriority="high">
      </Transition>
    </button>

    <div v-if="images.length > 1" class="controls">
      <button type="button" class="arrow" aria-label="Previous image" @click="step(-1)">‹</button>
      <button
        v-for="(image, i) in images"
        :key="image"
        type="button"
        class="dot"
        :class="{ 'is-on': i === at }"
        :aria-label="`Show image ${i + 1}`"
        :aria-current="i === at"
        @click="show(i)"
      />
      <button type="button" class="arrow" aria-label="Next image" @click="step(1)">›</button>
    </div>

    <Teleport to="body">
      <Transition name="fade">
        <div v-if="zoomed !== null" class="lightbox" role="dialog" aria-label="Image" @click.self="close">
          <button type="button" class="close" aria-label="Close" @click="close">×</button>
          <img :src="images[zoomed]" :alt="title">
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.show {
  position: relative;
  width: 100%;
  height: 100%;
}

.frame {
  position: relative;
  display: block;
  width: 100%;
  height: 100%;
  padding: 0;
  border: 0;
  background: none;
  cursor: zoom-in;
}

.frame img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.controls {
  position: absolute;
  left: 50%;
  bottom: var(--space-2);
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 0;
  padding: 0 var(--space-1);
  border-radius: var(--radius-full);
  background: color-mix(in srgb, var(--surface-page) 85%, transparent);
  box-shadow: var(--shadow-hairline);
}

/* 24px hit areas (WCAG target size); the dots stay 6px to the eye */
.arrow {
  min-width: 24px;
  min-height: 24px;
  padding: 0 4px;
  border: 0;
  background: none;
  color: var(--ink-muted);
  font: 400 16px/20px var(--font-sans);
  cursor: pointer;
}

.arrow:hover { color: var(--ink); }

.dot {
  --dot: var(--border-strong);
  width: 24px;
  height: 24px;
  padding: 0;
  border: 0;
  background: radial-gradient(circle, var(--dot) 3px, transparent 3.5px);
  cursor: pointer;
}

.dot.is-on { --dot: var(--ink); }

.lightbox {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-4);
  background: rgb(0 0 0 / 0.8);
}

.lightbox img {
  max-width: 90vw;
  max-height: 90vh;
  border-radius: var(--radius-sm);
  object-fit: contain;
}

.close {
  position: absolute;
  top: var(--space-4);
  right: var(--space-4);
  width: 40px;
  height: 40px;
  border: 0;
  border-radius: var(--radius-full);
  background: rgb(255 255 255 / 0.12);
  color: #fff;
  font: 400 24px/1 var(--font-sans);
  cursor: pointer;
}

.close:hover { background: rgb(255 255 255 / 0.22); }

.fade-enter-active,
.fade-leave-active { transition: opacity 500ms ease; }

.fade-enter-from,
.fade-leave-to { opacity: 0; }
</style>
