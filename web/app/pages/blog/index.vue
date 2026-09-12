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
const { data: listing, error: failed, refresh } = await useAsyncData(
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

// **A listing that could not be fetched is not an empty listing.** Left as
// one, the page renders "nothing here yet" and answers 200 — which the rule on
// `/blog/**` then lets the edge keep for a minute past the api coming back, and
// the browser keep longer. So the failure is thrown: nitro answers 5xx, and a
// 5xx is stored nowhere.
if (failed.value) {
  // The rule on `/blog/**` says `s-maxage`, and it says it whatever the status
  // is. A 502 the edge kept would outlive the outage it describes.
  // Straight onto the node response: `setResponseHeader` is h3's, and this is
  // app code, which does not have it.
  const event = import.meta.server ? useRequestEvent() : null
  event?.node.res.setHeader('cache-control', 'private, no-store')

  throw createError({
    statusCode: 502,
    statusMessage: 'The blog is not answering',
    fatal: true,
  })
}

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
      <div class="mx-auto max-w-3xl px-4 sm:px-6 lg:px-8 text-center">
        <h1 class="font-sans font-semibold text-4xl tracking-tight text-ink sm:text-5xl">Blog</h1>
      </div>
    </section>

    <!-- Topics on the left, what the reader can *do* on the right: the write
         button used to sit under the title, centred, where it read as part of
         the masthead rather than as an action. -->
    <section class="pb-6">
      <div class="mx-auto flex max-w-3xl flex-wrap items-center justify-between gap-4 px-4 sm:px-6 lg:px-8">
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

        <div v-if="isSignedIn" class="flex items-center gap-4">
          <NuxtLink
            v-if="mine"
            :to="mine"
            class="font-mono text-xs text-faint hover:text-ink"
          >
            my writing
          </NuxtLink>

          <NuxtLink to="/blog/write-something-amazing" class="btn-write">
            write
            <span class="btn-write-arrow" aria-hidden="true">→</span>
          </NuxtLink>
        </div>
      </div>
    </section>

    <section class="pb-20">
      <div class="mx-auto max-w-3xl px-4 sm:px-6 lg:px-8">
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

      <!-- Outside the reading column, which is sized for article cards rather
           than for a landscape envelope. -->
      <div class="mx-auto mt-16 max-w-4xl px-4 sm:px-6 lg:px-8">
        <MarketingNewsletterForm />
      </div>
    </section>
  </div>
</template>

<style scoped>
/* The one filled button on the page, so it reads as the action rather than as
   another link. `--color-on-ink` is the token for text on an ink fill — it
   flips with the ink, not with the paper. */
.btn-write {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.4rem 0.95rem;
    border-radius: 999px;
    background: var(--color-ink);
    color: var(--color-on-ink);
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 500;
    letter-spacing: -0.005em;
    box-shadow: 0 1px 2px rgb(0 0 0 / 0.18);
    transition:
        transform 140ms ease,
        box-shadow 140ms ease,
        background-color 140ms ease;
}

.btn-write:hover {
    background: var(--color-ink-hover);
    transform: translateY(-1px);
    box-shadow: 0 4px 10px rgb(0 0 0 / 0.18);
}

.btn-write:active {
    transform: translateY(0);
    box-shadow: 0 1px 2px rgb(0 0 0 / 0.18);
}

.btn-write-arrow {
    transition: transform 140ms ease;
}

.btn-write:hover .btn-write-arrow {
    transform: translateX(2px);
}

/* The lift is decoration. Anybody who asked not to be moved gets the colour
   change and nothing else. */
@media (prefers-reduced-motion: reduce) {
    .btn-write,
    .btn-write-arrow {
        transition: background-color 140ms ease;
    }

    .btn-write:hover {
        transform: none;
    }

    .btn-write:hover .btn-write-arrow {
        transform: none;
    }
}

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
