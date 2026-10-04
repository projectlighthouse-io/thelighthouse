<script setup lang="ts">
/**
 * The footer every page closes on: the block wordmark, the links, the build.
 */
interface FooterLink {
  to: string
  label: string
  external?: boolean
}

const links: FooterLink[] = [
  { to: '/books', label: 'books' },
  { to: '/projects', label: 'projects' },
  { to: '/syntax', label: 'syntax' },
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

/** 5×7 glyphs, each lit cell doubled so the block reads square in a mono face. */
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

const ascii = Array.from({ length: 7 }, (_, row) =>
  'PROJECTLIGHTHOUSE'
    .split('')
    .map(ch => (GLYPHS[ch]?.[row] ?? '     ').replace(/█/g, '██').replace(/ /g, '  '))
    .join('  '),
).join('\n')

const { version, commit } = useRuntimeConfig().public
const build = [`v${version}`, commit].filter(Boolean).join(' · ')
const year = new Date().getFullYear()
</script>

<template>
  <footer class="footer lh-wide">
    <pre class="ascii" role="img" aria-label="projectlighthouse">{{ ascii }}</pre>

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

.ascii {
  margin: 0;
  max-width: 100%;
  overflow: hidden;
  font: 400 clamp(3px, 0.55vw, 7px) / 1 var(--font-mono);
  letter-spacing: 0;
  color: var(--ink-secondary);
  white-space: pre;
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
