<script setup lang="ts">
import { AUTH_ART } from '@/data/Auth'

/**
 * The join button, and the sign-in panel it opens.
 *
 * A panel anchored under the button rather than a modal: signing in is not a
 * step in a flow the reader started, it is something they do on the way past.
 * A modal would black out the page they came for and demand to be dismissed.
 *
 * The panel is a replica of the login page's card, and shares its provider
 * buttons with it, so the two cannot drift.
 */
const open = ref<boolean>(false)
const root = ref<HTMLElement | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)

const route = useRoute()

/**
 * Back to the page they were reading, not to the dashboard.
 *
 * This is the whole reason the panel exists rather than a link to /login: a
 * reader half way down a lesson should come back to it.
 */
const redirect = computed<string>(() => route.fullPath)

const close = (restoreFocus = false): void => {
  open.value = false

  // Only when dismissed from the keyboard. Yanking focus back after a click
  // outside would fight whatever the reader just clicked on.
  if (restoreFocus) trigger.value?.focus()
}

const onPointerDown = (event: MouseEvent): void => {
  if (!open.value) return
  if (root.value?.contains(event.target as Node)) return

  close()
}

const onKeydown = (event: KeyboardEvent): void => {
  if (open.value && event.key === 'Escape') close(true)
}

onMounted(() => {
  // `pointerdown`, not `click`: a click that starts inside the panel and ends
  // outside it should not count as dismissing the panel.
  document.addEventListener('pointerdown', onPointerDown)
  document.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onPointerDown)
  document.removeEventListener('keydown', onKeydown)
})

// A panel left hanging over the next page is a panel the reader has to dismiss
// twice.
watch(() => route.fullPath, () => close())
</script>

<template>
  <div ref="root" class="relative">
    <button
      ref="trigger"
      type="button"
      class="cursor-pointer rounded-lg border border-stroke bg-ink px-4 py-2 text-sm font-semibold text-on-ink transition hover:bg-ink-hover sm:px-6"
      aria-haspopup="dialog"
      :aria-expanded="open"
      @click="open = !open"
    >
      join
    </button>

    <Transition name="join-panel">
      <div
        v-if="open"
        class="join-panel border-pencil-light rounded-xl bg-panel"
        role="dialog"
        aria-label="Join projectlighthouse"
      >
        <!-- Decorative, and hidden on narrow panels where it would push the
             buttons off screen. -->
        <div class="join-panel__art" aria-hidden="true">
          <img :src="AUTH_ART.image" alt="" class="join-panel__photo">
          <div class="join-panel__brand">
            <img src="/lighthouse.svg" alt="" class="join-panel__mark">
            <span>projectlighthouse</span>
          </div>
          <p class="join-panel__quote">{{ AUTH_ART.quote }}</p>
        </div>

        <div class="join-panel__card">
          <div class="font-mono text-xs tracking-wide text-teal-deep">
            <span class="text-faint">/ </span>welcome aboard
          </div>

          <h2 class="mt-2 font-serif text-2xl leading-tight tracking-tight text-ink">
            Join projectlighthouse
          </h2>

          <p class="mt-3 text-sm leading-relaxed text-mono-ink">
            Sign in to pick up where you left off — your labs, books, and progress.
          </p>

          <div class="mt-6">
            <AuthOauthButtons :redirect="redirect" />
          </div>

          <div class="my-6 flex items-center gap-3">
            <span class="h-px flex-1 bg-rule" />
            <span class="font-mono text-[11px] tracking-widest uppercase text-faint">secure oauth</span>
            <span class="h-px flex-1 bg-rule" />
          </div>

          <p class="text-sm leading-relaxed text-quiet">
            No passwords, no reset emails, no hassle.
          </p>

          <p class="mt-4 text-xs leading-relaxed text-quiet">
            By continuing, you agree to our
            <NuxtLink to="/terms" class="text-ink underline decoration-rule underline-offset-2 hover:text-link-hover">Terms of Service</NuxtLink>
            and
            <NuxtLink to="/privacy" class="text-ink underline decoration-rule underline-offset-2 hover:text-link-hover">Privacy Policy</NuxtLink>.
          </p>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* Plain css rather than tailwind arbitrary values. `top-[calc(100%+0.6rem)]`
   is a trap: css calc() requires whitespace around + and -, so without it the
   declaration is invalid and the browser drops it silently — the panel then
   falls back to its static position and renders over the header. Written here
   there is nothing to escape and nothing for the class scanner to miss.

   `top: 100%` plus a margin, not a calc, for the same reason. */
.join-panel {
  position: absolute;
  top: 100%;
  right: 0;
  z-index: 50;
  margin-top: 0.6rem;
  display: grid;
  grid-template-columns: 16rem 1fr;
  width: min(46rem, calc(100vw - 2rem));
  /* The art bleeds to the panel's edge, so the rounding has to clip it. */
  overflow: hidden;
  box-shadow: 0 24px 60px -30px rgb(22 20 15 / 45%);
}

.join-panel__art {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 1.4rem;
  /* Shows only while the photo loads. Fixed in both themes — it is behind a
     photograph, not part of the palette. */
  background: #1c3a5e;
  color: #fff;
}

.join-panel__photo {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center 28%;
}

.join-panel__art::after {
  content: '';
  position: absolute;
  inset: 0;
  background:
    linear-gradient(to bottom, rgb(12 22 40 / 55%) 0%, rgb(12 22 40 / 0%) 24%),
    linear-gradient(to top, rgb(10 18 34 / 82%) 0%, rgb(10 18 34 / 30%) 30%, rgb(10 18 34 / 0%) 52%);
}

/* Above the photo and its wash. */
.join-panel__brand,
.join-panel__quote {
  position: relative;
  z-index: 1;
}

.join-panel__brand {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  font-family: 'Inter', -apple-system, system-ui, sans-serif;
  font-weight: 600;
  font-size: 0.9rem;
  letter-spacing: -0.01em;
}

.join-panel__mark {
  width: 1.6rem;
  height: 1.6rem;
  object-fit: contain;
  border-radius: 3px;
  padding: 2px;
  background: var(--color-teal-wash);
}

.join-panel__quote {
  margin: 0;
  max-width: 14ch;
  font-family: 'Newsreader', Georgia, serif;
  font-weight: 500;
  font-size: 1.5rem;
  line-height: 1.16;
  letter-spacing: -0.015em;
  text-wrap: balance;
  text-shadow: 0 1px 24px rgb(8 14 28 / 45%);
}

.join-panel__card {
  padding: 1.9rem;
}

/* No room for the art beside the card, and a card squeezed to half a panel is
   worse than no picture. */
@media (max-width: 44rem) {
  .join-panel {
    grid-template-columns: 1fr;
    width: min(22rem, calc(100vw - 2rem));
  }

  .join-panel__art {
    display: none;
  }

  .join-panel__card {
    padding: 1.5rem;
  }
}

.join-panel-enter-active,
.join-panel-leave-active {
  transition:
    opacity 140ms ease,
    transform 140ms ease;
}

.join-panel-enter-from,
.join-panel-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

@media (prefers-reduced-motion: reduce) {
  .join-panel-enter-active,
  .join-panel-leave-active {
    transition: none;
  }

  .join-panel-enter-from,
  .join-panel-leave-to {
    transform: none;
  }
}
</style>
