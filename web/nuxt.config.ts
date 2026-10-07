// https://nuxt.com/docs/api/configuration/nuxt-config
import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import tailwindcss from '@tailwindcss/vite'

const appDir = fileURLToPath(new URL('./app', import.meta.url))
const serverDir = fileURLToPath(new URL('./server', import.meta.url))

// What the footer prints. Both come from the build environment first — `make
// image` passes APP_VERSION (the repo's VERSION) and GIT_COMMIT (HEAD) as build
// args, because the image has neither the file nor a .git. Outside docker they
// fall back to the root VERSION file and to git, else to nothing.
function versionOf(): string {
  if (process.env.APP_VERSION) return process.env.APP_VERSION.trim()

  try {
    return readFileSync(new URL('../VERSION', import.meta.url), 'utf8').trim()
  }
  catch {
    return 'dev'
  }
}

const version = versionOf()

function commitOf(): string {
  if (process.env.GIT_COMMIT) return process.env.GIT_COMMIT.slice(0, 8)

  try {
    return execSync('git rev-parse --short=8 HEAD', { stdio: ['ignore', 'pipe', 'ignore'] })
      .toString()
      .trim()
  }
  catch {
    return ''
  }
}

export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  css: ['@/assets/css/main.css'],

  // Declared explicitly rather than relying on the built-in @ so the same names
  // resolve inside server/ too — nitro does not inherit vite's alias map, which
  // is why server routes were reaching for ../../../app.
  alias: {
    '@': appDir,
    '#server': serverDir,
  },

  vite: {
    plugins: [tailwindcss()],
  },

  app: {
    pageTransition: { name: 'page', mode: 'out-in' },
    layoutTransition: { name: 'layout', mode: 'out-in' },

    head: {
      htmlAttrs: { lang: 'en' },
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1, viewport-fit=cover' },
        { name: 'theme-color', content: '#fcfcfc', media: '(prefers-color-scheme: light)' },
        { name: 'theme-color', content: '#202020', media: '(prefers-color-scheme: dark)' },
      ],
      // The icon set, every file in public/ as the brand assets gave it. The
      // .ico first for anything that only reads one; browsers that take svg
      // prefer it, and the pngs cover the rest. See BRAND.md.
      link: [
        { rel: 'icon', href: '/favicon.ico', sizes: '48x48' },
        { rel: 'icon', href: '/favicon.svg', type: 'image/svg+xml' },
        { rel: 'icon', href: '/favicon-32.png', type: 'image/png', sizes: '32x32' },
        { rel: 'icon', href: '/favicon-16.png', type: 'image/png', sizes: '16x16' },
        { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' },
        { rel: 'mask-icon', href: '/safari-pinned-tab.svg', color: '#202020' },
        { rel: 'manifest', href: '/site.webmanifest' },
        { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
        { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
        {
          rel: 'stylesheet',
          // Inter, Libre Baskerville and Geist Mono at the two weights the system
          // uses; Fredericka the Great is the hero h1 and nothing else. The
          // serif italic is for the author's sign-off and the desk's "now.".
          // Newsreader, Lora and JetBrains Mono are the lesson reader's type,
          // carried over from the old app — see `--font-reading*`.
          href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500&family=Libre+Baskerville:ital,wght@0,400;1,400&family=Geist+Mono:wght@400;500&family=Fredericka+the+Great&family=Newsreader:ital,opsz,wght@0,6..72,400;0,6..72,500;0,6..72,600;1,6..72,400;1,6..72,500&family=Lora:wght@400;600&family=JetBrains+Mono:wght@400;700&display=swap',
        },
      ],
    },
  },

  modules: ['@nuxt/eslint'],

  // Server side only — this key never reaches the browser. The api binds
  // loopback and caddy is the only public listener, so an SSR fetch to it never
  // leaves the container. NUXT_API_BASE overrides it.
  runtimeConfig: {
    apiBase: 'http://127.0.0.1:9000',
    public: {
      version,
      commit: commitOf(),
    },
  },

  // Development only, and load-bearing: caddy fronts development too, and it
  // reaches this process as `host.docker.internal:3000` (see `NUXT_UPSTREAM`
  // in compose.yaml). Nuxt's default binding is `localhost`, which a container
  // cannot dial — so `nuxt dev` without this comes up looking healthy on :3000
  // and every request through :8000 is a 502 that names nothing.
  //
  // Not a production concern: the built server binds from the environment, and
  // in the container caddy is on the same loopback.
  devServer: {
    host: '0.0.0.0',
  },

  // Most public pages render from static data, so there is nothing to compute
  // per request — prerender them and serve files. What is left on the server is
  // only what depends on a session, or on content that changes without a build.
  routeRules: {
    // Prerendered HTML is a file on disk, but without a cache header every
    // browser and CDN falls back to its own heuristic — which for a page with
    // no Expires and no max-age usually means refetching every time.
    //
    // max-age=0 keeps the browser honest (it revalidates, and gets a 304), while
    // s-maxage lets a shared cache serve it outright. stale-while-revalidate
    // means a deploy does not cause a latency spike: the CDN keeps serving the
    // old copy while it fetches the new one.
    // stale-while-revalidate is a minute, not a day: the image ships only the
    // current build's /_nuxt, so a day-old document at the edge points at
    // chunks that no longer exist and client navigation dies with it.
    '/**': {
      headers: {
        'cache-control': 'public, max-age=0, s-maxage=600, stale-while-revalidate=60',
      },
    },

    // *Not* prerendered, for the reason `/books/**` is not: the shelf and the
    // project band on it come from ohara by way of the api, and the web image
    // is built from `web/` alone — no api, no database, no network. Prerendering
    // it therefore baked the fallback of every fetch, and the page shipped with
    // both carousels empty. Nothing about that failure was visible in the build
    // log: a refused connection is what `useAsyncData` is told to tolerate.
    //
    // Same cache-control as `/books/**`, mirroring `public_content()` on the
    // api, so the document and the two listings in it expire together.
    '/': {
      prerender: false,
      headers: {
        'cache-control':
          'public, max-age=0, s-maxage=300, stale-while-revalidate=60',
      },
    },

    // *Not* prerendered, unlike everything else public. Books come from ohara,
    // which is reread at runtime on SIGHUP — baking them into files at build
    // time would mean a content fix needs a deploy, which is the thing that
    // design exists to avoid.
    //
    // `prerender: false` is load-bearing and not a default being restated:
    // `nitro.prerender.crawlLinks` follows every link it finds, so these routes
    // get discovered and baked unless they say no here.
    //
    // The cache-control mirrors what the api sends for the same content, so the
    // document and the data it came from expire together. No `Vary: Cookie`:
    // these pages are identical for everyone, which is what lets the edge hold
    // them at all.
    '/books/**': {
      prerender: false,
      headers: {
        'cache-control':
          'public, max-age=0, s-maxage=300, stale-while-revalidate=60',
      },
    },

    // *Not* prerendered, and this one was failing twice over. The listing
    // baked empty like the home page, and because `crawlLinks` discovers routes
    // by following links in what it has already rendered, an empty listing
    // meant no project page was ever found — so `/projects/<slug>` was not
    // merely stale, it did not exist in the output at all.
    '/projects/**': {
      prerender: false,
      headers: {
        'cache-control':
          'public, max-age=0, s-maxage=300, stale-while-revalidate=60',
      },
    },

    // *Not* prerendered any more. The blog is articles readers write, so its
    // content changes without a deploy — baking it at build time would mean a
    // new article appearing only at the next one. Same reasoning as
    // `/books/**`, and `prerender: false` is load-bearing rather than a
    // default restated: `crawlLinks` finds these and bakes them otherwise.
    //
    // The cache-control mirrors what rust sends for `/api/articles`, so the
    // document and the data it came from expire together — which is what
    // bounds how long a taken down article can still be served.
    '/blog': {
      prerender: false,
      headers: {
        'cache-control': 'public, max-age=0, s-maxage=60, stale-while-revalidate=60',
      },
    },
    '/syntax/**': { prerender: true },
    '/pricing': { prerender: true },
    '/roadmap': { prerender: true },
    '/changelog': { prerender: true },
    '/connecting-the-dots': { prerender: true },
    '/support': { prerender: true },
    '/terms': { prerender: true },
    '/privacy': { prerender: true },

    // Session-dependent, so prerendering them would bake one user's view into a
    // file. They are noindex anyway, and rendering them on the client keeps the
    // server out of it entirely. Which is also why noindex is a header here:
    // the html a crawler gets is an empty shell, with no head to carry a meta.
    // no-store, not just private: these render per session, and the /** rule
    // above would otherwise hand a shared cache permission to keep them
    // The editor. Session-dependent, so prerendering it would bake one
    // author's draft into a file.
    //
    // `/blog?author=` is *not* here and must not be: it is the same url as the
    // public listing, so a rule cannot tell the two apart. The page fetches
    // that listing in the browser instead — see `app/pages/blog/index.vue`.
    '/blog/write-something-amazing': { ssr: false, headers: { 'cache-control': 'private, no-store', 'x-robots-tag': 'noindex' } },
    '/blog/edit/**': { ssr: false, headers: { 'cache-control': 'private, no-store', 'x-robots-tag': 'noindex' } },

    // Every other /blog/** is an article page: public, and the same bytes for
    // everyone. Listed after the three above so those win.
    '/blog/**': {
      prerender: false,
      headers: {
        'cache-control': 'public, max-age=0, s-maxage=60, stale-while-revalidate=60',
      },
    },

    '/dashboard': { ssr: false, headers: { 'cache-control': 'private, no-store', 'x-robots-tag': 'noindex' } },
    '/notes': { ssr: false, headers: { 'cache-control': 'private, no-store', 'x-robots-tag': 'noindex' } },
    '/profile': { ssr: false, headers: { 'cache-control': 'private, no-store', 'x-robots-tag': 'noindex' } },
    '/settings/**': { ssr: false, headers: { 'cache-control': 'private, no-store', 'x-robots-tag': 'noindex' } },

    // Per country, so never shared — see the handler.
    '/_api/billing/offer': { headers: { 'cache-control': 'private, no-store' } },

    // Hashed filenames, so they can never go stale.
    '/_nuxt/**': { headers: { 'cache-control': 'public, max-age=31536000, immutable' } },
  },

  nitro: {
    alias: {
      '@': appDir,
      '#server': serverDir,
    },

    // No devProxy. Caddy fronts development too — `make up` in the thelighthouse
    // repo — so `/api/*` and the OAuth callbacks reach the rust api the same way
    // they do in production, and this app is reached at :8000 rather than :3000.
    //
    // A proxy here would be a second way in, and then the shape you tested would
    // be whichever you happened to start. The session cookie is the thing that
    // suffers: it crosses different boundaries under each, which is exactly the
    // bug that does not reproduce locally.

    // brotli + gzip beside every public asset, so the CDN serves the compressed
    // copy instead of compressing on the fly
    compressPublicAssets: { brotli: true, gzip: true },

    prerender: {
      crawlLinks: true,
      failOnError: false,
      // Not reachable by the crawler, so it has to be named.
      //
      // `/sitemap.xml` is *not* here any more: it lists every book and lesson,
      // those come from ohara, and ohara is reread at runtime. Baked at build
      // time it would advertise the previous set of lessons until the next
      // deploy.
      routes: ['/robots.txt'],
    },
  },

  hooks: {
    // /blog/new, /blog/create and /blog/write are the paths people (and old
    // links) reach for. Registered as router redirects, so they resolve on the
    // client too rather than only on a full page load.
    'pages:extend'(pages) {
      for (const path of ['/blog/new', '/blog/create', '/blog/write']) {
        pages.push({
          name: `blog-write-redirect-${path.split('/').pop()}`,
          path,
          redirect: '/blog/write-something-amazing',
        })
      }
    },
  },

  experimental: {
    // one shared payload for prerendered routes rather than inlining state into
    // every HTML file
    payloadExtraction: true,
    // ship the smaller of the two hydration strategies per component
    componentIslands: true,
  },
})
