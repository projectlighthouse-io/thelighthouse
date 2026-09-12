<script setup lang="ts">
import { AUTH_ART } from '@/data/Auth'

/**
 * The subscribe form, drawn as an envelope.
 *
 * Posts straight to `/api/newsletter` — the api's one public write, with no
 * session and no CSRF token, because a visitor who has never signed in holds
 * neither. No nitro route in between: this is a browser action, not something
 * rendered on the server, so there is nothing for `/_api/*` to shape or cache.
 *
 * **The success wording never says whether the address was already on the
 * list.** The api does not tell this page, deliberately — see `newsletter` for
 * why: a form that distinguishes the two answers "is this person a reader of
 * yours" for anybody who cares to ask.
 */
const email = ref('')
const busy = ref(false)
const sent = ref(false)
const problem = ref('')

const field = useId()

/** Whether the address looks finished. The api is the real judge — this only
 *  decides when the button stops being disabled. */
const complete = computed<boolean>(
  () => /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(email.value.trim()),
)

/** The line under the address. Empty before anything is typed, so the envelope
 *  does not open with an instruction nobody asked for. */
const hint = computed<string>(() => {
  if (problem.value) return problem.value
  if (!email.value) return ''

  return complete.value ? 'Address complete — ready to seal.' : 'Finish the address…'
})

/**
 * How many readers are on the list.
 *
 * Hand-kept, because the list lives at Kit and nothing here counts it —
 * `users.newsletter_enabled` only knows about readers who have an account, so
 * it would undercount every visitor who subscribed from this form. Move it to
 * the api the day something asks Kit for the real number.
 */
const SUBSCRIBERS = 553

/** The postmark carries the day it was stamped, the way a real one does. */
const MONTHS = [
  'JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN',
  'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC',
]

const stampedOn = computed<string>(() => {
  const now = new Date()

  return `${now.getDate()} ${MONTHS[now.getMonth()]} ${String(now.getFullYear()).slice(2)}`
})

async function seal(): Promise<void> {
  if (busy.value || !complete.value) return

  busy.value = true
  problem.value = ''

  try {
    await $fetch('/api/newsletter', {
      method: 'POST',
      body: { email: email.value.trim() },
    })
    sent.value = true
  }
  catch (error: unknown) {
    // The api's own wording when it sent one — written for a person, and more
    // use than anything this form could invent. A 429 carries none, so that
    // case gets the fallback.
    problem.value = (error as { data?: { error?: string } })?.data?.error
      ?? 'Could not send it just now. Try again in a minute.'
  }
  finally {
    busy.value = false
  }
}

/** Back to an editable address, for a typo spotted a second too late. */
function readdress(): void {
  sent.value = false
  problem.value = ''
}
</script>

