<script setup lang="ts">
// full-bleed split screen — the site chrome would fight it
definePageMeta({ layout: false })

useSeo({
  title: 'Join projectlighthouse',
  description: 'Sign in with GitHub or Google to pick up where you left off.',
  noindex: true,
})

// rust owns the oauth dance: these are plain hrefs, never fetches
const githubRedirectUrl = '/api/auth/github'
const googleRedirectUrl = '/api/auth/google'

const artImageUrl = 'https://i.pinimg.com/1200x/99/4c/52/994c52b546f4b42847d87066172342fb.jpg'

const route = useRoute()
const status = computed<string | null>(() => (route.query.status as string) ?? null)
const error = computed<string | null>(() => (route.query.error as string) ?? null)
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
        <p class="lead">From C to containers, one lab at a time.</p>
      </div>
    </div>

    <!-- FORM -->
    <div class="panel">
      <div class="card">
        <div class="card__eyebrow">welcome aboard</div>
        <h1>Join projectlighthouse</h1>
        <p class="card__sub">
          Sign in with GitHub or Google to pick up where you left off — your labs, books, and
          progress, all in one place.
        </p>

        <div v-if="status" class="status status--success">{{ status }}</div>
        <div v-if="error" class="status status--error">{{ error }}</div>

        <div class="oauth">
          <a class="btn btn--dark" :href="githubRedirectUrl">
            <svg class="ico" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 .5C5.7.5.5 5.7.5 12c0 5.1 3.3 9.4 7.9 10.9.6.1.8-.2.8-.5v-2c-3.2.7-3.9-1.4-3.9-1.4-.5-1.3-1.3-1.7-1.3-1.7-1.1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1 1.8 2.7 1.3 3.4 1 .1-.8.4-1.3.7-1.6-2.6-.3-5.3-1.3-5.3-5.7 0-1.3.5-2.3 1.2-3.1-.1-.3-.5-1.5.1-3.1 0 0 1-.3 3.3 1.2a11.5 11.5 0 0 1 6 0C17.3 4.7 18.3 5 18.3 5c.6 1.6.2 2.8.1 3.1.8.8 1.2 1.8 1.2 3.1 0 4.4-2.7 5.4-5.3 5.7.4.4.8 1.1.8 2.2v3.3c0 .3.2.6.8.5 4.6-1.5 7.9-5.8 7.9-10.9C23.5 5.7 18.3.5 12 .5z" />
            </svg>
            <span class="lbl">Continue with GitHub</span>
            <svg class="arr" viewBox="0 0 16 16" fill="none">
              <path
                d="M3 8h9M8.5 4l4 4-4 4"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </a>

          <a class="btn" :href="googleRedirectUrl">
            <svg class="ico" viewBox="0 0 24 24">
              <path
                fill="#4285F4"
                d="M23.5 12.3c0-.8-.1-1.5-.2-2.3H12v4.5h6.5c-.3 1.5-1.1 2.7-2.4 3.6v3h3.9c2.3-2.1 3.5-5.2 3.5-8.8z"
              />
              <path
                fill="#34A853"
                d="M12 24c3.2 0 5.9-1.1 7.9-2.9l-3.9-3c-1.1.7-2.5 1.2-4 1.2-3.1 0-5.7-2.1-6.6-4.9H1.4v3.1C3.4 21.4 7.4 24 12 24z"
              />
              <path
                fill="#FBBC05"
                d="M5.4 14.3c-.2-.7-.4-1.5-.4-2.3s.1-1.6.4-2.3V6.6H1.4C.5 8.2 0 10 0 12s.5 3.8 1.4 5.4l4-3.1z"
              />
              <path
                fill="#EA4335"
                d="M12 4.8c1.8 0 3.3.6 4.6 1.8l3.4-3.4C17.9 1.2 15.2 0 12 0 7.4 0 3.4 2.6 1.4 6.6l4 3.1C6.3 6.9 8.9 4.8 12 4.8z"
              />
            </svg>
            <span class="lbl">Continue with Google</span>
            <svg class="arr" viewBox="0 0 16 16" fill="none">
              <path
                d="M3 8h9M8.5 4l4 4-4 4"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </a>
        </div>

        <div class="divider"><span /><em>secure oauth</em><span /></div>

        <div class="note">
          <svg viewBox="0 0 18 18" fill="none">
            <path
              d="M9 1.5l6 2.2v4.1c0 3.7-2.5 6.4-6 7.7-3.5-1.3-6-4-6-7.7V3.7L9 1.5z"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linejoin="round"
            />
            <path
              d="M6.3 9l1.8 1.8L11.8 7"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          <span>No passwords to manage. We never post or read your repositories.</span>
        </div>

        <p class="legal">
          By continuing, you agree to our
          <NuxtLink to="/terms">Terms of Service</NuxtLink>
          and
          <NuxtLink to="/privacy">Privacy Policy</NuxtLink>.
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

