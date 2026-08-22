PORT ?= 8080

REGISTRY := registry.digitalocean.com/lighthouse-registry
IMAGE    := thelighthouse
VERSION  := $(shell cat VERSION)
TAG      := $(REGISTRY)/$(IMAGE):$(VERSION)

.DEFAULT_GOAL := help
.PHONY: help web db db-down db-reset migrate migrate-status fmt fmt-check lint test \
        build check audit image run login push clean

help: ## show this
	@grep -hE '^[a-z-]+:.*?## ' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# The frontend is a submodule, so a fresh clone has an empty web/ until this
# runs. Building without it fails inside docker with a confusing missing-file
# error rather than an obvious one.
web: ## fetch or update the frontend submodule
	git submodule update --init --remote web

# Postgres only. The site itself runs from the built image — see `run`.
db: ## start postgres and wait for it
	docker compose up -d --wait

# No -v. The volume survives, because throwing away local data should be typed
# out in full rather than reachable by muscle memory.
db-down: ## stop postgres, keep the data
	docker compose down

db-reset: ## stop postgres and delete the data
	docker compose down -v

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
	doctl registry login --context lighthouse

# login first — DO registry credentials expire, and a stale docker credential
# fails the push looking like a doctl auth problem
push: login ## push the image to the registry
	docker push $(TAG)
	@echo "\npushed $(TAG)"

clean: ## drop build artefacts
	rm -rf target
