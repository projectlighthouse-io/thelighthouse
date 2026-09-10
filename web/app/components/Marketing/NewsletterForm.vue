<script setup lang="ts">
/**
 * The subscribe form, wherever it appears.
 *
 * Posts straight to `/api/newsletter` — the api's one public write, with no
 * session and no CSRF token, because a visitor who has never signed in holds
 * neither. No nitro route in between: this is a browser action, not something
 * rendered on the server, so there is nothing for `/_api/*` to shape or cache.
 *
 * **The success wording never says whether the address was already on the
 * list.** The api does not tell this page, deliberately — see `newsletter` for
 * why: a form that distinguishes the two answers "is this person a reader of
 * yours" for anybody who cares to ask.
 */

interface Props {
  /** The footer's shape: one row, no heading, label for screen readers only. */
  compact?: boolean
}

const { compact = false } = defineProps<Props>()

const email = ref('')
const busy = ref(false)
const done = ref(false)
const problem = ref('')

/** Unique per instance: the footer and a page block can both be on screen, and
 * two inputs sharing an id makes one label point at the wrong one. */
const fieldId = useId()

async function submit() {
  if (busy.value) return

  busy.value = true
  problem.value = ''

  try {
    await $fetch('/api/newsletter', {
      method: 'POST',
      body: { email: email.value.trim() },
    })
    done.value = true
    email.value = ''
  }
  catch (error: unknown) {
    // The api's own wording when it sent one — it is written for a person and
    // is more use than anything this form could invent. A 429 carries none,
    // so that case gets the fallback.
    problem.value = (error as { data?: { error?: string } })?.data?.error
      ?? 'Could not sign you up just now. Try again in a minute.'
  }
  finally {
    busy.value = false
  }
}
</script>

<template>
  <div :class="compact ? '' : 'rounded-lg border border-rule bg-paper p-6'">
    <template v-if="!compact">
      <h2 class="font-serif text-xl text-ink">The newsletter</h2>
      <p class="mt-2 text-sm leading-relaxed text-quiet">
        New books, new projects, and what I learned building them. No more than
        one email a week, and nothing else.
      </p>
    </template>

    <!-- The form is replaced rather than hidden: leaving an empty field under
         a success message invites a second submission that does nothing. -->
    <p
      v-if="done"
      class="text-sm text-ink"
      :class="compact ? '' : 'mt-4'"
      role="status"
    >
      Thanks — you're on the list.
    </p>

    <form
      v-else
      class="flex items-center gap-2"
      :class="compact ? '' : 'mt-4'"
      @submit.prevent="submit"
    >
      <!-- Always screen-reader only: the placeholder carries it visually in
           both shapes, and a visible label would break the single row. -->
      <label :for="fieldId" class="sr-only">Email address</label>
      <input
        :id="fieldId"
        v-model="email"
        type="email"
        required
        autocomplete="email"
        placeholder="you@example.com"
        maxlength="254"
        class="min-w-0 flex-1 rounded-md border border-rule px-3 py-2.5 text-sm transition outline-none placeholder:text-faint focus:border-ink"
      >
      <button
        type="submit"
        :disabled="busy"
        class="rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover disabled:opacity-50"
      >
        {{ busy ? 'Signing up…' : 'Subscribe' }}
      </button>
    </form>

    <p
      v-if="problem"
      class="mt-2 text-sm text-red-700"
      role="alert"
    >
      {{ problem }}
    </p>
  </div>
</template>
