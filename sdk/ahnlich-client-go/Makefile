SHELL := /bin/bash

OS        := $(shell uname -s | awk '{print tolower($$0)}')
ARCH      := $(shell uname -m)
GO_VERSION           := 1.22.4
GOFMT_VERSION        := latest-0.3
GOIMPORTS_VERSION    := v0.22.0
PRE_COMMIT_VERSION   := 3.7.0
PRETTIER_VERSION     := 3.2.5
REVIVE_VERSION       := v1.3.7
SHFMT_VERSION        := v3.8.0
JUNIT_VERSION        := 2.1.0
STATICCHECK_VERSION  := 2023.1.7
OUT_DIR    := grpc 
TEST_DIR := $(shell realpath .)

.PHONY: all \
        install-go-junit install-go-test-sum install-gofmt install-goimports \
        install-pre-commit install-prettier install-revive update-shell-config \
        install-shfmt install-staticcheck install-dependencies \
        format-check format lint-check pre-commit-check \
        test-sequential test-race test-coverage test

all: install-dependencies


install-go-junit:
	@mkdir -p /tmp/temp && rm -rf /tmp/temp/*
ifeq ($(CI),1)
	@export PATH=$$PATH:/usr/local/go/bin:$(shell go env GOPATH)/bin; \
	go install github.com/jstemmer/go-junit-report@v1
else
	@ARC=$(ARCH); \
	if [ "$$ARC" = "x86_64" ]; then ARC=amd64; \
	elif [ "$$ARC" = "aarch64" ]; then ARC=arm64; fi; \
	wget -nv https://github.com/jstemmer/go-junit-report/releases/download/v$(JUNIT_VERSION)/go-junit-report-v$(JUNIT_VERSION)-$(OS)-$$ARC.tar.gz -P /tmp/temp; \
	tar -xzvf /tmp/temp/go-junit-report-v$(JUNIT_VERSION)-$(OS)-$$ARC.tar.gz -C /tmp/temp; \
	mv -v /tmp/temp/go-junit-report $$(go env GOPATH)/bin
endif

install-go-test-sum:
	@mkdir -p /tmp/temp && rm -rf /tmp/temp/*
	@export PATH=$$PATH:/usr/local/go/bin:$(shell go env GOPATH)/bin; \
	go install gotest.tools/gotestsum@latest

install-gofmt:
	docker pull cytopia/gofmt:$(GOFMT_VERSION)

install-goimports:
ifeq ($(CI),1)
	@export PATH=$$PATH:/usr/local/go/bin; \
	GOBIN=/usr/local/bin go install golang.org/x/tools/cmd/goimports@$(GOIMPORTS_VERSION)
else
	@GOBIN=$$HOME/.local/bin go install golang.org/x/tools/cmd/goimports@$(GOIMPORTS_VERSION)
endif

install-pre-commit:
	python3 -m pip install pre-commit==$(PRE_COMMIT_VERSION)
	pre-commit install --hook-type commit-msg --allow-missing-config 

install-prettier:
	docker pull tmknom/prettier:$(PRETTIER_VERSION)

install-revive:
	@mkdir -p /tmp/temp && rm -rf /tmp/temp/*
ifeq ($(CI),1)
	@export PATH=$$PATH:/usr/local/go/bin:$(shell go env GOPATH)/bin; \
	wget -nv https://github.com/mgechev/revive/releases/download/$(REVIVE_VERSION)/revive_linux_amd64.tar.gz -P /tmp/temp; \
	tar -xzvf /tmp/temp/revive_linux_amd64.tar.gz -C /tmp/temp; \
	mv -v /tmp/temp/revive $$(go env GOPATH)/bin
else
	@wget -q https://github.com/mgechev/revive/releases/download/$(REVIVE_VERSION)/revive_$(OS)_amd64.tar.gz -P /tmp/temp && \
	  tar -xzvf /tmp/temp/revive_$(OS)_amd64.tar.gz -C /tmp/temp && \
	  mv -v /tmp/temp/revive $$(go env GOPATH)/bin || \
	  (echo "No prebuilt for $(OS)/amd64; installing from source" && \
	   go install github.com/mgechev/revive@$(REVIVE_VERSION))
endif

update-shell-config:
	@if [ -f ~/.zshrc ]; then \
	  grep -qx 'PATH=$$PATH:~/.local/bin' ~/.zshrc || echo 'PATH=$$PATH:~/.local/bin' >> ~/.zshrc; \
	  echo "Reload zsh: source ~/.zshrc"; \
	fi
	@if [ -f ~/.bashrc ]; then \
	  grep -qx 'PATH=$$PATH:~/.local/bin' ~/.bashrc || echo 'PATH=$$PATH:~/.local/bin' >> ~/.bashrc; \
	  echo "Reload bash: source ~/.bashrc"; \
	fi

install-shfmt:
	@mkdir -p /tmp/temp && rm -rf /tmp/temp/*
ifeq ($(CI),1)
	@wget -nv -O /tmp/temp/shfmt https://github.com/mvdan/sh/releases/download/$(SHFMT_VERSION)/shfmt_$(SHFMT_VERSION)_linux_amd64; \
	chmod +x /tmp/temp/shfmt; \
	mv -v /tmp/temp/shfmt /usr/local/bin
else
	@wget -q -O /tmp/temp/shfmt https://github.com/mvdan/sh/releases/download/$(SHFMT_VERSION)/shfmt_$(SHFMT_VERSION)_$(OS)_amd64 && \
	  chmod +x /tmp/temp/shfmt && \
	  mv -v /tmp/temp/shfmt $$HOME/.local/bin || \
	  (echo "No prebuilt for $(OS)/amd64; installing from source" && \
	   GOBIN=$$HOME/.local/bin go install mvdan.cc/sh/v3/cmd/gosh@$(SHFMT_VERSION))
endif
	$(MAKE) update-shell-config

install-staticcheck:
	@mkdir -p /tmp/temp && rm -rf /tmp/temp/*
ifeq ($(CI),1)
	@wget -nv https://github.com/dominikh/go-tools/releases/download/$(STATICCHECK_VERSION)/staticcheck_linux_amd64.tar.gz -P /tmp/temp; \
	tar -xzvf /tmp/temp/staticcheck_linux_amd64.tar.gz -C /tmp/temp; \
	mv -v /tmp/temp/staticcheck/staticcheck $$(go env GOPATH)/bin; \
	rm -rf /tmp/temp/staticcheck
else
	@wget -q https://github.com/dominikh/go-tools/releases/download/$(STATICCHECK_VERSION)/staticcheck_$(OS)_amd64.tar.gz -P /tmp/temp && \
	  tar -xzvf /tmp/temp/staticcheck_$(OS)_amd64.tar.gz -C /tmp/temp && \
	  mv -v /tmp/temp/staticcheck/staticcheck $$HOME/.local/go/bin && \
	  rm -rf /tmp/temp/staticcheck || \
	  (echo "No prebuilt for $(OS)/amd64; installing from source" && \
	   GOBIN=$$HOME/.local/go/bin go install honnef.co/go/tools/cmd/staticcheck@$(STATICCHECK_VERSION))
endif

install-dependencies: install-go-junit \
                      install-gofmt \
                      install-goimports \
                      install-pre-commit \
                      install-prettier \
                      install-revive \
                      install-shfmt \
                      install-staticcheck \
                      install-go-test-sum


format-check:
	docker run --rm -u "$$(id -u):$$(id -g)" -v "$$(pwd)":/work tmknom/prettier:$(PRETTIER_VERSION) --check "**/*.md"
	# docker run --rm -u "$$(id -u):$$(id -g)" -v "$$(pwd)":/work tmknom/prettier:$(PRETTIER_VERSION) --check "**/*.yml"
	shfmt -f . | grep -v 'internal/' | grep -v 'tmp/' | xargs shfmt -i 4 -ci -d
	test -z "$$(find . -type f -name '*.go' ! -path './internal/*' | xargs goimports -local github.com/deven96/ahnlich/sdk/ahnlich-client-go -l)"
	test -z "$$(find . -type f -name '*.go' ! -path './internal/*' | xargs gofmt -l)"

