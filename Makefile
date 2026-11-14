# waymark development commands. Works with GNU Make 3.81 (the macOS default).
# Run `make help` to see all targets.

CARGO ?= cargo
TARGET := aarch64-apple-darwin
# Test files named tests/it_*.rs are integration tests. All other files in tests/ are unit or conformance tests.
INTEGRATION_FILES := $(wildcard tests/it_*.rs)
UNIT_FILES := $(filter-out $(INTEGRATION_FILES),$(wildcard tests/*.rs))
UNIT_ARGS := $(patsubst tests/%.rs,--test %,$(UNIT_FILES))
INTEGRATION_ARGS := $(patsubst tests/%.rs,--test %,$(INTEGRATION_FILES))

.DEFAULT_GOAL := help
.PHONY: help setup doctor build release test test-integration test-all fmt lint check spec-check run sample-kb brew-local public-check clean

help: ## List all targets
	@echo "waymark: development commands"
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | sed -E 's/^([a-z-]+):.*## (.*)/  \1|\2/' | awk -F'|' '{printf "  %-18s %s\n", $$1, $$2}'

setup: ## Install what is missing on a new Mac, then run doctor
	@bash scripts/setup.sh

doctor: ## Check that this machine is ready
	@bash scripts/doctor.sh

build: ## Debug build
	@echo "Building (debug)..."
	@$(CARGO) build

release: ## Release build for Apple silicon
	@echo "Building (release, $(TARGET))..."
	@$(CARGO) build --release --target $(TARGET)
	@ls -lh target/$(TARGET)/release/waymark

test: ## Unit and conformance tests
	@echo "Running unit and conformance tests..."
	@$(CARGO) test --lib --bins $(UNIT_ARGS)
	@bash scripts/tests/run.sh

test-integration: ## Integration tests (real binary, temporary Git repositories)
	@echo "Running integration tests..."
	@if [ -z "$(INTEGRATION_ARGS)" ]; then echo "No integration tests yet."; else $(CARGO) test $(INTEGRATION_ARGS); fi

test-all: test test-integration ## Unit, conformance, and integration tests

fmt: ## Format the code
	@$(CARGO) fmt

lint: ## Check format and run clippy with warnings as errors
	@$(CARGO) fmt --check
	@$(CARGO) clippy --all-targets -- -D warnings

check: lint test-all spec-check ## Everything to run before a commit

spec-check: ## Check the spec folders against INDEX.md
	@bash scripts/spec-check.sh

run: ## Start the server on a sample knowledge base
	@bash scripts/sample-kb.sh run

sample-kb: ## Create a sample knowledge base in tmp/
	@bash scripts/sample-kb.sh create

manual-tests: ## List the open [manual] tests of all specs
	@grep -rn --include=test.md "^- \[ \] \[manual\]" specs/features specs/foundation | sed "s|specs/||"

brew-local: ## Install from a local Homebrew tap (publishes nothing)
	@bash scripts/brew-local.sh

public-check: ## Scan tracked files for secrets and private details
	@bash scripts/public-check.sh

clean: ## Remove build output and test data
	@$(CARGO) clean
	@rm -rf tmp/sample-kb
