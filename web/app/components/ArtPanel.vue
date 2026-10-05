<script setup lang="ts">
/**
 * The tall photo panel that sits beside a form.
 *
 * Extracted from the blog editor so the next page that wants one does not copy
 * a background, two gradients and a set of sticky rules that have to stay in
 * step. `/login` has its own, older version of this in scoped css — worth
 * folding into this component when that page is next touched, rather than as a
 * drive-by here.
 *
 * The caption is a prop rather than baked in: the panel is a frame, and what
 * it says belongs to the page using it.
 *
 * Decoration, so `aria-hidden`: the photo carries no information, and the
 * caption is a flourish rather than content anybody navigating by screen
 * reader is missing. Nothing here is the only copy of anything.
 *
 * Positioning classes are on the root, so a caller can add or override them —
 * Vue merges fallthrough `class` onto the root element.
 */
import { AUTH_ART } from '@/data/Auth'

withDefaults(
  defineProps<{
    /** The line across the bottom. */
    caption?: string
    /** Defaults to the image `/login` uses, so the two surfaces match. */
    image?: string
  }>(),
  {
    caption: '',
    image: AUTH_ART.image,
  },
)
</script>

<template>
  <aside
    aria-hidden="true"
    class="relative hidden p-4 lg:block lg:h-full"
  >
    <!-- The image is on an inner box so the padding sits *outside* it: `p-4`
         on the element carrying the background would inset the content and
         leave the photo running to the edge underneath, which is the opposite
         of what the rounding is for. -->
    <div class="relative h-full overflow-hidden rounded-md">
      <!-- No fade at the right edge, deliberately.
           Two attempts were made and both removed: a page-coloured gradient
           painted over the photo, which tinted it milky, and a `mask-image`,
           which dissolved it into the white page and read as a lens flare
           across the brightest part of the sky. Both were softening the hard
           vertical line that a full-bleed panel created — and the rounded
           corners and the `p-4` gutter already separate this from whatever
           sits beside it, so there is no line left to soften. -->
      <div
        class="absolute inset-0"
        :style="{
          backgroundColor: '#1c3a5e',
          backgroundImage: `linear-gradient(to top, rgba(10, 18, 34, 0.82) 0%, rgba(10, 18, 34, 0.3) 30%, rgba(10, 18, 34, 0) 52%), url('${image}')`,
          backgroundSize: 'cover',
          backgroundPosition: 'center 28%',
          backgroundRepeat: 'no-repeat',
        }"
      />

      <!-- Above the photo rather than inside it, so the scrim never tints the
           text it exists to make legible. -->
      <div class="relative flex h-full items-end p-10">
        <p v-if="caption" class="font-serif text-2xl leading-relaxed text-white">
          {{ caption }}
        </p>
      </div>
    </div>
  </aside>
</template>
