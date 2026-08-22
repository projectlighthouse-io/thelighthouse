// https://nuxt.com/docs/api/configuration/nuxt-config
import { fileURLToPath } from 'node:url'
import tailwindcss from '@tailwindcss/vite'

const appDir = fileURLToPath(new URL('./app', import.meta.url))
const serverDir = fileURLToPath(new URL('./server', import.meta.url))

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
      script: [
        {
          // Runs before first paint so the page never renders light and snaps
          // to dark. Must stay inline and synchronous — deferred or hydrated
          // is worse than having no dark mode at all.
          innerHTML:
            '(function(){try{var s=localStorage.getItem("theme");'
            + 'var d=s==="dark"||(s!=="light"&&matchMedia("(prefers-color-scheme:dark)").matches);'
            + 'var r=document.documentElement;'
            + 'r.classList.toggle("dark",d);r.style.colorScheme=d?"dark":"light";}catch(e){}})()',
          tagPosition: 'head',
          tagPriority: 'critical',
        },
      ],
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/lighthouse.svg' },
        { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
        { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
        {
          rel: 'stylesheet',
          href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Lora:ital,wght@0,400..700;1,400..700&family=JetBrains+Mono:wght@400;500;600&family=Fredericka+the+Great&family=Caveat:wght@400..700&family=Newsreader:opsz,wght@6..72,400..700&family=IM+Fell+English:ital@0;1&display=swap',
        },
      ],
    },
  },

  modules: ['@nuxt/eslint'],

  // Every public page renders from static data, so there is nothing to compute
  // per request — prerender them and serve files. What is left on the server is
  // only what depends on a session.
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
    '/books/**': { prerender: true },
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
      // reachable only from the sitemap, so the crawler would miss them
      routes: ['/robots.txt', '/sitemap.xml'],
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