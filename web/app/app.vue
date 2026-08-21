<template>
  <div>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>

    <!-- Filter behind every .border-pencil / .btn-chalk. These attribute names
         are camelCase in the SVG spec and case-sensitive — kebab-case silently
         produces no displacement and the borders render flat. -->
    <svg aria-hidden="true" class="absolute size-0">
      <defs>
        <filter id="pencil-filter">
          <feTurbulence type="fractalNoise" baseFrequency="0.04" numOctaves="5" result="noise" />
          <feDisplacementMap
            in="SourceGraphic"
            in2="noise"
            scale="2"
            xChannelSelector="R"
            yChannelSelector="G"
          />
        </filter>
        <filter id="wavy-filter">
          <feTurbulence type="turbulence" baseFrequency="0.015" numOctaves="2" result="noise" />
          <feDisplacementMap
            in="SourceGraphic"
            in2="noise"
            scale="5"
            xChannelSelector="R"
            yChannelSelector="G"
          />
        </filter>
      </defs>
    </svg>
  </div>
</template>
