<script setup lang="ts">
/**
 * A page for the form, so it has a url worth linking to.
 *
 * The footer carries the same component on every page; this exists for the
 * times a link is the thing being sent — a talk, a README, a reply. Indexed,
 * unlike `/login`: it is a real page with a reason to be found.
 */

definePageMeta({ layout: 'default' })

useSeo({
  title: 'Newsletter — projectlighthouse',
  description:
    'New books, new projects, and what I learned building them. One email a week at most.',
})
</script>

<template>
  <div class="px-4 py-16 sm:px-6 lg:px-8">
    <div class="mx-auto max-w-2xl">
      <h1 class="font-editorial text-4xl leading-tight text-ink sm:text-5xl font-semibold">
        The newsletter
      </h1>

      <p class="mt-6 font-serif text-lg leading-relaxed text-quiet">
        I write about what goes into building this place — the books as they are
        drafted, the projects as they are broken, and the things that turned out
        to be harder than they looked.
      </p>

      <p class="mt-4 font-serif text-lg leading-relaxed text-quiet">
        One email a week at most, often fewer. No sponsors, no referrals, and
        nothing that is not mine. Unsubscribe from any of them.
      </p>

    </div>

    <!-- Its own column, wider than the prose above it: the envelope is a
         landscape thing and a reading measure squashes it. -->
    <div class="mx-auto mt-10 max-w-5xl">
      <MarketingNewsletterForm />
    </div>
  </div>
</template>
