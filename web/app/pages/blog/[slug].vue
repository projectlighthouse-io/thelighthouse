<script setup lang="ts">
import type { BlogPostResponse } from '@/types/Content'
const route = useRoute()
const slug = computed<string>(() => String(route.params.slug))
const { reader } = useReader()

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
  // Whoever wrote it — readers publish here too — with their profile when
  // they have a username to link it by.
  'author': {
    '@type': 'Person',
    'name': post.value?.author,
    ...(post.value?.authorUsername ? { url: `${SITE.url}/users/@${post.value.authorUsername}` } : {}),
  },
  'publisher': { '@type': 'Organization', 'name': SITE.name, 'url': SITE.url },
  'mainEntityOfPage': `${SITE.url}/blog/${slug.value}`,
}))
</script>

<template>
  <article v-if="post" class="post lh-text lh-gap">
    <header class="lh-head">
      <p class="lh-eyebrow">
        <NuxtLink to="/blog" class="lh-link">blog</NuxtLink>
        · <time :datetime="post.publishedAt">{{ post.publishedAt }}</time>
        · {{ post.readMinutes }} min read
      </p>
      <h1 class="lh-h1">{{ post.title }}</h1>
      <p class="lh-lede">{{ post.description }}</p>
      <ul v-if="post.tags.length" class="tags" aria-label="tags">
        <li v-for="tag in post.tags" :key="tag">#{{ tag }}</li>
      </ul>
      <!-- Client only: whether this reader wrote it is client state, and the
           page is one cached document for everyone. The api checks ownership
           on save regardless. -->
      <ClientOnly>
        <p v-if="post.authorUsername && post.authorUsername === reader?.username" class="lh-eyebrow">
          <NuxtLink :to="`/blog/edit/${post.slug}`" class="lh-link">edit this post</NuxtLink>
        </p>
      </ClientOnly>
    </header>

    <!-- eslint-disable-next-line vue/no-v-html -- authored markdown, rendered at build -->
    <div class="lesson-content body" v-html="post.html" />
  </article>
</template>

<style scoped>
.tags {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-wrap: wrap;
  gap: 6px var(--space-4);
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

.body { margin-top: var(--space-12); }
</style>
