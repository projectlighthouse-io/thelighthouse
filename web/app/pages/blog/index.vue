<script setup lang="ts">
import type { BlogPostResponse } from '@/types/Content'
// the listing needs titles and dates, not bodies
const { data: sorted } = await useAsyncData('posts', () => $fetch<BlogPostResponse[]>('/_api/blog'), {
  default: () => [],
})

const description
  = 'Notes on systems programming, Go, Rust, networking and the runtime under your code.'

useSeo({
  title: 'Blog — projectlighthouse',
  description,
})

useJsonLd('blog', {
  '@type': 'Blog',
  'name': 'projectlighthouse blog',
  'url': `${SITE.url}/blog`,
  'blogPost': sorted.value.map(p => ({
    '@type': 'BlogPosting',
    'headline': p.title,
    'description': p.description,
    'datePublished': p.publishedAt,
    'url': `${SITE.url}/blog/${p.slug}`,
  })),
})
</script>

<template>
  <div class="blog lh-text lh-gap">
    <div class="lh-head">
      <p class="lh-eyebrow">blog</p>
      <h1 class="lh-h1">Notes from the workshop floor</h1>
    </div>

    <ol class="list">
      <li v-for="post in sorted" :key="post.slug">
        <PostRow
          :to="`/blog/${post.slug}`"
          :eyebrow="`${post.publishedAt} · ${post.readMinutes} min read${post.tags.length ? ' · ' + post.tags.map(t => '#' + t).join(' ') : ''}`"
          :datetime="post.publishedAt"
          :title="post.title"
          :caption="post.description"
        />
      </li>
    </ol>
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
