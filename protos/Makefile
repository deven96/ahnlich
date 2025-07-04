# ------------------------------------------------------------------------
# Makefile – fixed: buf format gets one source argument
# ------------------------------------------------------------------------

.PHONY: format lint lock breaking generate all help

# Default target ─ run the full pipeline
all: generate

## Format code
format:
	@echo "➜ Formatting protobuf files"
	# Passing '.' keeps buf happy and recursively formats every *.proto
	buf format -w .

## Lint proto files
lint:
	@echo "➜ Linting protobuf files"
	buf lint .

## Lock dependencies
lock:
	@echo "➜ Updating buf module dependencies"
	buf mod update

## Check for breaking changes (runs from parent dir)
breaking:
	@echo "➜ Checking for breaking changes against main branch"
	(cd .. && buf breaking --against '.git#branch=main')

## Generate code with protoc plugins
generate: lock format lint
	@echo "➜ Generating code"
	buf generate
	@echo "✓ Done"

## Show available targets with descriptions
help:
	@grep -E '^##' -n Makefile | sed 's/^.*## //' | column -t -s ':'