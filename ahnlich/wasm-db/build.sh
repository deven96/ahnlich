#!/bin/bash
set -e

echo "Building ahnlich-wasm-db with threading support..."

# Clean previous build
rm -rf pkg

# Build with wasm-pack (uses .cargo/config.toml for flags)
wasm-pack build --target web --out-dir pkg

echo "Fixing worker imports..."
# Fix worker helper to use /wasm-pkg/ path for Docusaurus
WORKER_HELPER=$(find pkg/snippets -name "workerHelpers.js" 2>/dev/null | head -1)
if [ -n "$WORKER_HELPER" ]; then
    sed -i.bak "s|await import('../../..')|await import('/wasm-pkg/ahnlich_wasm_db.js')|g" "$WORKER_HELPER"
    rm "${WORKER_HELPER}.bak"
    echo "✓ Worker imports fixed"
else
    echo "⚠ Warning: workerHelpers.js not found"
fi

echo "Bundling protobuf types..."
# Check if protobuf SDK exists
if [ ! -d "../../sdk/ahnlich-client-node/dist/grpc" ]; then
    echo "⚠ Warning: SDK not found at ../../sdk/ahnlich-client-node/dist/grpc"
    echo "   Skipping protobuf bundle"
elif [ -f "protobuf-entry.js" ]; then
    # Use existing entry point from repo
    npx --yes esbuild --bundle --format=esm --outfile=pkg/protobuf-bundle.js protobuf-entry.js
    if [ $? -eq 0 ]; then
        # Add explicit default export at the end
        echo "export default protobuf_entry_exports;" >> pkg/protobuf-bundle.js
        echo "✓ Protobuf bundle created ($(du -h pkg/protobuf-bundle.js | cut -f1))"
    else
        echo "⚠ Warning: protobuf bundling failed"
    fi
else
    echo "⚠ Warning: protobuf-entry.js not found, skipping bundle"
fi

echo "Creating package.json..."
if [ -f "pkg/package.json.template" ]; then
    cp pkg/package.json.template pkg/package.json
    echo "✓ package.json created"
fi

echo ""
echo "✓ Build complete!"
echo ""
echo "Package contents:"
ls -lh pkg/*.{js,wasm} 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "To test locally:"
echo "  cd wasm-db && python3 examples/server.py"
echo "  Open http://localhost:8000/benchmark.html"
echo ""
echo "To publish to npm:"
echo "  cd pkg && npm publish"
