<template>
    <div class="text-[10px] uppercase tracking-[0.15em] text-term-dim">recipe</div>
    <div class="mt-1 text-term-text">cat *.log \</div>
    <div class="text-term-text">| grep error \</div>
    <div class="text-term-text">| awk '{print $4}' \</div>
    <div class="text-term-text">| sort | uniq -c \</div>
    <div class="text-term-text">| sort -rn | head -5</div>
    <div class="mt-2 text-term-yellow">↓ run</div>
    <div class="mt-1 grid grid-cols-[3rem_1fr] gap-3">
        <span class="text-term-red">42</span>
        <span class="text-term-text">db-timeout</span>
    </div>
    <div class="grid grid-cols-[3rem_1fr] gap-3">
        <span class="text-term-red">17</span>
        <span class="text-term-text">auth-fail</span>
    </div>
    <div class="grid grid-cols-[3rem_1fr] gap-3">
        <span class="text-term-orange">9</span>
        <span class="text-term-text">rate-limit</span>
    </div>
</template>
