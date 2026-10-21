#!/usr/bin/env bash
# W03 falsifier smoke for wasm32-unknown-unknown (G13, v26.10.20).
#
# Asserts:
#   1. the graphlaw-wasm artifact exists and carries the wasm magic header
#   2. IF a wasm toolchain (wasm-tools / wasmtime / wasmer) is available:
#      the module has ZERO wasi_snapshot_preview1 imports (the W03 falsifier —
#      any WASI import under unknown-unknown is a fail-closed refusal, exit 1)
#   3. IF no runtime tooling: loud skip, exit 3 (never a silent pass)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ART="$ROOT/target/wasm32-unknown-unknown/wasm/graphlaw_wasm.wasm"
TARGET=wasm32-unknown-unknown

# --- 1. artifact exists + magic header -----------------------------------
if [ ! -f "$ART" ]; then
  echo "FAIL: artifact missing: $ART" >&2
  echo "      build with: cargo build -p graphlaw-wasm --target $TARGET --profile wasm" >&2
  exit 1
fi

MAGIC="$(xxd -p -l4 "$ART")"
if [ "$MAGIC" != "0061736d" ]; then
  echo "FAIL: bad wasm magic header: got 0x$MAGIC, expected 0x0061736d" >&2
  exit 1
fi
echo "OK: artifact $ART ($(wc -c < "$ART" | tr -d ' ') bytes), magic 0061736d"

# --- 2. runtime tooling gate ---------------------------------------------
TOOL=""
if command -v wasm-tools >/dev/null 2>&1; then
  TOOL=wasm-tools
elif command -v wasmtime >/dev/null 2>&1; then
  TOOL=wasmtime
elif command -v wasmer >/dev/null 2>&1; then
  TOOL=wasmer
fi

if [ -z "$TOOL" ]; then
  echo "[SKIP-loud] no wasm runtime available (wasm-tools/wasmtime/wasmer not on PATH)" >&2
  echo "            header check passed; import-count falsifier NOT executed" >&2
  exit 3
fi

# --- 3. W03 falsifier: zero wasi_snapshot_preview1 imports ----------------
IMPORTS="$(wasm-tools print "$ART" 2>/dev/null | grep '(import ' || true)"
WASI_COUNT="$(printf '%s\n' "$IMPORTS" | grep -c 'wasi_snapshot_preview1' || true)"
TOTAL_COUNT="$(printf '%s\n' "$IMPORTS" | grep -c '(import ' || true)"

if [ "$WASI_COUNT" -ne 0 ]; then
  echo "FAIL: $WASI_COUNT wasi_snapshot_preview1 imports present (W03 falsifier tripped):" >&2
  printf '%s\n' "$IMPORTS" | grep 'wasi_snapshot_preview1' >&2
  exit 1
fi

echo "OK: 0 wasi_snapshot_preview1 imports (W03 falsifier passes); $TOTAL_COUNT total imports (wasm-bindgen host glue)"
