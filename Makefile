.DEFAULT_GOAL := help
.PHONY: help sync test build serve ci validate clean tables

PORT ?= 8080
OUT  ?= public
# URL the site is served from; empty keeps the config base_url
BASE_URL ?=
# jf2 file of received webmentions to show on posts; empty shows none
WEBMENTIONS ?=
# Nu HTML Checker for `make validate` (ADR 0011): pinned release from npm, checked by hash
VNU_VERSION := 26.9.16
VNU_SHA512  := 843772e456ea003f9e3998d481837389d4b38b9e2ffd595e07f6ac3b2cd81823dc9ac03d2b86afebea0a4be64ff43235376b3fcfa6d7ee4f24739c16af796c62
VNU_JAR     := target/vnu/vnu-$(VNU_VERSION).jar

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-8s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

sync: ## Install/update Python test-harness deps from uv.lock
	uv sync --frozen

test: ## Run Rust unit tests, then the spec harness
	cargo test --release --quiet
	uv run pytest

build: ## Build the site → ./$(OUT) (Rust binary, std only)
	cargo run --release -- --out $(OUT) $(if $(BASE_URL),--base-url $(BASE_URL)) $(if $(WEBMENTIONS),--webmentions $(WEBMENTIONS))

serve: ## Build, watch content/, and serve at :$(PORT)
	cargo run --release -- --out $(OUT) --serve --port $(PORT)

ci: sync test build ## Full local CI: same checks .github/workflows/ci.yml runs

validate: build $(VNU_JAR) ## Check every built page with the Nu HTML Checker (needs Java, ADR 0011)
	java -jar $(VNU_JAR) --skip-non-html --Werror \
		--filterpattern '.*Trailing slash on void elements.*' $(OUT)

$(VNU_JAR):
	mkdir -p target/vnu
	curl -fsSL -o target/vnu/vnu.tgz https://registry.npmjs.org/vnu-jar/-/vnu-jar-$(VNU_VERSION).tgz
	echo "$(VNU_SHA512)  target/vnu/vnu.tgz" | shasum -a 512 -c -
	tar -xzf target/vnu/vnu.tgz -C target/vnu package/build/dist/vnu.jar
	mv target/vnu/package/build/dist/vnu.jar $@
	rm -rf target/vnu/package target/vnu/vnu.tgz

tables: ## Regenerate build/entities.rs and build/unicode.rs tables (ADR 0007)
	python3 scripts/gen_tables.py

clean: ## Remove build + cache artifacts
	rm -rf public _smoke .pytest_cache .ruff_cache target
