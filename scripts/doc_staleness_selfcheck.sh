#!/usr/bin/env bash
# doc_staleness_selfcheck.sh — P17 G4: anti-vacuity selfcheck for the
# doc-staleness gate's SYMBOLIC anchor form (`path#SYMBOL`).
#
# Owns the flip-proof for: scripts/doc_staleness_gate.sh#ANCHOR_SYM
# (R5-SYMBOL rule). The gate itself is lane E10's file — this selfcheck only
# DRIVES it, on a scratch corpus, the way the gate header documents for
# anti-vacuity proofs (ROOT argument = standalone scratch tree).
#
# Blocked-on-E10: none — the #SYMBOL form is landed on disk
# (scripts/doc_staleness_gate.sh:189 ANCHOR_SYM; R5-SYMBOL findings :213-222).
# If E10 moves the anchor form, this selfcheck flips red by design; coordinate
# the two files in the same commit.
#
# Legs:
#   1. FRESH:  corpus doc citing a live symbol  -> gate exits 0
#   2. ROT:    symbol renamed under the doc     -> gate exits 1 with R5-SYMBOL
#   3. FRESH REMOVAL: anchor removed from doc   -> gate exits 0
set -u
REPO="$(cd "$(dirname "$0")/.." && pwd)"
GATE="$REPO/scripts/doc_staleness_gate.sh"
[ -f "$GATE" ] || { echo "selfcheck: REFUSED — gate missing at $GATE"; exit 2; }

CORPUS="$(mktemp -d "${TMPDIR:-/tmp}/doc-staleness-selfcheck.XXXXXX")"
trap 'rm -rf "$CORPUS"' EXIT

# Scratch corpus: a standalone repo-shaped root. README.md/CLAUDE.md are
# mandatory gate inputs (Rule 1 fails "file missing" without them); the anchor
# doc lives in the newest docs/jira window (the gate's Rule 5 scan set).
mkdir -p "$CORPUS/docs/jira/v26.10.24" "$CORPUS/src"
printf '# selfcheck corpus readme\n\nNo fenced commands here.\n' > "$CORPUS/README.md"
printf '# selfcheck corpus claude.md\n\nNo fenced commands here.\n' > "$CORPUS/CLAUDE.md"

write_target() { # $1 = symbol name
  printf 'pub fn %s() -> usize {\n    7\n}\n' "$1" > "$CORPUS/src/selfcheck_target.rs"
}
write_doc() { # $1 = anchor line or empty
  {
    printf '# Anchor doc\n\n'
    if [ -n "${1:-}" ]; then printf '%s\n' "$1"; fi
  } > "$CORPUS/docs/jira/v26.10.24/anchor-doc.md"
}

fail=0

# --- Leg 1: FRESH — live symbol anchor passes (positive control).
write_target "selfcheck_symbol"
write_doc "The live entry point is src/selfcheck_target.rs#selfcheck_symbol (symbolic anchor)."
out1="$(bash "$GATE" "$CORPUS" 2>&1)"; rc1=$?
if [ "$rc1" -eq 0 ]; then
  echo "LEG1 FRESH: gate exit 0 on live symbol anchor — PASS"
else
  echo "LEG1 FRESH: FAIL (exit $rc1, expected 0)"; echo "$out1"; fail=1
fi

# --- Leg 2: ROT — rename the symbol under an unchanged doc; gate must exit 1
# with an R5-SYMBOL finding naming the rotted anchor.
write_target "renamed_away_symbol"
out2="$(bash "$GATE" "$CORPUS" 2>&1)"; rc2=$?
if [ "$rc2" -eq 1 ] && printf '%s' "$out2" | grep -q 'R5-SYMBOL' \
   && printf '%s' "$out2" | grep -q 'selfcheck_symbol'; then
  echo "LEG2 ROT: gate exit 1 with R5-SYMBOL finding on the rotted anchor — PASS"
else
  echo "LEG2 ROT: FAIL (exit $rc2, expected 1 + R5-SYMBOL finding)"; echo "$out2"; fail=1
fi

# --- Leg 3: FRESH REMOVAL — drop the stale anchor from the doc; gate exits 0.
write_doc ""
out3="$(bash "$GATE" "$CORPUS" 2>&1)"; rc3=$?
if [ "$rc3" -eq 0 ]; then
  echo "LEG3 FRESH-REMOVAL: gate exit 0 after anchor removal — PASS"
else
  echo "LEG3 FRESH-REMOVAL: FAIL (exit $rc3, expected 0)"; echo "$out3"; fail=1
fi

echo "$out2" | grep 'R5-SYMBOL' | sed 's/^/  witness: /'

if [ "$fail" -eq 0 ]; then
  echo "doc_staleness_selfcheck: PASS (corpus $CORPUS removed)"
  exit 0
fi
echo "doc_staleness_selfcheck: FAIL"
exit 1
