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
  <div>
    <section class="py-16">
      <div class="mx-auto max-w-3xl px-2 sm:px-6 lg:px-8 text-center">
        <h1 class="mb-4 font-serif text-4xl tracking-tight text-ink sm:text-5xl">Blog</h1>
        <p class="text-mono-body">notes from the workshop floor.</p>
      </div>
    </section>

    <section class="pb-20">
      <div class="mx-auto max-w-3xl px-2 sm:px-6 lg:px-8">
        <article
          v-for="post in sorted"
          :key="post.slug"
          class="border-b border-dashed border-rule-soft py-8 last:border-b-0"
        >
          <div class="mb-2 flex flex-wrap items-center gap-3 font-mono text-xs text-faint">
            <time :datetime="post.publishedAt">{{ post.publishedAt }}</time>
            <span>·</span>
            <span>{{ post.readMinutes }} min read</span>
            <span v-for="tag in post.tags" :key="tag" class="text-teal">#{{ tag }}</span>
          </div>

          <h2 class="font-editorial text-2xl text-ink sm:text-3xl font-semibold">
            <NuxtLink :to="`/blog/${post.slug}`" class="hover:text-link-hover">
              {{ post.title }}
            </NuxtLink>
          </h2>

          <p class="mt-3 font-serif text-base leading-relaxed text-quiet">{{ post.description }}</p>

          <NuxtLink :to="`/blog/${post.slug}`" class="btn-chalk mt-4 text-sm font-medium text-ink">
            read <span class="ml-1">———→</span>
          </NuxtLink>
        </article>
      </div>
    </section>
  </div>
</template>
