<script setup lang="ts">
import type { SyntaxResponse } from '@/types/Content'
import '@/assets/css/reader.css'

const route = useRoute()
const slug = computed<string>(() => String(route.params.slug))

// Fetched rather than imported: the markdown for every language is ~100kb, and
// importing it would ship all of it to the browser to render one page. During
// SSR this calls the handler directly — no HTTP round trip — and useAsyncData
// dedupes so hydration does not fetch it a second time.
const { data: lang } = await useAsyncData(
  () => `syntax:${slug.value}`,
  () => $fetch<SyntaxResponse>(`/_api/syntax/${slug.value}`),
  { watch: [slug] },
)

if (!lang.value) {
  throw createError({ statusCode: 404, statusMessage: 'Language not found', fatal: true })
}

useSeo(() => ({
  title: `${lang.value?.name} Syntax Reference — projectlighthouse`,
  description: lang.value?.description ?? '',
  type: 'article',
}))

useJsonLd('syntax', () => ({
  '@type': 'TechArticle',
  'headline': `${lang.value?.name} Syntax Reference`,
  'description': lang.value?.description,
  'proficiencyLevel': 'Beginner',
  'publisher': { '@type': 'Organization', 'name': SITE.name, 'url': SITE.url },
}))
</script>

<template>
  <div v-if="lang" class="reader-shell">
    <div class="reader-layout">
      <aside v-if="lang.toc.length" class="reader-toc" aria-label="on this page">
        <span class="lh-eyebrow">on this page · {{ lang.toc.length }}</span>
        <nav class="reader-toc__list">
          <a v-for="item in lang.toc" :key="item.id" class="reader-toc__item" :href="`#${item.id}`">
            {{ item.text }}
          </a>
        </nav>
      </aside>

      <article class="reader-article">
        <header class="reader-head">
          <p class="lh-eyebrow">
            <NuxtLink to="/syntax" class="lh-link">syntax</NuxtLink> · reference
          </p>
          <h1 class="lh-h1">{{ lang.name }}</h1>
          <p class="lh-lede">{{ lang.description }}</p>
          <div class="reader-meta">
            <span class="lh-num">{{ lang.readMinutes }} min read</span>
          </div>
        </header>

        <div class="reader-body">
          <!-- eslint-disable-next-line vue/no-v-html -- authored markdown, rendered server side -->
          <div class="lesson-content" v-html="lang.html" />
        </div>
      </article>
    </div>
  </div>
</template>
