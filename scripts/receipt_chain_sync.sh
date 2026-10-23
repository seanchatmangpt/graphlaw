#!/usr/bin/env bash
# receipt_chain_sync.sh — BLAKE3 sidecar chain over receipts/ (lane L52).
#
# Enumerates every file under receipts/ (recursive, sorted by relpath),
# computes b3sum per file, and writes receipts/CHAIN.blake3:
#   one line per receipt:   <digest>  <relpath>
#   final line:             chain_head  <digest-of-concatenated-digests>
#
# Modes:
#   (default)        sync: verify existing chain, then write/update CHAIN.blake3.
#                    TAMPER-DETECTED + exit 1 if any receipt file's digest
#                    changed vs the existing chain (receipts are immutable;
#                    a new file only extends the chain, it never rewrites one).
#   --check          verify only, no write. exit 0 verified / 1 tamper-or-missing.
#
# Target tree: defaults to the repo containing this script; an optional second
# argument names another root (e.g. a /tmp copy for tamper-testing):
#   scripts/receipt_chain_sync.sh [--check] [repo-root]
#
# Idempotent: second sync run with no receipt changes verifies and writes
# a byte-identical CHAIN.blake3.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [ "${2:-}" != "" ]; then
  REPO_ROOT="$(cd "$2" && pwd)"
fi
RECEIPTS_DIR="$REPO_ROOT/receipts"
CHAIN="$RECEIPTS_DIR/CHAIN.blake3"
MODE="${1:-sync}"

if [ "$MODE" != "--check" ] && [ "$MODE" != "sync" ]; then
  echo "USAGE: $0 [sync|--check]" >&2
  exit 2
fi

cd "$REPO_ROOT"

# Enumerate all receipt files recursively, sorted, relative to repo root.
mapfile -t FILES < <(cd "$RECEIPTS_DIR" && find . -type f ! -name 'CHAIN.blake3' -print | sed 's|^\./||' | LC_ALL=C sort | sed 's|^|receipts/|')

if [ "${#FILES[@]}" -eq 0 ]; then
  echo "TAMPER-CHECK-SKIP: no receipt files under $RECEIPTS_DIR" >&2
  exit 1
fi

# Current per-file digests, sorted order.
CURRENT="$(b3sum "${FILES[@]}")"
CURRENT_HEAD="$(printf '%s' "$CURRENT" | b3sum | awk '{print $1}')"

verify_against_chain() {
  if [ ! -f "$CHAIN" ]; then
    echo "NO-CHAIN: $CHAIN missing" >&2
    return 1
  fi
  local existing_head
  existing_head="$(awk '$1=="chain_head"{print $2}' "$CHAIN")"
  if [ -z "$existing_head" ]; then
    echo "TAMPER-DETECTED: $CHAIN has no chain_head line" >&2
    return 1
  fi
  local drifted=0
  # Verify each receipt file listed in the existing chain against its recorded digest.
  while read -r digest relpath; do
    [ "$digest" = "chain_head" ] && continue
    if [ ! -f "$relpath" ]; then
      echo "TAMPER-DETECTED: $relpath (file listed in chain is missing)" >&2
      drifted=1
      continue
    fi
    local actual
    actual="$(b3sum "$relpath" | awk '{print $1}')"
    if [ "$actual" != "$digest" ]; then
      echo "TAMPER-DETECTED: $relpath (chain=$digest actual=$actual)" >&2
      drifted=1
    fi
  done < <(awk 'NF>=2{print $1, $2}' "$CHAIN")
  return "$drifted"
}

if [ "$MODE" = "--check" ]; then
  if verify_against_chain; then
    echo "VERIFIED: $(grep -cv '^chain_head' "$CHAIN" || true) receipts match CHAIN.blake3; head=$CURRENT_HEAD"
    exit 0
  fi
  exit 1
fi

# sync mode: existing chain must verify before any write.
if [ -f "$CHAIN" ]; then
  if ! verify_against_chain; then
    echo "REFUSED: chain drift above; receipts are immutable — no CHAIN.blake3 write (exit 1)" >&2
    exit 1
  fi
fi

NEW_CHAIN="$(printf '%s\nchain_head %s\n' "$CURRENT" "$CURRENT_HEAD")"
if [ -f "$CHAIN" ] && [ "$(cat "$CHAIN")" = "$NEW_CHAIN" ]; then
  echo "NO-OP: CHAIN.blake3 already current; head=$CURRENT_HEAD"
  exit 0
fi

printf '%s\n' "$NEW_CHAIN" > "$CHAIN"
echo "SYNCED: ${#FILES[@]} receipts chained; head=$CURRENT_HEAD -> $CHAIN"