format:
	docker run --rm -u "$$(id -u):$$(id -g)" -v "$$(pwd)":/work tmknom/prettier:$(PRETTIER_VERSION) --write "**/*.md"
	# docker run --rm -u "$$(id -u):$$(id -g)" -v "$$(pwd)":/work tmknom/prettier:$(PRETTIER_VERSION) --write "**/*.yml"
	shfmt -f . | grep -v 'internal/' | grep -v 'tmp/' | xargs shfmt -i 4 -ci -w
	find . -type f -name '*.go' ! -path './internal/*' | xargs goimports -local github.com/deven96/ahnlich/sdk/ahnlich-client-go -w
	find . -type f -name '*.go' ! -path './internal/*' | xargs gofmt -w

lint-check:
ifeq ($(CI),1)
	@export PATH=$$PATH:/usr/local/go/bin:$(shell go env GOPATH)/bin;
endif
	go list $(TEST_DIR)/... | grep -v 'internal/' | xargs go vet
	go list $(TEST_DIR)/... | grep -v 'internal/' | xargs staticcheck
	revive $$(find . -type f -name '*.go' -not -path './internal/*')

pre-commit-check: format-check lint-check


test-sequential:
	tomlDir=$$(realpath ../../ahnlich/Cargo.toml); cargo run --manifest-path $$tomlDir --bin ahnlich-db --help
