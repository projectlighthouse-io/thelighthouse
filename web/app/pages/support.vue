<script setup lang="ts">
const description
  = 'Get help with projectlighthouse. Contact us about courses, labs, subscriptions, or technical issues.'

useSeo({
  title: 'Support — projectlighthouse',
  description,
})

useJsonLd('support', {
  '@type': 'ContactPage',
  'name': 'Support',
  'mainEntity': {
    '@type': 'Organization',
    'name': SITE.name,
    'email': 'thearyanahmed@projectlighthouse.io',
  },
})
</script>

<template>
  <section class="py-16">
    <div class="mx-auto w-full max-w-4xl px-4 sm:px-6 lg:px-8">
      <div class="mb-10 text-center">
        <h1 class="mb-4 font-mono text-4xl font-medium text-ink">support</h1>
      </div>

      <div class="text-mono-body mx-auto max-w-2xl space-y-6">
        <p>
          If something isn't working, you have a question, or you just want to say hi&mdash;send an
          email to
          <a
            href="mailto:thearyanahmed@projectlighthouse.io"
            class="border-b border-stroke font-medium text-ink transition hover:border-quiet"
          >thearyanahmed@projectlighthouse.io</a>.
        </p>

        <p>I read every email and typically reply within a day or two.</p>
      </div>
    </div>
  </section>
</template>
