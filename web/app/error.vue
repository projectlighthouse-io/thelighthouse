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
  () => copy.value.headline ?? (isMissing.value ? 'Where did this page go?' : 'What just broke?'),
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
    <div class="lh-text lh-gap">
      <UiLogo variant="tile" :size="32" label="Project Lighthouse" class="error-logo" />
      <EmptyState
        as="h1"
        :eyebrow="`${error.statusCode} · ${isMissing ? 'not found' : 'error'}`"
        :heading="headline"
        :detail="detail"
        :action="canRetry ? undefined : { label: 'Browse the shelf →', to: '/books' }"
      >
        <template v-if="canRetry" #secondary>
          <UiButton variant="inverse" size="lg" @click="retry">Try again</UiButton>
        </template>
      </EmptyState>
    </div>
  </NuxtLayout>
</template>

<style scoped>
.error-logo { margin-bottom: var(--space-4); }
</style>
