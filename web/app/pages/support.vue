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
  <article class="page lh-text lh-gap">
    <header class="lh-head">
      <p class="lh-eyebrow">support</p>
      <h1 class="lh-h1">Write to me</h1>
    </header>

    <div class="lesson-content body">
      <p>
        If something isn't working, you have a question, or you just want to say hi — send an
        email to
        <a href="mailto:thearyanahmed@projectlighthouse.io">thearyanahmed@projectlighthouse.io</a>.
      </p>
      <p>I read every email and typically reply within a day or two.</p>
    </div>
  </article>
</template>

<style scoped>
.body { margin-top: var(--space-12); }
</style>
