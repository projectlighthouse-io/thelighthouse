<script setup lang="ts">
import { AUTH_ART_IMAGE, AUTH_TAGLINE } from '@/data/Auth'

/**
 * The join dropdown: the /login screen's split layout, hung under the header.
 *
 * Same art, same two buttons, same words — signing in from the chrome should
 * not look like a different product from signing in at /login. The page stays
 * because it is a real URL: the middleware redirects to it, and rust sends
 * failures back to it with `?error=`.
 */
const open = defineModel<boolean>({ required: true })

const { pinned = false } = defineProps<{
  /**
   * Whether the reader committed to this — clicked — rather than drifting a
   * cursor over the trigger.
   *
   * Hover opens a preview: no scrim, no focus steal, no `aria-modal`. All three
   * would be hostile for something the reader never asked for, and a scrim in
   * particular would swallow clicks meant for the page. Clicking pins it and it
   * becomes a real dialog.
   */
  pinned?: boolean
}>()

const emit = defineEmits<{ hoverIn: []; hoverOut: [] }>()

const route = useRoute()

// Where the reader already was, so signing in from the header does not throw
// away their place. /login is excluded — landing back on the sign-in screen
// after signing in is a loop.
const redirect = computed<string | undefined>(() =>
  route.path === '/login' ? undefined : route.fullPath,
)

const panel = useTemplateRef<HTMLElement>('panel')

function close() {
  open.value = false
}

// Focus moves into the panel so a keyboard reader is not left behind at the
// trigger — but only once pinned. Pulling focus because a cursor passed over
// the button would yank the caret out from under whatever they were doing.
watch([open, () => pinned], async ([isOpen, isPinned]) => {
  if (!isOpen || !isPinned) return
  await nextTick()
  panel.value?.focus()
})

// Navigating away closes it — otherwise it hangs over the new page.
watch(() => route.fullPath, close)
</script>

<template>
  <Transition name="join">
    <div v-if="open" class="join">
      <!-- Catches the click that lands outside the panel. A sibling rather than
           a wrapper, so the panel itself is not inside the dismiss target.

           Only once pinned: an unpinned panel is a preview the reader drifted
           into, and covering the page with a click-eating layer they did not
           ask for is the worst thing a hover menu can do. -->
      <div v-if="pinned" class="join__scrim" @click="close" />

      <div
        ref="panel"
        class="join__panel"
        role="dialog"
        :aria-modal="pinned"
        aria-label="Join projectlighthouse"
        tabindex="-1"
        @keydown.esc="close"
        @mouseenter="emit('hoverIn')"
        @mouseleave="emit('hoverOut')"
      >
        <div class="art" :style="{ backgroundImage: `url('${AUTH_ART_IMAGE}')` }">
          <div class="art__brand">
            <img src="/lighthouse.svg" alt="" class="art__logo">
            <span class="wm">projectlighthouse</span>
          </div>
          <p class="art__lead">{{ AUTH_TAGLINE }}</p>
        </div>

        <div class="side">
          <button type="button" class="close" aria-label="Close" @click="close">
            <svg viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path
                d="M4 4l8 8M12 4l-8 8"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
          </button>

          <div class="eyebrow">welcome aboard</div>
          <h2>Join projectlighthouse</h2>
          <p class="sub">
            Sign in with GitHub or Google to pick up where you left off — your labs, books, and
            progress, all in one place.
          </p>

          <AuthProviderButtons :redirect="redirect" class="buttons" />

          <p class="legal">
            By continuing, you agree to our
            <NuxtLink to="/terms" @click="close">Terms of Service</NuxtLink>
            and
            <NuxtLink to="/privacy" @click="close">Privacy Policy</NuxtLink>.
          </p>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.join {
  position: fixed;
  inset: 0;
  /* Below the header (z-50) so the trigger stays visible and the panel reads as
     hanging from it rather than covering it. */
  z-index: 40;
  /* The wrapper spans the viewport purely to position its children. Left
     interactive it would eat every click on the page behind an unpinned
     preview, which is the same bug the scrim's v-if avoids. Children opt back
     in below. */
  pointer-events: none;
}

.join__scrim {
  position: absolute;
  inset: 0;
  background: rgba(10, 18, 34, 0.38);
  backdrop-filter: blur(2px);
  pointer-events: auto;
}

.join__panel {
  position: absolute;
  /* Clears the 4rem header. */
  top: calc(4rem + 12px);
  left: 50%;
  transform: translateX(-50%);
  width: min(900px, calc(100vw - 32px));
  display: grid;
  grid-template-columns: 1fr 1fr;
  overflow: hidden;
  border-radius: 16px;
  border: 1px solid var(--color-read-line);
  background: var(--color-read-bg);
  box-shadow: 0 30px 80px -30px rgba(10, 18, 34, 0.55);
  outline: none;
  pointer-events: auto;
}

