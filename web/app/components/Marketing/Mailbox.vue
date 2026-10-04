<script setup lang="ts">
import type { Testimonial } from '@/types/Content'

/**
 * Reviews as mail read in a terminal: the inbox on the left, the open letter
 * on the right. Advances every seven seconds; picking a row or a dot opens it
 * and restarts the clock. Under reduced motion it never advances by itself.
 * Names are never shown.
 */
const props = defineProps<{ letters: Testimonial[] }>()

const active = ref(0)
const count = computed<number>(() => props.letters.length)
const open = computed<Testimonial | undefined>(() => props.letters[active.value])

const pad = (n: number): string => String(n).padStart(2, '0')

let timer: ReturnType<typeof setInterval> | undefined
let reduced = true

function start(): void {
  clearInterval(timer)
  if (reduced || count.value < 2) return

  timer = setInterval(() => {
    active.value = (active.value + 1) % count.value
  }, 7000)
}

function pick(i: number): void {
  active.value = i
  start()
}

onMounted(() => {
  reduced = matchMedia('(prefers-reduced-motion: reduce)').matches
  start()
})

onBeforeUnmount(() => clearInterval(timer))
</script>

<template>
  <div class="mail-section">
    <div class="lh-text lh-gap">
      <h2 class="lh-h2">A few words from folks on the voyage</h2>
    </div>

    <div class="lh-wide frame-wrap">
      <div class="frame lh-inverse">
        <div class="inbox">
          <div class="cmd">
            <span><span class="dim">$ </span>mail -f /var/mail/readers</span>
            <span class="dim">"/var/mail/readers": {{ count }} messages from readers</span>
          </div>

          <div class="rows">
            <button
              v-for="(letter, i) in letters"
              :key="letter.subject"
              type="button"
              class="row"
              :class="{ 'is-active': i === active }"
              :aria-pressed="i === active"
              @click="pick(i)"
            >
              <span class="marker" aria-hidden="true">{{ i === active ? '>' : '' }}</span>
              <span class="dim lh-num">{{ pad(i + 1) }}</span>
              <span class="subject">{{ letter.subject }}</span>
            </button>
          </div>

          <span class="dim" aria-hidden="true">&amp; <span class="caret" /></span>
        </div>

        <article v-if="open" class="letter" aria-live="polite">
          <dl class="headers">
            <dt>From:</dt>
            <dd>
              <span class="redacted" aria-hidden="true">a reader who asked us not to say</span>
              <span class="dim">(withheld)</span>
            </dd>
            <dt>To:</dt>
            <dd>thearyanahmed@projectlighthouse.io</dd>
            <dt>Subject:</dt>
            <dd class="bright">{{ open.subject }}</dd>
          </dl>

          <p class="quote">{{ open.quote }}</p>

          <div class="foot">
            <span>message {{ pad(active + 1) }} of {{ pad(count) }} · printed unedited</span>
            <span class="dots">
              <button
                v-for="(letter, i) in letters"
                :key="letter.subject"
                type="button"
                class="dot"
                :class="{ 'is-active': i === active }"
                :aria-label="`letter ${i + 1}`"
                :aria-pressed="i === active"
                @click="pick(i)"
              />
            </span>
          </div>
        </article>
      </div>
    </div>
  </div>
</template>

<style scoped>
.frame-wrap { margin-top: var(--space-12); }

.frame {
  display: grid;
  grid-template-columns: minmax(0, 5fr) minmax(0, 7fr);
  gap: var(--space-2);
}

.inbox {
  display: grid;
  gap: var(--space-4);
  align-content: start;
  min-width: 0;
  padding: var(--space-6);
  border: 1px solid var(--ink-inverse-faint);
  border-radius: var(--radius-md);
  font: var(--text-code);
}

.cmd {
  display: grid;
  gap: var(--space-1);
}

.dim { color: var(--ink-inverse-muted); }
.bright { color: var(--ink-inverse); }

.rows {
  display: grid;
  gap: 2px;
}

.row {
  display: grid;
  grid-template-columns: 16px 28px minmax(0, 1fr);
  gap: 0 var(--space-3);
  align-items: baseline;
  min-width: 0;
  padding: var(--space-2) 10px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--ink-inverse-secondary);
  font: var(--text-code);
  text-align: left;
  cursor: pointer;
  transition: background-color var(--duration) var(--ease-out);
}

.row:hover { background: rgba(255, 255, 255, 0.06); }

.row.is-active {
  background: rgba(255, 255, 255, 0.08);
  color: var(--ink-inverse);
}

.marker { color: var(--accent); }

.subject {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.caret {
  display: inline-block;
  width: 7px;
  height: 13px;
  vertical-align: -2px;
  background: var(--accent);
  animation: lh-caret 1.1s steps(1, end) infinite;
}

.letter {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  gap: 28px;
  min-width: 0;
  height: 480px;
  padding: var(--space-8) 36px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: var(--radius-md);
  background: rgba(255, 255, 255, 0.04);
}

.headers {
  margin: 0;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 6px var(--space-4);
  font: var(--text-code);
  color: var(--ink-inverse-secondary);
}

.headers dt { color: var(--ink-inverse-muted); }
.headers dd { margin: 0; min-width: 0; overflow-wrap: anywhere; }

.redacted + .dim { margin-left: 0.5ch; }

.redacted {
  border-radius: 3px;
  background: var(--ink-inverse-faint);
  color: transparent;
  user-select: none;
}

.quote {
  align-self: center;
  margin: 0;
  overflow: hidden;
  font: 400 26px/38px var(--font-serif);
  letter-spacing: -0.005em;
  color: var(--ink-inverse);
  text-wrap: pretty;
}

.foot {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: var(--space-4);
  padding-top: var(--space-5);
  border-top: 1px dashed var(--ink-inverse-faint);
  font: var(--text-label-mono);
  color: var(--ink-inverse-muted);
}

.dots {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 6px;
}

.dot {
  width: 6px;
  height: 6px;
  padding: 0;
  border: 0;
  border-radius: 50%;
  background: var(--ink-inverse-faint);
  cursor: pointer;
}

/* a 6px dot is too small to hit; widen the target without drawing it */
.dot { position: relative; }
.dot::after { content: ''; position: absolute; inset: -6px; }

.dot.is-active { background: var(--accent); }

@media (max-width: 900px) {
  .frame { grid-template-columns: minmax(0, 1fr); }
}

@media (max-width: 600px) {
  /* fixed, like the desktop letter, so auto-advance never moves the page */
  .letter { height: 600px; padding: var(--space-6); }
  .quote { font-size: 20px; line-height: 30px; }
  .foot { flex-direction: column; align-items: flex-start; }
}
</style>
