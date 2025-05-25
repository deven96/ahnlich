## GRPC rewrite TODOs

- [x] Fixing FIXMEs in DB test
- [ ] Fixing AI tests to use new grpc methods
- [ ] Fixing ahnlich client tests to use new grpc methods
- [ ] Fixing ahnlich client README documentation that still references creating connection pools and TCP stuff
- [X] Fixing DSL to use grpc methods
- [ ] Renaming grpc_types to ahnlich_types
- [ ] Fixing CLI to use grpc methods
- [ ] Starting Python rewrite to use new grpc methods



### REWRITE NOTES:
 - Client DB tests are flaky:
    ```rust
    thread 'grpc::db::test::test_simple_pipeline' panicked at /Users/davidonuh/Sandbox/rust/ahnlich/ahnlich/utils/src/server.rs:68:33:
    Could not set up ahnlich-db with allocator_size

    thread 'grpc::db::test::test_simple_pipeline' panicked at client/src/grpc/db.rs:584:14:
    Could not initialize client: Tonic(tonic::transport::Error(Transport, ConnectError(ConnectError("tcp connect error", Os { code: 61, kind: ConnectionRefused, message: "Connection refused" }))))
    ```