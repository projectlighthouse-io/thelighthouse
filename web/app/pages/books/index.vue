<script setup lang="ts">
import type { Book } from '@/types/Content'

const description
  = 'Interactive programming books on Go, Rust, data structures and algorithms, networking fundamentals, and operating systems. Learn software engineering fundamentals from first principles with hands-on examples.'

// From ohara, through the rust api. During SSR this calls the handler directly,
// so it costs no HTTP round trip.
const { data } = await useShelf()

const books = computed<Book[]>(() => data.value ?? [])

/**
 * The track tabs, named here so their order is a decision rather than
 * whatever the response happened to contain. A book on no track is still on
 * the shelf under `all`. `?track=` is the tab, so a track is a link.
 */
const TRACKS = ['go', 'rust', 'systems'] as const

type Track = 'all' | typeof TRACKS[number]

const isTrack = (value: unknown): value is Track =>
  value === 'all' || (TRACKS as readonly string[]).includes(value as string)

const route = useRoute()
const router = useRouter()

const track = ref<Track>(isTrack(route.query.track) ? route.query.track : 'all')

watch(track, (chosen) => {
  void router.replace({ query: chosen === 'all' ? {} : { track: chosen } })
})

const onTrack = (book: Book, key: Track): boolean =>
  key === 'all' || book.tracks[key] !== undefined

const tabs = computed(() =>
  (['all', ...TRACKS] as const)
    .map(key => ({ key, label: key, count: books.value.filter(b => onTrack(b, key)).length }))
    .filter(tab => tab.key === 'all' || tab.count > 0))

/** A track is a reading order, so a track shows in that order. */
const shown = computed<Book[]>(() => {
  const chosen = track.value
  const matching = books.value.filter(book => onTrack(book, chosen))

  if (chosen === 'all') return matching

  return [...matching].sort((a, b) => (a.tracks[chosen] ?? 0) - (b.tracks[chosen] ?? 0))
})

useSeo({
  title: 'Programming Books - Go, Rust, DSA, Networking, OS',
  description,
})

useJsonLd('books', () => ({
  '@type': 'CollectionPage',
  'name': 'Books',
  'mainEntity': {
    '@type': 'ItemList',
    'itemListElement': books.value.map((b, i) => ({
      '@type': 'ListItem',
      'position': i + 1,
      'url': `${SITE.url}/books/${b.slug}`,
      'name': b.title,
    })),
  },
}))
</script>

<template>
  <div class="books">
    <section class="lh-text lh-gap">
      <div class="lh-head">
        <h1 class="lh-h2">Books</h1>
        <p class="lh-sub">Carefully crafted books to help you level up your skills</p>
      </div>
    </section>

    <div class="lh-shelf shelf">
      <div v-if="tabs.length > 1" class="tabs">
        <SegmentedFilter v-model="track" :options="tabs" label="filter by track" />
      </div>

      <BookGrid :books="shown" />
    </div>
  </div>
</template>

<style scoped>
.shelf { margin-top: var(--space-12); }

.tabs {
  display: flex;
  justify-content: center;
  margin-bottom: 40px;
}
</style>
