<script setup lang="ts">
import type { BlogListResponse } from '@/types/Content'
import { CATEGORIES } from '@/composables/UseArticles'

const route = useRoute()
const topic = computed<string>(() => String(route.query.topic ?? ''))

/** `?author=<username>` — one author's articles rather than everybody's. */
const author = computed<string>(() => String(route.query.author ?? ''))

// `numberParam` coerces the same way `/_api/blog` does — see it for why.
const page = computed(() => numberParam(route.query.page, 1, 100_000))
const perPage = computed(() => numberParam(route.query.per_page, 1, 50))

// The listing needs cards, not bodies — `/_api/blog` sends an excerpt and a
// read time, never the raw markdown.
//
// `server: false` whenever an author is named, and that is not a preference:
// the api answers an author reading their own shelf with their taken down
// articles, and this document is cached by the edge on its url alone. Rendered
// here, one reader's private rows would be handed to the next person opening
// the same link. Fetched in the browser, they never reach the html.
const { data: listing, refresh } = await useAsyncData(
  () => `articles:${topic.value}:${author.value}:${page.value}:${perPage.value}`,
  () => $fetch<BlogListResponse>('/_api/blog', {
    query: {
      ...(topic.value ? { topic: topic.value } : {}),
      ...(author.value ? { author: author.value } : {}),
      ...(page.value ? { page: String(page.value) } : {}),
      ...(perPage.value ? { per_page: String(perPage.value) } : {}),
    },
  }),
  {
    default: () => ({ items: [], page: 1, per_page: 15, total: 0 }),
    watch: [topic, author, page, perPage],
    server: !route.query.author,
  },
)

const posts = computed(() => listing.value.items)

// The page and size the api answered with, not the ones asked for — it caps
// per_page, so a link built from the query would point at a page that does
// not exist.
const pageCount = computed(() => pageCountOf(listing.value.total, listing.value.per_page))

/** Same listing, another page: the topic and size have to survive the hop. */
const pageLink = (n: number) => ({
  path: '/blog',
  query: {
    ...(topic.value ? { topic: topic.value } : {}),
    ...(author.value ? { author: author.value } : {}),
    ...(perPage.value ? { per_page: String(perPage.value) } : {}),
    page: String(n),
  },
})

const { reader, isSignedIn } = useReader()

/** Where "my writing" goes. `null` until the reader lands, and for the rows
 *  that predate the username column. */
const mine = computed(() =>
  reader.value?.username ? ownWritingUrl(reader.value.username) : null,
)

// Whether this listing is the reader's own — for the controls only. What the
// listing *contains* is rust's decision against the session cookie, made in
// `handler::shelf`; this cannot show anybody a row the api did not send.
const isMine = computed(
  () => !!author.value && author.value === reader.value?.username,
)

const { remove } = useArticles()

const removing = ref<string | null>(null)
const error = ref<string | null>(null)

async function destroy(slug: string): Promise<void> {
  error.value = await remove(slug)
  removing.value = null

  if (!error.value) await refresh()
}

// Alphabetical here only. `CATEGORIES` keeps rust's order, which the editor's
// topic picker follows.
const topics = [...CATEGORIES].sort()

// ponytail: Intl.RelativeTimeFormat, no date library. Cascade down the units
// and hand the largest whole one to the formatter.
const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ['year', 31536000],
  ['month', 2592000],
  ['week', 604800],
  ['day', 86400],
  ['hour', 3600],
  ['minute', 60],
]

const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' })

function humanDate(iso: string): string {
  const seconds = (Date.parse(iso) - Date.now()) / 1000
  if (Number.isNaN(seconds)) return iso
  for (const [unit, size] of UNITS) {
    if (Math.abs(seconds) >= size) return rtf.format(Math.round(seconds / size), unit)
  }
  return rtf.format(Math.round(seconds), 'second')
}

const description
  = 'Notes on systems programming, Go, Rust, networking and the runtime under your code.'

useSeo({
  title: 'Blog — projectlighthouse',
  description,
})

useJsonLd('blog', () => ({
  '@type': 'Blog',
  'name': 'projectlighthouse blog',
  'url': `${SITE.url}/blog`,
  'blogPost': posts.value.map(p => ({
    '@type': 'BlogPosting',
    'headline': p.title,
    'description': p.description,
    'datePublished': p.publishedAt,
    'author': { '@type': 'Person', 'name': p.author },
    'url': `${SITE.url}/blog/${p.slug}`,
  })),
}))
</script>

