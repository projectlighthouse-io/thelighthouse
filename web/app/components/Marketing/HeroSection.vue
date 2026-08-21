<script setup lang="ts">
import type { HeroStat } from '@/types/Content'

const props = withDefaults(
  defineProps<{
    stats: HeroStat[]
    title?: string
    kicker?: string
    lede?: string
    phrases?: string[]
    primaryHref?: string
    secondaryHref?: string
  }>(),
  {
    title: 'fundamentals of software engineering ;',
    kicker: 'learn to build,',
    lede: 'a handcrafted shelf of books on systems, networks, and the runtime under your code — with projects you build, break, and ship on your own machine.',
    phrases: () => ['beyond the framework', 'learn by doing', 'build it yourself'],
    primaryHref: '/pricing',
    secondaryHref: '/books',
  },
)

const currentIndex = ref<number>(0)
const isAnimating = ref<boolean>(false)
let timer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  timer = setInterval(() => {
    isAnimating.value = true
    setTimeout(() => {
      currentIndex.value = (currentIndex.value + 1) % props.phrases.length
      isAnimating.value = false
    }, 300)
  }, 3000)
})

onBeforeUnmount(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <section class="relative isolate overflow-hidden">
    <div class="mx-auto max-w-7xl px-4 pt-10 pb-12 sm:pb-16 lg:flex lg:px-8">
      <div class="mx-auto max-w-2xl rounded-md bg-page p-6 lg:mx-0 lg:shrink-0 lg:pt-8">
        <p class="font-mono text-xs tracking-[0.2em] uppercase text-teal sm:mt-12 lg:mt-16">
          {{ kicker }} <span class="italic text-quiet">not just prompt</span>
        </p>

        <h1
          class="font-fredericka mt-3 text-5xl tracking-tight text-pretty text-ink sm:text-7xl"
        >
          {{ title }}
        </h1>

        <p
          class="mt-4 overflow-hidden text-2xl font-medium tracking-tight text-ink sm:text-3xl"
        >
          <span
            class="inline-block transition-all duration-300 ease-in-out"
            :class="isAnimating ? '-translate-y-full opacity-0' : 'translate-y-0 opacity-100'"
          >
            {{ phrases[currentIndex] }}
          </span>
        </p>

        <p class="mt-8 font-serif text-lg leading-relaxed text-ink sm:text-xl">
          {{ lede }}
        </p>

        <div
          class="mt-8 grid grid-cols-2 gap-x-4 gap-y-4 border-y border-rule py-5 sm:flex sm:flex-wrap sm:items-stretch sm:gap-0 sm:divide-x sm:divide-rule"
        >
          <div
            v-for="stat in stats"
            :key="stat.label"
            class="flex flex-col items-start sm:min-w-[5rem] sm:px-4"
          >
            <span class="font-serif text-3xl text-ink">{{ stat.value }}</span>
            <span class="mt-1 font-mono text-xs text-faint">{{ stat.label }}</span>
          </div>
        </div>

        <div class="mt-10 flex flex-col items-stretch gap-3 sm:flex-row sm:items-center sm:gap-4">
          <NuxtLink
            :to="primaryHref"
            class="w-full rounded-md bg-ink px-5 py-3 text-center text-base font-medium text-on-ink transition hover:bg-ink-hover sm:w-auto"
          >
            Get Pro
          </NuxtLink>
          <NuxtLink
            :to="secondaryHref"
            class="w-full rounded-md border border-stroke bg-panel px-5 py-3 text-center text-base font-medium text-ink transition hover:bg-paper-warm sm:w-auto"
          >
            Start Free
          </NuxtLink>
        </div>
      </div>

      <div
        class="mx-auto mt-16 flex max-w-2xl sm:mt-24 lg:my-auto lg:mr-0 lg:ml-10 lg:max-w-none lg:flex-none xl:ml-20"
      >
        <div class="relative max-w-3xl flex-none sm:max-w-5xl lg:max-w-none">
          <div class="w-[40rem] lg:w-[50rem] xl:w-[56rem]">
            <MarketingHeroTerminal />
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
