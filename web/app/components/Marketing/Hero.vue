<script setup lang="ts">
/**
 * The home page's opening: the hand-lettered title, the two CTAs, the counts,
 * and luxctl running beside them. The counts are passed in — they come from
 * the same listings the bands below render, so they cannot disagree.
 */
defineProps<{
  stats: { value: number, label: string }[]
}>()

const { getPro } = useGetPro()
</script>

<template>
  <div class="hero">
    <section class="text">
      <h1 class="title">fundamentals of software engineering&nbsp;;</h1>
      <p class="sub">
        handcrafted shelf of books on systems, networks, runtimes and more. build projects from scratch.
      </p>

      <div class="ctas">
        <UiButton variant="inverse" size="lg" cta="pro" flame @click="getPro">Get Pro</UiButton>
        <UiButton variant="ghost" size="lg" cta="free" to="/#shelf">Start Free</UiButton>
      </div>

      <dl class="stats">
        <div v-for="stat in stats" :key="stat.label">
          <dt class="lh-eyebrow">{{ stat.label }}</dt>
          <dd class="value lh-num">{{ stat.value.toLocaleString('en') }}</dd>
        </div>
      </dl>
    </section>

    <div class="figure">
      <MarketingLuxTerminal />
    </div>
  </div>
</template>

<style scoped>
.hero {
  max-width: 1400px;
  margin: 0 auto;
  padding: 72px var(--space-8) 0;
  display: flex;
  flex-wrap: wrap;
  column-gap: 72px;
  row-gap: 56px;
  align-items: center;
  justify-content: flex-end;
}

.text {
  flex: 0 1 460px;
  min-width: 0;
  margin-left: auto;
}

.title {
  margin: 0;
  font: 400 56px/62px var(--font-hero);
  letter-spacing: -0.01em;
  text-wrap: pretty;
}

.sub {
  margin: var(--space-6) 0 0;
  font: var(--text-body-sm);
  color: var(--ink-secondary);
  text-wrap: pretty;
}

.ctas {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  margin-top: var(--space-8);
}

.stats {
  margin: 40px 0 0;
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-8);
}

.stats > div {
  display: flex;
  flex-direction: column-reverse;
  gap: 2px;
}

.value {
  margin: 0;
  font: var(--text-h2);
  letter-spacing: var(--tracking-h2);
}

.figure {
  flex: 0 1 740px;
  min-width: 0;
}

@media (max-width: 1250px) {
  .hero { justify-content: flex-start; }
  .text { margin-left: 0; flex-basis: 100%; }
}

@media (max-width: 600px) {
  .hero { padding: 48px var(--space-6) 0; }
  .title { font-size: 44px; line-height: 50px; }
}
</style>
