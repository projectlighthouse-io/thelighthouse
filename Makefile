PORT ?= 8080

# Match compose.yaml's defaults, and overridable the same way: the file reads
# these from the environment too, so `POSTGRES_DB=other make psql` follows
# whatever `POSTGRES_DB=other make db` started.
POSTGRES_USER ?= lighthouse
POSTGRES_DB   ?= lighthouse

# Where caddy listens on the host. 8000 matches APP_URL and the oauth callbacks
# already registered with google and github.
CADDY_PORT ?= 8000

REGISTRY := registry.digitalocean.com/lighthouse-registry
IMAGE    := thelighthouse
VERSION  := $(shell cat VERSION)
TAG      := $(REGISTRY)/$(IMAGE):$(VERSION)

# Ohara, the content repo. Private, its own repo, never checked into this one —
# see .gitignore. Keep in step with CONTENT_PATH in .env.
CONTENT_PATH ?= ../ohara

# The running container, for the signal that makes it reread ohara.
CONTAINER ?= thelighthouse

# Where the api listens on loopback. Matches API_PORT in .env.
API_PORT ?= 9000

.DEFAULT_GOAL := help
.PHONY: help web up dev api down db db-down db-reset psql migrate migrate-status fmt fmt-check \
        lint test build check audit image run login push clean \
        content content-check content-sync content-db-sync content-reload \
        prices prices-apply catalogue

help: ## show this
	@grep -hE '^[a-z-]+:.*?## ' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# The frontend is a submodule, so a fresh clone has an empty web/ until this
# runs. Building without it fails inside docker with a confusing missing-file
# error rather than an obvious one.
web: ## fetch or update the frontend submodule
	git submodule update --init --remote web

# Everything development needs that is not the code: postgres, and the caddy
# that fronts it. The api and nuxt you start yourself — caddy proxies to them on
# the host, so start them in either order and reload the page.
#
#   make api                                        :9000
#   cd web && HOST=127.0.0.1 PORT=3000 npm run dev  :3000
#   http://localhost:8000                           ← visit this, never :3000
#
# HOST=127.0.0.1 is not optional. Nuxt otherwise binds [::1] — IPv6 loopback
# only — and caddy, running in docker, reaches the host over IPv4. The symptom
# is a 502 from caddy while http://localhost:3000 works perfectly in a browser.
up: ## start postgres and caddy, and wait for them
	docker compose up -d --wait
	@echo "\n  caddy    http://localhost:$(CADDY_PORT)"
	@echo "  api      make api"
	@echo "  nuxt     cd web && HOST=127.0.0.1 PORT=3000 npm run dev\n"

# Both in one terminal, with caddy already in front of them. Ctrl-C stops the
# pair — `kill 0` signals the whole process group, so nuxt does not survive the
# api and keep port 3000 for the next run.
#
# **The api restarts on a change, like nuxt does.** It did not, and the two
# behaving differently is a trap: nuxt reloads on every save, so the page in
# front of you is current while the api answering it can be hours old. What that
# looks like is not a stale server — it is a bug. A plan that sold ten books
# served three, because the binary predated plans carrying their own list.
#
# `billing.yaml` is watched alongside the code because the api reads it once, at
# boot. Running `prices apply` therefore changed what stripe charges and left
# the api quoting the old catalogue until somebody thought to restart it.
#
# `--no-vcs-ignores` is what makes that work: `billing.yaml` is generated and so
# it is in `.gitignore`, and cargo-watch skips ignored files by default — the
# watch was there and silently did nothing until this was added. `target/` and
# `.git/` are still ignored; only `--ignore-nothing` would include those.
#
# Needs `cargo install cargo-watch`. Nothing else here does, and it is one line
# to say so rather than a detection shim that guesses what you meant.
#
# HOST=127.0.0.1 is not optional; see the note above `up`.
dev: up ## run the api and nuxt together, behind caddy, both reloading
	@echo "  visit http://localhost:$(CADDY_PORT)\n"
	@trap 'kill 0' INT TERM; \
		cargo watch -q --no-vcs-ignores \
			-w crates -w Cargo.toml -w Cargo.lock -w billing.yaml \
			-x 'run -q -p lighthouse-api' & \
		(cd web && HOST=127.0.0.1 PORT=3000 npm run dev) & \
		wait

