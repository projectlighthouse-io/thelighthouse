// https://nuxt.com/docs/api/configuration/nuxt-config
import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import tailwindcss from '@tailwindcss/vite'

const appDir = fileURLToPath(new URL('./app', import.meta.url))
const serverDir = fileURLToPath(new URL('./server', import.meta.url))

// What the footer prints. VERSION is the release; the commit comes from the
// build environment (the image has no .git), else from git in a checkout, else
// nothing — the footer then shows the version alone.
const version = readFileSync(new URL('./VERSION', import.meta.url), 'utf8').trim()

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
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      ],
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/lighthouse.svg' },
        { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
        { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
        {
          rel: 'stylesheet',
          // Inter, Libre Baskerville and Geist Mono at the two weights the system
          // uses; Fredericka the Great is the hero h1 and nothing else. The
          // serif italic is for the author's sign-off and the desk's "now.".
          href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500&family=Libre+Baskerville:ital,wght@0,400;1,400&family=Geist+Mono:wght@400;500&family=Fredericka+the+Great&display=swap',
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
    '/**': {
      headers: {
        'cache-control': 'public, max-age=0, s-maxage=600, stale-while-revalidate=86400',
      },
    },

    '/': { prerender: true },

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
          'public, max-age=0, s-maxage=300, stale-while-revalidate=86400',
      },
    },

    '/projects/**': { prerender: true },
    '/blog/**': { prerender: true },
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
    // server out of it entirely.
    // no-store, not just private: these render per session, and the /** rule
    // above would otherwise hand a shared cache permission to keep them
    '/dashboard': { ssr: false, headers: { 'cache-control': 'private, no-store' } },
    '/notes': { ssr: false, headers: { 'cache-control': 'private, no-store' } },
    '/profile': { ssr: false, headers: { 'cache-control': 'private, no-store' } },
    '/settings/**': { ssr: false, headers: { 'cache-control': 'private, no-store' } },

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

  experimental: {
    // one shared payload for prerendered routes rather than inlining state into
    // every HTML file
    payloadExtraction: true,
    // ship the smaller of the two hydration strategies per component
    componentIslands: true,
  },
})