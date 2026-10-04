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
  <AccountShell title="Public profile" sub="What other learners see.">
    <div v-if="username" class="lh-card known">
      <dl class="lh-facts">
        <dt>username</dt>
        <dd>{{ username }}</dd>
        <template v-if="githubUsername">
          <dt>github</dt>
          <dd>{{ githubUsername }}</dd>
        </template>
      </dl>
    </div>

    <form class="form" @submit.prevent="save">
      <label class="lh-field">
        <span class="lh-label">tagline</span>
        <input
          v-model="form.tagline"
          type="text"
          :maxlength="MAX.tagline"
          placeholder="One line about you"
          class="lh-input"
        >
      </label>
      <label class="lh-field">
        <span class="lh-label">bio</span>
        <textarea v-model="form.bio" rows="3" :maxlength="MAX.bio" class="lh-input" />
      </label>
      <label class="lh-field">
        <span class="lh-label">company</span>
        <input
          v-model="form.company"
          type="text"
          :maxlength="MAX.short"
          class="lh-input"
        >
      </label>
      <label class="lh-field">
        <span class="lh-label">education</span>
        <input
          v-model="form.education"
          type="text"
          :maxlength="MAX.short"
          class="lh-input"
        >
      </label>
      <label class="lh-field">
        <span class="lh-label">location</span>
        <input
          v-model="form.location"
          type="text"
          :maxlength="MAX.location"
          placeholder="City, country"
          class="lh-input"
        >
      </label>
      <label class="lh-field">
        <span class="lh-label">linkedin</span>
        <input
          v-model="form.linkedin_url"
          type="url"
          :maxlength="MAX.short"
          placeholder="https://linkedin.com/in/your-handle"
          class="lh-input"
        >
      </label>
      <label class="lh-field">
        <span class="lh-label">x</span>
        <input
          v-model="form.x_url"
          type="url"
          :maxlength="MAX.short"
          placeholder="https://x.com/your-handle"
          class="lh-input"
        >
      </label>
      <label class="lh-field">
        <span class="lh-label">website</span>
        <input
          v-model="form.website_url"
          type="url"
          :maxlength="MAX.short"
          placeholder="https://yourdomain.dev"
          class="lh-input"
        >
      </label>

      <div class="actions">
        <UiButton type="submit" variant="inverse" size="lg" :disabled="busy">
          {{ busy ? 'Saving…' : 'Save' }}
        </UiButton>
        <span v-if="saved" class="lh-hint" role="status">Saved.</span>
        <span v-if="problem" class="lh-error" role="alert">{{ problem }}</span>
      </div>
    </form>
  </AccountShell>
</template>

<style scoped>
.known { margin-bottom: var(--space-8); }

.form {
  display: grid;
  gap: var(--space-5);
}

.actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-4);
  margin-top: var(--space-2);
}
</style>