# The api alone, reloading, for when nuxt is not wanted. Same watch set as
# `dev` — one of them quietly not restarting is the failure this exists to stop.
api: ## run the api on its own, reloading on a change
	cargo watch -q --no-vcs-ignores \
		-w crates -w Cargo.toml -w Cargo.lock -w billing.yaml \
		-x 'run -q -p lighthouse-api'

down: ## stop postgres and caddy, keep the data
	docker compose down

# Postgres only, for when caddy is not wanted.
db: ## start postgres and wait for it
	docker compose up -d --wait postgres

# No -v. The volume survives, because throwing away local data should be typed
# out in full rather than reachable by muscle memory.
db-down: ## stop postgres, keep the data
	docker compose down

db-reset: ## stop postgres and delete the data
	docker compose down -v

# `exec`, not `run`: attaches to the container `make db` already started, so it
# is the same database with the same data. `docker compose run` would spin up a
# second one. Add a command to inspect without the shell:
#   make psql ARGS='-c "select slug, status from books"'
psql: ## open a psql shell in the running postgres
	docker compose exec postgres psql -U $(POSTGRES_USER) -d $(POSTGRES_DB) $(ARGS)

# Fresh database: builds the schema from migration 0. A database the laravel app
# already owns wants `baseline` instead — see the README.
migrate: ## apply pending migrations
	cargo run -q -p lighthouse-migrate -- run

migrate-status: ## what is applied, what is pending
	cargo run -q -p lighthouse-migrate -- status

fmt: ## format the rust source
	cargo fmt --all

fmt-check: ## fail if anything is unformatted
	cargo fmt --all -- --check

# -D warnings so a pedantic warning fails the build too. Without it the deny
# list in Cargo.toml is the only thing with teeth and the warn tier is noise.
lint: ## clippy across all targets, warnings are errors
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test: ## run the test suite
	cargo test --workspace --all-features

build: ## release build
	cargo build --workspace --release

audit: ## report advisories against the dependency tree
	cargo audit || echo "install with: cargo install cargo-audit"

# What CI runs and what to run before pushing.
check: fmt-check lint test ## format check, clippy, tests

# --platform: App Platform is amd64 and this is likely built on an arm mac.
# Without it the image pushes fine and then refuses to start there.
image: ## build the combined caddy + nuxt + api image
	docker build --platform linux/amd64 -t $(TAG) -t $(IMAGE):local .
	@echo "\nbuilt $(TAG)"

# --env-file, not a list of -e flags: the api requires every key in
# .env.example and refuses to start with one missing, so naming them here means
# adding a key becomes two edits and a confusing boot failure between them.
# Set APP_URL=http://localhost:$(PORT) in .env or the oauth callbacks point at
# the dev server instead of this container.
run: ## run the built image locally on PORT, configured from .env
	docker run --rm -p $(PORT):8080 --env-file .env $(IMAGE):local

login: ## authenticate to the DO registry
	doctl registry login --context thelighthouse

# login first — DO registry credentials expire, and a stale docker credential
# fails the push looking like a doctl auth problem
push: login ## push the image to the registry
	docker push $(TAG)
	@echo "\npushed $(TAG)"

clean: ## drop build artefacts
	rm -rf target

# content
#
# Four steps that are deliberately separate, because they fail in different
# places and only the first two are safe to run without thinking:
#
#   content         pull ohara       — touches the content repo, not this one
#   content-check   does it parse?   — the gate; run it before a deploy
#   content-sync    pull, check, hup — the whole thing, against a container
#   content-db-sync upsert the rows  — writes; run it deliberately
#
# What a reader reads never comes from the database. Ohara is read from disk and
# held in memory, and `content-sync` is what makes a running process reread it.
# `content-db-sync` writes the rows that other tables point at — notes,
# bookmarks, entitlements — and nothing else; no prose goes into postgres.

