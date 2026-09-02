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
  <SettingsShell>
    <h2 class="font-serif text-2xl text-ink">Profile</h2>
    <p class="text-mono-body mt-2">Your name and email come from the provider you signed in with.</p>

    <dl class="mt-8 space-y-5">
      <div>
        <dt class="font-mono text-xs tracking-wider uppercase text-faint">name</dt>
        <dd class="mt-1 text-ink">{{ or(reader?.name) }}</dd>
      </div>
      <div>
        <dt class="font-mono text-xs tracking-wider uppercase text-faint">email</dt>
        <dd class="mt-1 text-ink">{{ or(reader?.email) }}</dd>
      </div>
      <div>
        <dt class="font-mono text-xs tracking-wider uppercase text-faint">signed in with</dt>
        <dd class="mt-1 text-ink capitalize">{{ or(reader?.provider) }}</dd>
      </div>
    </dl>

    <p class="mt-10 text-sm text-quiet">
      To change your name or avatar, change it with your provider and sign in again.
      What other learners see is on
      <NuxtLink to="/settings/public-profile" class="underline">public profile</NuxtLink>.
    </p>
  </SettingsShell>
</template>
