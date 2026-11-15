<script setup lang="ts">
/**
 * A lesson image, full screen, over the page.
 *
 * The body is `v-html`, so the images in it are not vue's to put a button
 * round. The page listens for clicks on the body and hands the image here;
 * this only draws it and closes. The look is the cover lightbox's, so a
 * reader who has zoomed one has zoomed the other.
 */

export interface ZoomedImage {
  src: string
  alt: string
}

const props = defineProps<{ image: ZoomedImage | null }>()
const emit = defineEmits<{ close: [] }>()

function onKeydown(event: KeyboardEvent): void {
  if (props.image && event.key === 'Escape') emit('close')
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', onKeydown))
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="image" class="lightbox" role="dialog" :aria-label="image.alt || 'Image'" @click.self="emit('close')">
        <button type="button" class="close" aria-label="Close" @click="emit('close')">×</button>
        <img :src="image.src" :alt="image.alt">
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.lightbox {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-4);
  background: rgb(0 0 0 / 0.8);
  cursor: zoom-out;
}

/* diagrams are drawn on a light ground, so they keep one in the dark */
.lightbox img {
  max-width: 90vw;
  max-height: 90vh;
  border-radius: var(--radius-sm);
  background: #fff;
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
.fade-leave-active { transition: opacity 200ms ease; }

.fade-enter-from,
.fade-leave-to { opacity: 0; }
</style>
