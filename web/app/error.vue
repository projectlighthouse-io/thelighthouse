<script setup lang="ts">
import type { NuxtError } from '#app'

/**
 * Copy a page can attach to its own failure.
 *
 * Every route ends up here, so the defaults have to be true of all of them —
 * which makes them vague. A page that knows more says so through
 * `createError({ data })`, and the two sentences below are replaced by ones
 * about the thing the reader was actually looking at.
 */
interface ErrorCopy {
  headline?: string
  detail?: string
  /** Show the retry. For a failure that is worth trying again — a 5xx, not a 404. */
  retry?: boolean
}

const props = defineProps<{ error: NuxtError }>()

const isMissing = computed<boolean>(() => props.error.statusCode === 404)

/**
 * `data` is whatever `createError` was handed, and it comes back over the
 * payload as it went in — an object from a page that set one, a string from
 * anything that put json in there, and undefined for the rest. None of those
 * is worth an error page failing on, so anything unreadable falls through to
 * the generic copy.
 */
const copy = computed<ErrorCopy>(() => {
  const raw: unknown = props.error.data

  if (typeof raw === 'string') {
    try {
      return JSON.parse(raw) as ErrorCopy
    }
    catch {
      return {}
    }
  }

  return typeof raw === 'object' && raw !== null ? raw as ErrorCopy : {}
})

const headline = computed<string>(
  () => copy.value.headline ?? (isMissing.value ? 'nothing here' : 'something broke'),
)

const detail = computed<string>(
  () =>
    copy.value.detail
    ?? (isMissing.value
      ? 'That page moved, or never existed. The shelf is still where you left it.'
      : 'That is on us, not you. Try again in a moment.'),
)

// A 404 does not become a page by being asked for twice. Anything else might,
// so the button is offered by default there and a page can force it either way.
const canRetry = computed<boolean>(() => copy.value.retry ?? !isMissing.value)

/**
 * A full reload rather than `clearError({ redirect })`.
 *
 * The failure was the route's own data, and clearing the error re-renders the
 * same app state — including the payload that failed. Reloading asks the
 * server again, which is the thing the reader is retrying.
 */
const retry = (): void => reloadNuxtApp({ persistState: false })
</script>

<template>
  <NuxtLayout>
    <div class="mx-auto flex max-w-2xl flex-col items-center px-4 py-32 text-center sm:px-6">
      <p class="font-mono text-xs tracking-[0.2em] uppercase text-teal">
        {{ error.statusCode }}
      </p>

      <h1 class="font-fredericka mt-4 text-5xl text-ink sm:text-6xl">
        {{ headline }}
      </h1>

      <p class="mt-6 font-serif text-lg leading-relaxed text-quiet">
        {{ detail }}
      </p>

      <div class="mt-10 flex flex-col gap-3 sm:flex-row">
        <button
          v-if="canRetry"
          type="button"
          class="rounded-md bg-ink px-5 py-3 text-base font-medium text-on-ink transition hover:bg-ink-hover"
          @click="retry"
        >
          Try again
        </button>
        <NuxtLink
          to="/books"
          class="rounded-md px-5 py-3 text-base font-medium transition"
          :class="canRetry
            ? 'border border-stroke bg-panel text-ink hover:bg-paper-warm'
            : 'bg-ink text-on-ink hover:bg-ink-hover'"
        >
          Browse the shelf
        </NuxtLink>
        <NuxtLink
          to="/"
          class="rounded-md border border-stroke bg-panel px-5 py-3 text-base font-medium text-ink transition hover:bg-paper-warm"
        >
          Go home
        </NuxtLink>
      </div>
    </div>
  </NuxtLayout>
</template>
