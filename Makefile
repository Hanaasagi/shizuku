SHELL := /bin/bash

.DEFAULT_GOAL := help

.PHONY: help
help:
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: test
test: ## Run tests
	@cargo test

.PHONY: fmt
fmt: ## Format code
	@cargo fmt

.PHONY: lint
lint: ## Lint code
	@cargo clippy --all --all-targets --verbose --

.PHONY: fix
fix: ## Fix code
	@cargo fix

.PHONY: asm
asm: ## Fix code
	@llvm-mc a.s --output-asm-variant=1 | bat -l asm