ifeq ($(CI),1)
	@export PATH=$$PATH:/usr/local/go/bin:$(shell go env GOPATH)/bin;
endif
ifneq ($(CACHE),1)
ifeq ($(REPORT),1)
	CGO_ENABLED=0 go test -count=1 -p 1 -failfast -v $$(go list $(TEST_DIR)/...) && gotestsum --junitfile go.xml
else
	CGO_ENABLED=0 go test -count=1 -p 1 -failfast -v $$(go list $(TEST_DIR)/...)
endif
else
ifeq ($(REPORT),1)
	CGO_ENABLED=0 go test -p 1 -failfast -v $$(go list $(TEST_DIR)/...) && gotestsum --junitfile go.xml
else
	CGO_ENABLED=0 go test -p 1 -failfast -v $$(go list $(TEST_DIR)/...)
endif
endif

test-race:
ifeq ($(CI),1)
	@export PATH=$$PATH:/usr/local/go/bin:$(shell go env GOPATH)/bin;
endif
ifneq ($(CACHE),1)
ifeq ($(REPORT),1)
	CGO_ENABLED=1 go test -race -count=1 -v $$(go list $(TEST_DIR)/...) && gotestsum --junitfile go.xml
else
	CGO_ENABLED=1 go test -race -count=1 -v $$(go list $(TEST_DIR)/...)
endif
else
ifeq ($(REPORT),1)
	CGO_ENABLED=1 go test -race -v $$(go list $(TEST_DIR)/...) && gotestsum --junitfile go.xml
else
	CGO_ENABLED=1 go test -race -v $$(go list $(TEST_DIR)/...)
endif
endif

test-coverage:
ifeq ($(CI),1)
	@export PATH=$$PATH:/usr/local/go/bin:$(shell go env GOPATH)/bin;
endif
	go test $(TEST_DIR)/... -coverpkg=$(TEST_DIR)/... -coverprofile=coverage.out
	go tool cover -html=coverage.out -o coverage.html

.PHONY: simple-test
simple-test:
	@echo "Running tests... Please wait."
	@go test $(TEST_DIR)/... -v
	@echo "✅ All tests passed successfully."

.PHONY: simple-install
simple-install:
	@echo "Installing Go module dependencies..."
	@go mod tidy
	@go mod download
	@go mod verify
	@echo "✅ Go module dependencies installed."

.PHONY: clean
PB_FILES := $(shell find $(OUT_DIR) -name "*.pb.gw.go")
PBW_FILES := $(shell find $(OUT_DIR) -name "*.pb.go")
GRPC_PB_FILES := $(shell find $(OUT_DIR) -name "*.grpc.pb.go")
clean:
	@echo "🧹 Cleaning generated files..."
	rm -f $(PB_FILES) $(PBW_FILES) $(GRPC_PB_FILES)
	@echo "✅ Clean complete."

test: test-sequential