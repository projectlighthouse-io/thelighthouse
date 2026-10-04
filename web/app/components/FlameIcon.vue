<script setup lang="ts">
/**
 * The flame on every Get Pro. Off-system orange on purpose; it flickers
 * through the four stops in the handoff and holds still under reduced motion.
 */
</script>

<template>
  <svg
    class="flame"
    width="16"
    height="18"
    viewBox="0 0 16 18"
    fill="none"
    stroke="#f2a41f"
    stroke-width="1.6"
    stroke-linejoin="round"
    stroke-linecap="round"
    aria-hidden="true"
  >
    <path d="M8 1.5c.4 2.6 2.2 3.8 3.6 5.4C13 8.5 14 10 14 11.8A6 6 0 0 1 2 11.8c0-1.9.9-3.3 2-4.4.2 1.2.9 2 1.8 2.3C5.6 7.2 6.4 4 8 1.5Z" />
    <path d="M8 16.5a2.6 2.6 0 0 0 2.6-2.6c0-1.3-1-2.1-1.6-3.2-.7.7-1.1 1.2-1.9 1.6-.6.4-1.1 1-1.1 1.6A2.6 2.6 0 0 0 8 16.5Z" />
  </svg>
</template>

<style scoped>
.flame {
  flex: none;
  transform-origin: 50% 100%;
  animation: flame 1.1s ease-in-out infinite;
}

@keyframes flame {
  0% { transform: scale(1) skewX(0deg); filter: drop-shadow(0 0 2px rgba(242, 90, 40, 0.7)); stroke: #f2a41f; }
  20% { transform: scale(1.06, 1.1) skewX(-3deg); filter: drop-shadow(0 0 5px rgba(242, 90, 40, 0.95)); stroke: #f56a2a; }
  40% { transform: scale(0.96, 0.98) skewX(2deg); filter: drop-shadow(0 0 3px rgba(242, 90, 40, 0.6)); stroke: #f2a41f; }
  60% { transform: scale(1.04, 1.08) skewX(3deg); filter: drop-shadow(0 0 6px rgba(240, 60, 40, 1)); stroke: #ef4d2f; }
  80% { transform: scale(0.98, 1.02) skewX(-2deg); filter: drop-shadow(0 0 3px rgba(242, 90, 40, 0.7)); stroke: #f58b2a; }
  100% { transform: scale(1) skewX(0deg); filter: drop-shadow(0 0 2px rgba(242, 90, 40, 0.7)); stroke: #f2a41f; }
}

@media (prefers-reduced-motion: reduce) {
  .flame { animation: none; }
}
</style>
