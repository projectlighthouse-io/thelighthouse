<script setup lang="ts">
definePageMeta({ layout: 'default', middleware: 'auth' })

useSeo({
  title: 'Two-factor — projectlighthouse',
  description: 'Two-factor authentication settings.',
  noindex: true,
})
</script>

<template>
  <SettingsShell>
    <h2 class="font-serif text-2xl text-ink">Two-factor authentication</h2>
    <p class="text-mono-body mt-2">
      An extra step at sign-in, from an authenticator app on your phone.
    </p>

    <div class="border-pencil-light mt-8 rounded-md p-6">
      <div class="flex items-center justify-between gap-4">
        <div>
          <div class="font-medium text-ink">Status</div>
          <div class="mt-1 font-mono text-sm text-quiet">not enabled</div>
        </div>
        <button
          type="button"
          class="rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover"
        >
          Enable
        </button>
      </div>
    </div>

    <p class="mt-6 text-sm text-quiet">
      Recovery codes are generated when you enable this. Store them somewhere that is not your
      phone.
    </p>
  </SettingsShell>
</template>
