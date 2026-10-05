<template>
  <section id="about" class="lh-text lh-gap-lg">
    <h2 class="lh-h2 heading">And a letter from the author…</h2>

    <div class="sheet">
      <div class="stamp" aria-hidden="true">
        <UiLogo variant="mark" :size="24" class="stamp-mark" />
      </div>

      <p class="lh-muted">to whoever reads this,</p>
      <p>Hi, I'm Aryan.</p>
      <p>
        The best way to learn programming is to build systems. You need the tools, and you also
        need the deeper understanding of how it works.
      </p>
      <p>
        Over the years, I've been part of teams that built large-scale systems from the ground up,
        kept them up and running under heavy load, and scaled them to meet growing demand.
        Currently I'm at
        <a href="https://www.digitalocean.com" target="_blank" rel="noopener" class="chip"><span class="dot" aria-hidden="true" />DigitalOcean</a>;
        building the simple cloud.
      </p>
      <p>
        I am building projectlighthouse to bridge some of the gap between theory, practice, tools
        and systems programming. This would be my approach if I could go back in time and teach
        myself. I hope you find it useful.
      </p>
      <p class="last">
        If you want to understand how projectlighthouse.io works,
        <NuxtLink to="/connecting-the-dots" class="lh-inline">read here in detail</NuxtLink>.
        If you get stuck, write to me — I answer.
      </p>
      <p class="welcome">Welcome to <strong>projectlighthouse.io</strong></p>

      <p class="sign">Aryan</p>
      <p class="email">thearyanahmed@projectlighthouse.io</p>
    </div>
  </section>
</template>

<style scoped>
.heading { margin-bottom: 40px; }

.sheet {
  position: relative;
  padding: 56px 56px var(--space-12);
  border-radius: var(--radius-md);
  background: var(--surface-raised);
  box-shadow: var(--shadow-sm);
  font: var(--text-code);
  line-height: 28px;
  color: var(--ink);
}

.sheet p { margin: 0 0 28px; text-wrap: pretty; }
.sheet p.last { margin-bottom: 40px; }
.sheet p.welcome { margin: 0; }

strong { font-weight: var(--weight-medium); }

.stamp {
  position: absolute;
  top: var(--space-6);
  right: var(--space-6);
  display: grid;
  place-items: center;
  width: 56px;
  height: 68px;
  padding: var(--space-1);
  background: var(--surface-raised);
  box-shadow: 0 0 0 3px var(--surface-raised), 0 0 0 4px var(--border);
  transform: rotate(3deg);
}

/* Counter-rotated: the stamp tilts, the mark it carries must not. */
.stamp-mark { opacity: 0.75; transform: rotate(-3deg); }

.chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 1px 10px 1px 6px;
  border: 1.5px solid var(--ink);
  border-radius: var(--radius-full);
  color: var(--ink);
  font-weight: var(--weight-medium);
  line-height: 22px;
  text-decoration: none;
  vertical-align: -1px;
  transition: background-color var(--duration) var(--ease-out);
}

.chip:hover { background: var(--surface-sunken); color: var(--ink); }

.dot {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--ink);
}

.sheet p.sign {
  margin: 40px 0 0;
  font: 400 26px/32px var(--font-serif);
}

.sheet p.email {
  margin: 6px 0 0;
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

@media (max-width: 600px) {
  .sheet { padding: 72px var(--space-6) 40px; }
}
</style>
