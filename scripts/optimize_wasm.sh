#!/usr/bin/env bash
# scripts/optimize_wasm.sh — post-link optimization and size gate for graphlaw-reconstituted.wasm
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_WASM="${1:-$REPO_ROOT/target/wasm32-wasip1/release/graphlaw_reconstituted.wasm}"
OUTPUT_WASM="${2:-$REPO_ROOT/target/wasm32-wasip1/release/graphlaw_reconstituted_opt.wasm}"
MAX_CEILING=35000000 # 35 MB

if [ ! -f "$TARGET_WASM" ]; then
    echo "NOTICE: Target wasm not yet built at $TARGET_WASM (skipping binaryen pass during early compilation)"
    exit 0
fi

if command -v wasm-opt >/dev/null 2>&1; then
    echo "Running wasm-opt -Oz --strip-debug on $TARGET_WASM..."
    wasm-opt -Oz --strip-debug "$TARGET_WASM" -o "$OUTPUT_WASM"
else
    echo "WARNING: wasm-opt not found in PATH; copying without binaryen optimization"
    cp "$TARGET_WASM" "$OUTPUT_WASM"
fi

SIZE=$(wc -c < "$OUTPUT_WASM" | tr -d ' ')
echo "Optimized WASM size: $SIZE bytes (ceiling: $MAX_CEILING bytes)"

if [ "$SIZE" -gt "$MAX_CEILING" ]; then
    echo "ERROR: Optimized WASM size $SIZE exceeds the 35 MB ceiling ($MAX_CEILING bytes)!" >&2
    exit 1
fi

echo "VERIFIED: WASM artifact is within the 35 MB ceiling."
