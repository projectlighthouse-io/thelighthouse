<template>
    <div class="text-[10px] uppercase tracking-[0.15em] text-term-dim">input · access.log</div>
    <div class="mt-1 text-term-text">10.0.0.1 get /api</div>
    <div class="text-term-text">10.0.0.1 get /pay</div>
    <div class="text-term-text">10.0.0.2 get /pay</div>
    <div class="text-term-text">10.0.0.1 get /api</div>
    <div class="mt-2 text-term-yellow">↓ sort | uniq -c |</div>
    <div class="text-term-yellow">&nbsp;&nbsp;sort -rn</div>
    <div class="mt-2 flex gap-2">
        <span class="text-term-dim">3</span>
        <span class="text-term-text">10.0.0.1 get /api</span>
    </div>
    <div class="flex gap-2">
        <span class="text-term-dim">1</span>
        <span class="text-term-text">10.0.0.2 get /pay</span>
    </div>
</template>
