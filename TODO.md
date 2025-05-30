## GRPC rewrite TODOs

- [x] Fixing FIXMEs in DB test
- [ ] Fixing AI tests to use new grpc methods
- [x] Fixing ahnlich client tests to use new grpc methods
- [x] Fixing ahnlich client README documentation that still references creating connection pools and TCP stuff
- [X] Fixing DSL to use grpc methods
- [x] Renaming grpc_types to ahnlich_types
- [x] Fixing CLI to use grpc methods
- [ ] Starting Python rewrite to use new grpc methods


## Python Rewrite
- [ ] Create a blocking client that wraps around betterproto async client
- [ ] Create CI step that checks that `grpc-update-client` does not produce any diffs so that our clients are always up to date.
- [X] Update tests.
- [ ] Fix Demo embed and demo tracing.
- [ ] Migrate README to reflect grpc client.
