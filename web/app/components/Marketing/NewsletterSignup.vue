<script setup lang="ts">
/**
 * The short subscribe form: a line of pitch, an address, a button. For the
 * places a reader passes on the way somewhere else — the end of the home page,
 * the end of a lesson. `/newsletter` has the envelope; this is its pocket size.
 *
 * Posts to `/api/newsletter` exactly as the envelope does, and like it never
 * says whether the address was already on the list.
 */
withDefaults(
  defineProps<{
    title?: string
    description?: string
  }>(),
  {
    title: 'stay in the loop',
    description: 'get notified about new labs, books or pages, and deep-dive articles. no spam, unsubscribe anytime.',
  },
)

const email = ref('')
const busy = ref(false)
const sent = ref(false)
const errors = useFieldErrors(['email'])

async function subscribe(): Promise<void> {
  if (busy.value) return

  busy.value = true
  errors.clear()

  try {
    await $fetch('/api/newsletter', {
      method: 'POST',
      body: { email: email.value.trim() },
    })
    sent.value = true
  }
  catch (error: unknown) {
    // The api's own wording when it sent one; a 429 carries none.
    errors.take(error, 'Could not subscribe just now. Try again in a minute.')
  }
  finally {
    busy.value = false
  }
}
</script>

<template>
  <section class="signup">
    <div class="words">
      <h2 class="title">{{ title }}</h2>
      <p class="lh-muted">{{ description }}</p>
    </div>

    <p v-if="sent" class="done" role="status">You're on the list. Watch for the next letter.</p>

    <!-- `novalidate`: the api judges the address, and its answer is drawn
         under the field rather than in the browser's own bubble. -->
    <form v-else class="row" novalidate @submit.prevent="subscribe">
      <input
        v-model="email"
        type="email"
        placeholder="you@example.com"
        maxlength="254"
        autocomplete="email"
        aria-label="email address"
        :aria-invalid="!!errors.fields.value.email"
        :disabled="busy"
      >
      <UiButton type="submit" variant="inverse" class="submit" :disabled="busy">
        {{ busy ? 'subscribing…' : 'subscribe' }}
      </UiButton>
    </form>

    <p v-if="errors.fields.value.email" class="problem" role="alert">{{ errors.fields.value.email }}</p>
    <p v-if="errors.message.value" class="problem" role="alert">{{ errors.message.value }}</p>
  </section>
</template>

<style scoped>
.signup {
  display: grid;
  gap: var(--space-4);
  padding: var(--space-8);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-raised);
}

.words {
  display: grid;
  gap: var(--space-2);
}

.title {
  margin: 0;
  font: 400 26px/32px var(--font-serif);
  color: var(--ink);
}

.words p { margin: 0; font: var(--text-body-sm); }

.row {
  display: flex;
  gap: var(--space-3);
}

input {
  flex: 1;
  min-width: 0;
  height: 40px;
  padding: 0 var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-page);
  color: var(--ink);
  font: var(--text-body-sm);
}

.submit { height: 40px; border-radius: var(--radius-md); }

input:focus { outline: none; border-color: var(--ink); }
input:disabled { opacity: 0.5; }

.done,
.problem {
  margin: 0;
  font: var(--text-body-sm);
}

.done { color: var(--ink); }
.problem { color: var(--ink-secondary); }

@media (max-width: 560px) {
  .signup { padding: var(--space-6); }
  .row { flex-direction: column; }
  /* flex: 1 is a zero basis, which in a column squeezes the height. */
  input { flex: none; }
}
</style>
