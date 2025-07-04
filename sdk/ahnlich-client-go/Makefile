PROTO_DIR  := ../../protos
OUT_DIR    := grpc            # whatever folder you like INSIDE your module

PROTO_FILES := $(shell find $(PROTO_DIR) -name "*.proto")

PROTOC := $(shell which protoc)
PROTOC_GEN_GO := $(shell which protoc-gen-go)
PROTOC_GEN_GO_GRPC := $(shell which protoc-gen-go-grpc)

.PHONY: proto
proto:
	mkdir -p $(OUT_DIR)
	protoc \
	  -I $(PROTO_DIR) \
	  --go_opt=paths=source_relative \
	  --go-grpc_opt=paths=source_relative \
	  --go_out=$(OUT_DIR) \
	  --go-grpc_out=$(OUT_DIR) \
	  $(PROTO_FILES)
	@echo "✅ Protobuf files generated in $(OUT_DIR)."



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

.PHONY: test
test:
	@echo "Running tests... Please wait."
	@go test ./... -v
	@echo "✅ All tests passed successfully."

.PHONY: install
install:
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

.PHONY: all
# Run all tasks: install dependencies, generate proto files, and run tests
all: clean deps proto install test
	@echo "All tasks completed successfully."



