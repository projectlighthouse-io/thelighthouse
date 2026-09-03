<script setup lang="ts">
import type { BlogPostResponse } from '@/types/Content'
const route = useRoute()
const slug = computed<string>(() => String(route.params.slug))

// Fetched, not imported — importing ships every post's full markdown to render
// one. SSR calls the handler directly, so this costs no round trip.
const { data: post } = await useAsyncData(
  () => `post:${slug.value}`,
  () => $fetch<BlogPostResponse>(`/_api/blog/${slug.value}`),
  { watch: [slug] },
)

if (!post.value) {
  throw createError({ statusCode: 404, statusMessage: 'Post not found', fatal: true })
}

useSeo(() => ({
  title: `${post.value?.title} — projectlighthouse`,
  description: post.value?.description ?? '',
  type: 'article',
  publishedAt: post.value?.publishedAt,
}))

useJsonLd('post', () => ({
  '@type': 'BlogPosting',
  'headline': post.value?.title,
  'description': post.value?.description,
  'datePublished': post.value?.publishedAt,
  'keywords': post.value?.tags.join(', '),
  'author': { '@type': 'Person', 'name': 'Aryan Ahmed' },
  'publisher': { '@type': 'Organization', 'name': SITE.name, 'url': SITE.url },
  'mainEntityOfPage': `${SITE.url}/blog/${slug.value}`,
}))
</script>

<template>
  <article v-if="post" class="mx-auto max-w-3xl px-2 py-16 sm:px-6 lg:px-8">
    <nav class="mb-8 font-mono text-sm text-faint">
      <NuxtLink to="/blog" class="hover:text-ink">blog</NuxtLink>
      <span class="mx-3 text-crumb">/</span>
      <span class="text-quiet">{{ post.slug }}</span>
    </nav>

    <div class="mb-4 flex flex-wrap items-center gap-3 font-mono text-xs text-faint">
      <time :datetime="post.publishedAt">{{ post.publishedAt }}</time>
      <span>·</span>
      <span>{{ post.readMinutes }} min read</span>
      <span v-for="tag in post.tags" :key="tag" class="text-teal">#{{ tag }}</span>
    </div>

    <h1 class="font-editorial text-4xl leading-tight text-ink sm:text-5xl font-semibold">
      {{ post.title }}
    </h1>

    <p class="mt-6 font-serif text-lg leading-relaxed text-quiet">{{ post.description }}</p>

    <!-- eslint-disable-next-line vue/no-v-html -- authored markdown, rendered at build -->
    <div class="lesson-content prose mt-12 max-w-none" v-html="post.html" />
  </article>
</template>
