#!/usr/bin/env bash
# Compare metadata representations while keeping Criterion benchmark IDs identical.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
AHNLICH_DIR="$(cd -- "$SCRIPT_DIR/../ahnlich" && pwd)"
REPRESENTATIONS="${METADATA_REPRESENTATIONS:-owned key_interned interned}"

command -v cargo >/dev/null 2>&1 || {
    echo "error: cargo is required" >&2
    exit 1
}
command -v critcmp >/dev/null 2>&1 || {
    echo "error: critcmp is required (cargo install critcmp)" >&2
    exit 1
}

read -r -a REPRESENTATION_LIST <<< "$REPRESENTATIONS"
[ "${#REPRESENTATION_LIST[@]}" -gt 1 ] || {
    echo "error: provide at least two METADATA_REPRESENTATIONS" >&2
    exit 1
}

cd "$AHNLICH_DIR"
for representation in "${REPRESENTATION_LIST[@]}"; do
    printf '\n==> metadata representation baseline: %s\n' "$representation"
    AHNLICH_METADATA_REPRESENTATION="$representation" \
        cargo bench --bench metadata_representation -- \
        "$@" --save-baseline "$representation"
done

printf '\n==> Comparison\n'
critcmp "${REPRESENTATION_LIST[@]}"
