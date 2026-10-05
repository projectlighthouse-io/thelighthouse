# lighthouse-web

The frontend for [projectlighthouse.io](https://projectlighthouse.io) — books,
hands-on projects and CLI challenges for people who want to know how the
machinery under their code actually works.

Nuxt 4 renders every page. A Rust API will own the data; until it does, the
content in `app/data` and `server/data` is real content extracted from the
private content repo, so the pages render the same shapes the API will return.

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
| `make image` / `make push` | container build, then push to the DO registry |
| `make bench` | benchmark the container, writes an HTML report |
| `make audit` / `make outdated` | dependency health |

Node version lives in `.nvmrc` and is shared by CI and the Dockerfile.

## Layout

```
app/
  assets/css/     theme tokens, the pencil border system, prose, reader
  components/     grouped by domain: Marketing, Book, Project, Challenge, Chrome
  composables/    UseTheme, UseSeo, UsePreviewAuth
  data/           content metadata — no prose
  layouts/        Default, Settings
  middleware/     Auth
  pages/          29 pages, file-based routing
  types/
server/
  routes/_api/    frontend's own endpoints; renders markdown and picks the body
  data/           markdown — server only, cannot be imported by a page
  middleware/     legacy /{locale}/ redirects
  routes/         robots.txt, sitemap.xml
  utils/
bench/            k6 load harness and HTML report
```

## Decisions worth knowing before you change something

**Content never reaches the browser as markdown.** `server/data` holds the
prose and `server/routes/_api` renders it. The `_api` prefix is deliberate:
`/api/*` belongs to the Rust backend, which Caddy routes to separately. Pages fetch the rendered HTML with
`useAsyncData`, which during SSR calls the handler directly — no HTTP round
trip. Importing `server/data` from a page is not possible, and that is the
point: the browser was previously downloading ~190kb of markdown to display one
already-rendered page.

**The backend decides free versus paid, not the frontend.** A locked lesson's
paid half never enters the client at all — the server picks one half and sends
it. The page renders whatever it is given and shows a CTA when a flag says
there is more. A frontend bug can fail to show content someone paid for; it
cannot leak content they did not.

**Pages are prerendered.** 275 static HTML files. Only session-dependent routes
(`/dashboard`, `/notes`, `/profile`, `/settings/*`) render at request time, and
those are `ssr: false` — prerendering them would bake one person's view into a
file.

**Themes swap CSS variables, not classes.** Every colour is a token in
`app/assets/css/theme.css`; `.dark` overrides the variables. No component knows
which theme is active. An inline script in `nuxt.config.ts` sets the class
before first paint, so there is no flash.

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

Runs on pull requests and pushes to `master`: lint, typecheck, build, `npm
audit`, and a container build that boots the image and curls it. Nothing is
deployed from CI — `make push` is run by hand.

## Licence

**PolyForm Strict 1.0.0** — see [LICENSE](../LICENSE).

The source is available to read, study and run for noncommercial purposes. It
is not open source: the licence grants no right to modify or distribute it.

The licence covers the code in this repository. The name, branding, visual
design, images and written content are **not** covered by it — all rights are
reserved; see [NOTICE](../NOTICE). The books, lessons and project briefs live
in a separate private repository, and forking this repo gives you the platform,
not the content.

## Architecture

The full rebuild plan, including the Rust API split, entitlements and the
paywall design, is in `docs/rebuild.md` in the projectlighthouse.io repo.
