<script setup lang="ts">
import { AUTH_ART } from '@/data/Auth'

// full-bleed split screen — the site chrome would fight it
definePageMeta({ layout: false })

useSeo({
  title: 'Why do you think I need an admin panel?',
  description: 'There is no admin panel. Keep things simple.',
  noindex: true,
})

const artImageUrl = AUTH_ART.image
</script>

<template>
  <div class="login-screen">
    <!-- ART -->
    <div class="art" :style="{ backgroundImage: `url('${artImageUrl}')` }">
      <NuxtLink to="/" class="art__brand">
        <img src="/lighthouse.svg" alt="projectlighthouse" class="art__logo">
        <span class="wm">projectlighthouse</span>
      </NuxtLink>
      <div class="art__quote">
        <p class="lead">{{ AUTH_ART.quote }}</p>
      </div>
    </div>

    <!-- FORM -->
    <div class="panel">
      <div class="card">
        <div class="card__eyebrow">no such panel</div>
        <h1>Why do you think I need an admin panel?</h1>
        <p class="card__sub">
          Everything administrative is either from a static file on a private repo or a bash
          script running on the VM. Keep things simple
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-screen {
  --ink: var(--color-read-ink);
  --ink-2: var(--color-read-ink-soft);
  --mute: var(--color-read-mute);
  --faint: var(--color-read-faint);
  --line: var(--color-read-line);
  --line-2: var(--color-read-line-soft);
  --bg: var(--color-read-bg);
  --bg-2: var(--color-read-bg-soft);
  --teal: var(--color-teal-mid);
  --teal-d: var(--color-teal-deep);

  --serif: 'Newsreader', Georgia, serif;
  --sans: 'Inter', -apple-system, system-ui, sans-serif;
  --mono: 'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace;

  display: grid;
  grid-template-columns: 1.05fr 1fr;
  min-height: 100vh;
  font-family: var(--sans);
  color: var(--ink);
  background: var(--bg-2);
  -webkit-font-smoothing: antialiased;
}

.login-screen * {
  box-sizing: border-box;
}

/* ---------- left: art panel ---------- */
.art {
  position: relative;
  overflow: hidden;
  /* shows only while the art photo loads */
  background-color: #1c3a5e;
  background-size: cover;
  background-position: center 28%;
  background-repeat: no-repeat;
  padding: 44px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  color: #fff;
}
.art::before {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  background:
    linear-gradient(to bottom, rgba(12, 22, 40, 0.55) 0%, rgba(12, 22, 40, 0) 24%),
    linear-gradient(to top, rgba(10, 18, 34, 0.82) 0%, rgba(10, 18, 34, 0.3) 30%, rgba(10, 18, 34, 0) 52%);
}
.art > * {
  position: relative;
  z-index: 1;
}

.art__brand {
  display: flex;
  align-items: center;
  gap: 12px;
  color: inherit;
  text-decoration: none;
}
.art__logo {
  width: 36px;
  height: 36px;
  object-fit: contain;
  border-radius: 3px;
  background: var(--color-teal-wash);
  padding: 3px;
}
.art__brand .wm {
  font-family: var(--sans);
  font-weight: 600;
  font-size: 19px;
  letter-spacing: -0.01em;
}

.art__quote {
  max-width: 26ch;
}
.art__quote .lead {
  margin: 0;
  font-family: var(--serif);
  font-weight: 500;
  font-size: 41px;
  line-height: 1.16;
  letter-spacing: -0.015em;
  text-wrap: balance;
  text-shadow: 0 1px 24px rgba(8, 14, 28, 0.45);
}

/* ---------- right: form panel ---------- */
.panel {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 48px;
}
.card {
  width: 100%;
  max-width: 392px;
}

.card__eyebrow {
  font-family: var(--mono);
  font-size: 12px;
  letter-spacing: 0.06em;
  color: var(--teal-d);
}
.card__eyebrow::before {
  content: '/ ';
  color: var(--faint);
}
.card h1 {
  font-family: var(--serif);
  font-weight: 600;
  font-size: 38px;
  line-height: 1.08;
  letter-spacing: -0.015em;
  color: var(--ink);
  margin: 14px 0 0;
}
.card__sub {
  font-family: var(--serif);
  font-size: 18px;
  line-height: 1.55;
  color: var(--ink-2);
  margin: 14px 0 0;
  max-width: 34ch;
}

@media (max-width: 880px) {
  .login-screen {
    grid-template-columns: 1fr;
  }
  .art {
    min-height: 232px;
    padding: 28px;
  }
  .art__quote .lead {
    font-size: 28px;
  }
  .panel {
    padding: 40px 28px 56px;
  }
}
</style>
