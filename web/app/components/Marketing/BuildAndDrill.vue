<script setup lang="ts">
import type { Project } from '@/types/Content'

/**
 * Projects and challenges in one grid, with a filter between them.
 *
 * `fade` is the home band: past six cards the grid fades out under a
 * "see all" button. The projects page renders the whole list with no fade.
 * The filter is client-side; `?kind=` keeps it in the url on the projects page
 * so a tab is a link somebody can send.
 */
type Kind = 'all' | 'project' | 'challenge'

const props = withDefaults(
  defineProps<{
    projects: Project[]
    fade?: boolean
    eyebrow?: string
    heading?: 'h1' | 'h2'
    syncUrl?: boolean
  }>(),
  { fade: false, eyebrow: '03 — build and drill', heading: 'h2', syncUrl: false },
)

const route = useRoute()
const router = useRouter()

const isKind = (value: unknown): value is Kind =>
  value === 'all' || value === 'project' || value === 'challenge'

const initial = props.syncUrl && isKind(route.query.kind) ? route.query.kind : 'all'
const kind = ref<Kind>(initial)

watch(kind, (chosen) => {
  if (!props.syncUrl) return

  router.replace({ query: chosen === 'all' ? {} : { kind: chosen } })
})

const projectsOnly = computed(() => props.projects.filter(p => !p.isChallenge))
const challengesOnly = computed(() => props.projects.filter(p => p.isChallenge))

// Projects first, then challenges — the order the band reads in.
const everything = computed(() => [...projectsOnly.value, ...challengesOnly.value])

const shown = computed<Project[]>(() => {
  if (kind.value === 'project') return projectsOnly.value
  if (kind.value === 'challenge') return challengesOnly.value

  return everything.value
})

const options = computed(() => [
  { key: 'all' as const, label: 'Everything', count: everything.value.length },
  { key: 'project' as const, label: 'Projects', count: projectsOnly.value.length },
  { key: 'challenge' as const, label: 'Challenges', count: challengesOnly.value.length },
])

const seeAll = computed<{ label: string, to: string }>(() => {
  if (kind.value === 'project') return { label: 'See all projects →', to: '/projects?kind=project' }
  if (kind.value === 'challenge') return { label: 'See all challenges →', to: '/projects?kind=challenge' }

  return { label: 'See everything →', to: '/projects' }
})

const overflowing = computed<boolean>(() => props.fade && shown.value.length > 6)

// Six cards in full and one more row for the gradient to fall across.
const cards = computed<Project[]>(() => (overflowing.value ? shown.value.slice(0, 8) : shown.value))
</script>

<template>
  <section id="build" class="build lh-wide lh-gap-lg">
    <div class="head">
      <p class="lh-eyebrow">{{ eyebrow }}</p>
      <component :is="heading" class="title">Rebuild the software you use every day</component>
      <p class="sub">
        Multi-stage projects that take weeks, and single-sitting challenges that take an evening.
        Same rules for both: any language, and the tests talk to your binary.
      </p>
    </div>

    <div class="filter">
      <SegmentedFilter v-model="kind" :options="options" label="filter by kind" />
    </div>

    <div class="grid-wrap" :class="{ 'is-clipped': overflowing }">
      <div class="grid">
        <CatalogueCard v-for="project in cards" :key="project.slug" :project="project" />
      </div>

      <p v-if="shown.length === 0" class="lh-sub empty">Nothing here yet.</p>

      <template v-if="fade">
        <div v-if="overflowing" class="fade" aria-hidden="true" />
        <div class="more">
          <UiButton variant="inverse" size="lg" :to="seeAll.to">{{ seeAll.label }}</UiButton>
        </div>
      </template>
    </div>
  </section>
</template>

<style scoped>
.head {
  max-width: var(--measure-text);
  margin: 0 auto;
  display: grid;
  gap: var(--space-5);
  text-align: center;
}

.title {
  margin: 0;
  font: 400 56px/64px var(--font-serif);
  letter-spacing: -0.01em;
  text-wrap: balance;
}

.sub {
  margin: 0;
  justify-self: center;
  max-width: 560px;
  font: var(--text-body);
  color: var(--ink-secondary);
  text-wrap: pretty;
}

.filter {
  display: flex;
  justify-content: center;
  margin-top: var(--space-12);
}

.grid-wrap {
  position: relative;
  margin-top: 40px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-4);
}

.fade {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 260px;
  pointer-events: none;
  background: linear-gradient(180deg, rgba(252, 252, 252, 0), var(--surface-page) 78%);
}

.more {
  display: flex;
  justify-content: center;
  margin-top: 40px;
}

.is-clipped .more {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 56px;
  margin: 0;
}

.empty { text-align: center; }

@media (max-width: 700px) {
  .grid { grid-template-columns: minmax(0, 1fr); }
  .title { font-size: 40px; line-height: 48px; }
}
</style>