/* left: art */

.art {
  position: relative;
  min-height: 380px;
  padding: 28px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  /* Shows only while the art loads. */
  background-color: #1c3a5e;
  background-size: cover;
  background-position: center 28%;
  color: #fff;
}

.art::before {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: linear-gradient(to top, rgba(10, 18, 34, 0.82) 0%, rgba(10, 18, 34, 0) 55%);
}

.art > * {
  position: relative;
  z-index: 1;
}

.art__brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.art__logo {
  width: 30px;
  height: 30px;
  object-fit: contain;
  border-radius: 3px;
  background: var(--color-teal-wash);
  padding: 3px;
}

.art__brand .wm {
  font-family: 'Inter', -apple-system, system-ui, sans-serif;
  font-weight: 600;
  font-size: 16px;
  letter-spacing: -0.01em;
}

.art__lead {
  margin: 0;
  max-width: 20ch;
  font-family: 'Newsreader', Georgia, serif;
  font-weight: 500;
  font-size: 27px;
  line-height: 1.18;
  letter-spacing: -0.015em;
  text-wrap: balance;
  text-shadow: 0 1px 24px rgba(8, 14, 28, 0.45);
}

/* right: the choice */

.side {
  position: relative;
  padding: 30px 30px 26px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.close {
  position: absolute;
  top: 14px;
  right: 14px;
  display: flex;
  padding: 7px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--color-read-mute);
  cursor: pointer;
  transition:
    background 140ms,
    color 140ms;
}

.close:hover {
  background: var(--color-read-bg-soft);
  color: var(--color-read-ink);
}

.close svg {
  width: 16px;
  height: 16px;
}

.eyebrow {
  font-family: 'JetBrains Mono', ui-monospace, Menlo, monospace;
  font-size: 11.5px;
  letter-spacing: 0.06em;
  color: var(--color-teal-deep);
}

.eyebrow::before {
  content: '/ ';
  color: var(--color-read-faint);
}

h2 {
  margin: 10px 0 0;
  font-family: 'Newsreader', Georgia, serif;
  font-weight: 600;
  font-size: 29px;
  line-height: 1.1;
  letter-spacing: -0.015em;
  color: var(--color-read-ink);
}

.sub {
  margin: 10px 0 0;
  max-width: 34ch;
  font-family: 'Newsreader', Georgia, serif;
  font-size: 16px;
  line-height: 1.5;
  color: var(--color-read-ink-soft);
}

.buttons {
  margin-top: 22px;
}

.legal {
  margin: 20px 0 0;
  font-family: 'Inter', -apple-system, system-ui, sans-serif;
  font-size: 12px;
  line-height: 1.6;
  color: var(--color-read-mute);
}

.legal a {
  color: var(--color-read-ink-soft);
  text-decoration: none;
  border-bottom: 1px solid var(--color-read-line);
}

.legal a:hover {
  color: var(--color-teal-deep);
  border-color: var(--color-teal-mid);
}

/* open and close */

/* Enter only carries movement; the leave is a plain fade so dismissing feels
   immediate rather than animated at you. */
.join-enter-active .join__panel {
  transition:
    opacity 220ms ease,
    transform 220ms cubic-bezier(0.22, 1, 0.36, 1);
}

.join-enter-active .join__scrim,
.join-leave-active .join__scrim,
.join-leave-active .join__panel {
  transition: opacity 150ms ease;
}

.join-enter-from .join__scrim,
.join-leave-to .join__scrim,
.join-enter-from .join__panel,
.join-leave-to .join__panel {
  opacity: 0;
}

.join-enter-from .join__panel {
  transform: translateX(-50%) translateY(-10px);
}

@media (prefers-reduced-motion: reduce) {
  .join-enter-active .join__panel,
  .join-leave-active .join__panel,
  .join-enter-active .join__scrim,
  .join-leave-active .join__scrim {
    transition: none !important;
    transform: translateX(-50%) !important;
  }
}

/* Narrow: the art becomes a banner rather than half the panel, which at this
   width would leave the buttons in a column too tight to read. */
@media (max-width: 720px) {
  .join__panel {
    grid-template-columns: 1fr;
    top: calc(4rem + 8px);
  }

  .art {
    min-height: 132px;
    padding: 18px;
  }

  .art__lead {
    font-size: 20px;
  }

  .side {
    padding: 22px 22px 20px;
  }
}
</style>
