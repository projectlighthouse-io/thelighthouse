<template>
    <div class="text-[10px] uppercase tracking-[0.15em] text-term-dim">input · prod.log</div>
    <div class="mt-1 text-term-text">12:01 200 142ms /api</div>
    <div class="text-term-text">12:01 500 990ms /api</div>
    <div class="text-term-text">12:02 200 110ms /pay</div>
    <div class="mt-2 text-term-yellow">↓ awk '$2==500 {</div>
    <div class="text-term-yellow">&nbsp;&nbsp;sum+=$3 }'</div>
    <div class="mt-2 grid grid-cols-[6rem_1fr] gap-3">
        <span class="text-term-text">errors:</span>
        <span class="text-term-red">1</span>
    </div>
    <div class="grid grid-cols-[6rem_1fr] gap-3">
        <span class="text-term-text">total ms:</span>
        <span class="text-term-orange">990</span>
    </div>
    <div class="grid grid-cols-[6rem_1fr] gap-3">
        <span class="text-term-text">slowest:</span>
        <span class="text-term-text">12:01 /api</span>
    </div>
</template>
