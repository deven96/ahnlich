# Claude Code Review Instructions for Ahnlich

When reviewing pull requests for Ahnlich, follow these guidelines:

## Project Context

Ahnlich is a high-performance in-memory vector database system written in Rust.

- **Language**: Rust (edition 2024, toolchain 1.97.1)
- **Architecture**: Multi-crate workspace with 12+ crates
- **Core components**: 
  - ahnlich-db (vector storage)
  - ahnlich-ai (AI proxy with ONNX)
- **Communication**: gRPC/Protocol Buffers
- **Concurrency**: Lock-free data structures (Papaya), Tokio async runtime

## Review Checklist

### Code Quality & Style
- [ ] Code follows Rust formatting standards (cargo fmt)
- [ ] Passes Clippy linting without warnings
- [ ] Uses idiomatic Rust patterns (proper error handling, no unwrap() in production code)
- [ ] Meaningful variable and function names (snake_case for functions, PascalCase for types)
- [ ] No commented-out code
- [ ] DRY principle followed

### Testing & Safety
- [ ] Unit tests for new functionality
- [ ] Integration tests for new features (especially gRPC endpoints)
- [ ] Edge cases covered
- [ ] Unsafe code is properly documented and justified
- [ ] No panics in production code paths

### Performance & Concurrency
- [ ] Efficient use of lock-free data structures where appropriate
- [ ] Proper async/await usage with Tokio
- [ ] No blocking operations in async contexts
- [ ] Appropriate use of Arc/Mutex vs message passing
- [ ] SIMD optimizations considered for hot paths (similarity algorithms)

### Protocol & API Stability
- [ ] Protobuf changes are backward compatible
- [ ] gRPC endpoint changes are documented
- [ ] Client SDKs updated if protocol changes (check if `make grpc-update-clients` needed)
- [ ] Version bumps appropriate for breaking changes

### Security
- [ ] No hardcoded credentials or API keys
- [ ] Input validation for all external inputs
- [ ] Proper error handling without exposing sensitive information
- [ ] Safe predicate parsing and evaluation (prevent code injection)
- [ ] Safe handling of user-provided vectors and metadata
- [ ] Proper bounds checking for array/vector operations

### Documentation
- [ ] Public APIs have doc comments (///)
- [ ] Complex logic has inline comments
- [ ] README updated for new features
- [ ] CHANGELOG.md updated if needed
- [ ] Examples updated if API changes

### Ahnlich-Specific Considerations
- [ ] Store operations maintain consistency
- [ ] Similarity algorithm changes preserve deterministic behavior
- [ ] Persistence logic maintains snapshot integrity
- [ ] Persistence format changes are backward compatible (can load old snapshots)
- [ ] Deterministic hashing (AHash with fixed seed) not broken for existing data
- [ ] AI model preprocessing changes are tested with sample data
- [ ] Memory usage is reasonable for in-memory database operations

### Performance Regressions
- [ ] Check benchmark results in PR comments for any significant slowdowns
- [ ] Verify hot paths (similarity algorithms, vector operations) haven't regressed
- [ ] Look for new allocations or locks in critical sections
- [ ] Ensure no O(n²) algorithms introduced where O(n) or O(log n) existed
- [ ] Check if new dependencies add significant compile time or binary size
- [ ] Flag any >5% performance degradation in core operations (insertion, retrieval, similarity search)

## Review Instructions

1. Review the PR against the checklist above
2. Focus on correctness, safety, and performance
3. Use inline comments for specific code issues
4. Post a summary comment with checklist results
5. Be constructive and specific in feedback
6. Highlight both issues and well-implemented patterns
