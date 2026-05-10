.DEFAULT_GOAL := help
.PHONY: help sync test build serve ci clean

PORT ?= 8080
OUT  ?= public

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-8s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

sync: ## Install/update Python deps from uv.lock
	uv sync --frozen

test: ## Run the test harness
	uv run pytest

build: ## Build the site → ./$(OUT)
	uv run python -m build --out $(OUT)

serve: ## Build, watch content/, and serve at :$(PORT)
	uv run python -m build --serve --port $(PORT)

ci: sync test build ## Full local CI: same checks .github/workflows/ci.yml runs

clean: ## Remove build + cache artifacts
	rm -rf public _smoke .pytest_cache .ruff_cache
