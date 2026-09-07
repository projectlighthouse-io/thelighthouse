<script setup lang="ts">
// `chrome: false` drops the footer and wordmark — see `layouts/Default`.
// Without it the document is taller than the viewport and the action bar
// scrolls away.
definePageMeta({ middleware: 'auth', chrome: false })

useSeo({
  title: 'Write — projectlighthouse',
  description: 'Write an article.',
  noindex: true,
})

const { create } = useArticles()

const title = ref('')
const subtitle = ref('')
const topics = ref<string[]>([])
const body = ref('')
const pending = ref(false)
const error = ref<string | null>(null)

async function publish(): Promise<void> {
  pending.value = true
  error.value = null

  const result = await create(title.value, subtitle.value, topics.value, body.value)

  pending.value = false

  if ('error' in result) {
    error.value = result.error
    return
  }

  // The slug rust minted, not one guessed here.
  await navigateTo(`/blog/${result.article.slug}`)
}
</script>

<template>
  <BlogEditor
    v-model:title="title"
    v-model:subtitle="subtitle"
    v-model:topics="topics"
    v-model:body="body"
    action="publish"
    caption="Write it down while it is still annoying you."
    :pending="pending"
    :error="error"
    @submit="publish"
  />
</template>
