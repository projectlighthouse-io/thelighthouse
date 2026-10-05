<script setup lang="ts">
import { csrfHeader } from '@/composables/UseReader'

definePageMeta({ layout: 'default', middleware: 'auth' })

useSeo({
  title: 'API tokens — projectlighthouse',
  description: 'API tokens for the luxctl CLI.',
  noindex: true,
})

/** One token as the api lists it. Never the secret — see `settings::view`. */
interface Token {
  id: number
  name: string
  last_used_at: string | null
  created_at: string | null
}

/** What creating one answers: the row, plus the single sight of the secret. */
interface Minted extends Token {
  token_string: string
}

const tokens = ref<Token[]>([])
const name = ref('')
const busy = ref(false)
/** Loading and revoking, which have no field to point at. */
const problem = ref('')
/** Creating one: the name field, and anything else the api says. */
const errors = useFieldErrors(['name'])

/**
 * The one time the secret exists outside the api's response.
 *
 * Held in a ref and never written anywhere else — not to storage, not to the
 * url — because the api cannot show it again. Cleared when another is minted,
 * so two secrets are never on screen claiming to be current.
 */
const justMinted = ref<Minted | null>(null)
const copied = ref(false)

/** No `useAsyncData`: this is behind a session, so there is nothing to render
 * on the server and nothing a cache may hold. */
async function load() {
  try {
    tokens.value = await $fetch<Token[]>('/api/settings/tokens')
  }
  catch {
    problem.value = 'Could not load your tokens.'
  }
}

onMounted(load)

async function create() {
  if (busy.value || !name.value.trim()) return

  busy.value = true
  problem.value = ''
  errors.clear()

  try {
    justMinted.value = await $fetch<Minted>('/api/settings/tokens', {
      method: 'POST',
      headers: csrfHeader(),
      body: { name: name.value },
    })
    copied.value = false
    name.value = ''
    await load()
  }
  catch (error: unknown) {
    errors.take(error, 'Could not create the token.')
  }
  finally {
    busy.value = false
  }
}

async function revoke(token: Token) {
  if (busy.value) return
  if (!confirm(`Revoke "${token.name}"? Anything using it stops working.`)) return

  busy.value = true
  problem.value = ''

  try {
    await $fetch(`/api/settings/tokens/${token.id}`, {
      method: 'DELETE',
      headers: csrfHeader(),
    })
    // The secret on screen may be the one just revoked; it is no use either way.
    if (justMinted.value?.id === token.id) justMinted.value = null
    await load()
  }
  catch {
    problem.value = 'Could not revoke the token.'
  }
  finally {
    busy.value = false
  }
}

async function copy() {
  if (!justMinted.value) return

  try {
    await navigator.clipboard.writeText(justMinted.value.token_string)
    copied.value = true
  }
  catch {
    // Clipboard access can be refused; the field is selectable either way.
    copied.value = false
  }
}

/** "never" is a finding about a token, not a missing value. */
const when = (at: string | null): string =>
  at ? new Date(at).toLocaleDateString() : 'never'
</script>

<template>
  <AccountShell
    title="API tokens"
    sub="luxctl authenticates with a bearer token. Create one, copy it once, keep it somewhere safe."
  >
    <div class="lh-narrow">
      <p v-if="problem" class="lh-error problem" role="alert">{{ problem }}</p>

      <!-- Shown once. The api keeps only a hash, so this cannot be recovered. -->
      <div v-if="justMinted" class="lh-card minted">
        <p class="lh-eyebrow">copy “{{ justMinted.name }}” now — it is not shown again</p>
        <div class="row">
          <input
            :value="justMinted.token_string"
            readonly
            class="lh-input token"
            aria-label="your new token"
            @focus="(e) => (e.target as HTMLInputElement).select()"
          >
          <UiButton variant="inverse" size="md" @click="copy">{{ copied ? 'Copied' : 'Copy' }}</UiButton>
        </div>
        <p class="lh-hint">Then run <code>luxctl auth --token &lt;token&gt;</code>.</p>
      </div>

      <form class="row create" @submit.prevent="create">
        <label class="lh-sr" for="token-name">what the token is for</label>
        <input
          id="token-name"
          v-model="name"
          placeholder="What is it for? e.g. my laptop"
          maxlength="60"
          class="lh-input"
          :aria-invalid="!!errors.fields.value.name"
        >
        <UiButton type="submit" variant="inverse" size="md" :disabled="busy || !name.trim()">
          Create token
        </UiButton>
      </form>
      <p v-if="errors.fields.value.name" class="lh-error" role="alert">{{ errors.fields.value.name }}</p>
      <p v-if="errors.message.value" class="lh-error" role="alert">{{ errors.message.value }}</p>

      <ul v-if="tokens.length" class="tokens">
        <li v-for="token in tokens" :key="token.id" class="lh-card item">
          <div class="who">
            <span class="lh-h3">{{ token.name }}</span>
            <span class="lh-mono lh-muted">
              created {{ when(token.created_at) }} · last used {{ when(token.last_used_at) }}
            </span>
          </div>
          <UiButton variant="ghost" size="sm" :disabled="busy" @click="revoke(token)">Revoke</UiButton>
        </li>
      </ul>

      <p v-else class="lh-card lh-sub none">No tokens yet.</p>

      <p class="lh-hint after">
        A token is shown once at creation and never again. If you lose it, revoke it and make another.
      </p>
    </div>
  </AccountShell>
</template>

<style scoped>
.problem { margin-bottom: var(--space-4); }

.minted {
  display: grid;
  gap: var(--space-3);
  margin-bottom: var(--space-8);
}

.row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.row .lh-input { flex: 1; min-width: 0; }

.token { font: var(--text-code); }

code { font: var(--text-code); color: var(--ink); }

.tokens {
  margin: var(--space-8) 0 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: var(--space-2);
}

.item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
}

.who {
  display: grid;
  gap: var(--space-1);
  min-width: 0;
}

.none { margin-top: var(--space-8); }

.after { margin-top: var(--space-6); }

@media (max-width: 560px) {
  .row { flex-direction: column; align-items: stretch; }
}
</style>
