.DEFAULT_GOAL := help
.PHONY: help setup lint test build serve ci validate clean tables lighthouse

PORT ?= 8080
OUT  ?= public
# URL the site is served from; empty keeps the config base_url
BASE_URL ?=
# publish cutoff (YYYY-MM-DD, or a UTC datetime ending in Z) to preview scheduled posts; empty is now
TODAY ?=
# jf2 file of received webmentions to show on posts; empty shows none
WEBMENTIONS ?=
# Unicode Character Database release `make tables` reads (see build/unicode.rs)
UNICODE_VERSION ?= 15.1.0
TABLES_DATA ?= target/tables-data
# Nu HTML Checker for `make validate` (ADR 2026-10-11-nu-html-checker-in-ci): locked with its hash in
# scripts/vnu/package-lock.json, where Dependabot proposes bumps (ADR 2026-11-06-dependabot-nu-checker)
VNU_DIR := scripts/vnu
VNU_JAR := $(VNU_DIR)/node_modules/vnu-jar/build/dist/vnu.jar

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-8s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Install the pinned Rust toolchain from rust-toolchain.toml (needs rustup)
	rustup toolchain install

lint: ## Check formatting (rustfmt) and lints (clippy, warnings are errors)
	cargo fmt --check
	cargo clippy --locked --release --all-targets --quiet -- -D warnings

test: ## Run the Rust unit tests and the spec harness (tests/harness)
	cargo test --locked --release --quiet

build: ## Build the site → ./$(OUT) (Rust binary, std only)
	cargo run --locked --release -- --out $(OUT) $(if $(BASE_URL),--base-url $(BASE_URL)) $(if $(WEBMENTIONS),--webmentions $(WEBMENTIONS)) $(if $(TODAY),--today $(TODAY))

serve: ## Build, watch content/, and serve at :$(PORT)
	cargo run --locked --release -- --out $(OUT) --serve --port $(PORT)

ci: lint test build ## Full local CI: same checks .github/workflows/ci.yml runs

# The checker reads pages as file: URLs, where the CSP's 'self' matches
# nothing, so it flags every stylesheet and script. tests/harness/csp.rs
# checks the policy instead (ADR 2026-10-28-csp-meta-tag).
validate: build $(VNU_JAR) ## Check every built page with the Nu HTML Checker (needs Java and Node)
	java -jar $(VNU_JAR) --skip-non-html --Werror \
		--filterpattern '.*(Trailing slash on void elements|Resource violates Content Security Policy).*' $(OUT)

# npm ci checks the download against the lockfile's hash, and reinstalls when
# a bump changes the lockfile. --ignore-scripts skips the package's
# postinstall, which would download a Java runtime; CI installs Java itself.
$(VNU_JAR): $(VNU_DIR)/package-lock.json
	npm ci --prefix $(VNU_DIR) --ignore-scripts --no-audit --no-fund

lighthouse: ## Lighthouse budget on the built site (CI-only; needs Node + Chrome, ADR 2026-10-14-mobile-first)
	scripts/lighthouse.sh

tables: ## Regenerate build/entities.rs and build/unicode.rs (downloads the source data)
	mkdir -p $(TABLES_DATA)
	curl -sSfL -o $(TABLES_DATA)/entities.json https://html.spec.whatwg.org/entities.json
	curl -sSfL -o $(TABLES_DATA)/UnicodeData.txt https://www.unicode.org/Public/$(UNICODE_VERSION)/ucd/UnicodeData.txt
	curl -sSfL -o $(TABLES_DATA)/DerivedNumericType.txt https://www.unicode.org/Public/$(UNICODE_VERSION)/ucd/extracted/DerivedNumericType.txt
	cargo run --release --bin gen-tables -- $(TABLES_DATA) $(UNICODE_VERSION)
	cargo fmt

clean: ## Remove build artifacts
	rm -rf public target _lighthouse $(VNU_DIR)/node_modules
