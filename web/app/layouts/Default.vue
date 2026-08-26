<script setup lang="ts">
interface NavLink {
  to: string
  label: string
  external?: boolean
}

const navLinks: NavLink[] = [
  { to: '/books', label: 'books' },
  { to: '/projects', label: 'projects' },
  { to: '/syntax', label: 'syntax' },
  { to: '/pricing', label: 'pricing' },
]

const footerLinks: NavLink[] = [
  { to: '/books', label: 'books' },
  { to: '/projects', label: 'projects' },
  { to: '/roadmap', label: 'roadmap' },
  { to: '/connecting-the-dots', label: 'connecting the dots' },
  { to: '/terms', label: 'terms' },
  { to: '/privacy', label: 'privacy' },
  { to: '/changelog', label: 'changelog' },
  { to: '/support', label: 'support' },
  { to: 'https://www.linkedin.com/company/projectlighthouse-io', label: 'linkedin', external: true },
  { to: 'https://projectlighthouse.substack.com/', label: 'substack', external: true },
  { to: '/blog', label: 'blog' },
]

const { reader, isSignedIn, initials, resolve } = useReader()

// Client side, after hydration: the header is the only per-reader thing on an
// otherwise identical page, and asking during SSR would make every page
// uncacheable to render one avatar. See useReader.
onMounted(resolve)

// site-level identity, emitted once for every page that uses this layout
useJsonLd('site', {
  '@type': 'WebSite',
  'name': SITE.name,
  'url': SITE.url,
  'publisher': {
    '@type': 'Organization',
    'name': SITE.name,
    'url': SITE.url,
    'logo': `${SITE.url}/projectlighthouse.png`,
    'sameAs': [
      'https://www.linkedin.com/company/projectlighthouse-io',
      'https://projectlighthouse.substack.com/',
    ],
  },
})

// hardcoded until the rust api exposes it
const appVersion = { version: 'v0.9', build: 'rebuild' }
const year = new Date().getFullYear()
</script>

<template>
  <div class="flex min-h-screen flex-col">
    <header class="dotted-bg fixed top-0 right-0 left-0 z-50">
      <div class="mx-auto max-w-full px-4 sm:px-6 lg:px-8">
        <div class="relative flex h-16 items-center justify-between">
          <NuxtLink to="/" class="flex items-center gap-2">
            <img src="/lighthouse.svg" alt="projectlighthouse logo" class="h-8 w-8 object-contain">
            <span class="text-base tracking-tight text-ink">projectlighthouse</span>
          </NuxtLink>

          <nav class="absolute left-1/2 hidden -translate-x-1/2 items-center gap-2 sm:flex">
            <NuxtLink
              v-for="link in navLinks"
              :key="link.to"
              :to="link.to"
              class="px-4 py-1 font-sans text-xs text-ink transition hover:text-link-hover sm:text-sm"
            >
              {{ link.label }}
            </NuxtLink>
          </nav>

          <div class="flex items-center gap-1">
            <ChromeThemeToggle />

            <!-- Who the reader is is client state, so the server has nothing
                 correct to render. It renders the join button, which is also
                 what an anonymous visitor keeps — one cached document for
                 everyone, corrected in the browser for the signed in. -->
            <ClientOnly>
              <NuxtLink
                v-if="isSignedIn"
                to="/profile"
                class="flex items-center gap-2 rounded-md px-2 py-1.5 transition hover:bg-paper-warm"
                :title="reader?.email"
              >
                <img
                  v-if="reader?.avatar"
                  :src="reader.avatar"
                  alt=""
                  class="size-7 rounded-full object-cover"
                >
                <!-- Drawn empty while the hint cookie says signed in and the
                     session endpoint has not answered yet: the right shape
                     immediately beats the right letters a round trip later. -->
                <span
                  v-else
                  class="flex size-7 items-center justify-center rounded-full bg-ink font-mono text-xs text-on-ink"
                >{{ initials }}</span>
                <span class="hidden font-sans text-sm text-ink sm:inline">{{ reader?.name }}</span>
              </NuxtLink>

              <ChromeJoinDropdown v-else />

              <!-- The same component the anonymous branch renders, so the
                   server emits the real button rather than a stand-in for it.
                   The join button is identical for every visitor — only the
                   signed-in variant is per-reader — so nothing about it needs
                   to wait for hydration, and a page whose javascript has not
                   run yet still shows the thing it will become. -->
              <template #fallback>
                <ChromeJoinDropdown />
              </template>
            </ClientOnly>
          </div>
        </div>
      </div>
    </header>

    <main class="mt-16 flex flex-1 flex-col">
      <slot />
    </main>

    <!-- ascii wordmark — hidden on mobile, too wide to be legible -->
    <div class="hidden w-full sm:block">
      <ChromeBinaryLogo />
    </div>

    <footer class="mt-auto px-4 py-8 sm:px-6 lg:px-8">
      <div class="mx-auto w-full max-w-7xl text-center">
        <nav
          class="mb-6 grid grid-cols-2 gap-x-4 gap-y-2 sm:flex sm:flex-wrap sm:items-center sm:justify-center sm:gap-x-6 sm:gap-y-2"
        >
          <template v-for="link in footerLinks" :key="link.to">
            <a
              v-if="link.external"
              :href="link.to"
              target="_blank"
              rel="noopener noreferrer"
              class="text-sm text-quiet transition hover:text-ink"
            >
              {{ link.label }}
            </a>
            <NuxtLink
              v-else
              :to="link.to"
              class="text-sm text-quiet transition hover:text-ink"
            >
              {{ link.label }}
            </NuxtLink>
          </template>
        </nav>

        <div class="font-mono text-xs text-whisper">
          <span>{{ appVersion.version }}</span>
          <span class="mx-1">&middot;</span>
          <span>{{ appVersion.build }}</span>
        </div>
        <p class="mt-1 text-sm text-quiet">
          &copy; {{ year }} projectlighthouse. all rights reserved.
        </p>
      </div>
    </footer>
  </div>
</template>
