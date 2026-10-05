<script setup lang="ts">
/**
 * Paperkites, the notes app the chapters are written in: what it is on the
 * left, a screenshot gallery on the right. Only the selected screenshot is in
 * the DOM, so the stage sizes to it — four stacked and hidden would leave it
 * the height of the tallest. The images are paperkites.app's own.
 */
interface Shot {
  label: string
  src: string
}

const features = [
  { key: 'offline', text: 'No account, no server, no spinner.' },
  { key: 'private', text: 'No analytics, no telemetry, no model training.' },
  { key: '~/notes', text: 'Every note is a file. Grep it, keep it in git.' },
  { key: 'obsidian', text: 'Compatible with Obsidian. Open your vault as it is.' },
]

const shots: Shot[] = [
  { label: 'split panel', src: 'https://paperkites.app/images/split_view_of_graph_and_note.webp' },
  { label: 'fuzzy search', src: 'https://paperkites.app/images/fuzzy_finder.webp' },
  { label: 'the whole vault', src: 'https://paperkites.app/images/graph_full_view.webp' },
  { label: 'connections', src: 'https://paperkites.app/images/graph_neighbour_view.webp' },
]

const pad = (n: number): string => String(n).padStart(2, '0')

const selected = ref(0)
const active = computed<Shot | undefined>(() => shots[selected.value])
</script>

<template>
  <section id="paperkites" class="paperkites">
    <div class="frame">
      <div class="copy">
        <div class="top">
          <p class="eyebrow">07 — paperkites, by projectlighthouse</p>
          <h2 class="title">These chapters are written in <span>Paperkites</span>.</h2>
          <p class="lede">
            A quiet <strong>markdown</strong> notes app for macOS and Linux that I build alongside
            the shelf. Plain .md files you own, Vim keys, backlinks and a graph view. Every draft
            on the desk above lives in it.
          </p>
          <ul class="features">
            <li v-for="f in features" :key="f.key">
              <span class="key">{{ f.key }}</span>{{ f.text }}
            </li>
          </ul>
        </div>

        <div class="bottom">
          <div class="actions">
            <UiButton variant="inverse" cta="pro" to="https://paperkites.app/" target="_blank">
              Download
            </UiButton>
            <a href="https://paperkites.app/#premium" target="_blank" rel="noopener" class="more">
              see what's in the box →
            </a>
          </div>
          <p class="fine">free · Apple silicon · compatible with Obsidian</p>
        </div>
      </div>

      <div class="gallery">
        <div class="stage">
          <img v-if="active" :key="active.src" :src="active.src" :alt="`paperkites: ${active.label}`" loading="lazy">
        </div>

        <div class="tabs" role="tablist" aria-label="paperkites screenshots">
          <button
            v-for="(shot, i) in shots"
            :key="shot.src"
            type="button"
            role="tab"
            :aria-selected="i === selected"
            class="tab"
            @click="selected = i"
          >
            <span class="n">{{ pad(i + 1) }} /</span>
            <span class="tab-label">{{ shot.label }}</span>
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.paperkites {
  max-width: 1200px;
  margin: 0 auto;
  padding: 128px 24px 0;
}

.frame {
  display: grid;
  grid-template-columns: minmax(0, 5fr) minmax(0, 7fr);
  gap: 8px;
  padding: 8px;
  background: var(--surface-sunken);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
}

/* left: the copy */

.copy {
  display: grid;
  align-content: space-between;
  gap: 40px;
  padding: 32px 28px 28px;
}

.top {
  display: grid;
  gap: 20px;
}

.top > * { margin: 0; }

.eyebrow {
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

/* muted, so only the product name reads dark */
.title {
  font: var(--text-h1);
  letter-spacing: var(--tracking-h1);
  text-wrap: balance;
  color: var(--ink-muted);
}

.title span { color: var(--ink); }

.lede {
  font: var(--text-body-sm);
  color: var(--ink-secondary);
  text-wrap: pretty;
}

.lede strong {
  font-weight: var(--weight-medium);
  color: var(--ink);
  text-transform: lowercase;
}

.features {
  padding: 8px 0 0;
  list-style: none;
  border: 0;
  display: grid;
  gap: 10px;
}

.features li {
  display: grid;
  grid-template-columns: 96px 1fr;
  gap: 12px;
  font: var(--text-body-sm);
  color: var(--ink);
}

.key {
  padding-top: 4px;
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

.bottom {
  display: grid;
  gap: 12px;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.more {
  font: var(--text-caption);
  color: var(--ink-secondary);
  text-decoration: none;
}

.more:hover { color: var(--ink); }

.fine {
  margin: 0;
  font: var(--text-label-mono);
  color: var(--ink-faint);
}

/* right: the gallery */

/* Sized by the image, not stretched to the copy: when the left column runs
   taller than a wide screenshot, a 1fr stage would pad it with an empty band. */
.gallery {
  display: grid;
  grid-template-rows: auto auto;
  align-content: start;
  gap: 8px;
  min-width: 0;
}

.stage {
  padding: 24px;
  background: var(--surface-raised);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-sm);
}

/* One fixed box for every screenshot, so switching tabs (or the image
   arriving late, being lazy) never moves anything around it. `contain`
   rather than `cover`: a cropped app window reads as broken. */
.stage img {
  display: block;
  width: 100%;
  height: auto;
  aspect-ratio: 16 / 10;
  object-fit: contain;
  border-radius: 8px;
  box-shadow: var(--shadow-hairline);
}

.tabs {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}

.tab {
  display: flex;
  align-items: baseline;
  gap: 6px;
  min-width: 0;
  padding: 8px 12px;
  border: 0;
  border-radius: var(--radius-md);
  background: transparent;
  text-align: left;
  cursor: pointer;
  transition: background var(--duration) var(--ease-out);
}

.tab:hover,
.tab[aria-selected="true"] { background: var(--surface-raised); }

.tab[aria-selected="true"] { box-shadow: var(--shadow-sm); }

.n {
  flex: none;
  font: var(--text-label-mono);
  color: var(--ink-faint);
}

.tab-label {
  font: var(--text-caption);
  color: var(--ink-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab[aria-selected="true"] .tab-label { color: var(--ink); }

@media (max-width: 960px) {
  .frame { grid-template-columns: minmax(0, 1fr); }
}

@media (max-width: 600px) {
  .tabs { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
</style>
