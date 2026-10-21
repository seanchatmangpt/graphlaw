#!/usr/bin/env bash
# W04 — WASM artifact size ceiling + header court.
# Subject: the wasm32-wasip1 artifact for the CURRENT wasm/Cargo.toml VERSION.
# Exit codes: 0 pass / 1 breach / 3 missing-subject.
# Mirrors frozen-duckdb scripts/doctor/checks/bin_audit.sh discipline:
# measure a /tmp copy, every absence is loud, never substitute another artifact.
#
# Non-vacuity: WASM_AUDIT_FATFIXTURE=<path> points the court at a padded file
# instead of the real subject; a file over the ceiling MUST exit 1.
set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_MANIFEST="$REPO_ROOT/wasm/Cargo.toml"

# --- derive VERSION from wasm/Cargo.toml (single source of truth) ---
VERSION="$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$WASM_MANIFEST" | head -n1)"
if [[ -z "$VERSION" ]]; then
    echo "[FAIL] cannot derive version from $WASM_MANIFEST" >&2
    exit 3
fi

if [[ -n "${WASM_AUDIT_FATFIXTURE:-}" ]]; then
    SUBJECT="$WASM_AUDIT_FATFIXTURE"
    echo "== wasm_audit: FATFIXTURE mode (non-vacuity probe) =="
else
    # Subject resolution ladder (wasip1 layout varies by cargo profile era):
    # 1. GRAPHLAW_WASM override (explicit, wins)
    # 2. target/wasm32-wasip1/wasm/graphlaw_wasm.wasm   (legacy final-artifact dir)
    # 3. target/wasm32-wasip1/release/graphlaw_wasm.wasm (current release-dir layout)
    if [[ -n "${GRAPHLAW_WASM:-}" ]]; then
        SUBJECT="$GRAPHLAW_WASM"
    else
        SUBJECT=""
        for candidate in \
            "$REPO_ROOT/target/wasm32-wasip1/wasm/graphlaw_wasm.wasm" \
            "$REPO_ROOT/target/wasm32-wasip1/release/graphlaw_wasm.wasm"
        do
            if [[ -f "$candidate" ]]; then
                SUBJECT="$candidate"
                break
            fi
        done
        if [[ -z "$SUBJECT" ]]; then
            # keep the legacy path for the MISSING message; absence check below exits 3
            SUBJECT="$REPO_ROOT/target/wasm32-wasip1/wasm/graphlaw_wasm.wasm"
        fi
    fi
    echo "== wasm_audit: subject = wasm v${VERSION} artifact =="
fi

# --- 25 MiB ceiling ---
MAX_BYTES=26214400

if [[ ! -f "$SUBJECT" ]]; then
    echo "[MISSING] wasm artifact not found: $SUBJECT" >&2
    echo "[MISSING] build with: cargo build -p graphlaw-wasm --target wasm32-wasip1 --release" >&2
    exit 3
fi

echo "subject: $SUBJECT"
real_size="$(stat -f%z "$SUBJECT")"
echo "real size: ${real_size} bytes"

# --- (1) header magic: \0asm (canonical W04 court) ---
header="$(xxd -p -l4 "$SUBJECT")"
echo "header: ${header}"
if [[ "$header" != "0061736d" ]]; then
    echo "[FAIL] header magic ${header} is not 0061736d (\\0asm) — not a wasm module" >&2
    exit 1
fi
echo "[PASS] wasm header magic \\0asm present"

# --- (2) size ceiling on a stripped /tmp copy ---
TMP_COPY="$(mktemp -t wasm_audit.XXXXXX)"
trap 'rm -f "$TMP_COPY"' EXIT
cp "$SUBJECT" "$TMP_COPY"
if ! strip "$TMP_COPY" 2>/dev/null; then
    echo "[WARN] strip failed on tmp copy (wasm objects often unstrippable); measuring unstripped" >&2
    cp "$SUBJECT" "$TMP_COPY"
fi
measured_size="$(stat -f%z "$TMP_COPY")"
echo "measured size: ${measured_size} bytes (ceiling ${MAX_BYTES} = 25 MiB)"
if (( measured_size > MAX_BYTES )); then
    echo "[FAIL] wasm size ${measured_size} exceeds ceiling ${MAX_BYTES} (25 MiB)" >&2
    echo "== wasm_audit: BREACH =="
    exit 1
fi
echo "[PASS] wasm size within ceiling"

echo "== wasm_audit: PASS =="
exit 0
