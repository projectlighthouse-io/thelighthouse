<template>
    <div class="text-[10px] uppercase tracking-[0.15em] text-term-dim">on-call.sh</div>
    <div class="mt-1 text-term-text">for svc in api pay search; do</div>
    <div class="text-term-text">&nbsp;&nbsp;if ! curl -sf $svc/health; then</div>
    <div class="text-term-text">&nbsp;&nbsp;&nbsp;&nbsp;alert "$svc down"</div>
    <div class="text-term-text">&nbsp;&nbsp;fi</div>
    <div class="text-term-text">done</div>
    <div class="mt-2 text-term-yellow">↓ ./on-call.sh</div>
    <div class="mt-1 text-term-green">[ok] api</div>
    <div class="text-term-green">[ok] pay</div>
    <div class="text-term-red">[!!] search down · paged</div>
</template>
