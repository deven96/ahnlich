SHELL := /bin/bash

.PHONY: install-dependencies format format-check lint-check pre-commit-check test

install-dependencies:
	npm install

format:
	npx prettier --write "src/**/*.ts" "tests/**/*.ts" "grpc/**/*.ts"

format-check:
	npx prettier --check "src/**/*.ts" "tests/**/*.ts" "grpc/**/*.ts"

lint-check: format-check

pre-commit-check: lint-check

test:
ifeq ($(REPORT),1)
	JEST_JUNIT_OUTPUT_NAME=node.xml npm test -- --forceExit --reporters=default --reporters=jest-junit
else
	npm test -- --forceExit
endif
