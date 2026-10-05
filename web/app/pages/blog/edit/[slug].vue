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

const { find, edit, archive } = useArticles()
const { reader } = useReader()

const title = ref('')
const subtitle = ref('')
const topics = ref<string[]>([])
const body = ref('')
const pending = ref(false)
const loading = ref(true)
const errors = useFieldErrors(['title', 'subtitle', 'topics', 'body'])
const missing = ref(false)
/** Whether the article is archived, as the api last said. */
const archived = ref(false)
const archiving = ref(false)
const archiveProblem = ref('')

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
  archived.value = article.archivedAt !== null
  subtitle.value = article.subtitle
  topics.value = [...article.topics]
  body.value = article.body
  loading.value = false
})

async function save(): Promise<void> {
  if (pending.value) return

  errors.clear()
  pending.value = true

  const result = await edit(slug.value, title.value, subtitle.value, topics.value, body.value)

  pending.value = false

  if ('refused' in result) {
    errors.show(result.refused)
    return
  }

  await navigateTo(ownWritingUrl(reader.value?.username))
}

/** Archive, or bring back. Separate from save: it changes who can see the
 *  article, not what it says, and takes effect at once. */
async function toggleArchive(): Promise<void> {
  if (archiving.value) return

  archiving.value = true
  archiveProblem.value = ''
  const result = await archive(slug.value, !archived.value)
  archiving.value = false

  if ('refused' in result) {
    archiveProblem.value = result.refused.message || 'That could not be changed. Please try again.'
    return
  }

  archived.value = result.article.archivedAt !== null
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
    :error="errors.message.value || null"
    :field-errors="errors.fields.value"
    @submit="save"
  >
    <template #header>
      <nav class="font-mono text-sm text-faint">
        <NuxtLink :to="ownWritingUrl(reader?.username)" class="hover:text-ink">my writing</NuxtLink>
        <span class="mx-3 text-crumb">/</span>
        <span class="text-quiet">edit</span>
      </nav>

      <div class="mt-4 flex flex-wrap items-center gap-3">
        <span v-if="archived" class="font-mono text-xs text-quiet" role="status">
          archived · only you can see it
        </span>
        <UiButton variant="ghost" size="sm" :disabled="archiving" @click="toggleArchive">
          {{ archiving ? '…' : archived ? 'unarchive' : 'archive' }}
        </UiButton>
        <span v-if="archiveProblem" class="text-sm text-bad" role="alert">{{ archiveProblem }}</span>
      </div>
    </template>
  </BlogEditor>
</template>
