.DEFAULT_GOAL := help
.PHONY: help sync test build serve ci clean tables

PORT ?= 8080
OUT  ?= public
# URL the site is served from; empty keeps the config base_url
BASE_URL ?=

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-8s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

sync: ## Install/update Python test-harness deps from uv.lock
	uv sync --frozen

test: ## Run Rust unit tests, then the spec harness
	cargo test --release --quiet
	uv run pytest

build: ## Build the site → ./$(OUT) (Rust binary, std only)
	cargo run --release -- --out $(OUT) $(if $(BASE_URL),--base-url $(BASE_URL))

serve: ## Build, watch content/, and serve at :$(PORT)
	cargo run --release -- --out $(OUT) --serve --port $(PORT)

ci: sync test build ## Full local CI: same checks .github/workflows/ci.yml runs

tables: ## Regenerate build/entities.rs and build/unicode.rs tables (ADR 0007)
	python3 scripts/gen_tables.py

clean: ## Remove build + cache artifacts
	rm -rf public _smoke .pytest_cache .ruff_cache target
