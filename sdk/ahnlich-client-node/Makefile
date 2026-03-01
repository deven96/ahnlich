SHELL := /bin/bash

PROTOS_DIR := $(shell realpath ../../protos)

.PHONY: install-dependencies format format-check lint-check pre-commit-check generate test

install-dependencies:
	npm install

format:
	npx prettier --write "src/**/*.ts" "tests/**/*.ts" "grpc/**/*.ts"

format-check:
	npx prettier --check "src/**/*.ts" "tests/**/*.ts" "grpc/**/*.ts"

lint-check: format-check

pre-commit-check: lint-check

generate:
	@echo "➜ Generating TypeScript protobuf code from proto definitions"
	cd $(PROTOS_DIR) && buf generate
	@echo "✅ Proto generation complete."

test:
ifeq ($(REPORT),1)
	JEST_JUNIT_OUTPUT_NAME=node.xml npm test -- --forceExit --reporters=default --reporters=jest-junit
else
	npm test -- --forceExit
endif
