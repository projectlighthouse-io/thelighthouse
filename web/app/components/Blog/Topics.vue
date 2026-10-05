<script setup lang="ts">
/**
 * The topics an article is about, as chips.
 *
 * An article can be about Go *and* Docker, so this is a set rather than a
 * choice — `article_topics` in the database, `topics: string[]` on the wire.
 *
 * The list itself is fixed (`CATEGORIES`), so adding one is picking from a
 * menu rather than typing free text: rust refuses anything off its own list,
 * and a text field would invite a refusal on every typo.
 */
import { CATEGORIES, MAX_TOPICS } from '@/composables/UseArticles'

const props = defineProps<{ modelValue: string[] }>()
const emit = defineEmits<{ (e: 'update:modelValue', value: string[]): void }>()

const picking = ref(false)
const root = ref<HTMLElement | null>(null)

/** What is left to add, in the order `CATEGORIES` lists them. */
const available = computed(() =>
  CATEGORIES.filter(topic => !props.modelValue.includes(topic)),
)

const full = computed(() => props.modelValue.length >= MAX_TOPICS)

function add(topic: string): void {
  if (full.value || props.modelValue.includes(topic)) return

  emit('update:modelValue', [...props.modelValue, topic])
  picking.value = false
}

function remove(topic: string): void {
  emit('update:modelValue', props.modelValue.filter(it => it !== topic))
}

// A menu that only closes on its own trigger is a menu that gets left open
// behind whatever the reader clicks next.
function onDocumentClick(event: MouseEvent): void {
  if (!root.value?.contains(event.target as Node)) picking.value = false
}

onMounted(() => document.addEventListener('click', onDocumentClick))
onBeforeUnmount(() => document.removeEventListener('click', onDocumentClick))
</script>

<template>
  <div ref="root" class="relative flex flex-wrap items-center gap-2">
    <span class="sr-only">topics</span>

    <span
      v-for="topic in modelValue"
      :key="topic"
      class="inline-flex items-center gap-1.5 rounded-sm bg-paper-warm py-1 pr-1.5 pl-2.5 text-[13px] leading-5 text-ink ring-1 ring-rule-soft"
    >
      {{ topic }}
      <button
        type="button"
        :title="`remove ${topic}`"
        class="flex size-4 cursor-pointer items-center justify-center rounded-full bg-crumb text-[10px] leading-none text-page transition hover:bg-ink"
        @click="remove(topic)"
      >
        <span aria-hidden="true">✕</span>
        <span class="sr-only">remove {{ topic }}</span>
      </button>
    </span>

    <!-- Hidden once the set is full, rather than shown disabled: there is
         nothing left to pick, and a dead `+` invites a click that does
         nothing. -->
    <button
      v-if="!full && available.length"
      type="button"
      title="add a topic"
      class="flex size-7 cursor-pointer items-center justify-center rounded-sm text-base leading-none text-faint ring-1 ring-rule-soft transition hover:bg-paper-warm hover:text-ink"
      @click="picking = !picking"
    >
      <span aria-hidden="true">+</span>
      <span class="sr-only">add a topic</span>
    </button>

    <!-- Dark, like the editor's bubble menu — the two are the same kind of
         thing (a surface that floats over the page while you are working) and
         they should read as one system.
         It also fixes what the light version got wrong: `paper-warm` chips on
         a `page`-white panel is two near-identical off-whites, so the chips
         had almost no edge and the whole panel looked washed out. -->
    <div
      v-if="picking"
      class="absolute top-full left-0 z-10 mt-2 flex w-max max-w-full flex-wrap gap-1 rounded-md bg-ink p-1.5 shadow-lg"
    >
      <button
        v-for="topic in available"
        :key="topic"
        type="button"
        class="cursor-pointer rounded-sm px-2.5 py-1.5 text-[13px] leading-none text-on-ink/70 transition hover:bg-on-ink/15 hover:text-on-ink"
        @click="add(topic)"
      >
        {{ topic }}
      </button>
    </div>
  </div>
</template>
