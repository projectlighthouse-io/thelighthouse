<script setup lang="ts">
/**
 * The footer every page closes on: the wordmark, the links, the lockup and
 * the build.
 */
interface FooterLink {
  to: string
  label: string
  external?: boolean
}

/** Every footer destination, once. The two layouts below only order them. */
const LINKS = {
  books: { to: '/books', label: 'books' },
  projects: { to: '/projects', label: 'projects' },
  dots: { to: '/connecting-the-dots', label: 'connecting the dots' },
  terms: { to: '/terms', label: 'terms' },
  privacy: { to: '/privacy', label: 'privacy' },
  changelog: { to: '/changelog', label: 'changelog' },
  support: { to: '/support', label: 'support' },
  linkedin: { to: 'https://www.linkedin.com/company/projectlighthouse-io', label: 'linkedin', external: true },
  substack: { to: 'https://projectlighthouse.substack.com/', label: 'substack', external: true },
  blog: { to: '/blog', label: 'blog' },
  paperkites: { to: 'https://paperkites.app/', label: 'paperkites ↗', external: true },
} satisfies Record<string, FooterLink>

const links: FooterLink[] = [
  LINKS.books, LINKS.projects, LINKS.dots, LINKS.terms, LINKS.privacy, LINKS.changelog,
  LINKS.support, LINKS.linkedin, LINKS.substack, LINKS.blog, LINKS.paperkites,
]

/** The phone footer's three columns, anchored left, centre and right. */
const columns: { heading: string, links: FooterLink[] }[] = [
  { heading: 'read', links: [LINKS.books, LINKS.projects, LINKS.blog, LINKS.dots] },
  { heading: 'company', links: [LINKS.changelog, LINKS.support, LINKS.terms, LINKS.privacy] },
  { heading: 'follow', links: [LINKS.linkedin, LINKS.substack, LINKS.paperkites] },
]

/** 5×7 glyphs for the wordmark; a `█` is a lit cell. */
const GLYPHS: Record<string, string[]> = {
  P: ['████ ', '█   █', '█   █', '████ ', '█    ', '█    ', '█    '],
  R: ['████ ', '█   █', '█   █', '████ ', '█ █  ', '█  █ ', '█   █'],
  O: [' ███ ', '█   █', '█   █', '█   █', '█   █', '█   █', ' ███ '],
  J: ['  ███', '   █ ', '   █ ', '   █ ', '   █ ', '█  █ ', ' ██  '],
  E: ['█████', '█    ', '█    ', '████ ', '█    ', '█    ', '█████'],
  C: [' ████', '█    ', '█    ', '█    ', '█    ', '█    ', ' ████'],
  T: ['█████', '  █  ', '  █  ', '  █  ', '  █  ', '  █  ', '  █  '],
  L: ['█    ', '█    ', '█    ', '█    ', '█    ', '█    ', '█████'],
  I: ['█████', '  █  ', '  █  ', '  █  ', '  █  ', '  █  ', '█████'],
  G: [' ████', '█    ', '█    ', '█  ██', '█   █', '█   █', ' ████'],
  H: ['█   █', '█   █', '█   █', '█████', '█   █', '█   █', '█   █'],
  U: ['█   █', '█   █', '█   █', '█   █', '█   █', '█   █', ' ███ '],
  S: [' ████', '█    ', '█    ', ' ███ ', '    █', '    █', '████ '],
}

const WORD = 'PROJECTLIGHTHOUSE'

/** Every lit cell as an x, y on a grid one unit per cell, a column between letters. */
const cells = [...WORD].flatMap((ch, i) =>
  (GLYPHS[ch] ?? []).flatMap((row, y) =>
    [...row].flatMap((c, x) => (c === '█' ? [{ x: i * 6 + x, y }] : [])),
  ),
)

const GRID_W = WORD.length * 6 - 1

const { version, commit } = useRuntimeConfig().public
const build = [`v${version}`, commit].filter(Boolean).join(' · ')
const year = new Date().getFullYear()
</script>

