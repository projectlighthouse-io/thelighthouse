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
  'author': { '@type': 'Person', 'name': post.value?.author },
  'publisher': { '@type': 'Organization', 'name': SITE.name, 'url': SITE.url },
  'mainEntityOfPage': `${SITE.url}/blog/${slug.value}`,
}))
</script>

<template>
  <div v-if="post" class="mx-auto max-w-4xl bg-panel">
    <!-- The same slab the listing sits on: `bg-panel` rather than `bg-white`,
         so it follows the theme, and a little wider than the max-w-3xl column
         inside it so the dotted page still shows at the edges. -->
    <article class="mx-auto max-w-3xl px-2 py-16 sm:px-6 lg:px-8">
      <nav class="mb-8 font-mono text-sm text-whisper">
        <NuxtLink to="/blog" class="hover:text-ink">blog</NuxtLink>
        <span class="mx-3 text-crumb">/</span>
        <span>{{ post.slug }}</span>
      </nav>

      <h1 class="font-editorial text-4xl leading-tight text-ink sm:text-5xl font-semibold">
        {{ post.title }}
      </h1>

      <!-- The same line the listing carries: who wrote it, what it is about,
           when — and nothing else. -->
      <p class="mt-3 font-mono text-xs text-faint">
        <NuxtLink
          v-if="post.authorUsername"
          :to="`/blog?author=${post.authorUsername}`"
          class="text-rose-500 hover:underline"
        >{{ post.author }}</NuxtLink><span v-else class="text-rose-500">{{ post.author }}</span><template v-if="post.tags.length"> on <span class="inline-flex gap-2 align-baseline"><span v-for="tag in post.tags" :key="tag" class="text-teal">#{{ tag }}</span></span></template>,
        <time :datetime="post.publishedAt">{{ humanDate(post.publishedAt) }}</time>
      </p>

      <p class="mt-6 font-serif text-lg leading-relaxed text-quiet">{{ post.description }}</p>

      <!-- eslint-disable-next-line vue/no-v-html -- sanitised server side by SafeMarkdown.renderArticle; the raw markdown is never sent to this page -->
      <div class="lesson-content prose mt-12 max-w-none" v-html="post.html" />

      <!-- After the piece, not before it: somebody who has just finished
           reading is the one worth asking. -->
      <MarketingNewsletterForm class="mt-16" />
    </article>
  </div>
</template>
