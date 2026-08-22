# thelighthouse

Deployment for [projectlighthouse.io](https://projectlighthouse.io). One image,
three processes: Caddy in front, the Nuxt frontend and the Rust API behind it.

The frontend lives in [lighthouse-web](https://github.com/projectlighthouse-io/lighthouse-web)
and is pulled in as a submodule at `web/`. This repo owns the API, the routing
and the deploy.

## Shape

```
        :8080  caddy  (the only public listener)
                 │
   /api/*  ──────┼──► :9000  api    signed requests only
   /*      ──────┴──► :3000  nuxt
                          │
                          └──► :9000  api    loopback, never via caddy
```

Nuxt and the API bind loopback. Nothing reaches them except through Caddy, and
Nuxt reads the API over loopback because the frontend is the only thing that
does so on a reader's behalf.

## Running it

```bash
cp .env.example .env   # then fill it in — every key is required
make web               # fetch the frontend submodule
make image             # build the combined image
make run               # runs it, configured from .env
```

| | |
|---|---|
| `make web` | fetch or update the frontend submodule |
| `make image` | build caddy + nuxt + api into one image |
| `make run` | run it locally, reading `.env` |
| `make push` | push to the DO registry |

Deploys are manual. Nothing pushes from CI.

In production there is no `.env` — every key comes from the platform's own
environment, and the API reads whichever is present. What it will not do is
start with a key missing; the boot failure names it.

## The database

Postgres runs from `compose.yaml`, so a clone needs nothing installed:

```bash
make db          # start postgres and wait until it answers
make migrate     # build the schema from migration 0
make db-down     # stop it, keep the data
make db-reset    # stop it and delete the data
```

It publishes **5433**, not 5432, so it can coexist with the Laravel stack's
Postgres. The default connection string is:

```
DATABASE_URL=postgres://lighthouse:lighthouse@127.0.0.1:5433/lighthouse
```

Only Postgres is in compose. The api, Nuxt and Caddy ship as one image built by
the `Dockerfile` — running them a second way here would mean two definitions of
the same thing drifting apart. `make image && make run` runs the site.

## Migrations

```bash
cargo run -p lighthouse-migrate -- status     # what is applied, what is pending
cargo run -p lighthouse-migrate -- run        # apply everything pending
cargo run -p lighthouse-migrate -- baseline   # adopt an existing schema
```

**The API never runs migrations.** It opens a connection pool and stops. A
deploy that doubles as a schema change turns a failed migration into a container
that will not start, and two containers coming up together would race each other
through the same files. Applying them is a decision, so it is a separate binary
you run on purpose.

The migrations are compiled into that binary by `sqlx::migrate!`, so the
deployed artifact carries them — nothing to copy onto the box, no `sqlx-cli` to
install. One consequence worth knowing: adding a migration file needs a rebuild,
because Cargo does not notice a new file appearing in a directory the macro
read. A new migration that "isn't picked up" is a stale binary.

### Migration 0 is the Laravel schema

The rebuild replaces a Laravel app one phase at a time, against the database
that app already owns. So migration 0 is a `pg_dump --schema-only` of it —
41 tables, verbatim. It is a snapshot, not a design: nothing renamed, nothing
pruned. Everything after it is a forward migration written normally.

That gives one file two jobs, and which applies depends on the database:

| database | command | what happens |
|---|---|---|
| fresh — local, CI, a clone | `run` | migration 0 executes and builds the schema |
| one Laravel already built | `baseline` | migration 0 is recorded as applied, and **not** executed |

`run` against the second kind fails on the first `CREATE TABLE`, which is
correct — that schema does not need building, it needs adopting. Either way the
version lands in `_sqlx_migrations`, so from the next forward migration onward
both kinds of database behave identically.

`baseline` is deliberately narrow, because a tool that marks migrations "already
done" is otherwise one mistake from skipping a real one. It only ever records
the **first** migration, refuses on a database that does not already have the
schema, and refuses if that migration is already recorded.

**Why not `CREATE TABLE IF NOT EXISTS` instead?** It would collapse the two
commands into one, and it does not work here. Postgres has no
`ADD CONSTRAINT IF NOT EXISTS` and the dump carries 115 of them, so each would
need wrapping in a `DO` block that swallows `duplicate_object` — turning a
verbatim `pg_dump` into a hand-edited file. More importantly `IF NOT EXISTS`
compares nothing: it silently skips a table that exists with entirely different
columns, reporting a clean migration over a schema that has drifted. Baselining
records what is true instead, and leaves migration 0 meaning "build this from
nothing" — which is what a fresh clone needs it to mean.

## Signing in

Social auth only, GitHub and Google, and the API owns the whole dance — Nuxt
never sees a token and never talks to a provider.

```
GET  /api/auth/{provider}    mint a state, redirect to the provider
GET  /{provider}/callback    verify, exchange, set the session cookie
GET  /api/auth/session       the reader, or null
POST /api/auth/logout        clear the cookie
```

The callback sits at the root while the rest live under `/api/auth`. That
asymmetry is deliberate: its URL is registered with Google and GitHub, so it
keeps the path the Laravel app serves and the rebuild inherits both
registrations. Renaming it would mean editing two OAuth consoles before this
could ship.

The session is an HttpOnly, SameSite=Lax cookie, signed with
`SESSION_SECRET` — there is no session table, because there is no users table
yet. The consequence, written down where it can be found: **a session cannot be
revoked before it expires.** Rotating `SESSION_SECRET` signs everyone out and
is the only blunt instrument available until the users table lands.

Each provider's registered callback must be `APP_URL` plus the callback path
above, byte for byte. An empty client id skips that provider — its route 404s
and the other one still works.

## How the API is protected

Three ways in, and only three:

**luxctl** signs every request with an HMAC over the body. Caddy checks that
`X-Luxctl-Signature` is *present* and routes those to the API; the API verifies
the signature itself, in constant time, and rejects anything it cannot prove.

Caddy is not the security boundary here and must not be treated as one — it
cannot verify an HMAC, so header presence is a filter to keep unsigned traffic
off the API, nothing more. If the API ever stops verifying, the door is open.

**The frontend** reads the API over loopback for server-rendered pages. That
path never crosses Caddy, so it needs no signature and cannot be reached from
outside the container.

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

## Status

`crates/api` is a placeholder. It binds loopback, verifies luxctl signatures and
answers the health probe — the wiring around it is real and tested, so building
the actual backend means replacing that crate and nothing else.

The full plan, including entitlements, the paywall and the payment split, is in
`docs/rebuild.md` in the projectlighthouse.io repo.
