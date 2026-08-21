<script setup lang="ts">
import type { HorizonBook } from '@/types/Content'

withDefaults(
  defineProps<{
    books: HorizonBook[]
    forthcomingLabel?: string
    requestTopicHref?: string
  }>(),
  {
    forthcomingLabel: '~2026-27',
    requestTopicHref: 'mailto:hello@projectlighthouse.io?subject=Horizon%20topic%20request',
  },
)

const indexLabel = (i: number): string => String(i + 1).padStart(2, '0')
</script>

<template>
  <section class="rounded-md bg-panel px-6 py-10 sm:px-10 sm:py-12 lg:px-14 lg:py-14">
    <div class="grid gap-10 lg:grid-cols-[minmax(0,22rem)_1fr] lg:gap-16">
      <aside class="lg:sticky lg:top-24 lg:self-start">
        <div
          class="mb-6 flex items-center gap-2 font-mono text-xs tracking-wide uppercase text-quiet"
        >
          <span class="inline-block size-2 rounded-full bg-live" aria-hidden="true" />
          <span>forthcoming · {{ forthcomingLabel }}</span>
        </div>

        <h2
          class="font-editorial mt-6 text-ink sm:mt-10"
          style="
            font-weight: 500;
            font-size: clamp(2.5rem, 5vw, 4rem);
            line-height: 1;
            letter-spacing: -0.01em;
          "
        >
          On the<br>
          <em class="italic text-teal font-medium">Horizon</em>
        </h2>

        <svg
          class="mt-4 h-4 w-56 text-ink"
          viewBox="0 0 240 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
          aria-hidden="true"
        >
          <path d="M2 9 C 30 2, 60 14, 90 8 S 150 2, 180 10 S 230 6, 238 9" />
          <circle cx="60" cy="11" r="1.2" class="fill-teal" stroke="none" />
          <circle cx="130" cy="6" r="1.2" class="fill-teal" stroke="none" />
          <circle cx="200" cy="9" r="1.2" class="fill-teal" stroke="none" />
        </svg>

        <p class="mt-6 font-serif text-base leading-relaxed text-ink sm:text-lg">
          the books above teach you how programs work, how memory works, how networks work. these
          are where it all comes together —
          <mark class="bg-marker px-1">containers, distributed systems, databases, protocols</mark>.
          each one builds on what you already learned, or fills one of the gaps.
        </p>
      </aside>

      <ol class="max-h-[32rem] space-y-2 overflow-y-auto pr-2 [scrollbar-width:thin]">
        <li
          v-for="(book, i) in books"
          :key="book.title"
          class="group py-6 transition-colors hover:bg-paper"
        >
          <div class="flex items-start gap-4 sm:gap-6">
            <span class="pt-1.5 font-mono text-xs text-whisper">{{ indexLabel(i) }}</span>

            <div class="min-w-0 flex-1">
              <div class="flex items-baseline gap-2">
                <h3 class="font-editorial text-xl font-semibold text-ink sm:text-2xl">
                  {{ book.title }}
                </h3>
                <span class="font-editorial text-xl text-whisper sm:text-2xl" aria-hidden="true">—</span>
              </div>
              <p class="mt-2 font-serif text-base leading-relaxed text-mono-ink">
                {{ book.description }}
              </p>
            </div>
          </div>
        </li>
      </ol>
    </div>

    <div class="mt-10 pt-6">
      <div class="flex flex-col items-start gap-4 sm:flex-row sm:items-center sm:justify-between">
        <p class="font-mono text-xs text-faint">
          * subject to change based on priority &amp; ask from the learners
        </p>
        <p class="font-handwritten text-lg text-ink">
          order isn't final — your votes shape it.
        </p>
        <a
          :href="requestTopicHref"
          class="border-b border-stroke font-mono text-sm text-ink hover:border-teal hover:text-teal"
        >
          request a topic →
        </a>
      </div>
    </div>
  </section>
</template>
