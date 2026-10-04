<script setup lang="ts">
/**
 * The design system's Button.
 *
 * Three variants and three sizes, plus the two CTA overrides the site uses
 * everywhere: `pro` (Get Pro — serif, dark gradient, usually with a flame) and
 * `free` (Start Free — an outlined ghost). `pro-dark` is the Get Pro that sits
 * on an inverse panel.
 *
 * Renders a NuxtLink when given `to`, a real <button> otherwise, so a control
 * that does something is never a link pretending to be one.
 */
const props = withDefaults(
  defineProps<{
    variant?: 'inverse' | 'ghost' | 'accent'
    size?: 'sm' | 'md' | 'lg'
    cta?: 'pro' | 'pro-dark' | 'free'
    to?: string
    target?: string
    type?: 'button' | 'submit'
    disabled?: boolean
    flame?: boolean
    /** A full page load even for a path on this site — an OAuth start, say. */
    external?: boolean
    block?: boolean
  }>(),
  {
    variant: 'ghost',
    size: 'md',
    cta: undefined,
    to: undefined,
    target: undefined,
    type: 'button',
    disabled: false,
    flame: false,
    external: false,
    block: false,
  },
)

const isExternal = computed<boolean>(() => props.external || /^https?:\/\//.test(props.to ?? ''))

const classes = computed(() => [
  'btn',
  `btn-${props.size}`,
  `btn-${props.variant}`,
  props.cta ? `btn-cta btn-${props.cta}` : null,
  props.block ? 'btn-block' : null,
])
</script>

<template>
  <NuxtLink
    v-if="to && !disabled"
    :to="to"
    :external="isExternal"
    :target="target"
    :rel="target === '_blank' ? 'noopener' : undefined"
    :class="classes"
  >
    <FlameIcon v-if="flame" />
    <slot />
  </NuxtLink>

  <button
    v-else
    :type="type"
    :disabled="disabled"
    :class="classes"
  >
    <FlameIcon v-if="flame" />
    <slot />
  </button>
</template>

<style scoped>
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  border: 0;
  border-radius: var(--radius-full);
  font-family: var(--font-sans);
  font-weight: var(--weight-medium);
  text-decoration: none;
  white-space: nowrap;
  cursor: pointer;
  transition: var(--transition-control);
}

.btn:disabled { cursor: not-allowed; opacity: 0.4; }

.btn-block { display: flex; width: 100%; }

.btn-sm { height: 28px; padding: 0 12px; font-size: 14px; }
.btn-md { height: 32px; padding: 0 14px; font-size: 15px; }
.btn-lg { height: 44px; padding: 0 22px; font-size: 15px; }

.btn-inverse { background: var(--surface-inverse); color: var(--ink-inverse); }
.btn-ghost { background: var(--surface-raised); color: var(--ink); box-shadow: var(--shadow-sm); }
.btn-accent { background: var(--accent); color: var(--accent-ink); }

.btn-inverse:hover:not(:disabled) { opacity: 0.88; color: var(--ink-inverse); }
.btn-ghost:hover:not(:disabled) { box-shadow: var(--shadow-md); color: var(--ink); }
.btn-accent:hover:not(:disabled) { opacity: 0.9; color: var(--accent-ink); }

/* CTA overrides — radius 6, height 40, padding 0 16 */
.btn-cta { height: 40px; padding: 0 16px; border-radius: 6px; }

/* the two gradients are the handoff's named exceptions */
.btn-pro {
  font: 400 14px/1 var(--font-serif);
  background: linear-gradient(180deg, #2a2a2a, #161616);
  color: var(--ink-inverse);
}

.btn-pro-dark {
  font: 400 14px/1 var(--font-serif);
  background: linear-gradient(180deg, #3a3a3a, #1e1e1e);
  border: 1px solid rgba(255, 255, 255, 0.18);
  color: var(--ink-inverse);
}

.btn-pro:hover:not(:disabled),
.btn-pro-dark:hover:not(:disabled) { opacity: 0.88; color: var(--ink-inverse); }

.btn-free {
  font-size: 13px;
  box-shadow: none;
  border: 1px solid var(--ink);
}

.btn-free:hover:not(:disabled) { box-shadow: none; background: var(--surface-sunken); }
</style>
