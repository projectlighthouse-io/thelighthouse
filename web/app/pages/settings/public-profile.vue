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

const fields = [
  'tagline', 'bio', 'company', 'education', 'location', 'linkedin_url', 'x_url', 'website_url',
] as const

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
/** Loading, which has no field to point at. */
const problem = ref('')
const errors = useFieldErrors(fields)
const saved = ref(false)
/**
 * Whether the stored profile reached the form. Save stays off until it has:
 * the api writes every field it is sent, so saving a form that never loaded
 * would send eight blanks and wipe the profile.
 */
const loaded = ref(false)

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
    loaded.value = true
  }
  catch {
    problem.value = 'Could not load your profile. Reload the page before editing it.'
  }
})

async function save() {
  if (busy.value || !loaded.value) return

  busy.value = true
  errors.clear()
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
    errors.take(error, 'Could not save your profile.')
  }
  finally {
    busy.value = false
  }
}
</script>

<template>
  <AccountShell title="Public profile" sub="What other learners see.">
    <div class="lh-narrow">
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

      <!-- `novalidate`: the api is the judge, and its refusals are drawn beside
           each field; the browser's own bubbles would say something else, in
           another place, for only some of them. -->
      <form class="form" novalidate @submit.prevent="save">
        <label class="lh-field">
          <span class="lh-label">tagline</span>
          <input
            v-model="form.tagline"
            type="text"
            :maxlength="MAX.tagline"
            placeholder="One line about you"
            class="lh-input"            :aria-invalid="!!errors.fields.value.tagline"
          >
          <span v-if="errors.fields.value.tagline" class="lh-error" role="alert">{{ errors.fields.value.tagline }}</span>
        </label>
        <label class="lh-field">
          <span class="lh-label">bio</span>
          <textarea
            v-model="form.bio"
            rows="3"
            :maxlength="MAX.bio"
            class="lh-input"
            :aria-invalid="!!errors.fields.value.bio"
          />
          <span v-if="errors.fields.value.bio" class="lh-error" role="alert">{{ errors.fields.value.bio }}</span>
        </label>
        <label class="lh-field">
          <span class="lh-label">company</span>
          <input
            v-model="form.company"
            type="text"
            :maxlength="MAX.short"
            class="lh-input"            :aria-invalid="!!errors.fields.value.company"
          >
          <span v-if="errors.fields.value.company" class="lh-error" role="alert">{{ errors.fields.value.company }}</span>
        </label>
        <label class="lh-field">
          <span class="lh-label">education</span>
          <input
            v-model="form.education"
            type="text"
            :maxlength="MAX.short"
            class="lh-input"            :aria-invalid="!!errors.fields.value.education"
          >
          <span v-if="errors.fields.value.education" class="lh-error" role="alert">{{ errors.fields.value.education }}</span>
        </label>
        <label class="lh-field">
          <span class="lh-label">location</span>
          <input
            v-model="form.location"
            type="text"
            :maxlength="MAX.location"
            placeholder="City, country"
            class="lh-input"            :aria-invalid="!!errors.fields.value.location"
          >
          <span v-if="errors.fields.value.location" class="lh-error" role="alert">{{ errors.fields.value.location }}</span>
        </label>
        <label class="lh-field">
          <span class="lh-label">linkedin</span>
          <input
            v-model="form.linkedin_url"
            type="url"
            :maxlength="MAX.short"
            placeholder="https://linkedin.com/in/your-handle"
            class="lh-input"            :aria-invalid="!!errors.fields.value.linkedin_url"
          >
          <span v-if="errors.fields.value.linkedin_url" class="lh-error" role="alert">{{ errors.fields.value.linkedin_url }}</span>
        </label>
        <label class="lh-field">
          <span class="lh-label">x</span>
          <input
            v-model="form.x_url"
            type="url"
            :maxlength="MAX.short"
            placeholder="https://x.com/your-handle"
            class="lh-input"            :aria-invalid="!!errors.fields.value.x_url"
          >
          <span v-if="errors.fields.value.x_url" class="lh-error" role="alert">{{ errors.fields.value.x_url }}</span>
        </label>
        <label class="lh-field">
          <span class="lh-label">website</span>
          <input
            v-model="form.website_url"
            type="url"
            :maxlength="MAX.short"
            placeholder="https://yourdomain.dev"
            class="lh-input"            :aria-invalid="!!errors.fields.value.website_url"
          >
          <span v-if="errors.fields.value.website_url" class="lh-error" role="alert">{{ errors.fields.value.website_url }}</span>
        </label>

        <div class="actions">
          <UiButton type="submit" variant="inverse" size="lg" :disabled="busy || !loaded">
            {{ busy ? 'Saving…' : 'Save' }}
          </UiButton>
          <span v-if="saved" class="lh-hint" role="status">Saved.</span>
          <span v-if="problem" class="lh-error" role="alert">{{ problem }}</span>
          <span v-if="errors.message.value" class="lh-error" role="alert">{{ errors.message.value }}</span>
        </div>
      </form>
    </div>
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
