<script setup lang="ts">
import type { BlogListResponse } from '@/types/Content'

definePageMeta({ middleware: 'auth' })

useSeo({
  title: 'Profile — projectlighthouse',
  description: 'Your projectlighthouse profile.',
  noindex: true,
})

/** The public half of a profile — what `/settings/public-profile` writes. */
interface Profile {
  username: string | null
  github_username: string | null
  tagline: string | null
  bio: string | null
  company: string | null
  education: string | null
  location: string | null
  linkedin_url: string | null
  x_url: string | null
  website_url: string | null
}

// The route guard already resolved the session to let this page render, so
// this reads the state rather than asking again.
const { reader, initials } = useReader()

/**
 * Client side, always.
 *
 * Everything here is behind the session cookie and the page is `noindex`, so
 * there is nothing for the server to render that anyone else may see — and a
 * server-side call would need the cookie forwarded to say anything at all.
 *
 * The error is kept rather than swallowed: a profile that failed to load and a
 * profile with nothing in it both render as no profile, and those want
 * different words.
 */
const { data: profile, error: profileFailed } = await useAsyncData<Profile>(
  'profile',
  () => $fetch<Profile>('/api/settings/profile'),
  { server: false },
)

/** Whether anything has actually been written. `username` is not counted — it
 *  is assigned at sign-up, not something a reader filled in. */
const hasProfile = computed<boolean>(() => {
  const p = profile.value
  if (!p) return false

  return !!(p.tagline || p.bio || p.company || p.education || p.location
    || p.website_url || p.github_username || p.linkedin_url || p.x_url)
})

/** The facts under the name, in the order they read: where, then what. */
const facts = computed<string[]>(() =>
  [profile.value?.company, profile.value?.education, profile.value?.location]
    .filter((f): f is string => !!f))

const links = computed(() =>
  [
    { label: 'website', href: profile.value?.website_url },
    { label: 'github', href: profile.value?.github_username ? `https://github.com/${profile.value.github_username}` : null },
    { label: 'linkedin', href: profile.value?.linkedin_url },
    { label: 'x', href: profile.value?.x_url },
  ].filter((l): l is { label: string, href: string } => !!l.href))

/**
 * `?page=` rather than a ref, so the back button walks the pages and a link to
 * one still opens on it — the shape `/blog` already uses.
 */
const route = useRoute()
const router = useRouter()

// `numberParam` answers `null` for anything that is not a page number — a
// typo, a stray `?page=`, a stale link — and page one is the right place to
// land for all of them.
const page = computed<number>(() => numberParam(route.query.page, 1, 100_000) ?? 1)

const PER_PAGE = 10

/**
 * This reader's own writing.
 *
 * Keyed on the username, and skipped entirely while there is not one: a row
 * that predates the username column has nothing to ask the api with, and
 * `author=` empty would fetch the whole blog rather than nothing.
 */
const username = computed<string | null>(() => reader.value?.username ?? null)

const { data: listing, pending } = await useAsyncData<BlogListResponse>(
  () => `profile-articles:${username.value}:${page.value}`,
  () => username.value
    ? $fetch<BlogListResponse>('/_api/blog', {
        query: { author: username.value, page: page.value, per_page: PER_PAGE },
      })
    : Promise.resolve({ items: [], page: 1, per_page: PER_PAGE, total: 0 }),
  {
    server: false,
    default: () => ({ items: [], page: 1, per_page: PER_PAGE, total: 0 }),
    watch: [username, page],
  },
)

const pageCount = computed<number>(
  () => pageCountOf(listing.value.total, listing.value.per_page),
)

const pageLink = (n: number) => ({ path: '/profile', query: n > 1 ? { page: String(n) } : {} })

/** Keeps a stale `?page=` from stranding the reader on an empty list — the api
 *  clamps the page it answers, so this follows it back. */
watch(listing, (answer) => {
  if (answer.page !== page.value) void router.replace(pageLink(answer.page))
})

const when = (iso: string): string =>
  new Date(iso).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
</script>

