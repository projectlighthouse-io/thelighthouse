<script setup lang="ts">
// Binary ASCII art representation of "projectlighthouse"
const asciiArt = `
 ██████╗ ██████╗  ██████╗      ██╗███████╗ ██████╗████████╗██╗     ██╗ ██████╗ ██╗  ██╗████████╗██╗  ██╗ ██████╗ ██╗   ██╗███████╗███████╗
 ██╔══██╗██╔══██╗██╔═══██╗     ██║██╔════╝██╔════╝╚══██╔══╝██║     ██║██╔════╝ ██║  ██║╚══██╔══╝██║  ██║██╔═══██╗██║   ██║██╔════╝██╔════╝
 ██████╔╝██████╔╝██║   ██║     ██║█████╗  ██║        ██║   ██║     ██║██║  ███╗███████║   ██║   ███████║██║   ██║██║   ██║███████╗█████╗
 ██╔═══╝ ██╔══██╗██║   ██║██   ██║██╔══╝  ██║        ██║   ██║     ██║██║   ██║██╔══██║   ██║   ██╔══██║██║   ██║██║   ██║╚════██║██╔══╝
 ██║     ██║  ██║╚██████╔╝╚█████╔╝███████╗╚██████╗   ██║   ███████╗██║╚██████╔╝██║  ██║   ██║   ██║  ██║╚██████╔╝╚██████╔╝███████║███████╗
 ╚═╝     ╚═╝  ╚═╝ ╚═════╝  ╚════╝ ╚══════╝ ╚═════╝   ╚═╝   ╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚══════╝╚══════╝
`

const logoRef = ref<HTMLElement | null>(null)
const parallaxOffset = ref<number>(0)

const handleScroll = (): void => {
  if (!logoRef.value) return

  const rect = logoRef.value.getBoundingClientRect()
  const elementCenter = rect.top + rect.height / 2
  const viewportCenter = window.innerHeight / 2

  parallaxOffset.value = (elementCenter - viewportCenter) * 0.4
}

onMounted(() => {
  window.addEventListener('scroll', handleScroll, { passive: true })
  handleScroll()
})

onBeforeUnmount(() => {
  window.removeEventListener('scroll', handleScroll)
})
</script>

<template>
  <div
    ref="logoRef"
    class="binary-logo w-full overflow-hidden pt-8 pb-32 sm:pt-10 sm:pb-40 lg:pt-12 lg:pb-48"
  >
    <div
      class="parallax-content transition-transform duration-100 ease-out"
      :style="{ transform: `translateY(${parallaxOffset}px)` }"
    >
      <pre
        class="mx-auto w-fit font-mono text-[4px] leading-[1.2] whitespace-pre text-quiet select-none sm:text-[5px] md:text-[6px] lg:text-[7px]"
      >{{ asciiArt }}</pre>
    </div>
  </div>
</template>

<style scoped>
.binary-logo pre {
  font-family: 'Courier New', Courier, monospace;
  letter-spacing: 0.02em;
}

.binary-logo {
  perspective: 1000px;
}

.parallax-content {
  will-change: transform;
}
</style>
