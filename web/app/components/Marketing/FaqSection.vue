<script setup lang="ts">
import { faqs } from '@/data/Faqs'


const openIndex = ref<number | null>(null)

const toggle = (index: number): void => {
  openIndex.value = openIndex.value === index ? null : index
}
</script>

<template>
  <section id="faq" class="py-24">
    <div class="mx-auto max-w-3xl px-2 sm:px-6 lg:px-8">
      <div class="mb-12 text-center">
        <h2 class="mb-4 text-3xl font-semibold text-ink">Frequently Asked Questions</h2>
        <p class="text-lg text-quiet">The why behind the what</p>
      </div>

      <div class="space-y-4">
        <div
          v-for="(faq, index) in faqs"
          :key="index"
          class="border-pencil-solid-black overflow-hidden rounded-lg bg-panel transition-all"
          :class="{ 'shadow-sm': openIndex === index }"
        >
          <button
            type="button"
            class="flex w-full cursor-pointer items-center justify-between px-6 py-5 text-left"
            :aria-expanded="openIndex === index"
            @click="toggle(index)"
          >
            <span class="text-lg font-medium text-ink">{{ faq.question }}</span>
            <svg
              class="size-5 shrink-0 text-quiet transition-transform duration-200"
              :class="{ 'rotate-180': openIndex === index }"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              aria-hidden="true"
            >
              <path d="m6 9 6 6 6-6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>

          <div
            class="grid transition-all duration-200"
            :class="openIndex === index ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
          >
            <div class="overflow-hidden">
              <div
                class="px-6 py-5 transition-opacity duration-700 ease-in-out"
                :class="openIndex === index ? 'opacity-100 delay-150' : 'opacity-0'"
              >
                <p class="leading-relaxed whitespace-pre-line text-quiet">{{ faq.answer }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