content: ## pull the latest ohara
	@test -d "$(CONTENT_PATH)/.git" \
		|| { echo "no content repo at $(CONTENT_PATH)"; exit 1; }
	git -C "$(CONTENT_PATH)" pull --ff-only
	@echo "\nohara at $$(git -C "$(CONTENT_PATH)" rev-parse --short HEAD)"

# The same walk the api does at boot, so a book that would stop the process from
# starting fails here instead — with the offending file named.
content-check: ## parse every book and lesson, and say what is wrong
	@CONTENT_PATH="$(CONTENT_PATH)" cargo run --quiet --bin lighthouse-api -- --check-content

# `docker kill` only sends the signal; the container keeps running. The
# entrypoint traps HUP and forwards it to the api, which rereads ohara and swaps
# the catalogue over. In-flight requests finish against the old one, and a
# content repo that does not parse leaves the previous one in place.
content-sync: content content-check ## pull, check, and make the container reread
	docker kill -s HUP $(CONTAINER)
	@echo "\nsignalled $(CONTAINER); check its logs for 'content reloaded'"

# Its own binary, and its own step, for the reason `migrate` is both: this
# writes. It reads DATABASE_URL from .env like everything else, so it hits
# whichever database that points at — check before running it against
# production.
content-db-sync: ## upsert ohara's books and lessons into postgres
	CONTENT_PATH="$(CONTENT_PATH)" cargo run -q -p lighthouse-content -- sync

# ---------------------------------------------------------------------------
# prices
# ---------------------------------------------------------------------------
#
# `pricing.yaml` is the declaration. Two things are generated from it and
# neither is edited by hand:
#
#   billing.yaml           what the api charges     — needs stripe
#   web/app/data/Catalogue.ts  what the page advertises — needs ohara
#
# They are separate commands because only one of them talks to stripe, and
# because the frontend has to be buildable without a stripe key. `prices-apply`
# runs both, which is what keeps the page from advertising a price the api no
# longer charges.

prices: ## what stripe has, and where it differs from pricing.yaml
	cargo run -q -p lighthouse-prices -- status

# Writes to stripe. Creates prices and coupons that cannot be deleted, only
# deactivated — check `make prices` first, and check which key .env holds.
prices-apply: ## make stripe match pricing.yaml, then regenerate both outputs
	cargo run -q -p lighthouse-prices -- apply
	$(MAKE) catalogue

# No stripe key and no network: two files in, one file out. Its output is
# committed to the web submodule, because `/pricing` is prerendered and the
# image is built from web/ alone — the build cannot reach pricing.yaml, ohara,
# or the api.
catalogue: ## write the frontend's build-time catalogue from pricing.yaml + ohara
	CONTENT_PATH="$(CONTENT_PATH)" cargo run -q -p lighthouse-prices -- catalogue

# Rereads ohara and swaps the catalogue over in the *running* process. No
# restart, no dropped request — the alternative is `docker kill -s HUP`, which
# does the same thing without telling you whether it worked.
#
# Signed, because the endpoint is: the api verifies the HMAC itself and answers
# 404 without it. The payload is "{unix seconds}.{METHOD}.{path}" — luxctl's
# scheme, which this endpoint inherits by sharing the middleware. The timestamp
# goes in a header of its own and is checked against a five-minute window, so a
# signature copied out of a shell history stops working.
content-reload: ## make the running api reread ohara
	@test -n "$$LUXCTL_SECRET" || { . ./.env 2>/dev/null; }; \
		ts=$$(date +%s); \
		sig=$$(printf '%s.POST./reload' "$$ts" \
			| openssl dgst -sha256 -hmac "$${LUXCTL_SECRET}" -hex | sed 's/.*= *//'); \
		curl -fsS -X POST -H "X-Luxctl-Signature: $$sig" \
			-H "X-Luxctl-Timestamp: $$ts" \
			http://127.0.0.1:$(API_PORT)/reload && echo