<template>
  <!--
    Scoped CSS rather than utilities for most of this. Almost every measurement
    in an envelope is off the scale — 3px airmail stripes, a stamp rotated 1.6
    degrees, a postmark that lands on it — and as Tailwind each one would be a
    bracket value, which is the argument `contents.css` already makes. Colours
    are all tokens, so it themes.
  -->
  <section class="news">
    <h2 class="news__title">The newsletter</h2>

    <div class="env" :class="{ 'env--sent': sent, 'env--bad': problem }">
    <div class="env__in">
      <div class="env__addr">
        <h2 class="env__title">Addressed to you</h2>

        <p class="env__lede">
          New books, new projects, and what I learned building them, and
          nothing else.
        </p>

        <div class="env__rows">
          <div class="env__row">
            <span class="env__key">from</span>
            <span class="env__val">Aryan, keeper of the lighthouse</span>
          </div>

          <div class="env__row">
            <span class="env__key">re</span>
            <span class="env__val">Systems programming, written slowly</span>
          </div>

          <!-- The one blank on the envelope, and the only thing to fill in. -->
          <div class="env__row env__row--to">
            <label class="env__key env__key--to" :for="field">to</label>
            <span class="env__val env__blank">
              <!-- A written-in address, not a form field: the placeholder is
                   drawn behind the input so the caret sits on the line where a
                   pen would start. -->
              <span v-if="!email" class="env__ghost" aria-hidden="true">
                <b />your email here
              </span>
              <input
                :id="field"
                v-model="email"
                type="email"
                autocomplete="email"
                spellcheck="false"
                :readonly="sent"
                placeholder="you@example.com"
                @keydown.enter.prevent="seal"
              >
            </span>
          </div>
        </div>

        <!--
          Always rendered, hidden rather than removed: `v-show` sets
          `display: none`, so the line took no space until there was something
          to say and everything under it jumped when there was. `visibility`
          keeps the row reserved and empty.
        -->
        <p class="env__hint" :class="{ 'env__hint--quiet': !hint }" role="status">
          <span class="env__dot" />{{ hint }}
        </p>
      </div>

      <div class="env__rail">
        <div class="env__stamp">
          <div class="env__stamp-frame">
            <!-- The same art the login page and the join panel show, from the
                 one constant they share — a stamp is a small picture, and this
                 is the picture this site already uses for itself. -->
            <img
              :src="AUTH_ART.image"
              alt=""
              width="96"
              height="120"
              loading="lazy"
              referrerpolicy="no-referrer"
            >
          </div>

          <!-- Stamped across the corner once it is sent, the way a post office
               cancels a stamp so it cannot be used twice. -->
          <svg class="env__mark" viewBox="0 0 120 120" fill="none" aria-hidden="true">
            <circle cx="60" cy="60" r="44" stroke="currentColor" stroke-width="2.4" />
            <circle cx="60" cy="60" r="35" stroke="currentColor" stroke-width="1.3" />
            <path d="M19 60h82" stroke="currentColor" stroke-width="1.1" />
            <text x="60" y="44" text-anchor="middle" font-size="9.5" letter-spacing="1.4" fill="currentColor">SUBSCRIBED</text>
            <text x="60" y="79" text-anchor="middle" font-size="11" letter-spacing="1.2" fill="currentColor">{{ stampedOn }}</text>
          </svg>
        </div>

        <button
          type="button"
          class="env__send"
          :disabled="!complete || busy || sent"
          @click="seal"
        >
          {{ busy ? 'sealing…' : 'seal & send' }}
          <svg viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M3 8h9M8.5 4l4 4-4 4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Replaces nothing: the address stays on screen under it, dimmed, so what
         was sent is still legible. -->
    <div v-if="sent" class="env__done" role="status">
      <span class="env__seal" aria-hidden="true">✦</span>
      <span>
        <span class="env__done-t">On its way.</span>
        <span class="env__done-s">Sent to {{ email.trim() }}.</span>
      </span>
        <button type="button" class="env__undo" @click="readdress">edit address</button>
      </div>
    </div>

    <p class="news__foot">{{ SUBSCRIBERS.toLocaleString('en-US') }} readers</p>
  </section>
</template>

<style scoped>
/* The panel the envelope sits on. White like the envelope, so the two are told
   apart by the envelope's border rather than by a change of ground — which is
   what makes it read as a letter lying on a desk. */
.news {
    background: var(--color-panel);
    border-radius: 14px;
    /* Padded on all four sides. Dropping the side padding made the widths
       line up but left the heading flush against the panel's edge — invisible
       while the panel was white on white, and plainly wrong the moment dark
       mode gave the panel a ground of its own. The column it sits in is what
       sets the width instead. */
    padding: 44px;
}

.news__title {
    margin: 0 0 22px;
    font-family: var(--font-serif);
    font-weight: 600;
    font-size: 34px;
    letter-spacing: -0.02em;
    color: var(--color-ink);
}

.news__foot {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    margin: 16px 0 0;
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--color-faint);
}

.env {
    position: relative;
    border-radius: 6px;
    background: var(--color-panel);
    border: 1px solid var(--color-rule);
    box-shadow: 0 18px 40px -30px rgb(26 26 26 / 40%);
    overflow: hidden;
}

/* The airmail border, inset from the edge the way a real one is. Four
   repeating gradients rather than a border image, so it takes the token and
   flips with the theme. */
.env::before {
    content: '';
    position: absolute;
    inset: 9px;
    border-radius: 3px;
    pointer-events: none;
    z-index: 3;
    opacity: 0.5;
    background:
        repeating-linear-gradient(90deg, var(--color-pencil-red) 0 12px, transparent 12px 24px) top left / 100% 3px no-repeat,
        repeating-linear-gradient(90deg, var(--color-pencil-red) 0 12px, transparent 12px 24px) bottom left / 100% 3px no-repeat,
        repeating-linear-gradient(180deg, var(--color-pencil-red) 0 12px, transparent 12px 24px) top left / 3px 100% no-repeat,
        repeating-linear-gradient(180deg, var(--color-pencil-red) 0 12px, transparent 12px 24px) top right / 3px 100% no-repeat;
}