<template>
  <div class="mx-auto max-w-3xl px-4 py-16 sm:px-6 lg:px-8">
    <header class="flex items-start gap-5">
      <img
        v-if="reader?.avatar"
        :src="reader.avatar"
        alt=""
        class="size-16 shrink-0 rounded-full object-cover"
      >
      <div
        v-else
        class="flex size-16 shrink-0 items-center justify-center rounded-full bg-ink font-mono text-lg text-on-ink"
      >
        {{ initials }}
      </div>

      <div class="min-w-0">
        <h1 class="font-serif text-3xl tracking-tight text-ink">
          {{ reader?.name ?? 'Your profile' }}
        </h1>

        <!-- No email. This page is about what a reader chose to say about
             themselves, and their address is neither that nor anyone's
             business — it lives in settings, where it can be changed. -->
        <p v-if="profile?.username" class="mt-1 font-mono text-xs text-quiet">
          @{{ profile.username }}
        </p>

        <p v-if="profile?.tagline" class="mt-3 font-serif text-lg leading-relaxed text-ink">
          {{ profile.tagline }}
        </p>
      </div>
    </header>

    <p v-if="profileFailed" class="mt-6 font-mono text-sm text-quiet" role="alert">
      Your profile could not be loaded.
    </p>

    <p
      v-else-if="profile && !hasProfile"
      class="mt-6 font-serif text-base leading-relaxed text-read-ink-soft"
    >
      Nothing here yet — a tagline, a bio and where to find you all live in
      <NuxtLink to="/settings/public-profile" class="text-teal-deep underline underline-offset-2">
        your public profile
      </NuxtLink>.
    </p>

    <p v-if="profile?.bio" class="mt-6 font-serif text-base leading-relaxed text-read-ink-soft">
      {{ profile.bio }}
    </p>

    <p v-if="facts.length" class="mt-4 flex flex-wrap gap-x-4 gap-y-1 font-mono text-xs text-faint">
      <span v-for="fact in facts" :key="fact">{{ fact }}</span>
    </p>

    <p v-if="links.length" class="mt-3 flex flex-wrap gap-x-4 gap-y-1 font-mono text-xs">
      <a
        v-for="link in links"
        :key="link.label"
        :href="link.href"
        target="_blank"
        rel="noopener noreferrer me"
        class="text-teal-deep hover:text-ink"
      >{{ link.label }}</a>
    </p>

    <!-- ---------- their writing ---------- -->
    <section class="mt-16">
      <h2 class="font-serif text-2xl text-ink">Writing</h2>

      <p v-if="pending" class="mt-6 font-mono text-sm text-quiet">Loading…</p>

      <p v-else-if="!username" class="mt-6 font-serif text-base text-read-ink-soft">
        Your account predates usernames, so your posts cannot be listed here yet.
        <NuxtLink to="/settings/public-profile" class="text-teal-deep underline underline-offset-2">
          Pick one
        </NuxtLink>
        and they will appear.
      </p>

      <p v-else-if="listing.total === 0" class="mt-6 font-serif text-base text-read-ink-soft">
        Nothing published yet.
        <NuxtLink to="/blog/write-something-amazing" class="text-teal-deep underline underline-offset-2">
          Write something
        </NuxtLink>.
      </p>

      <template v-else>
        <ul class="mt-6 space-y-px">
          <li v-for="post in listing.items" :key="post.slug">
            <NuxtLink
              :to="`/blog/${post.slug}`"
              class="hover-border-pencil group block rounded-md px-4 py-3"
            >
              <span class="block font-serif text-lg leading-snug text-ink transition group-hover:text-teal-deep">
                {{ post.title }}
              </span>
              <span class="mt-1 flex flex-wrap items-center gap-x-3 font-mono text-xs text-read-faint">
                <time :datetime="post.publishedAt">{{ when(post.publishedAt) }}</time>
                <span>{{ post.readMinutes }} min read</span>
                <!-- Only the author sees this listing, so a taken down post is
                     shown rather than hidden — it is theirs, and knowing it is
                     down is the point. -->
                <span v-if="post.takenDownAt" class="text-amber">taken down</span>
              </span>
            </NuxtLink>
          </li>
        </ul>

        <nav v-if="pageCount > 1" class="mt-8 flex items-center justify-between font-mono text-sm">
          <NuxtLink
            v-if="listing.page > 1"
            :to="pageLink(listing.page - 1)"
            class="text-faint hover:text-ink"
          >
            ←——— newer
          </NuxtLink>
          <span v-else />

          <span class="text-faint">{{ listing.page }} / {{ pageCount }} — {{ listing.total }} posts</span>

          <NuxtLink
            v-if="listing.page < pageCount"
            :to="pageLink(listing.page + 1)"
            class="text-faint hover:text-ink"
          >
            older ———→
          </NuxtLink>
          <span v-else />
        </nav>
      </template>
    </section>
  </div>
</template>
