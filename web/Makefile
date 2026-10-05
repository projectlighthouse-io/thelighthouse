PORT ?= 3000

REGISTRY := registry.digitalocean.com/lighthouse-registry
IMAGE    := lighthouse-web
VERSION  := $(shell cat ../VERSION)
TAG      := $(REGISTRY)/$(IMAGE):$(VERSION)

.DEFAULT_GOAL := help
.PHONY: help install dev build preview lint fix types check audit outdated \
        image run-image login push bench clean fresh

help: ## show this
	@grep -hE '^[a-z-]+:.*?## ' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

# `ci` not `install`: it installs exactly what package-lock.json records and
# fails if the lockfile and package.json disagree. `npm install` is free to
# re-resolve the 900-odd transitive packages, which is where a poisoned release
# actually gets in — pinning the 11 direct deps does nothing for those.
install: ## install exactly what the lockfile records
	npm ci

dev: ## run the dev server (PORT=3001 make dev to move it)
	PORT=$(PORT) npm run dev

build: ## production build
	npm run build

preview: ## serve the production build
	PORT=$(PORT) node .output/server/index.mjs

lint: ## report lint problems
	npx eslint .

fix: ## fix what eslint can fix on its own
	npx eslint . --fix

types: ## typecheck templates and script blocks
	npx nuxt typecheck

# What CI should run, and what to run before pushing. Types come before build:
# `nuxt build` transpiles without typechecking, so a build alone will happily
# ship a type error.
check: lint types build ## lint, typecheck, then build

# --platform: App Platform runs amd64 and this is likely built on an arm mac,
# so without it the pushed image will not start there.
image: ## build the production container
	docker build --platform linux/amd64 -t $(TAG) -t $(IMAGE):local .
	@echo "\nbuilt $(TAG)"

run-image: ## run the built container locally on PORT
	docker run --rm -p $(PORT):3000 $(IMAGE):local

login: ## authenticate to the DO registry
	doctl registry login --context lighthouse

# login first — DO registry credentials expire, and a stale docker credential
# fails the push looking like a doctl auth problem
push: login ## push the image to the registry
	docker push $(TAG)
	@echo "\npushed $(TAG)"

# Benchmarks the container, not `nuxt dev` — dev has HMR, source maps and no
# prerendering, so its timings say nothing about production.
bench: image ## benchmark the production container
	./bench/run.sh

audit: ## report known vulnerabilities
	npm audit --audit-level=moderate

outdated: ## show which pinned versions have moved on
	npm outdated || true

clean: ## drop build output and caches
	rm -rf .nuxt .output node_modules/.vite node_modules/.cache

fresh: clean ## clean, then reinstall from scratch
	rm -rf node_modules
	npm ci
