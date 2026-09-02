<script setup lang="ts">
/**
 * The two ways into the site, in one place.
 *
 * Shared by the login page and the header's join panel so there is one
 * definition of where sign-in goes. Two copies would drift, and the copy that
 * drifted would be an auth entry point.
 */
const props = defineProps<{
  /**
   * Where to land afterwards. Passed straight through to rust, which refuses
   * anything that is not a same-site absolute path before it becomes a
   * `Location` — see `safe_redirect`. Nothing here validates it: two checks
   * that can disagree are worse than one.
   */
  redirect?: string
}>()

const query = computed<string>(() =>
  props.redirect ? `?redirect=${encodeURIComponent(props.redirect)}` : '',
)

// rust owns the oauth dance: these are plain hrefs, never fetches
const githubUrl = computed<string>(() => `/api/auth/github${query.value}`)
const googleUrl = computed<string>(() => `/api/auth/google${query.value}`)
</script>

<template>
  <div class="oauth">
    <a class="oauth-btn oauth-btn--dark" :href="githubUrl">
      <svg class="oauth-btn__ico" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 .5C5.7.5.5 5.7.5 12c0 5.1 3.3 9.4 7.9 10.9.6.1.8-.2.8-.5v-2c-3.2.7-3.9-1.4-3.9-1.4-.5-1.3-1.3-1.7-1.3-1.7-1.1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1 1.8 2.7 1.3 3.4 1 .1-.8.4-1.3.7-1.6-2.6-.3-5.3-1.3-5.3-5.7 0-1.3.5-2.3 1.2-3.1-.1-.3-.5-1.5.1-3.1 0 0 1-.3 3.3 1.2a11.5 11.5 0 0 1 6 0C17.3 4.7 18.3 5 18.3 5c.6 1.6.2 2.8.1 3.1.8.8 1.2 1.8 1.2 3.1 0 4.4-2.7 5.4-5.3 5.7.4.4.8 1.1.8 2.2v3.3c0 .3.2.6.8.5 4.6-1.5 7.9-5.8 7.9-10.9C23.5 5.7 18.3.5 12 .5z" />
      </svg>
      <span class="oauth-btn__lbl">Continue with GitHub</span>
      <svg class="oauth-btn__arr" viewBox="0 0 16 16" fill="none">
        <path
          d="M3 8h9M8.5 4l4 4-4 4"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </a>

    <a class="oauth-btn" :href="googleUrl">
      <!-- Google's brand colours, fixed in both themes. A recoloured Google
           mark is not Google's mark. -->
      <svg class="oauth-btn__ico" viewBox="0 0 24 24">
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
      <span class="oauth-btn__lbl">Continue with Google</span>
      <svg class="oauth-btn__arr" viewBox="0 0 16 16" fill="none">
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
/* Tokens rather than the hexes the login page used, because this now renders
   on the header too — and the header goes dark. */
.oauth {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.oauth-btn {
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

.oauth-btn:hover {
  border-color: var(--color-teal-mid);
  background: var(--color-read-bg-soft);
  box-shadow: 0 10px 26px -16px rgb(22 20 15 / 40%);
}

.oauth-btn:active {
  transform: translateY(1px);
}

.oauth-btn__ico {
  width: 20px;
  height: 20px;
  flex: none;
}

.oauth-btn__lbl {
  flex: 1;
  text-align: left;
}

.oauth-btn__arr {
  width: 16px;
  height: 16px;
  color: var(--color-read-faint);
  transition:
    transform 160ms,
    color 140ms;
}

.oauth-btn:hover .oauth-btn__arr {
  color: var(--color-teal-deep);
  transform: translateX(3px);
}

.oauth-btn--dark {
  background: var(--color-ink);
  border-color: var(--color-ink);
  color: var(--color-on-ink);
}

.oauth-btn--dark:hover {
  background: var(--color-ink-hover);
  border-color: var(--color-ink-hover);
}

.oauth-btn--dark .oauth-btn__arr {
  color: var(--color-on-ink);
  opacity: 0.5;
}

.oauth-btn--dark:hover .oauth-btn__arr {
  color: var(--color-on-ink);
  opacity: 1;
}
</style>