.env__in {
    position: relative;
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 40px;
    padding: 46px;
}

.env__addr {
    min-width: 0;
}

.env__title {
    margin: 0;
    font-family: var(--font-serif);
    font-weight: 600;
    font-size: 27px;
    letter-spacing: -0.018em;
    color: var(--color-ink);
}

.env__lede {
    margin: 10px 0 0;
    max-width: 46ch;
    line-height: 1.62;
    color: var(--color-read-ink-soft);
    text-wrap: pretty;
}

.env__rows {
    margin-top: 28px;
}

.env__row {
    display: grid;
    grid-template-columns: 64px 1fr;
    align-items: baseline;
    gap: 12px;
    padding: 9px 0;
}

.env__key {
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--color-faint);
}

.env__key--to {
    color: var(--color-pencil-red);
    cursor: text;
}

.env__val {
    min-width: 0;
    font-family: var(--font-serif);
    font-size: 16px;
    color: var(--color-read-ink-soft);
}

.env__dim {
    color: var(--color-quiet);
    font-style: italic;
}

.env__blank {
    position: relative;
}

.env__blank input {
    display: block;
    width: 100%;
    /* No padding: 5px of it made the `to` row taller than the two above it. */
    padding: 0;
    border: 0;
    background: none;
    outline: none;
    font-family: var(--font-serif);
    font-weight: 500;
    /* Inherited from `.env__val`, not stated: at 22px the `to` row was taller
       than `from` and `re`, and the row changed height as the ghost gave way
       to the typed address. */
    font-size: inherit;
    line-height: inherit;
    letter-spacing: -0.01em;
    color: var(--color-ink);
    caret-color: var(--color-pencil-red);
}

/* The real placeholder is hidden and redrawn as `.env__ghost`, so the blinking
   rule can sit in front of it — a placeholder cannot carry a child element. */
.env__blank input::placeholder {
    color: transparent;
}

.env__ghost {
    position: absolute;
    top: 0;
    left: 0;
    display: flex;
    align-items: center;
    pointer-events: none;
    font-family: var(--font-serif);
    font-style: italic;
    font-size: inherit;
    line-height: inherit;
    color: var(--color-faint);
}

.env__ghost b {
    display: inline-block;
    width: 1.5px;
    height: 1.05em;
    margin-right: 4px;
    vertical-align: -0.12em;
    background: var(--color-pencil-red);
    animation: env-blink 1.05s steps(1, end) infinite;
}

@keyframes env-blink {
    0%,
    50% {
        opacity: 1;
    }

    50.01%,
    100% {
        opacity: 0;
    }
}

.env__hint {
    display: flex;
    align-items: center;
    gap: 9px;
    margin: 11px 0 0;
    min-height: 17px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--color-quiet);
}

.env__hint--quiet {
    visibility: hidden;
}

.env__dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-faint);
    flex: none;
    transition: background 200ms;
}

.env--bad .env__hint {
    color: var(--color-pencil-red);
}

.env--bad .env__dot {
    background: var(--color-pencil-red);
}

.env__rail {
    display: flex;
    flex: none;
    flex-direction: column;
    align-items: flex-end;
    justify-content: space-between;
    gap: 30px;
    /* Stated, not sized to content. The button's label goes from `seal & send`
       to `sealing…` on submit, and a rail that measures itself against the
       label was 151px wide, then 112px, then 151px again — dragging the stamp
       39px sideways and back on every send. Wide enough for the longer one. */
    width: 152px;
}

.env__stamp {
    position: relative;
    width: 112px;
    align-self: start;
    transform: rotate(1.6deg);
}

.env__stamp-frame {
    padding: 8px;
    border: 2px dashed color-mix(in oklab, var(--color-pencil-red) 60%, transparent);
    border-radius: 5px;
    background: var(--color-panel);
    text-align: center;
}

/* `cover`, not `contain`: this is a photograph filling a stamp, not a mark
   that has to stay whole. */
