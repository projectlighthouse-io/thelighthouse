<script setup lang="ts">
// The laravel app had both; with OAuth only there is nothing to tell apart.
definePageMeta({ alias: '/register' })

useSeo({
  title: 'Join projectlighthouse',
  description: 'Sign in with GitHub or Google to pick up where you left off.',
  noindex: true,
})

const route = useRoute()

// Handed to AuthOauthButtons, which builds the hrefs. Rust is what checks it
// is a same-site path — see `safe_redirect`.
const redirect = computed<string | undefined>(() => {
  const target = route.query.redirect

  return typeof target === 'string' && target !== '' ? target : undefined
})

const status = computed<string | null>(() => (route.query.status as string) ?? null)

// Already signed in — back from the provider, or here by a stale link: go on to
// where the reader was headed rather than asking them to sign in again.
const { resolve, isSignedIn } = useReader()
onMounted(async () => {
  await resolve()
  if (isSignedIn.value) await navigateTo(redirect.value ?? '/', { replace: true })
})
const error = computed<string | null>(() => (route.query.error as string) ?? null)
</script>

<template>
  <div class="login lh-text lh-gap">
    <div class="lh-card card">
      <!-- Decorative: the heading below already names the site. -->
      <UiLogo variant="tile" :size="32" />
      <p class="lh-eyebrow">welcome aboard</p>
      <h1 class="lh-h2">Join projectlighthouse</h1>
      <p class="lh-sub">
        Sign in with GitHub or Google to pick up where you left off — your books, projects and
        progress, in one place. No passwords, no reset emails.
      </p>

      <p v-if="status" class="note" role="status">{{ status }}</p>
      <p v-if="error" class="note" role="alert">{{ error }}</p>

      <AuthOauthButtons :redirect="redirect" />

      <p class="lh-hint">
        By continuing, you agree to the
        <NuxtLink to="/terms" class="lh-inline">terms</NuxtLink>
        and the
        <NuxtLink to="/privacy" class="lh-inline">privacy policy</NuxtLink>.
      </p>
    </div>
  </div>
</template>

<style scoped>
.card {
  display: grid;
  gap: var(--space-5);
  max-width: 440px;
  margin: 0 auto;
}

.note {
  margin: 0;
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
  background: var(--surface-sunken);
  font: var(--text-body-sm);
  color: var(--ink);
}
</style>
