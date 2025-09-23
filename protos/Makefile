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
	(cd .. && buf breaking --against '.git#branch=main' protos) 

## Generate code with protoc plugins
generate: lock format lint
	@echo "➜ Generating code"
	buf generate
	@echo "✓ Done"

## Show available targets with descriptions
help:
	@grep -E '^##' -n Makefile | sed 's/^.*## //' | column -t -s ':'

# Ensure protoc and plugins are installed
PROTOC := $(shell which protoc)
PROTOC_GEN_GO := $(shell which protoc-gen-go)
PROTOC_GEN_GO_GRPC := $(shell which protoc-gen-go-grpc)
BUF := $(shell which buf)

.PHONY: deps
deps:
ifndef PROTOC
	@echo "❌ protoc is not installed. Please install it manually:"
	@echo "  - macOS: brew install protobuf"
	@echo "  - Ubuntu: sudo apt install -y protobuf-compiler"
	@echo "  - Or download from: https://github.com/protocolbuffers/protobuf/releases"
	@exit 1
endif
ifndef PROTOC_GEN_GO
	@echo "Installing protoc-gen-go..."
	go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
endif
ifndef PROTOC_GEN_GO_GRPC
	@echo "Installing protoc-gen-go-grpc..."
	go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
endif
	@echo "✅ All dependencies installed."
ifndef BUF
	@echo "Installing buf..."
	go install github.com/bufbuild/buf/cmd/buf@latest
endif
	@echo "✅ buf installed."
