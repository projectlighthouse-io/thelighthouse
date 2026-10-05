<script setup lang="ts">
/**
 * The footer every page closes on: the wordmark, the links, the build.
 */
interface FooterLink {
  to: string
  label: string
  external?: boolean
}

const links: FooterLink[] = [
  { to: '/books', label: 'books' },
  { to: '/projects', label: 'projects' },
  { to: '/connecting-the-dots', label: 'connecting the dots' },
  { to: '/terms', label: 'terms' },
  { to: '/privacy', label: 'privacy' },
  { to: '/changelog', label: 'changelog' },
  { to: '/support', label: 'support' },
  { to: 'https://www.linkedin.com/company/projectlighthouse-io', label: 'linkedin', external: true },
  { to: 'https://projectlighthouse.substack.com/', label: 'substack', external: true },
  { to: '/blog', label: 'blog' },
  { to: 'https://paperkites.app/', label: 'paperkites ↗', external: true },
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
      <span class="lh-mono lh-faint">{{ build }}</span>
      <span class="copyright">© {{ year }} projectlighthouse. all rights reserved.</span>
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
  gap: var(--space-2);
}

.copyright {
  font: var(--text-body-sm);
  color: var(--ink-secondary);
}
</style>
