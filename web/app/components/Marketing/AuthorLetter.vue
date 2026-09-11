<template>
  <section id="about" class="py-12">
    <div class="mx-auto max-w-3xl px-4 sm:px-6 lg:px-8">
      <h2 class="mb-8 font-serif text-3xl tracking-tight text-ink sm:text-4xl">
        and a letter from the author...
      </h2>
      <div
        class="letter-paper border-pencil-solid-black relative bg-panel px-2 py-12 sm:rotate-[-0.4deg] sm:px-16 sm:py-32"
      >
        <div class="space-y-5 font-mono text-sm leading-relaxed text-mono-ink">
          <p>Hi, I'm Aryan</p>
          <p>
            The best way to learn programming is to build systems. You need the tools, and you also
            need the deeper understanding on how it works.
          </p>
          <p>
            Over the years, I've been part of teams that built large scale systems from the ground
            up, keep up and running under heavy load, and scale to meet growing demand. Currently,
            I'm at
            <span
              class="border-pencil mx-1 inline-flex items-center gap-1.5 rounded-full !bg-white px-3 py-1 align-middle"
            >
              <img src="/do-black.svg" alt="DigitalOcean" class="h-4 w-auto">
              <span class="text-sm font-medium text-ink">DigitalOcean</span>
            </span>
            <span class="text-gray-400">;</span> building the simple cloud.
          </p>

          <p>
            I am building projectlighthouse to bridge some of the gap between theory, practice,
            tools and systems programming. This would be my approach if I could go back in time and
            teach myself. I hope you find it useful!
          </p>

          <p>
            If you want to understand how projectlighthouse.io works,
            <NuxtLink
              to="/connecting-the-dots"
              class="font-medium text-ink underline underline-offset-2 transition hover:text-quiet"
            >read here in detail</NuxtLink>.
          </p>

          <p class="pt-4">Welcome to <span class="font-bold">projectlighthouse.io</span></p>
        </div>
      </div>
    </div>
  </section>
</template>
