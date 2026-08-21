<template>
    <div class="text-[10px] uppercase tracking-[0.15em] text-term-dim">input · errors.json</div>
    <div class="mt-1 truncate text-term-text">[{"ts":"...","lvl":"err",…</div>
    <div class="mt-2 text-term-yellow">↓ jq '.[] |</div>
    <div class="text-term-yellow">&nbsp;&nbsp;select(...)'</div>
    <div class="mt-2 grid grid-cols-[3rem_1fr_3rem] gap-3">
        <span class="text-term-red">503</span>
        <span class="text-term-text">api/orders</span>
        <span class="text-term-dim">12×</span>
    </div>
    <div class="grid grid-cols-[3rem_1fr_3rem] gap-3">
        <span class="text-term-red">503</span>
        <span class="text-term-text">api/payments</span>
        <span class="text-term-dim">8×</span>
    </div>
    <div class="grid grid-cols-[3rem_1fr_3rem] gap-3">
        <span class="text-term-orange">429</span>
        <span class="text-term-text">api/search</span>
        <span class="text-term-dim">3×</span>
    </div>
</template>
