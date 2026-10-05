<script setup lang="ts">
import { languages } from '@/data/Syntax'

const description
  = 'Quick syntax reference for Go, Rust, and C. One page per language, heavily commented code you can scan in minutes.'

useSeo({
  title: 'Syntax Reference — projectlighthouse',
  description,
})

useJsonLd('syntax', {
  '@type': 'CollectionPage',
  'name': 'Syntax Reference',
  'mainEntity': {
    '@type': 'ItemList',
    'itemListElement': languages.map((l, i) => ({
      '@type': 'ListItem',
      'position': i + 1,
      'url': `${SITE.url}/syntax/${l.slug}`,
      'name': `${l.name} Syntax Reference`,
    })),
  },
})
</script>

<template>
  <div class="syntax lh-text lh-gap">
    <div class="lh-head">
      <p class="lh-eyebrow">reference</p>
      <h1 class="lh-h1">Syntax reference</h1>
      <p class="lh-sub">
        One page per language. Heavily commented code you can scan in minutes. Not a tutorial —
        a reference you keep open in a tab.
      </p>
    </div>

    <ul class="list">
      <li v-for="lang in languages" :key="lang.slug">
        <PostRow :to="`/syntax/${lang.slug}`" eyebrow="reference" :title="lang.name" :caption="lang.description" />
      </li>
    </ul>
  </div>
</template>

<style scoped>
.list {
  margin: var(--space-12) calc(var(--space-4) * -1) 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 2px;
}
</style>
