<script setup lang="ts">
/**
 * The mark, and the only place the site draws it. Nothing else references a
 * logo file: the drawings below are the shapes of `public/brand/app-icon.svg`,
 * `logo.svg` and `logo-dark.svg`, copied element for element (only the
 * files' provenance `<metadata>` is left out). Inline rather than an `<img>`,
 * so there is no request and no flash, and SSR and the client render the same
 * bytes.
 *
 * - `tile`      the mark on the dark #202020 tile — the navbar logo, and the
 *               one to use on any ground, dark ones included.
 * - `mark`      the bare ink mark, for light grounds.
 * - `mark-dark` the bare light mark, for dark grounds.
 *
 * Decorative by default (`aria-hidden`), because it almost always sits next to
 * text or inside a link that carries the name. Pass `label` where it stands
 * alone, and it becomes an image with that name.
 *
 * Brand rules — no recolouring the lamp, no filters, no effects, no rotation,
 * minimum 16px tile / 20px bare mark — are in BRAND.md.
 */
const props = withDefaults(defineProps<{
  variant?: 'tile' | 'mark' | 'mark-dark'
  size?: number
  label?: string
}>(), {
  variant: 'tile',
  size: 24,
  label: undefined,
})
</script>

<template>
  <svg
    class="logo"
    :class="`logo--${props.variant}`"
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 32 32"
    :width="props.size"
    :height="props.size"
    :role="props.label ? 'img' : undefined"
    :aria-label="props.label"
    :aria-hidden="props.label ? undefined : 'true'"
    focusable="false"
  >
    <template v-if="props.variant === 'tile'">
      <rect width="32" height="32" rx="5" fill="#202020" />
      <g transform="translate(5.5 5.5) scale(0.656)">
        <path d="M9.5 3.5H4.5V28.5H27.5V3.5H22.5" fill="none" stroke="#f2f2f2" stroke-width="3.6" stroke-linecap="square" />
        <path d="M11.8 28.5 13.9 13.6H18.1L20.2 28.5Z" fill="#f2f2f2" />
        <rect x="12.6" y="11.4" width="6.8" height="1.9" fill="#f2f2f2" />
        <rect x="13.9" y="7.2" width="4.2" height="3.8" fill="#f2a41f" />
        <path d="M13.2 6.8 16 3.8 18.8 6.8Z" fill="#f2f2f2" />
      </g>
    </template>

    <template v-else-if="props.variant === 'mark-dark'">
      <path d="M9.5 3.5H4.5V28.5H27.5V3.5H22.5" fill="none" stroke="#f2f2f2" stroke-width="2.76" stroke-linecap="square" />
      <path d="M11.8 28.5 13.9 13.6H18.1L20.2 28.5Z" fill="#f2f2f2" />
      <rect x="12.6" y="11.4" width="6.8" height="1.9" fill="#f2f2f2" />
      <rect x="13.9" y="7.2" width="4.2" height="3.8" fill="#f2a41f" />
      <path d="M13.2 6.8 16 3.8 18.8 6.8Z" fill="#f2f2f2" />
    </template>

    <template v-else>
      <path d="M9.5 3.5H4.5V28.5H27.5V3.5H22.5" fill="none" stroke="#202020" stroke-width="2.76" stroke-linecap="square" />
      <path d="M11.8 28.5 13.9 13.6H18.1L20.2 28.5Z" fill="#202020" />
      <rect x="12.6" y="11.4" width="6.8" height="1.9" fill="#202020" />
      <rect x="13.9" y="7.2" width="4.2" height="3.8" fill="#f2a41f" />
      <path d="M13.2 6.8 16 3.8 18.8 6.8Z" fill="#202020" />
    </template>
  </svg>
</template>

<style scoped>
.logo {
  display: block;
  flex: none;
}

/*
 * The ink mark is for light grounds only. Under the site's dark theme the page
 * it sits on is dark, so it takes `logo-dark.svg`'s ink — the sanctioned dark
 * variant, the same #f2f2f2 — rather than putting #202020 on #111. The lamp is
 * untouched. Not a filter: the brand rules forbid `filter: invert()`.
 */
:root[data-theme="dark"] .logo--mark [fill="#202020"] { fill: #f2f2f2; }
:root[data-theme="dark"] .logo--mark [stroke="#202020"] { stroke: #f2f2f2; }
</style>
