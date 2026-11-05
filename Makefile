.DEFAULT_GOAL := help
.PHONY: help setup lint test build serve ci validate clean tables lighthouse

PORT ?= 8080
OUT  ?= public
# URL the site is served from; empty keeps the config base_url
BASE_URL ?=
# publish cutoff date (YYYY-MM-DD) to preview scheduled posts; empty is today in UTC
TODAY ?=
# jf2 file of received webmentions to show on posts; empty shows none
WEBMENTIONS ?=
# Unicode Character Database release `make tables` reads (see build/unicode.rs)
UNICODE_VERSION ?= 15.1.0
TABLES_DATA ?= target/tables-data
# Nu HTML Checker for `make validate` (ADR 2026-10-11-nu-html-checker-in-ci): pinned release from npm, checked by hash
VNU_VERSION := 26.9.16
VNU_SHA512  := 843772e456ea003f9e3998d481837389d4b38b9e2ffd595e07f6ac3b2cd81823dc9ac03d2b86afebea0a4be64ff43235376b3fcfa6d7ee4f24739c16af796c62
VNU_JAR     := target/vnu/vnu-$(VNU_VERSION).jar

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
validate: build $(VNU_JAR) ## Check every built page with the Nu HTML Checker (needs Java)
	java -jar $(VNU_JAR) --skip-non-html --Werror \
		--filterpattern '.*(Trailing slash on void elements|Resource violates Content Security Policy).*' $(OUT)

$(VNU_JAR):
	mkdir -p target/vnu
	curl -fsSL -o target/vnu/vnu.tgz https://registry.npmjs.org/vnu-jar/-/vnu-jar-$(VNU_VERSION).tgz
	echo "$(VNU_SHA512)  target/vnu/vnu.tgz" | shasum -a 512 -c -
	tar -xzf target/vnu/vnu.tgz -C target/vnu package/build/dist/vnu.jar
	mv target/vnu/package/build/dist/vnu.jar $@
	rm -rf target/vnu/package target/vnu/vnu.tgz

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
	rm -rf public target _lighthouse
