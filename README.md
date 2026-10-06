# thelighthouse

The platform behind [projectlighthouse.io](https://projectlighthouse.io). One
image, three processes: Caddy in front, the Nuxt frontend and the Rust API
behind it.

The frontend is the Nuxt app in `web/`. This repo owns it, the API, the
routing and the deploy. Content — the books, lessons and projects — is the
separate, private ohara repo, read from `CONTENT_PATH`. `fixture/` is a small
synthetic stand-in for it, so a clone builds and tests without access; see
[Without the private repos](#without-the-private-repos).

## What is where

| path | |
|---|---|
| `crates/api` | `lighthouse-api` — the HTTP surface: routes, the luxctl signature boundary, sessions, cache policy |
| `crates/ohara` | reads the content repo — books, lessons, projects, prices. No database, no HTTP. Named after the repo it reads |
| `crates/content` | `lighthouse-content sync` — upserts ohara's books and lessons into postgres |
| `crates/migrate` | `lighthouse-migrate run\|status\|baseline` — the only thing that applies migrations |
| `crates/prices` | `lighthouse-prices status\|apply\|resolve\|catalogue` — reconciles `pricing.yaml` with Stripe and writes the api's `billing.yaml` |
| `crates/billing` | subscription billing behind a trait, with a Stripe driver |
| `crates/loginwith` | OAuth 2.0 sign-in for GitHub and Google — see its [README](crates/loginwith/README.md) |
| `crates/subscriber` | `lighthouse-subscriber` — reads, and revokes, one reader's subscription |
| `web/` | the Nuxt frontend — see its [README](web/README.md) |
| `migrations/` | the schema, compiled into `lighthouse-migrate` |
| `fixture/` | a fake content repo, for tests — see its [README](fixture/README.md) |

## Shape

```
        :8080  caddy  (the only public listener)
                 │
   /api/*  ──────┼──► :9000  api    signed, or a path caddy names
   /*      ──────┴──► :3000  nuxt
                          │
                          └──► :9000  api    loopback, never via caddy
```

Nuxt and the API bind loopback. Nothing reaches them except through Caddy, and
Nuxt reads the API over loopback because the frontend is the only thing that
does so on a reader's behalf.

## Running it

```bash
cp .env.example .env   # then fill it in — every uncommented key is required
make image             # build the combined image
make run               # runs it, configured from .env
```

| | |
|---|---|
| `make image` | build caddy + nuxt + api into one image |
| `make run` | run it locally, reading `.env` |
| `make run-built` | run the last built image against the local postgres |
| `make push` | push to the DigitalOcean registry |

`make help` lists every target.

Production is DigitalOcean App Platform, running the pushed image behind
Cloudflare. Deploys are manual. Nothing pushes from CI.

In production there is no `.env` — every key comes from the platform's own
environment, and the API reads whichever is present. What it will not do is
start with a required key missing; the boot failure names it. The keys
commented out in `.env.example` — rate limits, `DISCOUNT_BANNER`,
`ENSURE_PRICES_AT_BOOT`, Sentry — are optional, and each says its default
there.

At start the container resolves `billing.yaml` from Stripe and applies pending
migrations before anything serves — see `docker/entrypoint.sh`.

## Without the private repos

The content repo, `pricing.yaml` and the Stripe keys are all private, and none
of them is needed to build and test:

```bash
make test                          # no database, no content repo, no stripe
make db                            # then, for the tests whose subject is SQL:
DATABASE_URL=postgres://lighthouse:lighthouse@127.0.0.1:5433/lighthouse make test-db
```

The tests read `fixture/` in place of ohara. To build the image the same way —
which is what CI does:

```bash
cp pricing.sample.yaml pricing.yaml
make image CONTENT_PATH=fixture
```

A clone's `BILLING_PLANS` points at `crates/billing/billing.sample.yaml`, which
parses and boots but cannot charge anybody.

## Running it locally

Caddy fronts development too, so the browser reaches Nuxt and the API exactly as
it will in production — same routing rules, same origin, same boundaries for the
session cookie to cross.

```bash
make up          # postgres + caddy
make migrate     # build the schema

make api                                         # :9000, reloading on a change
cd web && HOST=127.0.0.1 PORT=3000 npm run dev   # :3000
```

Or `make dev`, which runs both in one terminal behind the same Caddy. Both
`make api` and `make dev` need `cargo install cargo-watch`.

Then visit **http://localhost:8000** — never `:3000` directly, or you are testing
a different shape than the one that ships.

`HOST=127.0.0.1` is not optional. Nuxt otherwise binds `[::1]`, IPv6 loopback
only, and Caddy reaches the host over IPv4. The symptom is a 502 from Caddy while
`http://localhost:3000` works perfectly in a browser.

Port 8000 is the origin the project's own OAuth apps have registered, which is
why `.env.example` ships `APP_URL=http://localhost:8000`. `make run` serves on
`8080` instead; set `APP_URL` to match when using it. With your own GitHub or Google OAuth app, register
`<APP_URL>/github/callback` or `<APP_URL>/google/callback`; leave a client id
empty and that provider is simply skipped.

There is one Caddyfile, not a dev copy. A second one would drift, and the drift
would be in the rules that decide what a browser can reach. Only the upstreams
differ, and those are environment variables: in the production image both default
to loopback inside the container; in compose they point at the host.

## The database

Postgres runs from `compose.yaml`, so a clone needs nothing installed:

```bash
make db          # postgres alone, without caddy
make migrate     # build the schema from migration 0
make psql        # open a shell on it
make db-down     # stop it, keep the data
make db-reset    # stop it and delete the data
make down        # stop postgres and caddy together
```

`make psql` takes a query too, for when a shell is more ceremony than the
question deserves:

```bash
make psql ARGS='-c "select slug, status from books"'
```

It publishes **5433**, not 5432, so it can coexist with the Laravel stack's
Postgres. The default connection string is:

```
DATABASE_URL=postgres://lighthouse:lighthouse@127.0.0.1:5433/lighthouse
```

Only Postgres is in compose. The api, Nuxt and Caddy ship as one image built by
the `Dockerfile` — running them a second way here would mean two definitions of
the same thing drifting apart. `make image && make run` runs the site.

## Content

```bash
make content-check    # does ohara parse? the gate, before a deploy
make content-sync     # pull, check, and make a running container reread it
make content-db-sync  # upsert ohara's books and lessons into postgres
```

The first two never touch the database. What a reader reads is served from
disk — ohara is walked at boot and rereadable on SIGHUP — so publishing is a
signal, not a write.

`content-db-sync` is the one that writes, and it writes metadata only: a row per
book and per lesson so that notes, bookmarks, completions and entitlements have
something to point at. No prose goes into postgres, and nothing is written back
into the content repo. It upserts on the uuid each `book.yaml` and `lesson.yaml`
carries, so it is safe to re-run; it deletes nothing, so a lesson that leaves
ohara keeps its row and the notes written against it.

## Migrations

```bash
cargo run -p lighthouse-migrate -- status     # what is applied, what is pending
cargo run -p lighthouse-migrate -- run        # apply everything pending
cargo run -p lighthouse-migrate -- baseline   # adopt an existing schema
```

**The API never runs migrations.** It opens a connection pool and stops.
Applying them is a separate binary, `lighthouse-migrate`.

**The container does run them, at start.** `docker/entrypoint.sh` runs
`lighthouse-migrate run` before the api comes up, because App Platform gives no
console to run it by hand. The cost: a migration that fails is a container that
will not start, and the deploy never comes up — read the log, fix the
migration, deploy again.

The migrations are compiled into that binary by `sqlx::migrate!`, so the
deployed artifact carries them — nothing to copy onto the box, no `sqlx-cli` to
install. One consequence worth knowing: adding a migration file needs a rebuild,
because Cargo does not notice a new file appearing in a directory the macro
read. A new migration that "isn't picked up" is a stale binary.

## Signing in

Social auth only, GitHub and Google, and the API owns the whole dance — Nuxt
never sees a token and never talks to a provider.

```
GET  /api/auth/{provider}    mint a state, redirect to the provider
GET  /{provider}/callback    verify, exchange, resolve a user, open a session
GET  /api/auth/session       the reader, or null
POST /api/auth/logout        delete the session row, clear the cookie
```

The callback sits at the root while the rest live under `/api/auth`. That
asymmetry is deliberate: its URL is registered with Google and GitHub, so it
keeps the path the Laravel app serves and the rebuild inherits both
registrations. Renaming it would mean editing two OAuth consoles before this
could ship.

The session is an HttpOnly, SameSite=Lax cookie carrying 32 random bytes and
nothing else. Those bytes name a row in `sessions`, which holds the user id, the
provider, and a CSRF token for the writes that come later. `last_activity` is a
sliding expiry: thirty days without a request, not thirty days from sign-in.

**Signing out deletes the row**, so a copy of the cookie lifted off a browser
dies with it. That is the reason the session is a row rather than a signed
payload the process can verify without remembering it.

The callback resolves the profile to a `users` row: by the provider's id, else
by email — which is what lets one person sign in with either Google or GitHub
and land in the same account — else a new row with a generated username. A
provider that shares no email address cannot complete sign-in, because
`users.email` is the link key and the column is `NOT NULL`.

Each provider's registered callback must be `APP_URL` plus the callback path
above, byte for byte. An empty client id skips that provider — its route 404s
and the other one still works.

## How the API is protected

Four ways in, and only four:

**[luxctl](https://github.com/projectlighthouse-io/luxctl)** signs every
request: an HMAC-SHA256 over `{unix seconds}.{METHOD}.{path}`, sent as
`X-Luxctl-Signature` with the timestamp in `X-Luxctl-Timestamp`, and good for
five minutes. Caddy checks that `X-Luxctl-Signature` is *present* and routes
those to the API; the API verifies the signature itself, in constant time, and
rejects anything it cannot prove.

Caddy is not the security boundary here and must not be treated as one — it
cannot verify an HMAC, so header presence is a filter to keep unsigned traffic
off the API, nothing more. If the API ever stops verifying, the door is open.

**The frontend** reads the API over loopback for server-rendered pages. That
path never crosses Caddy, so it needs no signature and cannot be reached from
outside the container.

**The browser** reaches the paths the Caddyfile names one by one — sign-in,
notes, bookmarks, articles, settings, the newsletter, billing, project progress,
and a lesson and its public comments — and nothing else. Those authenticate with
the session cookie where they need a reader, and a write made as a reader also
carries the CSRF token minted at sign-in.

**OAuth callbacks and webhooks** are browser navigations and third-party POSTs,
so they cannot carry a luxctl signature. They get their own Caddy rules and
authenticate themselves — OAuth through the provider, webhooks through their own
signature.

Anything else on `/api/*` gets a 404. Not a 403: a 403 confirms the endpoint is
there.

## The single-container trade

Three processes in one container is deliberate — one deploy unit, no
cross-container networking, one App Platform component. The cost is real
though: no per-process restarts, and scaling scales all three together.

What makes it safe is that the entrypoint ties their lifetimes together. If any
one of them exits, the container exits, and the platform restarts a known-good
state rather than leaving a half-running site answering requests.

## Background

This replaces an earlier Laravel app, against the database that app built. The
plan it follows, including entitlements, the paywall and the payment split, is
`docs/rebuild.md` in that app's repository, which is private.

## Licence

**PolyForm Strict 1.0.0** — see [LICENSE](LICENSE).

The source is available to read, study and run for noncommercial purposes. It
is not open source: the licence grants no right to modify or distribute it, or
to use it commercially.

The licence applies to the source code only. It gives you **no rights at all**
to the ProjectLighthouse name, branding, visual design, images or written
content: those are all rights reserved, and may not be copied, reused or
adapted in any way, commercial or not, without written permission — see
[NOTICE](NOTICE). The books, lessons and project briefs live in a separate
private repository, and this repo gives you the platform, not the content.

For permission beyond what the licence allows: thearyanahmed@projectlighthouse.io.

See [CONTRIBUTING.md](CONTRIBUTING.md) before opening an issue or pull request.
