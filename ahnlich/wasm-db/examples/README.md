# WASM DB Tests

```bash
# Build SDK
cd ../../../sdk/ahnlich-client-node
npm install && npm run build

# Build WASM
cd ../../ahnlich/wasm-db
wasm-pack build --target web --out-dir pkg

# Run tests
cd examples
node --test test.mjs
```
