<script setup lang="ts" generic="T">
/**
 * A row of things that slides itself, endlessly.
 *
 * The mechanism only — what rides on it is the caller's, through the slot. It
 * exists as its own component because the seam is the hard part and it should
 * be got right once: the track holds the items twice and slides exactly half
 * its width less half a gap, so the copy arrives where the original began. Any
 * other distance shows a jump, once per pass, that is easy to see and hard to
 * name.
 *
 * **A CSS animation, not a timer.** Nothing measures, nothing ticks, and it
 * cannot drift out of step with a scroll position the way an interval calling
 * `scrollBy` does.
 */
const props = withDefaults(defineProps<{
  items: T[]
  /** Right to left by default; `reverse` runs it the other way. */
  reverse?: boolean
  /** How wide one item is. A track of mixed widths cannot have a clean seam. */
  width?: string
  /** Seconds each item takes to cross, so the speed holds however many there
   *  are — a fixed duration makes a long row race and a short one crawl. */
  secondsPerItem?: number
}>(), {
  reverse: false,
  width: '340px',
  secondsPerItem: 7,
})

defineSlots<{ default: (props: { item: T }) => unknown }>()

const duration = computed<string>(
  () => `${Math.max(props.items.length, 1) * props.secondsPerItem}s`,
)
</script>

<template>
  <!--
    Hover and focus pause it. Moving content that cannot be stopped is unusable
    for anybody reading slowly, and unclickable for anybody whose pointer is
    chasing a card across the screen.
  -->
  <div
    class="marquee"
    :class="{ 'marquee--reverse': reverse }"
    :style="{ '--marquee-duration': duration, '--marquee-item': width }"
  >
    <ul class="marquee__track">
      <li v-for="(item, i) in items" :key="`a-${i}`" class="marquee__item">
        <slot :item="item" />
      </li>

      <!--
        The second pass. `aria-hidden` and `inert`, because it is the same items
        — a screen reader reading the row twice, or a tab key walking it twice,
        would be describing a trick of the animation rather than anything on the
        page.
      -->
      <li v-for="(item, i) in items" :key="`b-${i}`" class="marquee__item" aria-hidden="true" inert>
        <slot :item="item" />
      </li>
    </ul>
  </div>
</template>

<style scoped>
.marquee {
    overflow: hidden;
}

.marquee__track {
    display: flex;
    gap: 20px;
    margin: 0;
    padding: 0;
    list-style: none;
    width: max-content;
    animation: marquee-slide var(--marquee-duration, 60s) linear infinite;
}

/* Half the track is the first set, so translating exactly -50% lands the copy
   where the original started — less half a gap, because the gap sits between
   all the items and not only inside each half. */
@keyframes marquee-slide {
    from {
        transform: translateX(0);
    }

    to {
        transform: translateX(calc(-50% - 10px));
    }
}

/* The same slide, run backwards: the row starts half-shifted and returns to
   zero, so it reads as coming from the left with no second keyframe block. */
.marquee--reverse .marquee__track {
    animation-direction: reverse;
}

.marquee:hover .marquee__track,
.marquee:focus-within .marquee__track {
    animation-play-state: paused;
}

.marquee__item {
    display: flex;
    flex: 0 0 auto;
    width: var(--marquee-item, 340px);
}

/* The card fills the height the track stretches its item to, so a row of them
   ends level however long one title runs. */
.marquee__item > :deep(*) {
    width: 100%;
}

/*
 * Asked not to be moved: the track stops and the row becomes an ordinary
 * scroller instead, so everything is still reachable — a stopped marquee with
 * `overflow: hidden` would hide most of it.
 */
@media (prefers-reduced-motion: reduce) {
    .marquee {
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        padding-bottom: 14px;
    }

    .marquee__track {
        animation: none;
    }

    .marquee__item {
        scroll-snap-align: start;
    }
}
</style>
