<script setup lang="ts">
import { csrfHeader } from '@/composables/UseReader'

definePageMeta({ layout: 'default', middleware: 'auth' })

useSeo({
  title: 'Public profile — projectlighthouse',
  description: 'What other learners see on your profile.',
  noindex: true,
})

/** What the api sends. Snake case, because it is the wire. */
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

/** The eight a reader owns. `username` and `github_username` are shown, not sent. */
interface Editable {
  tagline: string
  bio: string
  company: string
  education: string
  location: string
  linkedin_url: string
  x_url: string
  website_url: string
}

const MAX = { tagline: 160, bio: 1000, short: 255, location: 120 }

const form = reactive<Editable>({
  tagline: '',
  bio: '',
  company: '',
  education: '',
  location: '',
  linkedin_url: '',
  x_url: '',
  website_url: '',
})

const username = ref<string | null>(null)
const githubUsername = ref<string | null>(null)
const busy = ref(false)
const problem = ref('')
const saved = ref(false)

function fill(profile: Profile) {
  // Null is "not written", and an input's value is a string; the two meet here
  // and nowhere else, so the form never has to think about null.
  form.tagline = profile.tagline ?? ''
  form.bio = profile.bio ?? ''
  form.company = profile.company ?? ''
  form.education = profile.education ?? ''
  form.location = profile.location ?? ''
  form.linkedin_url = profile.linkedin_url ?? ''
  form.x_url = profile.x_url ?? ''
  form.website_url = profile.website_url ?? ''
  username.value = profile.username
  githubUsername.value = profile.github_username
}

/** Behind a session, so nothing to render on the server and nothing to cache. */
onMounted(async () => {
  try {
    fill(await $fetch<Profile>('/api/settings/profile'))
  }
  catch {
    problem.value = 'Could not load your profile.'
  }
})

async function save() {
  if (busy.value) return

  busy.value = true
  problem.value = ''
  saved.value = false

  try {
    // The whole form every time — the api writes every field it is sent, so
    // clearing an input is how a field is cleared. See `users::update_profile`.
    fill(await $fetch<Profile>('/api/settings/profile', {
      method: 'PATCH',
      headers: csrfHeader(),
      body: { ...form },
    }))
    saved.value = true
  }
  catch (error: unknown) {
    // The api's wording when it sent one: it names the field that was wrong.
    problem.value = (error as { data?: { error?: string } })?.data?.error
      ?? 'Could not save your profile.'
  }
  finally {
    busy.value = false
  }
}
</script>

<template>
  <SettingsShell>
    <h2 class="font-serif text-2xl text-ink">Public profile</h2>
    <p class="mt-2 text-sm leading-relaxed text-quiet">What other learners see.</p>

    <dl v-if="username" class="mt-6 overflow-hidden rounded-lg border border-rule">
      <div class="flex items-baseline justify-between gap-4 px-4 py-3" :class="githubUsername && 'border-b border-rule'">
        <dt class="shrink-0 text-xs font-medium tracking-wider uppercase text-faint">username</dt>
        <dd class="min-w-0 truncate text-sm text-ink">{{ username }}</dd>
      </div>
      <div v-if="githubUsername" class="flex items-baseline justify-between gap-4 px-4 py-3">
        <dt class="shrink-0 text-xs font-medium tracking-wider uppercase text-faint">github</dt>
        <dd class="min-w-0 truncate text-sm text-ink">{{ githubUsername }}</dd>
      </div>
    </dl>

    <form class="mt-8 space-y-6" @submit.prevent="save">
      <label class="block">
        <span class="text-xs font-medium tracking-wider uppercase text-faint">tagline</span>
        <input
          v-model="form.tagline"
          type="text"
          :maxlength="MAX.tagline"
          placeholder="One line about you"
          class="mt-2 w-full rounded-md border border-rule bg-panel px-4 py-2.5 text-sm text-ink transition outline-none placeholder:text-faint focus:border-ink"
        >
      </label>

      <label class="block">
        <span class="text-xs font-medium tracking-wider uppercase text-faint">bio</span>
        <textarea
          v-model="form.bio"
          rows="3"
          :maxlength="MAX.bio"
          class="mt-2 w-full rounded-md border border-rule bg-panel px-4 py-2.5 text-sm text-ink transition outline-none placeholder:text-faint focus:border-ink"
        />
      </label>

      <label class="block">
        <span class="text-xs font-medium tracking-wider uppercase text-faint">company</span>
        <input
          v-model="form.company"
          type="text"
          :maxlength="MAX.short"
          class="mt-2 w-full rounded-md border border-rule bg-panel px-4 py-2.5 text-sm text-ink transition outline-none placeholder:text-faint focus:border-ink"
        >
      </label>

      <label class="block">
        <span class="text-xs font-medium tracking-wider uppercase text-faint">education</span>
        <input
          v-model="form.education"
          type="text"
          :maxlength="MAX.short"
          class="mt-2 w-full rounded-md border border-rule bg-panel px-4 py-2.5 text-sm text-ink transition outline-none placeholder:text-faint focus:border-ink"
        >
      </label>

      <label class="block">
        <span class="text-xs font-medium tracking-wider uppercase text-faint">location</span>
        <input
          v-model="form.location"
          type="text"
          :maxlength="MAX.location"
          placeholder="City, country"
          class="mt-2 w-full rounded-md border border-rule bg-panel px-4 py-2.5 text-sm text-ink transition outline-none placeholder:text-faint focus:border-ink"
        >
      </label>

      <label class="block">
        <span class="text-xs font-medium tracking-wider uppercase text-faint">linkedin</span>
        <input
          v-model="form.linkedin_url"
          type="url"
          :maxlength="MAX.short"
          placeholder="https://linkedin.com/in/your-handle"
          class="mt-2 w-full rounded-md border border-rule bg-panel px-4 py-2.5 text-sm text-ink transition outline-none placeholder:text-faint focus:border-ink"
        >
      </label>

      <label class="block">
        <span class="text-xs font-medium tracking-wider uppercase text-faint">x</span>
        <input
          v-model="form.x_url"
          type="url"
          :maxlength="MAX.short"
          placeholder="https://x.com/your-handle"
          class="mt-2 w-full rounded-md border border-rule bg-panel px-4 py-2.5 text-sm text-ink transition outline-none placeholder:text-faint focus:border-ink"
        >
      </label>

      <label class="block">
        <span class="text-xs font-medium tracking-wider uppercase text-faint">website</span>
        <input
          v-model="form.website_url"
          type="url"
          :maxlength="MAX.short"
          placeholder="https://yourdomain.dev"
          class="mt-2 w-full rounded-md border border-rule bg-panel px-4 py-2.5 text-sm text-ink transition outline-none placeholder:text-faint focus:border-ink"
        >
      </label>

      <div class="flex items-center gap-4">
        <button
          type="submit"
          :disabled="busy"
          class="rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover disabled:opacity-50"
        >
          {{ busy ? 'Saving…' : 'Save' }}
        </button>
        <span v-if="saved" class="text-sm text-quiet">Saved.</span>
        <span v-if="problem" class="text-sm text-red-700">{{ problem }}</span>
      </div>
    </form>
  </SettingsShell>
</template>
