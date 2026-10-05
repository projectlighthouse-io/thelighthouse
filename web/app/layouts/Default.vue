<script setup lang="ts">
const { resolve } = useReader()

/**
 * `definePageMeta({ chrome: false })`: no footer, and from `lg` up the page is
 * exactly the viewport tall, so a full-height tool — the article editor — fits
 * without the document scrolling around it.
 */
const route = useRoute()
const bare = computed<boolean>(() => route.meta.chrome === false)

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
  <div class="site" :class="{ 'site--bare': bare }">
    <PppBanner />
    <SiteHeader />

    <main class="main">
      <slot />
    </main>

    <SiteFooter v-if="!bare" />
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

@media (min-width: 1024px) {
  .site--bare { height: 100dvh; min-height: 0; }
  .site--bare .main { min-height: 0; }
}
</style>
