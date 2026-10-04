<script setup lang="ts">
/**
 * The two ways in. Plain links to the api's OAuth start, so the browser does a
 * full navigation — the provider's redirect has to happen at the top level.
 * `redirect` rides along so the reader lands back where they started.
 */
const props = defineProps<{
  redirect?: string
}>()

const query = computed<string>(() =>
  props.redirect ? `?redirect=${encodeURIComponent(props.redirect)}` : '',
)
</script>

<template>
  <div class="oauth">
    <UiButton variant="inverse" size="lg" block external :to="`/api/auth/github${query}`">
      Continue with GitHub →
    </UiButton>
    <UiButton variant="ghost" size="lg" block external :to="`/api/auth/google${query}`">
      Continue with Google →
    </UiButton>
  </div>
</template>

<style scoped>
.oauth {
  display: grid;
  gap: var(--space-3);
}
</style>
