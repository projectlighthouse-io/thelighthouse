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
    <div class="reader-subbar">
      <div class="reader-subbar__inner">
        <NuxtLink class="reader-subbar__book" to="/syntax">syntax</NuxtLink>
        <span class="reader-subbar__sep">›</span>
        <span class="reader-subbar__cur">{{ lang.name }}</span>
        <span class="reader-subbar__spacer" />
        <span class="reader-subbar__rt">{{ lang.readMinutes }} min read</span>
      </div>
    </div>

    <div class="reader-layout">
      <aside class="reader-toc">
        <div class="reader-toc__label">On this page</div>
        <nav class="reader-toc__list">
          <a
            v-for="item in lang.toc"
            :key="item.id"
            class="reader-toc__item"
            :href="`#${item.id}`"
          >
            {{ item.text }}
          </a>
        </nav>
      </aside>

      <article class="reader-article">
        <div class="reader-eyebrow">reference</div>
        <h1>{{ lang.name }}</h1>
        <p class="reader-dek">{{ lang.description }}</p>

        <div class="reader-prose" style="margin-top: 44px">
          <!-- eslint-disable-next-line vue/no-v-html -- authored markdown, rendered server side -->
          <div class="lesson-content" v-html="lang.html" />
        </div>
      </article>

      <aside class="reader-aside">
        <div class="font-mono text-xs text-read-mute">{{ lang.toc.length }} sections</div>
      </aside>
    </div>
  </div>
</template>
