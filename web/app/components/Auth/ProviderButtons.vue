<script setup lang="ts">
/**
 * The two OAuth buttons, and nothing else.
 *
 * Extracted because /login and the header dropdown offer the same choice, and
 * the alternative was a second copy of both provider marks — the kind of
 * duplication that goes stale silently when a provider is added or a brand mark
 * is redrawn.
 */
// Kept as a props object rather than destructured: a bare `redirect` collides
// with an auto-import of the same name.
const props = withDefaults(
  defineProps<{
    /** Where to land after signing in. Rust validates it again and refuses
        anything that leaves the site. Undefined means rust's own default. */
    redirect?: string
  }>(),
  { redirect: undefined },
)

const { signInUrl } = useAuth()

// Plain hrefs, never fetches: the browser has to navigate to the provider, and
// an XHR to a cross-origin redirect would be blocked.
const githubUrl = computed(() => signInUrl('github', props.redirect))
const googleUrl = computed(() => signInUrl('google', props.redirect))
</script>

<template>
  <div class="oauth">
    <a class="btn btn--dark" :href="githubUrl">
      <svg class="ico" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 .5C5.7.5.5 5.7.5 12c0 5.1 3.3 9.4 7.9 10.9.6.1.8-.2.8-.5v-2c-3.2.7-3.9-1.4-3.9-1.4-.5-1.3-1.3-1.7-1.3-1.7-1.1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1 1.8 2.7 1.3 3.4 1 .1-.8.4-1.3.7-1.6-2.6-.3-5.3-1.3-5.3-5.7 0-1.3.5-2.3 1.2-3.1-.1-.3-.5-1.5.1-3.1 0 0 1-.3 3.3 1.2a11.5 11.5 0 0 1 6 0C17.3 4.7 18.3 5 18.3 5c.6 1.6.2 2.8.1 3.1.8.8 1.2 1.8 1.2 3.1 0 4.4-2.7 5.4-5.3 5.7.4.4.8 1.1.8 2.2v3.3c0 .3.2.6.8.5 4.6-1.5 7.9-5.8 7.9-10.9C23.5 5.7 18.3.5 12 .5z" />
      </svg>
      <span class="lbl">Continue with GitHub</span>
      <svg class="arr" viewBox="0 0 16 16" fill="none">
        <path
          d="M3 8h9M8.5 4l4 4-4 4"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </a>

    <a class="btn" :href="googleUrl">
      <!-- Google's brand colours are fixed in both themes; see docs/rebuild.md. -->
      <svg class="ico" viewBox="0 0 24 24">
        <path
          fill="#4285F4"
          d="M23.5 12.3c0-.8-.1-1.5-.2-2.3H12v4.5h6.5c-.3 1.5-1.1 2.7-2.4 3.6v3h3.9c2.3-2.1 3.5-5.2 3.5-8.8z"
        />
        <path
          fill="#34A853"
          d="M12 24c3.2 0 5.9-1.1 7.9-2.9l-3.9-3c-1.1.7-2.5 1.2-4 1.2-3.1 0-5.7-2.1-6.6-4.9H1.4v3.1C3.4 21.4 7.4 24 12 24z"
        />
        <path
          fill="#FBBC05"
          d="M5.4 14.3c-.2-.7-.4-1.5-.4-2.3s.1-1.6.4-2.3V6.6H1.4C.5 8.2 0 10 0 12s.5 3.8 1.4 5.4l4-3.1z"
        />
        <path
          fill="#EA4335"
          d="M12 4.8c1.8 0 3.3.6 4.6 1.8l3.4-3.4C17.9 1.2 15.2 0 12 0 7.4 0 3.4 2.6 1.4 6.6l4 3.1C6.3 6.9 8.9 4.8 12 4.8z"
        />
      </svg>
      <span class="lbl">Continue with Google</span>
      <svg class="arr" viewBox="0 0 16 16" fill="none">
        <path
          d="M3 8h9M8.5 4l4 4-4 4"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </a>
  </div>
</template>

<style scoped>
.oauth {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.btn {
  display: flex;
  align-items: center;
  gap: 14px;
  width: 100%;
  padding: 15px 18px;
  border-radius: 11px;
  border: 1px solid var(--color-read-line);
  background: var(--color-read-bg);
  color: var(--color-read-ink);
  font-family: 'Inter', -apple-system, system-ui, sans-serif;
  font-weight: 500;
  font-size: 15px;
  letter-spacing: -0.01em;
  text-decoration: none;
  cursor: pointer;
  transition:
    border-color 140ms,
    background 140ms,
    transform 80ms,
    box-shadow 140ms;
}

.btn:hover {
  border-color: var(--color-teal-mid);
  background: #fff;
  box-shadow: 0 10px 26px -16px rgba(22, 20, 15, 0.4);
}

.btn:active {
  transform: translateY(1px);
}

.btn .ico {
  width: 20px;
  height: 20px;
  flex: none;
}

.btn .lbl {
  flex: 1;
  text-align: left;
}

.btn .arr {
  width: 16px;
  height: 16px;
  color: var(--color-read-faint);
  transition:
    transform 160ms,
    color 140ms;
}

.btn:hover .arr {
  color: var(--color-teal-deep);
  transform: translateX(3px);
}

.btn--dark {
  background: var(--color-read-ink);
  border-color: var(--color-read-ink);
  color: #fff;
}

.btn--dark:hover {
  background: #000;
  border-color: #000;
}

.btn--dark .arr {
  color: rgba(255, 255, 255, 0.5);
}

.btn--dark:hover .arr {
  color: #fff;
}
</style>
