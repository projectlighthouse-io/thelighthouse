<script setup lang="ts">
/**
 * Another reader's public page, at `/users/@username` — the path laravel used,
 * so links already out there keep working.
 *
 * Server rendered: unlike `/profile`, nothing here depends on who is looking.
 */
interface PublicProfile {
  name: string
  avatar: string | null
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

const route = useRoute()
const username = computed(() => String(route.params.username ?? '').replace(/^@/, ''))

const { data: profile } = await useFetch<PublicProfile>(
  () => `/_api/users/${encodeURIComponent(username.value)}`,
)

if (!profile.value) {
  throw createError({ statusCode: 404, statusMessage: 'Reader not found', fatal: true })
}

useSeo({
  title: `${profile.value.name} — projectlighthouse`,
  // Their own words first; the site line makes sure even an empty profile has
  // a description worth showing in a result.
  description: [
    profile.value.tagline,
    profile.value.bio,
    `${profile.value.name}'s public profile on projectlighthouse, where developers learn systems programming by building real software.`,
  ].filter(Boolean).join(' '),
})

const initials = computed(() =>
  (profile.value?.name ?? '').split(/\s+/).map(w => w[0] ?? '').join('').slice(0, 2).toUpperCase())

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
</script>

<template>
  <div v-if="profile" class="mx-auto max-w-3xl px-4 py-16 sm:px-6 lg:px-8">
    <header class="flex items-start gap-5">
      <img
        v-if="profile.avatar"
        :src="profile.avatar"
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
        <h1 class="font-serif text-3xl tracking-tight text-ink">{{ profile.name }}</h1>
        <p class="mt-1 font-mono text-xs text-quiet">@{{ username }}</p>
        <p v-if="profile.tagline" class="mt-3 font-serif text-lg leading-relaxed text-ink">
          {{ profile.tagline }}
        </p>
      </div>
    </header>

    <p v-if="profile.bio" class="mt-6 font-serif text-base leading-relaxed text-read-ink-soft">
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

    <p class="mt-12 font-mono text-sm">
      <NuxtLink :to="`/blog?author=${username}`" class="text-teal-deep hover:text-ink">
        Writing by @{{ username }} →
      </NuxtLink>
    </p>
  </div>
</template>
