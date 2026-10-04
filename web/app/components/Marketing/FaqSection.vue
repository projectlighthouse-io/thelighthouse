<script setup lang="ts">
import { faqs } from '@/data/Faqs'

/**
 * The FAQ as native <details> cards, first one open. The same list the pages
 * emit as FAQPage structured data, so the two cannot drift.
 */
const paragraphs = (answer: string): string[] =>
  answer.split(/\n{2,}/).map(p => p.trim()).filter(Boolean)
</script>

<template>
  <section class="faq lh-text lh-gap">
    <h2 class="lh-h2 heading">Frequently asked questions</h2>

    <div class="list">
      <details v-for="(faq, i) in faqs" :key="faq.question" class="item" :open="i === 0">
        <summary class="question">
          {{ faq.question }}
          <span class="plus" aria-hidden="true">+</span>
        </summary>
        <div class="answer">
          <p v-for="(p, k) in paragraphs(faq.answer)" :key="k">{{ p }}</p>
        </div>
      </details>
    </div>
  </section>
</template>

<style scoped>
.heading { margin-bottom: var(--space-12); }

.list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.item {
  padding: var(--space-5) var(--space-6);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-raised);
}

.question {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  font: 500 16px/22px var(--font-sans);
  letter-spacing: var(--tracking-title);
}

.plus {
  margin-left: auto;
  font: var(--text-label-mono);
  color: var(--ink-faint);
}

.answer {
  display: grid;
  gap: var(--space-3);
  margin-top: var(--space-4);
}

.answer p {
  margin: 0;
  font: var(--text-body-sm);
  color: var(--ink-secondary);
  text-wrap: pretty;
}
</style>
