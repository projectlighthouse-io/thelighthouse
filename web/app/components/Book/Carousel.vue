<script setup lang="ts">
import type { Book } from '@/types/Content'

/**
 * The books on the home page, as a row that slides itself.
 *
 * The nav's dropdown is where browsing happens now — every book, grouped by
 * track, one click from any page. So this band's job stopped being "list the
 * catalogue" and became "show that there is one, and what it looks like".
 *
 * The card came from the shelf this replaced — same `.bk` rules, same pencil
 * border — and lives here now, since that component is gone. The sliding is
 * `MarqueeRow`'s, shared with the projects row below it.
 */
defineProps<{ books: Book[] }>()
</script>

<template>
  <MarqueeRow :items="books" width="340px">
    <template #default="{ item }">
      <!--
        One link around the whole card, where the shelf has three elements and a
        lightbox. A cover that zooms is worth a click on a page about the books;
        on a card sliding past, the only thing anyone wants is the book.
      -->
      <NuxtLink :to="`/books/${item.slug}`" class="bk border-pencil">
        <span class="bk-thumb">
          <img v-if="item.thumbnailUrl" :src="item.thumbnailUrl" :alt="item.title" loading="lazy">
        </span>

        <span class="bk-link">
          <span class="bk-title">{{ item.title }}</span>
          <span class="bk-desc">{{ item.description }}</span>
        </span>

        <span class="bk-meta">
          <span>{{ item.pages }} pages</span>
          <span v-if="item.price" class="bk-price">{{ item.price }}</span>
        </span>
      </NuxtLink>
    </template>
  </MarqueeRow>
</template>

<style scoped>
/* ---------- the shelf's card ---------- */

/*
 * Every card the same height, and the meta line on the same baseline.
 *
 * In the shelf these sit in a grid row, which equalises them for free. In a
 * track they size to their own content, so a title that wraps to two lines made
 * one card taller than its neighbours and the row of drawn borders came out
 * ragged. `1fr` on the middle row gives the slack to the blurb instead.
 */
.bk {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto 1fr auto;
    gap: 16px;
    align-items: start;
    width: 100%;
    height: 100%;
    padding: 22px;
    border-radius: 5px;
}

/* Landscape and `contain`, not a portrait tile and `cover`. The covers are
 * drawn wide, with the title lettering running edge to edge — cropped to a
 * 3/4 box they lose the first and last word of their own name. Letterboxing a
 * short one is the cheaper failure. */
.bk-thumb {
    width: 100%;
    aspect-ratio: 16 / 10;
    border-radius: 3px;
    overflow: hidden;
    display: block;
    transition: transform 220ms cubic-bezier(0.2, 0.75, 0.25, 1);
}

.bk-thumb img {
    width: 100%;
    height: 100%;
    object-fit: contain;
}

.bk:hover .bk-thumb {
    transform: translateY(-3px);
}

.bk-link {
    display: block;
    min-width: 0;
}

.bk-title {
    display: block;
    font-family: 'Newsreader', Georgia, serif;
    font-optical-sizing: auto;
    font-weight: 600;
    font-size: 24px;
    line-height: 1.2;
    color: var(--color-read-ink);
    transition: color 140ms;
}

.bk:hover .bk-title {
    color: var(--color-teal-deep);
}

.bk-desc {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    overflow: hidden;
    margin-top: 10px;
    font-family: 'Newsreader', Georgia, serif;
    font-optical-sizing: auto;
    font-size: 16px;
    line-height: 1.72;
    color: var(--color-read-ink-soft);
    text-wrap: pretty;
}

.bk-meta {
    display: flex;
    align-self: end;
    align-items: center;
    gap: 10px;
    padding-top: 7px;
    font-family: 'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace;
    font-size: 10px;
    color: var(--color-read-faint);
    white-space: nowrap;
}

.bk:hover .bk-meta {
    color: var(--color-read-mute);
}

.bk-price {
    color: var(--color-amber);
    background: var(--color-amber-soft);
    padding: 2px 7px;
    border-radius: 20px;
}

@media (prefers-reduced-motion: reduce) {
    .bk-thumb,
    .bk:hover .bk-thumb {
        transition: none;
        transform: none;
    }
}
</style>
