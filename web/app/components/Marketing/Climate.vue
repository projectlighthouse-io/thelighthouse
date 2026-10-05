<script setup lang="ts">
/**
 * "Oh, one more thing": 1% of every purchase to carbon removal through Stripe
 * Climate. Ported from paperkites.app's "for the planet" panel, green and all —
 * the one section allowed off the house palette, so its greens are local to
 * this file rather than tokens anything else could reach for. The globe is
 * inline SVG with a fixed aspect ratio, so nothing shifts as the page loads;
 * its gradient and clip ids are per-instance, so two strips never share one.
 */
const climateUrl = 'https://climate.stripe.com/O4dzrx'

const id = useId()
const seaId = `climate-sea-${id}`
const clipId = `climate-clip-${id}`
</script>

<template>
  <section id="climate" class="climate" aria-label="For the planet">
    <div class="frame">
      <div class="copy">
        <p class="eyebrow">For the planet</p>
        <h2 class="title">Oh, one more thing.</h2>
        <p class="lede">
          Don’t forget about the planet. 1% of every purchase goes to carbon removal through
          <a :href="climateUrl" target="_blank" rel="noopener">Stripe Climate</a>.
        </p>

        <div class="actions">
          <div class="stat">
            <span class="pct">1%</span>
            <span class="pct-label">of every purchase,<br>to carbon removal</span>
          </div>
          <a class="button" :href="climateUrl" target="_blank" rel="noopener">
            Stripe Climate <span aria-hidden="true">↗</span>
          </a>
        </div>
      </div>

      <svg class="globe" viewBox="0 0 300 260" width="280" height="243" aria-hidden="true">
        <defs>
          <radialGradient :id="seaId" cx="0.42" cy="0.36" r="0.7">
            <stop offset="0" stop-color="#f7faf3" />
            <stop offset="0.65" stop-color="#dde9d4" />
            <stop offset="1" stop-color="#c3d7b7" />
          </radialGradient>
          <clipPath :id="clipId"><circle cx="150" cy="118" r="92" /></clipPath>
        </defs>
        <ellipse cx="150" cy="242" rx="66" ry="7" fill="#d5e2cc" />
        <path
          d="M58 150 C40 168 60 192 110 194 M232 76 C262 82 268 104 236 132"
          fill="none"
          stroke="#8fae82"
          stroke-width="1.5"
          stroke-dasharray="2 5"
          stroke-linecap="round"
        />
        <circle cx="150" cy="118" r="92" :fill="`url(#${seaId})`" />
        <g :clip-path="`url(#${clipId})`" fill="#5e8a48">
          <path d="M100 52c22-8 40 6 34 30-5 18-14 28-12 48 2 18-14 26-26 18-16-10-24-36-20-60 2-16 10-31 24-36z" />
          <path d="M200 30c18 2 34 16 32 30-2 10-14 10-28 6-16-4-26-12-24-24 2-8 10-13 20-12z" />
          <path d="M188 104c20-6 40 8 42 30 2 20-10 34-24 30-12-4-12-18-22-28-12-12-14-27 4-32z" />
          <path d="M108 180c14-6 30 2 30 18 0 14-12 22-24 20-14-2-20-12-18-22 1-8 5-13 12-16z" />
          <g fill="#8db477">
            <ellipse cx="112" cy="64" rx="9" ry="6" transform="rotate(-30 112 64)" />
            <ellipse cx="214" cy="42" rx="8" ry="5" transform="rotate(20 214 42)" />
            <ellipse cx="208" cy="122" rx="8" ry="5" transform="rotate(30 208 122)" />
          </g>
        </g>
        <g fill="#ffffff">
          <path d="M140 108a8 8 0 0 1 14-5 7 7 0 0 1 12 2 6 6 0 0 1 2 11h-28a5 5 0 0 1 0-8z" />
          <path d="M178 92a5 5 0 0 1 9-3 5 5 0 0 1 8 6h-17a3 3 0 0 1 0-3z" />
          <path d="M98 150a5 5 0 0 1 9-2 4 4 0 0 1 4 7h-15a3 3 0 0 1 2-5z" />
        </g>
        <path d="M128 36a92 92 0 0 1 52 2" fill="none" stroke="#ffffff" stroke-width="4" stroke-linecap="round" />
        <g fill="#8fae82">
          <path d="M238 70c8 4 10 14 4 22-6-6-8-14-4-22z" />
          <path d="M50 150c-4 6-2 14 4 18 4-6 2-14-4-18z" />
        </g>
        <g fill="#9cba8f">
          <circle cx="42" cy="40" r="3" />
          <circle cx="252" cy="20" r="2.5" />
          <circle cx="262" cy="190" r="3" />
          <circle cx="76" cy="214" r="2" />
        </g>
      </svg>
    </div>
  </section>
</template>

<style scoped>
/* paperkites.app's greens, kept here: nothing outside this section uses them. */
.climate {
  --leaf: #3f7a35;
  --leaf-deep: #1f3a1b;
  --leaf-ink: #3d4a39;
  --leaf-muted: #4f5c4b;
  --leaf-line: #a9c79d;
  --leaf-border: #dbe6d4;

  max-width: 1200px;
  margin: 0 auto;
  padding: 128px 24px 0;
}

.frame {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 48px;
  padding: clamp(32px, 5vw, 64px);
  background: linear-gradient(135deg, #f1f6ec, #e7f0df);
  border: 1px solid var(--leaf-border);
  border-radius: var(--radius-lg);
}

.copy {
  flex: 1 1 360px;
  max-width: 680px;
  display: grid;
  gap: 16px;
}

.copy > * { margin: 0; }

.eyebrow {
  font: var(--text-label-mono);
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--leaf);
}

.title {
  font: 500 clamp(30px, 3.4vw, 40px)/1.15 var(--font-serif);
  letter-spacing: -0.015em;
  color: var(--leaf-deep);
}

.lede {
  font: 400 19px/1.75 var(--font-serif);
  color: var(--leaf-ink);
  text-wrap: pretty;
}

.lede a {
  color: var(--leaf);
  text-decoration-color: var(--leaf-line);
  text-underline-offset: 5px;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 20px 40px;
  margin-top: 20px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 20px;
}

.pct {
  font: 400 64px/1 var(--font-serif);
  letter-spacing: -0.03em;
  font-variant-numeric: tabular-nums;
  color: var(--leaf);
}

.pct-label {
  font: 400 15px/1.5 var(--font-sans);
  color: var(--leaf-muted);
}

.button {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 48px;
  padding: 0 24px;
  border-radius: var(--radius-full);
  background: var(--leaf);
  color: #ffffff;
  font: 500 16px/1 var(--font-sans);
  text-decoration: none;
}

.button:hover { opacity: 0.9; }

/* Width and height attributes plus a fixed ratio reserve the box before
   layout; the clamp only scales it. */
.globe {
  display: block;
  flex: 0 1 auto;
  width: clamp(200px, 24vw, 280px);
  height: auto;
  aspect-ratio: 300 / 260;
  margin: 0 auto;
}
</style>