.status {
  margin-top: 20px;
  padding: 10px 14px;
  border-radius: 9px;
  font-family: var(--sans);
  font-size: 13.5px;
  line-height: 1.5;
}
.status--success {
  color: var(--color-ok);
  background: var(--color-ok-bg);
  border: 1px solid var(--color-ok-line);
}
.status--error {
  color: var(--color-bad);
  background: var(--color-bad-bg);
  border: 1px solid var(--color-bad-line);
}

.oauth {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 34px;
}
.btn {
  display: flex;
  align-items: center;
  gap: 14px;
  width: 100%;
  padding: 15px 18px;
  border-radius: 11px;
  border: 1px solid var(--line);
  background: var(--bg);
  color: var(--ink);
  font-family: var(--sans);
  font-weight: 500;
  font-size: 15px;
  letter-spacing: -0.01em;
  text-decoration: none;
  cursor: pointer;
  transition:
    border-color 140ms,
    background 140ms,
    transform 80ms,
    box-shadow 140ms;
}
.btn:hover {
  border-color: var(--teal);
  background: #fff;
  box-shadow: 0 10px 26px -16px rgba(22, 20, 15, 0.4);
}
.btn:active {
  transform: translateY(1px);
}
.btn .ico {
  width: 20px;
  height: 20px;
  flex: none;
}
.btn .lbl {
  flex: 1;
  text-align: left;
}
.btn .arr {
  width: 16px;
  height: 16px;
  color: var(--faint);
  transition:
    transform 160ms,
    color 140ms;
}
.btn:hover .arr {
  color: var(--teal-d);
  transform: translateX(3px);
}
.btn--dark {
  background: var(--ink);
  border-color: var(--ink);
  color: #fff;
}
.btn--dark:hover {
  background: #000;
  border-color: #000;
}
.btn--dark .arr {
  color: rgba(255, 255, 255, 0.5);
}
.btn--dark:hover .arr {
  color: #fff;
}

.divider {
  display: flex;
  align-items: center;
  gap: 16px;
  margin: 26px 0;
}
.divider span {
  flex: 1;
  height: 1px;
  background: var(--line);
}
.divider em {
  font-family: var(--mono);
  font-style: normal;
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--faint);
}

.note {
  display: flex;
  align-items: flex-start;
  gap: 11px;
  font-family: var(--serif);
  font-size: 15px;
  line-height: 1.5;
  color: var(--ink-2);
}
.note svg {
  width: 17px;
  height: 17px;
  color: var(--teal-d);
  flex: none;
  margin-top: 2px;
}

.legal {
  font-family: var(--sans);
  font-size: 12.5px;
  line-height: 1.6;
  color: var(--mute);
  margin-top: 30px;
}
.legal a {
  color: var(--ink-2);
  text-decoration: none;
  border-bottom: 1px solid var(--line);
  padding-bottom: 1px;
}
.legal a:hover {
  color: var(--teal-d);
  border-color: var(--teal);
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
