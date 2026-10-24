#!/usr/bin/env bash
# TC-MUT-01 shape-flip court (anti-vacuity over ontologies/sql-ast.ttl shapes).
#
# For each mutation: snapshot sha256 of the shape file, mutate in place, run
# the sql-rdf courts, restore byte-exact via EXIT trap (fail-closed).
#
# Mutations:
#   M1: remove "LIKE" "ILIKE" from the sql:operator sh:in list.
#       NOTE (reader backstop, by design): the operator enum is ALSO enforced
#       code-side (sql_ast.rs Operator::from_str), so the shape-layer operator
#       court is E2 coverage, not a flip court — M1/M2-style operator-enum
#       mutations are NOT expected to flip named courts on their own.
#   M2: remove sh:minCount 1 from the SelectStatementShape from-clause branches
#       (sql:fromTable / sql:from sh:or) — a from-less SELECT must newly admit
#       and named courts must flip (witnessed flip class from shim-court work).
#
# Exit 0 = at least one named court flipped under the mutations.
# Exit 3 = no named court flipped (gate is vacuous).
# Exit 4 = restore failed byte-exact verification (loud, never silent).
# Exit 5 = a mutation did not apply (refusing to judge).
#
# Usage: bash scripts/shape_mutation_court.sh [M1|M2|all]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SHAPE="$ROOT/ontologies/sql-ast.ttl"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target-shapemut}"
COURTS=(--test sql_rdf_dialect --test sql_rdf_laundering_corpus)
BACKUP="$(mktemp /tmp/sql-ast.ttl.snapshot.XXXXXX)"
RESTORED_OK=1

cp "$SHAPE" "$BACKUP"
BASE_SHA="$(shasum -a 256 "$SHAPE" | awk '{print $1}')"

restore() {
  if cp "$BACKUP" "$SHAPE"; then
    local now
    now="$(shasum -a 256 "$SHAPE" | awk '{print $1}')"
    if [ "$now" != "$BASE_SHA" ]; then
      echo "SHAPE-COURT: RESTORE FAILED (sha256 mismatch $now != $BASE_SHA)" >&2
      RESTORED_OK=0
    fi
  else
    echo "SHAPE-COURT: RESTORE FAILED (copy) — snapshot kept at $BACKUP" #: $BACKUP" >&2
    RESTORED_OK=0
  fi
  rm -f "$BACKUP"
  if [ "$RESTORED_OK" -ne 1 ]; then
    echo "SHAPE-COURT: fail-closed exit 4" >&2
    exit 4
  fi
}
trap restore EXIT

mutate_m1() {
  # M1: remove LIKE/ILIKE from the sh:in operator list. Successor-anchored:
  # the enum grew set-op tokens, sql-ast.ttl:490 now reads
  # ..."LIKE" "ILIKE" "->" "->>" — anchor the lookahead on those successors
  # (a trailing-\) anchor no longer matches and M1 would "not apply" -> exit 5).
  perl -pi -e 's/ "LIKE" "ILIKE"(?= "->" "->>" \))//' "$SHAPE"
}
mutate_m2() {
  # M2: drop sh:minCount 1 from the from-clause branches of
  # sql:SelectStatementShape so a from-less SELECT newly admits.
  perl -0pi -e 's/(sh:path sql:from(?:Table)? ;\n\s*)sh:minCount 1 ;\n/$1/g' "$SHAPE"
}

# A mutation "applied" iff the shape file's content hash changed from baseline.
assert_mutation_applied() {
  local now
  now="$(shasum -a 256 "$SHAPE" | awk '{print $1}')"
  [ "$now" != "$BASE_SHA" ]
}

run_courts() {
  # Prints the names of courts that FLIPPED (failed) under the mutation.
  (cd "$ROOT" && env CARGO_TARGET_DIR="$TARGET_DIR" \
    cargo test "${COURTS[@]}" 2>&1 || true) \
    | awk '/^test .* \.\.\. FAILED$/ { print $2 }' | sort -u
}

which="${1:-all}"
case "$which" in
  M1) run_mutations=(mutate_m1) ;;
  M2) run_mutations=(mutate_m2) ;;
  all) run_mutations=(mutate_m1 mutate_m2) ;;
  *) echo "usage: $0 [M1|M2|all]" >&2; exit 2 ;;
esac

total_flips=0
for m in "${run_mutations[@]}"; do
  cp "$BACKUP" "$SHAPE"   # each mutation applies to the pristine shape
  "$m"
  if ! assert_mutation_applied "$m"; then
    echo "SHAPE-COURT: mutation $m did not apply — refusing to judge" >&2
    exit 5
  fi
  echo "SHAPE-COURT: mutation $m applied, running courts..."
  flips="$(run_courts)"
  if [ -z "$flips" ]; then
    echo "SHAPE-COURT: $m — NO named court flipped" >&2
  else
    while IFS= read -r t; do
      [ -n "$t" ] && echo "SHAPE-COURT: $m FLIP: $t"
    done <<< "$flips"
  fi
  n="$(printf '%s' "$flips" | grep -c . || true)"
  total_flips=$((total_flips + n))
done

if [ "$total_flips" -eq 0 ]; then
  echo "SHAPE-COURT: VACUOUS — no court flips under any mutation" >&2
  exit 3
fi
echo "SHAPE-COURT: PASS — $total_flips named court flip(s) witnessed"
