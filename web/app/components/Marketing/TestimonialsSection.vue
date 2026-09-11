<script setup lang="ts">
import type { Testimonial } from '@/types/Content'

// ponytail: not shuffled. the laravel version randomises on mount, which under
// SSR renders one order on the server and another on the client — a hydration
// mismatch for zero benefit.
withDefaults(
  defineProps<{
    eyebrow?: string
    testimonials?: Testimonial[]
  }>(),
  {
    eyebrow: 'few words from folks on the voyage',
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
</script>

<template>
  <section class="py-8 sm:py-12 lg:py-16">
    <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
      <div class="mx-auto max-w-2xl text-center">
        <h2
          class="font-handwritten text-3xl text-ink sm:text-4xl lg:text-5xl font-medium tracking-handwritten"
        >
          {{ eyebrow }}
        </h2>
      </div>

      <div class="mx-auto mt-10 flow-root max-w-2xl sm:mt-14 lg:mx-0 lg:mt-16 lg:max-w-none">
        <div class="-mt-6 sm:-mx-3 sm:columns-2 sm:gap-0 sm:text-[0] lg:columns-3">
          <div
            v-for="(t, i) in testimonials"
            :key="i"
            class="pt-6 sm:inline-block sm:w-full sm:px-3 sm:pt-6 lg:px-4 lg:pt-8"
          >
            <figure
              class="border-pencil-solid-black rounded-md bg-panel p-5 text-sm/6 shadow-lg sm:p-6 sm:text-[0.9rem]/6 lg:p-8 lg:text-[0.95rem]/6 lg:shadow-xl"
            >
              <blockquote class="text-ink">
                <p>&ldquo;{{ t.quote }}&rdquo;</p>
              </blockquote>
            </figure>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
