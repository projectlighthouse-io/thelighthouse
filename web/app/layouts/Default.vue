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

// A client-side island: this layout wraps prerendered, edge-cached pages, so
// the session must never reach the rendered HTML. The header draws signed-out
// and fills in on mount — see UseAuth.ts.
// The reader's name, avatar and email belong to ChromeUserMenu now — the layout
// only needs to know whether to show it or the join button.
//
// `chrome`, not a boolean: it has a third state for "no answer yet", so the
// header can draw a reserved space instead of guessing. Guessing wrong is the
// join button appearing in front of somebody who is signed in.
const { chrome, load } = useAuth()

onMounted(load)

// Join opens the panel instead of navigating. /login still exists and is still
// where the middleware and rust's failure redirects send people — this is the
// same screen brought to the reader rather than the reader sent to it.
const joinOpen = ref(false)

// Clicked, as opposed to drifted into. A pinned panel gets the scrim, focus and
// aria-modal; a hovered one is only a preview and stays out of the way.
const joinPinned = ref(false)

// Hover is an enhancement over the click, never a replacement: it is off for
// touch and stylus, where `mouseenter` fires on tap and would make the panel
// open and shut in the same gesture.
const canHover = ref(false)
onMounted(() => {
  canHover.value = window.matchMedia('(hover: hover) and (pointer: fine)').matches
})

// Two different waits, for two different mistakes. Opening waits long enough
// that a cursor crossing the button on its way elsewhere does not flash the
// panel open. Closing waits long enough to cross the gap between the button and
// the panel, which is a diagonal of a few hundred pixels — too short and the
// panel closes while the reader is on their way to it.
const OPEN_DELAY = 120
const CLOSE_DELAY = 260

let openTimer: ReturnType<typeof setTimeout> | undefined
let closeTimer: ReturnType<typeof setTimeout> | undefined

function clearTimers() {
  clearTimeout(openTimer)
  clearTimeout(closeTimer)
}

function hoverIn() {
  if (!canHover.value) return
  clearTimers()
  openTimer = setTimeout(() => (joinOpen.value = true), OPEN_DELAY)
}

function hoverOut() {
  if (!canHover.value) return
  clearTimers()
  // A pinned panel is the reader's decision and only they close it — with the
  // ×, Escape or the scrim. Drifting the cursor off is not a decision.
  if (joinPinned.value) return
  closeTimer = setTimeout(() => (joinOpen.value = false), CLOSE_DELAY)
}

function toggleJoin() {
  clearTimers()
  joinOpen.value = !joinOpen.value
  joinPinned.value = joinOpen.value
}

// Whatever closed it — scrim, Escape, a route change — the pin goes with it.
watch(joinOpen, (isOpen) => {
  if (!isOpen) joinPinned.value = false
})

onBeforeUnmount(clearTimers)

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

            <!-- Exactly one of these ever renders, and it never changes into
                 the other.

                 Server-rendered HTML has to be identical for every anonymous
                 visitor or it cannot be edge-cached, so the server cannot draw
                 this at all — hence ClientOnly. And until the session answers,
                 neither a name nor a join button is known to be true, so the
                 slot stays empty at the right size. Empty says "not yet"; a
                 join button says something that may be false. -->
            <ClientOnly>
              <ChromeUserMenu v-if="chrome === 'reader'" />

              <button
                v-else-if="chrome === 'anonymous'"
                type="button"
                aria-haspopup="dialog"
                :aria-expanded="joinOpen"
                class="cursor-pointer rounded-lg border border-stroke bg-ink px-4 py-2 text-sm font-semibold text-on-ink transition hover:bg-ink-hover sm:px-6"
                @click="toggleJoin"
                @mouseenter="hoverIn"
                @mouseleave="hoverOut"
              >
                join
              </button>

              <!-- chrome === 'unknown', and the ClientOnly fallback before
                   mount. Same size either way, so nothing shifts when the
                   answer lands. -->
              <div v-else class="h-10 w-24" aria-hidden="true" />

              <template #fallback>
                <div class="h-10 w-24" aria-hidden="true" />
              </template>
            </ClientOnly>
          </div>
        </div>
      </div>
    </header>

    <AuthJoinPanel
      v-model="joinOpen"
      :pinned="joinPinned"
      @hover-in="hoverIn"
      @hover-out="hoverOut"
    />

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
