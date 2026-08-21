<template>
    <div class="text-[10px] uppercase tracking-[0.15em] text-term-dim">before</div>
    <div class="mt-1 text-term-text">db_host: prod-db-1.old.io</div>
    <div class="text-term-text">api_key: sk_live_abc123def</div>
    <div class="mt-2 text-term-yellow">↓ sed -i -e</div>
    <div class="text-term-yellow">'s/sk_live_\w+/redacted/'</div>
    <div class="mt-2 text-term-text">db_host: prod-db-1.new.io</div>
    <div class="text-term-text">api_key: <span class="text-term-red">redacted</span></div>
</template>
