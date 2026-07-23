#!/bin/bash
# This script copies the built WASM package to Docusaurus static files.
# It runs automatically in GitHub Actions and via prestart/prebuild npm hooks.
# For local development, ensure you've built WASM first: cd ../../ahnlich/wasm-db && ./build.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEB_DIR="$(dirname "$SCRIPT_DIR")"
WASM_PKG_SRC="$WEB_DIR/../../ahnlich/wasm-db/pkg"
WASM_PKG_DEST="$WEB_DIR/static/wasm-pkg"

echo "🔍 Checking for WASM package at $WASM_PKG_SRC"

# Check if source exists
if [ ! -d "$WASM_PKG_SRC" ]; then
  echo "❌ Error: WASM package not found at $WASM_PKG_SRC"
  echo "💡 Please build it first with: cd ../../ahnlich/wasm-db && ./build.sh"
  exit 1
fi

# Create destination if it doesn't exist
mkdir -p "$WASM_PKG_DEST"

echo "📦 Copying WASM package from $WASM_PKG_SRC to $WASM_PKG_DEST"

# Copy all files except package.json and node_modules
rsync -av --exclude 'package.json' --exclude 'node_modules' --exclude '.gitignore' --exclude '*.tgz' \
  "$WASM_PKG_SRC/" "$WASM_PKG_DEST/"

echo "✅ WASM package copied successfully!"
echo "📂 Files available at static/wasm-pkg/ for imports"
