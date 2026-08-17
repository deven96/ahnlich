#!/bin/bash
set -euo pipefail

# Get the directory where this script lives
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLCHAIN_FILE="${SCRIPT_DIR}/../rust-toolchain.toml"

if [[ ! -f "$TOOLCHAIN_FILE" ]]; then
    echo "Error: rust-toolchain.toml not found at $TOOLCHAIN_FILE" >&2
    exit 1
fi

# Extract version from: channel = "1.97.1"
grep -E '^channel\s*=' "$TOOLCHAIN_FILE" | sed 's/.*"\(.*\)".*/\1/'
