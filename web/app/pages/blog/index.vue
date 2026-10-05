<script setup lang="ts">
import type { BlogListResponse } from '@/types/Content'

const route = useRoute()
const { reader, isSignedIn } = useReader()

/** `?author=` narrows the listing to one writer — the editor sends authors
 *  back here to see their own. */
const author = computed<string | null>(() => {
  const value = route.query.author

  return typeof value === 'string' && value !== '' ? value : null
})

/** Reading your own listing: each row gets an edit link. */
const mine = computed<boolean>(() => !!author.value && author.value === reader.value?.username)

/**
 * Your own articles, archived and taken down ones included — the public
 * listing leaves both out, so it cannot be your shelf. Read from the api with
 * your session, in the browser; nothing about it is cached or shared.
 */
const own = useArticles()
watch(mine, (isMine) => {
  if (isMine) void own.load()
}, { immediate: true })

// the listing needs titles and dates, not bodies
const { data: sorted } = await useAsyncData(
  () => `posts:${author.value ?? ''}`,
  () => $fetch<BlogListResponse>('/_api/blog', {
    query: author.value ? { author: author.value } : {},
  }).then(page => page.items),
  { default: () => [], watch: [author] },
)

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
      <h1 class="lh-h1">
        <template v-if="mine">Your writing</template>
        <template v-else-if="author">Writing by @{{ author }}</template>
        <template v-else>Notes from the workshop floor</template>
      </h1>
      <!-- Client only: who is reading is client state, and the listing itself
           is one cached document for everyone. -->
      <ClientOnly>
        <p v-if="isSignedIn" class="write">
          <UiButton variant="ghost" size="sm" to="/blog/write-something-amazing">Write something →</UiButton>
        </p>
      </ClientOnly>
    </div>

    <ol v-if="mine" class="list">
      <li v-for="post in own.articles.value" :key="post.slug">
        <!-- Archived and taken down articles have no public page, for their
             author either, so those rows open the editor instead. -->
        <PostRow
          :to="post.archivedAt || post.takenDownAt ? `/blog/edit/${post.slug}` : `/blog/${post.slug}`"
          :eyebrow="[
            (post.createdAt ?? '').slice(0, 10),
            post.archivedAt ? 'archived' : '',
            post.takenDownAt ? 'taken down' : '',
          ].filter(Boolean).join(' · ')"
          :title="post.title"
          :caption="post.subtitle"
        />
        <NuxtLink :to="`/blog/edit/${post.slug}`" class="lh-link edit">edit</NuxtLink>
      </li>
      <li v-if="own.loaded.value && !own.articles.value.length" class="lh-muted">Nothing written yet.</li>
    </ol>

    <ol v-else class="list">
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
.write { margin: var(--space-4) 0 0; }

.edit {
  display: inline-block;
  margin: 0 var(--space-4);
  font: var(--text-label-mono);
}

.list {
  margin: var(--space-12) calc(var(--space-4) * -1) 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 2px;
}
</style>
