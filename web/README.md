# lighthouse-web

The frontend for [projectlighthouse.io](https://projectlighthouse.io) — books,
hands-on projects and CLI challenges for people who want to know how the
machinery under their code actually works.

Nuxt 4 renders every page. Books, lessons, projects and the blog come from the
Rust API in `../crates/api`, which reads them from the private content repo and
the database; this app reaches it over loopback, server side only. What is left
in `app/data` and `server/data` is static: site copy, the catalogue generated
by `make catalogue` at the repo root, and the syntax references.

In production this app ships inside the combined image built from the repo
root — see the [root README](../README.md). The `Dockerfile` and `make image`
here build it on its own.

## Running it

```bash
make install   # npm ci — exact lockfile
make dev       # http://localhost:3000
```

`make` on its own lists every target.

| | |
|---|---|
| `make check` | lint, typecheck, build — run this before pushing |
| `make lint` / `make fix` | ESLint |
| `make types` | `vue-tsc` over templates and script blocks |
| `make build` / `make preview` | production build, then serve it |
| `make image` / `make push` | standalone container build, then push to the DO registry |
| `make bench` | benchmark the container, writes an HTML report |
| `make audit` / `make outdated` | dependency health |

Node version lives in `.nvmrc`, which CI reads; the Dockerfiles pin the same
version.

## Layout

```
app/
  assets/css/     design tokens, prose, reader
  components/     grouped by domain: Marketing, Book, Reader, Blog, Nav, Site, …
  composables/    UseTheme, UseSeo, UseReader, UseNotes, UseBilling, …
  data/           static data and the generated Catalogue.ts — no lesson prose
  layouts/        Default
  middleware/     Auth
  pages/          file-based routing
  types/
  utils/
server/
  routes/_api/    frontend's own endpoints; fetch from the Rust API, shape the answer
  data/           syntax-reference markdown — server only, cannot be imported by a page
  middleware/     legacy /{locale}/ redirects
  routes/         robots.txt, sitemap.xml
  utils/          the API client, markdown sanitising
bench/            k6 load harness and HTML report
tests/            SEO checks against a running server
```

## Decisions worth knowing before you change something

**Content never reaches the browser as markdown.** The Rust API renders
lessons to HTML; `server/routes/_api` fetches them from it, and renders blog
articles (sanitised) and the syntax references in `server/data` itself. The
`_api` prefix is deliberate: `/api/*` belongs to the Rust backend, which Caddy
routes to separately. Pages fetch the rendered HTML with `useAsyncData`, which
during SSR calls the handler directly — no HTTP round trip. Importing
`server/data` from a page is not possible, and that is the point: the browser
was previously downloading ~190kb of markdown to display one already-rendered
page.

**The backend decides free versus paid, not the frontend.** A locked lesson's
paid half never enters the client at all — the server picks one half and sends
it. The page renders whatever it is given and shows a CTA when a flag says
there is more. A frontend bug can fail to show content someone paid for; it
cannot leak content they did not.

**Static pages are prerendered; content pages are not.** `/syntax/**`,
`/pricing` and the other pages built from static data are HTML files. The home
page, `/books/**`, `/projects/**` and `/blog/**` render per request, because
their content comes from the API and changes without a deploy — see the
`routeRules` in `nuxt.config.ts`. Session-dependent routes (`/dashboard`,
`/notes`, `/profile`, `/settings/*`, the blog editor) are `ssr: false` —
prerendering them would bake one person's view into a file.

**Themes swap CSS variables, not classes.** Every colour is a token in
`app/assets/css/tokens/colors.css`; `data-theme="dark"` on `<html>` overrides
the variables. No component knows which theme is active. Dark is opt-in only.

**Legacy `/{locale}/` URLs 301 to their unprefixed equivalent.** Those URLs are
indexed and in newsletters. The redirect strips every leading locale segment in
one hop and remembers the language in a cookie.

## Conventions

- `@/` for `app/`, `#server/` for `server/`. Never `../`.
- StudlyCase for files that export code. `app/pages/**` stays lowercase — the
  filename *is* the URL.
- Dependencies are pinned exactly. `.npmrc` sets `save-exact`; use
  `make install`, not `npm install`.
- No banner comments. Comments say why, not what.

## CI

The `web` jobs in `../.github/workflows/ci.yml`: lint, typecheck, unit tests,
build and `npm audit`. The workflow is currently manual-only
(`workflow_dispatch`). Nothing is deployed from CI — `make push` at the repo
root is run by hand.

## Licence

**PolyForm Strict 1.0.0** — see [LICENSE](../LICENSE).

The source is available to read, study and run for noncommercial purposes. It
is not open source: the licence grants no right to modify or distribute it, or
to use it commercially.

The licence covers the code in this repository. The name, branding, visual
design, images and written content are **not** covered by it — all rights are
reserved; see [NOTICE](../NOTICE). The books, lessons and project briefs live
in a separate private repository, and this repo gives you the platform, not the
content.

## Architecture

The full rebuild plan, including the Rust API split, entitlements and the
paywall design, is `docs/rebuild.md` in the earlier Laravel app's repository,
which is private.
