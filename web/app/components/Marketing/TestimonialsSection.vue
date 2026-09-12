<script setup lang="ts">
import type { Testimonial } from '@/types/Content'

// ponytail: not shuffled. the laravel version randomises on mount, which under
// SSR renders one order on the server and another on the client — a hydration
// mismatch for zero benefit.
const props = withDefaults(
  defineProps<{
    heading?: string
    testimonials?: Testimonial[]
  }>(),
  {
    heading: 'What readers say',
    testimonials: () => [
      {
        quote:
          'Really appreciate that you go beyond just syntax and explain things with real-world scenarios and practical implementations.',
      },
      {
        quote:
          'I read the OS material — there are very few resources that cover this in such depth.',
      },
      {
        quote: 'Great tutorial, I really like that you give us exercises at the end of the tutorial.',
      },
      {
        quote:
          'Jam-packed article, thank you! Learned about pointer dereference, nil pointer, panic, recover, and Go’s simple error handling philosophy all in one place.',
      },
      {
        quote:
          'In a reality where our reasoning is constantly clouded, I found clarity in your article. I have been following Project Lighthouse and found it amazing. Particularly for me, because reading is more effective for knowledge retention. All the very best.',
      },
      {
        quote:
          'I had no idea how memory was actually handled at a low level, especially cache lines and spatial locality — thanks to Project Lighthouse for the great write-up.',
      },
      {
        quote:
          'I wanted to read Alex Edwards’ Let’s Go and Let’s Go Further, but they were paid and I never got to read them. Today, seeing Project Lighthouse’s books, that regret is gone. Thank you for this incredible contribution — in one word, an extraordinary learning resource.',
      },
      { quote: 'Project Lighthouse resource is amazing.' },
      {
        quote:
          'The website (projectlighthouse.io) claims it will teach you fundamentals, but it gives you a lot more.',
      },
    ],
  },
)

/**
 * The layout, from a flat list.
 *
 * The first quote is the featured one — the caller controls which by ordering
 * the list, rather than a second prop that can disagree with it. The rest are
 * dealt into four columns so the grid below can place them the way the design
 * does: two columns that run the full height either side, two that sit on the
 * top row.
 */
const featured = computed<Testimonial | undefined>(() => props.testimonials[0])

const columns = computed<Testimonial[][]>(() => {
  const rest = props.testimonials.slice(1)
  const out: Testimonial[][] = [[], [], [], []]

  // Dealt round robin, not sliced: sliced columns leave the last one short
  // whenever the count does not divide by four, and the ragged column is
  // always the same one.
  rest.forEach((t, i) => out[i % out.length]?.push(t))

  return out
})
</script>

<template>
  <section class="py-8 sm:py-12 lg:py-16">
    <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
      <!-- The page's own heading convention — the serif the books and
           projects bands use. -->
      <div class="mx-auto max-w-2xl text-center">
        <h2 class="font-serif text-4xl text-ink sm:text-5xl">{{ heading }}</h2>
      </div>

      <!--
        No author, avatar or company logo anywhere: these quotes came in without
        them. A card whose footer is a grey circle and a placeholder name reads
        as invented, which is the one thing a testimonial cannot afford.

        `border-pencil` rather than the design's hairline ring — a drawn edge and
        a 1px ring on the same card fight each other. `-solid-black` is the wrong
        variant here: it draws only top and bottom rules and overhangs the box by
        10px each side, which is ink outside the card in a grid.
      -->
      <div
        class="mx-auto mt-10 grid max-w-2xl grid-cols-1 grid-rows-1 gap-6 sm:mt-14 sm:grid-cols-2 lg:mt-16 xl:mx-0 xl:max-w-none xl:grid-flow-col xl:grid-cols-4"
      >
        <figure
          v-if="featured"
          class="border-pencil rounded-lg bg-panel shadow-lg sm:col-span-2 xl:col-start-2 xl:row-end-1"
        >
          <blockquote
            class="p-6 font-serif text-lg leading-relaxed text-ink sm:p-12 sm:text-xl"
          >
            <p>&ldquo;{{ featured.quote }}&rdquo;</p>
          </blockquote>
        </figure>

        <!--
          `xl:contents` from `lg` up: the wrapper stops being a box and its
          columns become grid items of the grid above, which is what lets two of
          them span both rows while the other two sit on the first.
        -->
        <div
          v-for="(column, columnIdx) in columns"
          :key="columnIdx"
          class="space-y-6 xl:contents xl:space-y-0"
        >
          <div
            :class="[
              columnIdx === 0 || columnIdx === columns.length - 1
                ? 'xl:row-span-2'
                : 'xl:row-start-1',
              'space-y-6',
            ]"
          >
            <figure
              v-for="(t, i) in column"
              :key="i"
              class="border-pencil rounded-lg bg-panel p-6 shadow-lg"
            >
              <blockquote class="font-serif text-base leading-relaxed text-ink">
                <p>&ldquo;{{ t.quote }}&rdquo;</p>
              </blockquote>
            </figure>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
