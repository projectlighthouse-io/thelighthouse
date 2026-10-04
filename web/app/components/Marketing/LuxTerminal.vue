<script setup lang="ts">
/**
 * The hero's terminal: luxctl validating a project, typed out and replayed.
 *
 * The script is the handoff's, verbatim. Under reduced motion it renders the
 * finished run and stays there — no typing, no loop.
 */
type Tone = 'dim' | 'grey' | 'text' | 'bright' | 'green' | 'blue' | 'orange' | 'yellow' | 'red'
type Line = [string, Tone][]

const PROMPT_DIR = '~/projects/redis-mini'
const COMMAND = 'luxctl validate --project redis-mini'

const OUTPUT: Line[] = [
  [['→ ', 'grey'], ['resolving project manifest...', 'text']],
  [['→ ', 'grey'], ['spinning up isolated runtime ', 'text'], ['(rust 1.75)', 'orange']],
  [['→ ', 'grey'], ['running ', 'text'], ['31 checks', 'blue'], [' against your solution', 'text']],
  [['✓ ', 'green'], ['[ 1/31] parses resp simple strings  ', 'text'], ['6ms', 'grey']],
  [['✓ ', 'green'], ['[ 2/31] parses resp bulk strings  ', 'text'], ['4ms', 'grey']],
  [['✓ ', 'green'], ['[ 3/31] handles inline commands  ', 'text'], ['3ms', 'grey']],
  [['   ... 23 more passing', 'dim']],
  [['! ', 'yellow'], ['[27/31] pipelined commands  ', 'text'], ['slow', 'yellow'], ['  412ms', 'grey']],
  [['✕ ', 'red'], ['[28/31] expire / ttl precision < 10ms  ', 'text'], ['fail', 'red']],
  [['  → expected ', 'grey'], ['9ms', 'green'], [' drift, got ', 'grey'], ['47ms', 'red'], [' — see ', 'grey'], ['notes/expire.md', 'blue']],
  [['✓ ', 'green'], ['[29/31] config get maxmemory  ', 'text'], ['2ms', 'grey']],
  [['✓ ', 'green'], ['[30/31] persistence: aof replay  ', 'text'], ['38ms', 'grey']],
  [['✓ ', 'green'], ['[31/31] concurrent clients (50)  ', 'text'], ['121ms', 'grey']],
  [['', 'text']],
  [['summary · ', 'text'], ['29 pass', 'green'], [' · ', 'text'], ['1 slow', 'yellow'], [' · ', 'text'], ['1 fail', 'red'], [' · 31 total', 'text']],
  [['tip: ', 'grey'], ['luxctl explain 28', 'blue'], [' for a guided walkthrough of the failing test.', 'grey']],
]

const typed = ref(COMMAND.length)
const shown = ref(OUTPUT.length)
const scroller = useTemplateRef<HTMLElement>('scroller')

const done = computed<boolean>(() => typed.value >= COMMAND.length && shown.value >= OUTPUT.length)

let timer: ReturnType<typeof setTimeout> | undefined

function tick(): void {
  if (typed.value < COMMAND.length) {
    typed.value += 1
    timer = setTimeout(tick, 42)
  }
  else if (shown.value < OUTPUT.length) {
    shown.value += 1
    timer = setTimeout(tick, shown.value === 1 ? 420 : 130)
  }
  else {
    timer = setTimeout(() => {
      typed.value = 0
      shown.value = 0
      tick()
    }, 6000)
  }
}

watch([typed, shown], async () => {
  await nextTick()
  if (scroller.value) scroller.value.scrollTop = scroller.value.scrollHeight
})

onMounted(() => {
  // The server renders the finished run, which is also what a reader who
  // asked for less motion keeps — scrolled to its end, like a real terminal.
  if (matchMedia('(prefers-reduced-motion: reduce)').matches) {
    if (scroller.value) scroller.value.scrollTop = scroller.value.scrollHeight
    return
  }

  typed.value = 0
  shown.value = 0
  timer = setTimeout(tick, 120)
})

onBeforeUnmount(() => clearTimeout(timer))
</script>

<template>
  <TerminalPanel tag="luxctl" note="the runner every chapter ends in">
    <div
      ref="scroller"
      class="body lh-noscroll"
      role="img"
      :aria-label="`a terminal running ${COMMAND}: 29 checks pass, 1 is slow, 1 fails`"
    >
      <div aria-hidden="true">
        <div class="t-grey">aryan@lighthouse in <span class="t-bright">{{ PROMPT_DIR }}</span></div>
        <div class="t-bright">
          <span class="t-grey">$ </span>{{ COMMAND.slice(0, typed) }}<span v-if="typed < COMMAND.length" class="lh-caret" />
        </div>
        <div v-for="(line, i) in OUTPUT.slice(0, shown)" :key="i" class="line">
          <span v-for="(seg, j) in line" :key="j" :class="`t-${seg[1]}`">{{ seg[0] }}</span>
        </div>
        <div v-if="done" class="t-grey">$ <span class="lh-caret" /></div>
      </div>
    </div>
  </TerminalPanel>
</template>

<style scoped>
.body {
  height: 400px;
  overflow: auto;
  padding: var(--space-4) 28px var(--space-6);
  font: var(--text-code);
  font-size: 12.5px;
  line-height: 22px;
  letter-spacing: 0.01em;
  font-variant-numeric: tabular-nums;
  color: var(--term-text);
  white-space: pre;
}

.line { min-height: 22px; }

.t-dim { color: var(--term-dim); }
.t-grey { color: var(--term-grey); }
.t-text { color: var(--term-text); }
.t-bright { color: var(--term-bright); }
.t-green { color: var(--term-green); }
.t-blue { color: var(--term-blue); }
.t-orange { color: var(--term-orange); }
.t-yellow { color: var(--term-yellow); }
.t-red { color: var(--term-red); }

@media (max-width: 600px) {
  .body { padding: var(--space-4) 20px var(--space-5); }
}
</style>
