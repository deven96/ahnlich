#!/bin/bash
# Fix wasm-bindgen-rayon worker imports for --target web

WORKER_HELPER=$(find pkg/snippets -name "workerHelpers.js" 2>/dev/null | head -1)

if [ -z "$WORKER_HELPER" ]; then
    echo "Error: workerHelpers.js not found"
    exit 1
fi

echo "Patching $WORKER_HELPER"

# Replace import('../../..') with import('/pkg/ahnlich_wasm_db.js')
sed -i.bak "s|await import('../../..')|await import('/pkg/ahnlich_wasm_db.js')|g" "$WORKER_HELPER"

if [ $? -eq 0 ]; then
    echo "Successfully patched worker imports"
    rm "${WORKER_HELPER}.bak"
else
    echo "Failed to patch worker imports"
    exit 1
fi
