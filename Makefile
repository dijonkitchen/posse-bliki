.DEFAULT_GOAL := help
.PHONY: help setup lint test build serve ci clean tables

PORT ?= 8080
OUT  ?= public
# URL the site is served from; empty keeps the config base_url
BASE_URL ?=
# jf2 file of received webmentions to show on posts; empty shows none
WEBMENTIONS ?=
# Unicode Character Database release `make tables` reads (see build/unicode.rs)
UNICODE_VERSION ?= 15.1.0
TABLES_DATA ?= target/tables-data

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-8s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Install the pinned Rust toolchain from rust-toolchain.toml (needs rustup)
	rustup toolchain install

lint: ## Check formatting (rustfmt) and lints (clippy, warnings are errors)
	cargo fmt --check
	cargo clippy --release --all-targets --quiet -- -D warnings

test: ## Run the Rust unit tests and the spec harness (tests/harness)
	cargo test --release --quiet

build: ## Build the site → ./$(OUT) (Rust binary, std only)
	cargo run --release -- --out $(OUT) $(if $(BASE_URL),--base-url $(BASE_URL)) $(if $(WEBMENTIONS),--webmentions $(WEBMENTIONS))

serve: ## Build, watch content/, and serve at :$(PORT)
	cargo run --release -- --out $(OUT) --serve --port $(PORT)

ci: lint test build ## Full local CI: same checks .github/workflows/ci.yml runs

tables: ## Regenerate build/entities.rs and build/unicode.rs (downloads the source data)
	mkdir -p $(TABLES_DATA)
	curl -sSfL -o $(TABLES_DATA)/entities.json https://html.spec.whatwg.org/entities.json
	curl -sSfL -o $(TABLES_DATA)/UnicodeData.txt https://www.unicode.org/Public/$(UNICODE_VERSION)/ucd/UnicodeData.txt
	curl -sSfL -o $(TABLES_DATA)/DerivedNumericType.txt https://www.unicode.org/Public/$(UNICODE_VERSION)/ucd/extracted/DerivedNumericType.txt
	cargo run --release --bin gen-tables -- $(TABLES_DATA) $(UNICODE_VERSION)
	cargo fmt

clean: ## Remove build artifacts
	rm -rf public target