<template>
  <div class="mx-auto max-w-4xl bg-panel">
    <!-- `bg-panel`, not `bg-white`: the token is #ffffff in light and the dark
         panel in dark, so the slab follows the theme. Slightly wider than the
         max-w-3xl content column inside it, so the dotted page still shows at
         the edges. -->
    <section class="py-16">
      <div class="mx-auto max-w-3xl px-2 sm:px-6 lg:px-8 text-center">
        <h1 class="mb-4 font-sans font-semibold text-4xl tracking-tight text-ink sm:text-5xl">Blog</h1>

        <div class="mt-6 flex flex-wrap items-center justify-center gap-4">
          <NuxtLink v-if="isSignedIn" to="/blog/write-something-amazing" class="btn-chalk text-sm font-medium text-ink">
            write <span class="ml-1">———→</span>
          </NuxtLink>
          <NuxtLink
            v-if="isSignedIn && mine"
            :to="mine"
            class="font-mono text-sm text-faint hover:text-ink"
          >
            my writing
          </NuxtLink>
        </div>
      </div>
    </section>

    <section class="pb-6">
      <div class="mx-auto max-w-3xl px-2 sm:px-6 lg:px-8">
        <nav class="flex flex-wrap gap-x-4 gap-y-2 font-mono text-xs">
          <NuxtLink
            to="/blog"
            class="hover:text-ink"
            :class="topic === '' ? 'text-ink' : 'text-faint'"
          >
            all
          </NuxtLink>
          <NuxtLink
            v-for="it in topics"
            :key="it"
            :to="`/blog?topic=${it}`"
            class="hover:text-ink"
            :class="topic === it ? 'text-ink' : 'text-faint'"
          >
            #{{ it }}
          </NuxtLink>

          <NuxtLink v-if="author" to="/blog" class="text-teal hover:text-ink">
            {{ isMine ? 'mine' : `@${author}` }} ✕
          </NuxtLink>
        </nav>
      </div>
    </section>

    <section class="pb-20">
      <div class="mx-auto max-w-3xl px-2 sm:px-6 lg:px-8">
        <p
          v-if="error"
          class="mb-6 rounded-md border border-stroke px-4 py-3 font-mono text-sm text-ink"
          role="alert"
        >
          {{ error }}
        </p>

        <p v-if="posts.length === 0" class="text-mono-body py-8">
          nothing here yet.
        </p>

        <article
          v-for="post in posts"
          :key="post.slug"
          class="py-4"
        >
          <h2 class="post-title break-words">
            <NuxtLink :to="`/blog/${post.slug}`" class="hover:text-link-hover">
              {{ post.title }}
            </NuxtLink>
          </h2>

          <p class="mt-2 font-mono text-xs text-faint">
            {{ post.author }}<template v-if="post.tags.length"> on <span class="inline-flex gap-2 align-baseline"><span v-for="tag in post.tags" :key="tag" class="text-teal">#{{ tag }}</span></span></template>,
            <time :datetime="post.publishedAt">{{ humanDate(post.publishedAt) }}</time>
          </p>

          <p class="mt-3 font-serif text-base leading-relaxed text-quiet break-words">{{ post.description }}</p>

          <!-- Both only ever arrive on the author's own listing: rust leaves
               the columns off every other shape, so this markup cannot show a
               takedown to anybody who is not its author. -->
          <p
            v-if="post.takenDownAt"
            class="mt-3 rounded-md border border-stroke px-4 py-3 font-mono text-sm text-quiet"
          >
            not being shown<span v-if="post.takenDownReason">: {{ post.takenDownReason }}</span>
          </p>

          <div v-if="isMine" class="mt-3 flex flex-wrap items-center gap-4 font-mono text-xs">
            <NuxtLink :to="`/blog/edit/${post.slug}`" class="text-link hover:text-link-hover">
              edit
            </NuxtLink>

            <button
              v-if="removing !== post.slug"
              type="button"
              class="cursor-pointer text-faint hover:text-ink"
              @click="removing = post.slug"
            >
              delete
            </button>

            <span v-else class="flex items-center gap-3">
              <span class="text-quiet">delete for good?</span>
              <button type="button" class="cursor-pointer text-ink underline" @click="destroy(post.slug)">
                yes
              </button>
              <button type="button" class="cursor-pointer text-faint hover:text-ink" @click="removing = null">
                no
              </button>
            </span>
          </div>
        </article>

        <nav v-if="pageCount > 1" class="mt-10 flex items-center justify-between font-mono text-xs">
          <NuxtLink
            v-if="listing.page > 1"
            :to="pageLink(listing.page - 1)"
            class="text-faint hover:text-ink"
          >
            ←——— newer
          </NuxtLink>
          <span v-else />

          <span class="text-faint">{{ listing.page }} / {{ pageCount }}</span>

          <NuxtLink
            v-if="listing.page < pageCount"
            :to="pageLink(listing.page + 1)"
            class="text-faint hover:text-ink"
          >
            older ———→
          </NuxtLink>
          <span v-else />
        </nav>
      </div>
    </section>
  </div>
</template>

<style scoped>
/* Same size as the chapter headings on /books/:slug, but Inter rather than
   their serif. */
.post-title {
    font-family: var(--font-sans);
    font-weight: 600;
    font-size: 19px;
    line-height: 1.15;
    letter-spacing: -0.005em;
    color: var(--color-read-ink);
}
</style>
