<script setup lang="ts">
// preview scaffolding, never in a production build
if (!import.meta.dev) {
  throw createError({ statusCode: 404, statusMessage: 'Not found', fatal: true })
}

const { user, isSignedIn, signIn, signOut } = usePreviewAuth()

useSeo({
  title: 'Preview auth — projectlighthouse',
  description: 'Development-only preview of the signed-in state.',
  noindex: true,
})

const signedInPages = [
  { to: '/dashboard', label: 'dashboard' },
  { to: '/notes', label: 'notes' },
  { to: '/profile', label: 'profile' },
  { to: '/settings/profile', label: 'settings' },
]
</script>

<template>
  <div class="mx-auto max-w-2xl px-4 py-20 sm:px-6 lg:px-8">
    <div class="font-mono text-xs tracking-[0.2em] uppercase text-teal">dev only</div>
    <h1 class="mt-3 font-serif text-3xl text-ink sm:text-4xl">Preview auth</h1>
    <p class="text-mono-body mt-4">
      Real auth is phase 3. This flips a cookie so the signed-in chrome renders, letting you look
      at those pages before OAuth exists.
    </p>

    <div class="border-pencil-light mt-10 rounded-md bg-panel p-7">
      <div class="flex items-center justify-between gap-4">
        <div>
          <div class="font-mono text-xs tracking-wider uppercase text-faint">status</div>
          <div class="mt-1 text-ink">
            {{ isSignedIn ? `signed in as ${user?.name}` : 'signed out' }}
          </div>
        </div>

        <button
          v-if="!isSignedIn"
          type="button"
          class="cursor-pointer rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover"
          @click="signIn"
        >
          Sign in
        </button>
        <button
          v-else
          type="button"
          class="cursor-pointer rounded-md border border-stroke bg-panel px-5 py-2.5 text-sm font-medium text-ink transition hover:bg-paper-warm"
          @click="signOut"
        >
          Sign out
        </button>
      </div>
    </div>

    <div class="mt-10">
      <div class="font-mono text-xs tracking-wider uppercase text-faint">signed-in pages</div>
      <ul class="mt-4 space-y-2">
        <li v-for="page in signedInPages" :key="page.to">
          <NuxtLink :to="page.to" class="btn-chalk text-sm font-medium text-ink">
            {{ page.label }} <span class="ml-1">———→</span>
          </NuxtLink>
        </li>
      </ul>
      <p class="mt-6 text-sm text-quiet">
        These are reachable whether or not you flip the cookie — there is no route guard yet. The
        cookie only changes the header chrome.
      </p>
    </div>
  </div>
</template>