<template>
  <footer class="footer lh-wide">
    <svg
      class="mark"
      :viewBox="`0 0 ${GRID_W + 0.4} 7.4`"
      role="img"
      aria-label="projectlighthouse"
    >
      <!-- The outline shadow, offset down and right, behind the lit cells. -->
      <rect
        v-for="c in cells"
        :key="`s${c.x}-${c.y}`"
        class="shadow"
        :x="c.x + 0.32"
        :y="c.y + 0.32"
        width="0.88"
        height="0.88"
      />
      <rect v-for="c in cells" :key="`c${c.x}-${c.y}`" class="cell" :x="c.x" :y="c.y" width="0.88" height="0.88" />
    </svg>

    <div class="middle">
      <nav class="links" aria-label="footer">
        <template v-for="link in links" :key="link.to">
          <a
            v-if="link.external"
            :href="link.to"
            target="_blank"
            rel="noopener noreferrer"
            class="lh-link"
          >{{ link.label }}</a>
          <NuxtLink v-else :to="link.to" class="lh-link">{{ link.label }}</NuxtLink>
        </template>
      </nav>

      <!-- The one language there is. A label, not a control, until there is a
           second one to switch to. -->
      <span class="language" lang="en">English</span>
    </div>

    <div class="build">
      <NuxtLink to="/" class="lockup" aria-label="Project Lighthouse home">
        <UiLogo variant="tile" :size="24" />
        <span class="wordmark">projectlighthouse</span>
      </NuxtLink>
      <span class="lh-mono lh-faint">{{ build }}</span>
      <span class="copyright">© {{ year }} projectlighthouse. all rights reserved.</span>
    </div>
  </footer>

  <!-- Below 720px this replaces the footer above: one compact block, no rules,
       sized to sit above the floating tab bar. -->
  <footer class="mfooter">
    <div class="m-brand-row">
      <NuxtLink to="/" class="m-brand">
        <UiLogo variant="tile" :size="24" aria-hidden="true" />
        <span class="wordmark">projectlighthouse</span>
      </NuxtLink>
      <span class="m-language" lang="en">English</span>
    </div>

    <nav class="m-columns" aria-label="Footer">
      <div v-for="col in columns" :key="col.heading" class="m-col">
        <span class="m-heading">{{ col.heading }}</span>
        <template v-for="link in col.links" :key="link.to">
          <a
            v-if="link.external"
            :href="link.to"
            target="_blank"
            rel="noopener noreferrer"
            class="m-link"
          >{{ link.label }}</a>
          <NuxtLink v-else :to="link.to" class="m-link">{{ link.label }}</NuxtLink>
        </template>
      </div>
    </nav>

    <div class="m-legal">
      <span class="m-copyright">© {{ year }} projectlighthouse</span>
      <span class="m-build">{{ build }}</span>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  display: grid;
  justify-items: center;
  gap: 56px;
  margin-top: 128px;
  padding-bottom: var(--space-16);
  text-align: center;
}

.mark {
  display: block;
  width: max(50%, 300px);
  max-width: 100%;
  height: auto;
  opacity: 0.85;
}

.cell { fill: var(--ink); }

.shadow {
  fill: none;
  stroke: var(--ink-muted);
  stroke-width: 0.06;
}

/* Rises in as it scrolls into view, where the browser can tie an animation
   to scroll; elsewhere it is simply there. */
@supports (animation-timeline: view()) {
  .mark {
    animation: rise linear both;
    animation-timeline: view();
    animation-range: entry 0% entry 80%;
  }
}

@keyframes rise {
  from { opacity: 0; transform: translateY(24px); }
}

@media (prefers-reduced-motion: reduce) {
  .mark { animation: none; }
}

.middle {
  display: grid;
  justify-items: center;
  gap: var(--space-6);
}

.links {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: var(--space-3) var(--space-8);
  font: var(--text-body-sm);
}

.language {
  display: inline-flex;
  align-items: center;
  height: 40px;
  padding: 0 18px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-raised);
  color: var(--ink);
  font: 500 15px/1 var(--font-sans);
}

.build {
  display: grid;
  justify-items: center;
  gap: var(--space-2);
}

/* The header's lockup, at the same size: tile, 8px, the wordmark. */
.lockup {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
  color: var(--ink);
  text-decoration: none;
}

.lockup:hover { color: var(--ink); }

.wordmark { font: var(--text-label-mono); color: var(--ink); }

.copyright {
  font: var(--text-body-sm);
  color: var(--ink-secondary);
}

.mfooter { display: none; }

@media (max-width: 720px) {
  .footer { display: none; }

  .mfooter {
    display: flex;
    flex-direction: column;
    gap: 24px;
    margin-top: 72px;
    padding: 24px 20px 0;
  }
}

.m-brand-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.m-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--ink);
  text-decoration: none;
}

.m-language {
  display: inline-flex;
  align-items: center;
  height: 28px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-full);
  background: var(--surface-raised);
  font: var(--text-label-mono);
  color: var(--ink);
}

.m-language:hover { background: var(--surface-sunken); }

.m-columns {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: 0 16px;
}

.m-col {
  display: grid;
  align-content: start;
  min-width: 0;
}

.m-col:nth-child(1) { justify-self: start; }
.m-col:nth-child(2) { justify-self: center; }
.m-col:nth-child(3) { justify-self: end; justify-items: end; text-align: right; }

.m-heading {
  padding-bottom: 4px;
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

.m-link {
  display: flex;
  align-items: center;
  min-height: 32px;
  font: var(--text-caption);
  color: var(--ink-secondary);
  text-decoration: none;
  white-space: nowrap;
}

.m-link:hover { color: var(--ink); }

.m-brand:focus-visible,
.m-link:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}

.m-legal {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  justify-content: space-between;
  gap: 4px 12px;
}

.m-copyright {
  font: var(--text-caption);
  color: var(--ink-secondary);
}

.m-build {
  font: var(--text-label-mono);
  color: var(--ink-faint);
}
</style>
