<script setup lang="ts">
definePageMeta({ layout: 'default', middleware: 'auth' })

useSeo({
  title: 'Profile — projectlighthouse',
  description: 'Your account details.',
  noindex: true,
})

/**
 * Read-only, and there is no endpoint behind it.
 *
 * Name, email and provider all arrive with the session, so asking a second
 * time would be a request to learn what the page already knows. They are not
 * editable for the same reason they are free: the provider sends them on every
 * sign-in, so a field offering to change them would offer a change that
 * reverts at the next login.
 */
const { reader } = useReader()

/** A dash while the session is still in flight, so the row keeps its shape. */
const or = (value: string | null | undefined): string => value || '—'
</script>

<template>
  <AccountShell title="Profile" sub="Your name and email come from the provider you signed in with.">
    <div class="lh-card">
      <dl class="lh-facts">
        <dt>name</dt>
        <dd>{{ or(reader?.name) }}</dd>
        <dt>username</dt>
        <dd>{{ reader?.username ? `@${reader.username}` : '—' }}</dd>
        <dt>email</dt>
        <dd>{{ or(reader?.email) }}</dd>
        <dt>signed in with</dt>
        <dd>{{ or(reader?.provider) }}</dd>
      </dl>
    </div>

    <p class="lh-hint note">
      To change your name or avatar, change it with your provider and sign in again.
      What other learners see is on
      <NuxtLink to="/settings/public-profile" class="lh-inline">public profile</NuxtLink>.
    </p>
  </AccountShell>
</template>

<style scoped>
.note { margin-top: var(--space-6); }
</style>
