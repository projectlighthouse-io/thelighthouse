<script setup lang="ts">
import { csrfHeader } from '@/composables/UseReader'

definePageMeta({ layout: 'settings', middleware: 'auth' })

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
const problem = ref('')

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
    // The api's refusal wording, when it sent one — it is written for a person
    // and is more use than anything this page could invent.
    problem.value = (error as { data?: { error?: string } })?.data?.error
      ?? 'Could not create the token.'
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
  <div>
    <h2 class="font-serif text-2xl text-ink">API tokens</h2>
    <p class="text-mono-body mt-2">
      luxctl authenticates with a bearer token. Create one, copy it once, keep it somewhere safe.
    </p>

    <p v-if="problem" class="mt-4 text-sm text-red-700">{{ problem }}</p>

    <!-- Shown once. The api keeps only a hash, so this cannot be recovered. -->
    <div v-if="justMinted" class="border-pencil-light mt-8 rounded-md p-4">
      <p class="text-sm font-medium text-ink">
        Copy “{{ justMinted.name }}” now — it is not shown again.
      </p>
      <div class="mt-3 flex items-center gap-2">
        <input
          :value="justMinted.token_string"
          readonly
          class="border-pencil-light flex-1 rounded-md px-3 py-2 font-mono text-xs"
          @focus="(e) => (e.target as HTMLInputElement).select()"
        >
        <button
          type="button"
          class="rounded-md bg-ink px-4 py-2 text-sm font-medium text-on-ink"
          @click="copy"
        >
          {{ copied ? 'Copied' : 'Copy' }}
        </button>
      </div>
      <p class="mt-2 text-xs text-quiet">
        Then run <code>lux auth --token &lt;token&gt;</code>.
      </p>
    </div>

    <form class="mt-8 flex items-center gap-2" @submit.prevent="create">
      <input
        v-model="name"
        placeholder="What is it for? e.g. my laptop"
        maxlength="60"
        class="border-pencil-light flex-1 rounded-md px-3 py-2 text-sm"
      >
      <button
        type="submit"
        :disabled="busy || !name.trim()"
        class="rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover disabled:opacity-50"
      >
        Create token
      </button>
    </form>

    <div v-if="tokens.length" class="mt-8">
      <div
        v-for="token in tokens"
        :key="token.id"
        class="border-pencil-light flex items-center justify-between border-b py-3"
      >
        <div>
          <p class="text-sm font-medium text-ink">{{ token.name }}</p>
          <p class="text-xs text-quiet">
            created {{ when(token.created_at) }} · last used {{ when(token.last_used_at) }}
          </p>
        </div>
        <button
          type="button"
          :disabled="busy"
          class="text-sm text-quiet transition hover:text-red-700 disabled:opacity-50"
          @click="revoke(token)"
        >
          Revoke
        </button>
      </div>
    </div>

    <div v-else class="border-pencil-light mt-8 rounded-md p-6 text-center">
      <p class="text-sm text-quiet">No tokens yet.</p>
    </div>

    <p class="mt-6 text-sm text-quiet">
      A token is shown once at creation and never again. If you lose it, revoke it and make another.
    </p>
  </div>
</template>