.env__stamp-frame img {
    display: block;
    width: 100%;
    height: auto;
    aspect-ratio: 4 / 5;
    object-fit: cover;
    border-radius: 3px;
    box-shadow: 0 0 0 1px var(--color-rule);
}

.env__mark {
    position: absolute;
    top: -8px;
    left: -52px;
    width: 118px;
    height: 118px;
    pointer-events: none;
    color: var(--color-teal-deep);
    opacity: 0;
    transform: rotate(-16deg) scale(1.5);
    transition:
        opacity 420ms ease,
        transform 420ms cubic-bezier(0.2, 0.9, 0.3, 1.2);
}

.env__mark text {
    font-family: var(--font-mono);
}

.env--sent .env__mark {
    opacity: 0.42;
    transform: rotate(-13deg) scale(1);
}

.env__send {
    display: inline-flex;
    width: 100%;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 12px 20px;
    border: 0;
    border-radius: 7px;
    background: var(--color-ink);
    color: var(--color-on-ink);
    font-family: var(--font-mono);
    font-size: 13px;
    cursor: pointer;
    transition:
        background 160ms,
        transform 120ms,
        opacity 160ms;
}

.env__send:hover:not(:disabled) {
    background: var(--color-ink-hover);
}

.env__send:active:not(:disabled) {
    transform: translateY(1px);
}

.env__send:disabled {
    opacity: 0.34;
    cursor: not-allowed;
}

.env__send svg {
    width: 15px;
    height: 15px;
}

/* Sent: the address stays readable underneath, dimmed, so the reader can see
   what was actually sent rather than a panel that replaced it. */
.env--sent .env__rows,
.env--sent .env__hint,
.env--sent .env__send {
    opacity: 0.42;
    transition: opacity 300ms;
}

.env__done {
    position: absolute;
    right: 46px;
    bottom: 38px;
    left: 46px;
    display: flex;
    align-items: center;
    gap: 13px;
    padding: 14px 16px;
    border: 1px solid var(--color-rule);
    border-radius: 8px;
    background: var(--color-panel);
    box-shadow: 0 14px 30px -22px rgb(26 26 26 / 50%);
}

.env__seal {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    flex: none;
    border-radius: 50%;
    background: var(--color-pencil-red);
    color: var(--color-panel);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
}

.env__done-t {
    display: block;
    font-family: var(--font-serif);
    font-weight: 500;
    font-size: 15px;
    color: var(--color-ink);
}

.env__done-s {
    display: block;
    margin-top: 2px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--color-quiet);
    overflow-wrap: anywhere;
}

.env__undo {
    margin-left: auto;
    flex: none;
    border: 0;
    background: none;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--color-quiet);
}

.env__undo:hover {
    color: var(--color-ink);
}

/* One column, and no stamp: 112px of stamp is a third of a phone, and the
   postmark it carries has nothing to cancel once it is gone. */
@media (max-width: 640px) {
    /*
     * No panel on a phone — the ground goes, not just its padding.
     *
     * Keeping the box and dropping the side padding leaves the heading flush
     * against a visible edge, which is how this looked wrong in dark mode
     * before. Without the box there is nothing to be flush against: the
     * heading and the envelope both start at the page gutter, like every
     * other card on the page. The envelope has an edge of its own and does
     * not need a second one around it at this width.
     */
    .news {
        padding: 24px 0;
        background: none;
        border-radius: 0;
    }

    .news__title {
        font-size: 26px;
        margin-bottom: 18px;
    }

    .env__in {
        grid-template-columns: 1fr;
        gap: 20px;
        padding: 30px 26px 26px;
    }

    .env__stamp {
        display: none;
    }

    .env__rail {
        align-items: stretch;
        /* One column down here, so the rail is the full width of the envelope
           and there is no stamp beside it to knock out of place. */
        width: auto;
    }

    .env__done {
        right: 26px;
        bottom: 26px;
        left: 26px;
    }
}

/* The stamp landing and the caret blinking are decoration. Anybody who asked
   not to be moved gets the states without the movement. */
@media (prefers-reduced-motion: reduce) {
    .env__ghost b {
        animation: none;
    }

    .env__mark {
        transition: opacity 420ms ease;
        transform: rotate(-13deg);
    }

    .env--sent .env__mark {
        transform: rotate(-13deg);
    }
}
</style>
