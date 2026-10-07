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

// Site-level identity, emitted once for every page that uses this layout: the
// organization on its own, and the website pointing at it by `@id`.
const ORGANIZATION_ID = `${SITE.url}/#organization`

useJsonLd('organization', {
  '@type': 'Organization',
  '@id': ORGANIZATION_ID,
  'name': SITE.name,
  'url': SITE.url,
  'logo': `${SITE.url}${SITE.logo}`,
  'sameAs': [
    'https://www.linkedin.com/company/projectlighthouse-io',
    'https://projectlighthouse.substack.com/',
  ],
})

useJsonLd('site', {
  '@type': 'WebSite',
  'name': SITE.name,
  'url': SITE.url,
  'publisher': { '@id': ORGANIZATION_ID },
})

// The defaults every page starts from. A page's own `useSeo` registers after
// this and replaces each one it sets; a page that sets none — the error page —
// still shares as the site's card rather than as nothing. No image width and
// height here: `useSeo` sets those only for the site's own card, and a default
// left over under a page's own image would claim a size it does not have.
useSeoMeta({
  ogSiteName: SITE.name,
  ogType: 'website',
  ogImage: `${SITE.url}${SITE.ogImage}`,
  ogImageAlt: SITE.ogImageAlt,
  twitterCard: 'summary_large_image',
  twitterImage: `${SITE.url}${SITE.ogImage}`,
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
    <SiteTabBar />
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

/* Room for the floating tab bar, so it never sits on the footer's last line. */
@media (max-width: 720px) {
  .site { padding-bottom: calc(80px + env(safe-area-inset-bottom)); }
}

@media (min-width: 1024px) {
  .site--bare { height: 100dvh; min-height: 0; }
  .site--bare .main { min-height: 0; }
}
</style>
