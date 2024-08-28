# # ----------------Commands----------------
#
# # change the 20 value in printf to adjust width
# # Use ' ## some comment' behind a command and it will be added to the help message automatically


# Define paths to the project files
RUST_PROJECT_DIR := ../ahnlich
RUST_TYPES_NAME := ahnlich_types
RUST_CLIENT_NAME := client
AHNLICH_DB_NAME := db
AHNLICH_AI_NAME := ai
PYTHON_PROJECT_DIR := ../sdk/ahnlich-client-py
CARGO_TOML := $(RUST_PROJECT_DIR)/Cargo.toml
PYPOETRY_TOML := $(PYTHON_PROJECT_DIR)/pyproject.toml

# Default version bump rule 
BUMP_RULE := patch

# Default crate name
CRATE_NAME := db

help: ## Show this help message
	@awk 'BEGIN {FS = ":.*?## "}; /^[a-zA-Z0-9_-]+:.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST) | grep -v '^help:.*?## '

format-check: ## cargo fmt --check
	  cargo fmt --all -- --check

format: ## cargo fmt
	  cargo fmt

clippy: ## cargo clippy 
	cargo clippy -- -D warnings

check: ## cargo check 
	cargo check

test: ## cargo test
	cargo nextest run --no-capture

generate-specs: ## generate type specs 
	cargo run --bin typegen generate

run-db: ## Run ahnlich_db bin 
	cargo run --bin ahnlich-db run

run-db-tracing: ## Run ahnlich_db bin with  tracing enabled
	cargo run --bin ahnlich-db run --enable-tracing

run-ai: ## Run ahnlich_ai bin with default supported models
	cargo run --bin ahnlich-ai run --supported-models dalle3,llama3

run-ai-tracing: ## Run ahnlich_ai bin with default supported models and tracing_enabled
	cargo run --bin ahnlich-ai run --supported-models dalle3,llama3 --enable-tracing


bump-protocol-version: ## Bump project versions. Rules for bumpversion: patch, minor, major.
	@echo "Bumping Rust Protocol version to $${RULE:-$(BUMP_RULE)}"
	@cd $(RUST_PROJECT_DIR) && cargo set-version --bump $(BUMP_RULE) --package $(RUST_TYPES_NAME)
	@echo "Rust Protocol version bumped to $(BUMP_RULE)"

	@echo "Bumping ahnlich db version because of changed protocol with rule $(BUMP_RULE)"
	@cd $(RUST_PROJECT_DIR) && cargo set-version --bump $(BUMP_RULE) --package $(AHNLICH_DB_NAME)
	@echo "Ahnlich DB version bumped to $(BUMP_RULE)"

	@echo "Bumping ahnlich ai version because of changed protocol with rule $(BUMP_RULE)"
	@cd $(RUST_PROJECT_DIR) && cargo set-version --bump $(BUMP_RULE) --package $(AHNLICH_AI_NAME)
	@echo "Ahnlich AI version bumped to $(BUMP_RULE)"

	@echo "Bumping Python project version with rule $(BUMP_RULE)"
	@cd $(PYTHON_PROJECT_DIR) && poetry run bumpversion --component Protocol --bump-type $(BUMP_RULE)
	@echo "Python project version bumped using rule $(BUMP_RULE)"

bump-py-client: ## Bump python client versions. Rules for bump-py-client: patch, minor, major.
	@echo "Bumping Python client version with rule $(BUMP_RULE)"
	@cd $(PYTHON_PROJECT_DIR) && poetry run bumpversion --component Client --bump-type $(BUMP_RULE)
	@echo "Python client version bumped using rule $(BUMP_RULE)"

bump-rs-client: ## Bump Rust client versions. Rules for bump-rs-client: patch, minor, major.
	@echo "Bumping Rust client version with rule $(BUMP_RULE)"
	@cd $(RUST_PROJECT_DIR) && cargo set-version --bump $(BUMP_RULE) --package $(RUST_CLIENT_NAME)
	@echo "Rust client version bumped to $(BUMP_RULE)"

bump-ahnlich-db: ## Bump ahnlich db version. Rules for bump-ahnlich-db: patch, minor, major.
	@echo "Bumping ahnlich db version with rule $(BUMP_RULE)"
	@cd $(RUST_PROJECT_DIR) && cargo set-version --bump $(BUMP_RULE) --package $(AHNLICH_DB_NAME)
	@echo "Ahnlich DB version bumped to $(BUMP_RULE)"

bump-ahnlich-ai: ## Bump ahnlich ai version. Rules for bump-ahnlich-ai: patch, minor, major.
	@echo "Bumping ahnlich ai version with rule $(BUMP_RULE)"
	@cd $(RUST_PROJECT_DIR) && cargo set-version --bump $(BUMP_RULE) --package $(AHNLICH_AI_NAME)
	@echo "Ahnlich AI version bumped to $(BUMP_RULE)"

workspace-crate-version: ## View current version of crate in workspace. Rules workspace-crate-version: <CRATE_NAME>.
	@echo "Getting version of crate: $(CRATE_NAME)"
	@cd $(RUST_PROJECT_DIR) && cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | select(.name == "$(CRATE_NAME)") | "\(.version)"'


demo-tracing:
	@echo "Starting Docker services..."
	docker-compose up -d

	@echo "Setup Python Env Dependencies..."
	@cd $(PYTHON_PROJECT_DIR) && poetry config --local virtualenvs.in-project true
	@cd $(PYTHON_PROJECT_DIR) && poetry install
	@echo "Run demo_tracing..."
	@cd $(PYTHON_PROJECT_DIR) && python demo_tracing.py 


all: format check test clippy generate-specs bumpversion bump-py-client bump-rs-client ## format, clip, generate type specs, and bumpversions 


# --------------Configuration-------------
#
#  .NOTPARALLEL: ; # wait for this target to finish
.EXPORT_ALL_VARIABLES: ; # send all vars to shell

.PHONY: docs all # All targets are accessible for user
	.DEFAULT: help # Running Make will run the help target

MAKEFLAGS += --no-print-directory # dont add message about entering and leaving the working directory

