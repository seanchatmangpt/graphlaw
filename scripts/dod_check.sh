#!/usr/bin/env bash
# dod_check.sh - read-only Definition-of-Done probes for v26.11.25.
# Cheap non-compiling checks only. Exit 1 if any FAIL. SKIP is loud, not a pass.
set -euo pipefail
command -v rg >/dev/null 2>&1 || { echo "refusing to fake-pass: rg absent"; exit 2; }
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
V="26.11.25"
MP="${GGEN_MARKETPLACE:-$HOME/ggen-marketplace}"
fails=0; skips=0; passes=0
pass() { echo "PASS  $1"; passes=$((passes+1)); }
fail() { echo "FAIL  $1 -- $2"; fails=$((fails+1)); }
skip() { echo "SKIP  $1 -- $2 (NOT a pass)"; skips=$((skips+1)); }

exists() { # label path
  if [ -e "$2" ]; then pass "$1 exists ($2)"; else fail "$1 exists" "missing $2"; fi
}
ver_eq() { # label file rg-pattern
  if [ ! -f "$2" ]; then fail "$1 version" "missing $2"; return; fi
  if rg -q -e "$3" "$2"; then pass "$1 version = $V"; else fail "$1 version = $V" "no match of /$3/ in $2"; fi
}

echo "== Stage 1: Rust =="
ver_eq "Cargo.toml" Cargo.toml '^version = "26\.11\.25"'
ver_eq "registry/mcp-manifest.json" registry/mcp-manifest.json '"version": "26\.11\.25"'
exists "registry/capability-registry.json" registry/capability-registry.json
exists "registry/ARTIFACTS.blake3" registry/ARTIFACTS.blake3
exists "docs/refusals.md" docs/refusals.md
ver_eq "CHANGELOG.md" CHANGELOG.md '26\.11\.25'
for s in mutation_court.sh spc_court.sh; do exists "script $s" "scripts/$s"; done
free_gib=$(df -k / | awk 'NR==2{printf "%d", $4/1048576}')
if [ "$free_gib" -ge 20 ]; then pass "df / free ${free_gib} GiB >= 20"; else fail "df / >= 20 GiB" "${free_gib} GiB free"; fi
skip "cargo check / courts / full suite / mutation / spc / wasm courts" "compile+run gates are not run here; see DEFINITION-OF-DONE.md"

echo "== wasm pin =="
W=graphlaw_ex/priv/graphlaw/graphlaw.wasm
M=graphlaw_ex/priv/graphlaw/MANIFEST.json
ver_eq "MANIFEST.json graphlaw_version" "$M" '"graphlaw_version": "26\.11\.25"'
if [ -f "$W" ] && [ -f "$M" ]; then
  have_sha=$(shasum -a 256 "$W" | awk '{print $1}')
  if rg -q "\"sha256\": \"$have_sha\"" "$M"; then pass "wasm sha256 matches MANIFEST.json"; else fail "wasm sha256 vs MANIFEST.json" "$have_sha not in $M"; fi
  if command -v b3sum >/dev/null 2>&1; then
    have_b3=$(b3sum "$W" | awk '{print $1}')
    if rg -q "\"blake3\": \"$have_b3\"" "$M"; then pass "wasm blake3 matches MANIFEST.json"; else fail "wasm blake3 vs MANIFEST.json" "$have_b3 not in $M"; fi
  else skip "wasm blake3" "b3sum absent"; fi
else fail "wasm + MANIFEST present" "missing $W or $M"; fi

echo "== Stage 2: Elixir =="
ver_eq "graphlaw_ex/mix.exs" graphlaw_ex/mix.exs '@version "26\.11\.25"'
ver_eq "graphlaw_ex/CHANGELOG.md" graphlaw_ex/CHANGELOG.md '26\.11\.25'
ver_eq "pack graphlaw-ash-capability-pack" "$MP/packs/graphlaw-ash-capability-pack/pack.toml" '^version = "26\.11\.25"'
if [ -f "$MP/packs/chicago-graphlaw-court-pack/pack.toml" ]; then
  ver_eq "pack chicago-graphlaw-court-pack" "$MP/packs/chicago-graphlaw-court-pack/pack.toml" '^version = "26\.11\.25"'
else skip "chicago-graphlaw-court-pack version" "pack.toml absent"; fi
G="$MP/packs/graphlaw-ash-capability-pack/gates"
if [ -f "$G/190_refusal_kind_class_contract.rq" ]; then pass "gate 190 present"; else fail "gate 190 present" "missing in $G"; fi
exists "ggen_sync script" graphlaw_ex/scripts/ggen_sync.sh
for g in count_gate doc_staleness_gate exit_capture_check machine_shaped_gate witness_gate \
         partition_court receipt_replay_court shape_mutation_court; do
  exists "script gate $g.sh" "scripts/$g.sh"
done
if command -v reuse >/dev/null 2>&1; then
  if (cd graphlaw_ex && reuse lint >/dev/null 2>&1); then pass "reuse lint"; else fail "reuse lint" "non-zero"; fi
else skip "reuse lint" "reuse not installed"; fi
T=graphlaw_ex/graphlaw_ex-$V.tar
if [ -f "$T" ]; then
  sz=$(wc -c < "$T" | tr -d ' ')
  if [ "$sz" -lt 8000000 ]; then pass "hex tarball $sz B < 8000000"; else fail "hex tarball < 8000000 B" "$sz B"; fi
  if tar -tf "$T" 2>/dev/null | rg -q 'graphlaw\.wasm$'; then fail "tarball excludes wasm" "wasm entry found"; else pass "outer tar lists no graphlaw.wasm (contents.tar.gz not unpacked)"; fi
else skip "hex tarball size" "$T absent (mix hex.build not run)"; fi
if [ -f "$HOME/ash_r2rml/mix.exs" ]; then pass "ash_r2rml present"; else skip "ash_r2rml present" "~/ash_r2rml/mix.exs absent"; fi
skip "mix compile/format/test/dialyzer/verify/doctor/zero-egress/ash_r2rml suite" "mix not run here"

echo "== Stage 3: handoff =="
skip "explicit-path staging, ls-remote x3, Release, hex.publish" "operator-gated; HOLD for operator go"

echo "== summary: PASS=$passes FAIL=$fails SKIP=$skips =="
[ "$fails" -eq 0 ] || exit 1
