<script setup lang="ts">
interface Props {
    tool: string;
    slug: string;
}
defineProps<Props>();
</script>

<template>
    <div class="text-[10px] uppercase tracking-[0.15em] text-term-dim">{{ slug }}</div>
    <div class="mt-1 text-term-text">$ {{ tool }} --help</div>
    <div class="text-term-dim">···</div>
    <div class="mt-2 text-term-yellow">↓ run drill</div>
    <div class="mt-1 text-term-green">[ok] scenario primed</div>
    <div class="text-term-text">awaiting your input</div>
</template>
