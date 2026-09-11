<script setup lang="ts">
// `chrome: false` drops the footer and wordmark — see `layouts/Default`.
// Without it the document is taller than the viewport and the action bar
// scrolls away.
definePageMeta({ middleware: 'auth', chrome: false })

useSeo({
  title: 'Edit — projectlighthouse',
  description: 'Edit your article.',
  noindex: true,
})

const route = useRoute()
const slug = computed<string>(() => String(route.params.slug))

const { find, edit } = useArticles()
const { reader } = useReader()

const title = ref('')
const subtitle = ref('')
const topics = ref<string[]>([])
const body = ref('')
const pending = ref(false)
const loading = ref(true)
const error = ref<string | null>(null)
const missing = ref(false)

// `mine` is the only endpoint that answers about an article of the reader's
// own — including one that has been taken down, which is exactly the article
// somebody most wants to edit.
onMounted(async () => {
  const article = await find(slug.value)

  if (!article) {
    missing.value = true
    loading.value = false
    return
  }

  title.value = article.title
  subtitle.value = article.subtitle
  topics.value = [...article.topics]
  body.value = article.body
  loading.value = false
})

async function save(): Promise<void> {
  pending.value = true
  error.value = null

  const result = await edit(slug.value, title.value, subtitle.value, topics.value, body.value)

  pending.value = false

  if ('error' in result) {
    error.value = result.error
    return
  }

  await navigateTo(ownWritingUrl(reader.value?.username))
}
</script>

<template>
  <div v-if="loading" class="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:px-8">
    <p class="text-mono-body">loading…</p>
  </div>

  <div v-else-if="missing" class="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:px-8">
    <h1 class="font-serif text-3xl tracking-tight text-ink">Not found</h1>
    <p class="text-mono-body mt-2">
      no article of yours at that address.
      <NuxtLink :to="ownWritingUrl(reader?.username)" class="text-link hover:text-link-hover">your writing</NuxtLink>
    </p>
  </div>

  <BlogEditor
    v-else
    v-model:title="title"
    v-model:subtitle="subtitle"
    v-model:topics="topics"
    v-model:body="body"
    action="save"
    caption="Second drafts are where the thinking shows."
    :pending="pending"
    :error="error"
    @submit="save"
  >
    <template #header>
      <nav class="font-mono text-sm text-faint">
        <NuxtLink :to="ownWritingUrl(reader?.username)" class="hover:text-ink">my writing</NuxtLink>
        <span class="mx-3 text-crumb">/</span>
        <span class="text-quiet">edit</span>
      </nav>
    </template>
  </BlogEditor>
</template>
