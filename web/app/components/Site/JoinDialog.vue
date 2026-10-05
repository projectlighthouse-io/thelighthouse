<script setup lang="ts">
/**
 * Sign-in in place, over whatever page the reader is on: the lighthouse on the
 * left, the two ways in on the right. `/login` stays as the same thing on its
 * own page, for the redirects that land there and for a browser without JS.
 *
 * A native <dialog>: `showModal` gives the backdrop, the focus trap and Escape
 * for free. It grows out of the button that opened it. A click on the
 * backdrop lands on the dialog element itself, which is how it closes on an
 * outside click.
 */
const dialog = useTemplateRef<HTMLDialogElement>('dialog')
const route = useRoute()

/**
 * Opens from where it was asked for: the offset from the trigger's centre to
 * the viewport's is where the grow animation starts.
 */
function open(event?: MouseEvent): void {
  const el = dialog.value
  if (!el) return

  const from = (event?.currentTarget as HTMLElement | null)?.getBoundingClientRect()
  if (from) {
    el.style.setProperty('--from-x', `${from.left + from.width / 2 - window.innerWidth / 2}px`)
    el.style.setProperty('--from-y', `${from.top + from.height / 2 - window.innerHeight / 2}px`)
  }

  el.showModal()
}

function close(): void {
  dialog.value?.close()
}

function onClick(event: MouseEvent): void {
  if (event.target === dialog.value) close()
}

watch(() => route.fullPath, close)

defineExpose({ open })
</script>

<template>
  <dialog ref="dialog" class="join" aria-labelledby="join-title" @click="onClick">
    <div class="art" aria-hidden="true">
      <img src="/pricing-lighthouse.jpg" alt="">
    </div>

    <div class="body">
      <button type="button" class="close" aria-label="close" @click="close">×</button>

      <p class="lh-eyebrow">welcome aboard</p>
      <h2 id="join-title" class="lh-h2">Join projectlighthouse</h2>
      <p class="lh-sub">
        Sign in with GitHub or Google to pick up where you left off: your books, projects and
        progress, in one place. No passwords, no reset emails.
      </p>

      <AuthOauthButtons :redirect="route.fullPath" />

      <p class="lh-hint">
        By continuing, you agree to the
        <NuxtLink to="/terms" class="lh-inline">terms</NuxtLink>
        and the
        <NuxtLink to="/privacy" class="lh-inline">privacy policy</NuxtLink>.
      </p>
    </div>
  </dialog>
</template>

<style scoped>
.join {
  inset: 0;
  margin: auto;
  width: min(880px, calc(100vw - 32px));
  max-height: calc(100dvh - 32px);
  padding: 0;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-page);
  color: var(--ink);
  overflow: hidden;
}

.join[open] {
  display: grid;
  grid-template-columns: minmax(0, 5fr) minmax(0, 6fr);
  animation: grow 260ms cubic-bezier(0.2, 0.8, 0.2, 1);
}

.join::backdrop {
  background: rgb(0 0 0 / 0.45);
  backdrop-filter: blur(2px);
}

.art { position: relative; min-height: 460px; }

.art img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: 50% 40%;
}

.body {
  position: relative;
  display: grid;
  align-content: center;
  gap: var(--space-5);
  padding: var(--space-12) var(--space-8);
}

.body > * { margin: 0; }

.close {
  position: absolute;
  top: var(--space-3);
  right: var(--space-3);
  width: 32px;
  height: 32px;
  border: 0;
  border-radius: var(--radius-md);
  background: none;
  color: var(--ink-secondary);
  font: 400 22px/1 var(--font-sans);
  cursor: pointer;
}

.close:hover { color: var(--ink); background: var(--surface-sunken); }

@keyframes grow {
  from {
    opacity: 0;
    transform: translate(var(--from-x, 0), var(--from-y, 0)) scale(0.08);
  }
}

.join[open]::backdrop { animation: fade 260ms ease-out; }

@keyframes fade {
  from { opacity: 0; }
}

@media (prefers-reduced-motion: reduce) {
  .join[open], .join[open]::backdrop { animation: none; }
}

@media (max-width: 700px) {
  .join[open] { grid-template-columns: minmax(0, 1fr); }
  .art { min-height: 160px; }
  .body { padding: var(--space-8) var(--space-6); }
}
</style>
