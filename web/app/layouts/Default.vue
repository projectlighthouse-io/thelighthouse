<script setup lang="ts">
const { resolve } = useReader()

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
</script>

<template>
  <div class="site">
    <PppBanner />
    <SiteHeader />

    <main class="main">
      <slot />
    </main>

    <SiteFooter />
  </div>
</template>

<style scoped>
.site {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: var(--surface-page);
  color: var(--ink);
}

.main {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.main > :deep(*) { width: 100%; }
</style>
