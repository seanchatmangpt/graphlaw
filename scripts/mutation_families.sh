#!/usr/bin/env bash
# mutation_families.sh — P12 B1 mutation fleet driver over scripts/mutation_court.sh.
#
#   scripts/mutation_families.sh --preflight
#       Gate only: concurrent-court guard, per-family --list counts, universe
#       consistency across the four family configs, pairwise-disjointness of
#       the three explicit families, union==inventory cover, and the
#       fleet-vs-default-config skew check (kill-set-equality precondition).
#       Exit 0 clean; exit 8 typed refusal; exit 2 usage/missing config.
#
#   scripts/mutation_families.sh [--family NAME]...
#       Full fleet (or the --family subset): preflight gate, then per family
#       the court run, then ESCALATION (per family with missed>0: re-run the
#       missed mutants' survivor regex under the DEFAULT committed config so
#       the full killer set gets one shot at each survivor), then the verdict:
#       merged missed.txt + the committed mutation_court.sh GATED predicate
#       grep + per-family caught/missed table + (full fleet only) the
#       kill-set-equality check of the three explicit families against the
#       catchall reference run.
#
# Exit codes: 0 pass | 2 usage/missing-config | 3 FLEET-GATE-FAIL (court GATED
# predicate nonempty on merged missed) | 4 family-run infrastructure failure |
# 5 KILL-SET-INEQUALITY | 8 preflight refusal (concurrent court, disjointness
# violation, inventory skew, gate-predicate drift).
#
# Families and --case regexes (P12 B1):
#   builders   limit_response|missing_buffer_response|unknown_buffer_response
#   specstate  dialect_by_name|spec_dialect|spec_of|state_of|state_field
#   dispatch   \bcall\b|\bcall_json\b|\bdispatch\b|\brespond\b|refusal_json|\bbad(_refusal)?\b
#   catchall   (no --case: full universe, full committed 8+builders killer set;
#               doubles as the kill-set-equality reference run)
#
# Solo-run law: the court mutates src/ in place; the fleet refuses to start
# while another court process is live (typed refusal, exit 8) and runs its
# families strictly serially.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

COURT="$ROOT/scripts/mutation_court.sh"
OUTROOT="${MUTANTS_FLEET_OUTPUT:-target-mutants/families}"

# ---- arguments ---------------------------------------------------------------
PREFLIGHT_ONLY=0
FAMILIES_SEL=()
while [ $# -gt 0 ]; do
  case "$1" in
    --preflight) PREFLIGHT_ONLY=1; shift ;;
    --family)
      [ $# -ge 2 ] || { echo "mutation_families: --family requires a name" >&2; exit 2; }
      FAMILIES_SEL+=("$2"); shift 2 ;;
    -h|--help) sed -n '2,33p' "$0"; exit 0 ;;
    *) echo "mutation_families: unknown argument: $1 (use --preflight | --family NAME)" >&2; exit 2 ;;
  esac
done

# ---- family table (P12 B1) ---------------------------------------------------
ALL_FAMILIES=(builders specstate dispatch catchall)
declare -A REGEX=(
  [builders]='limit_response|missing_buffer_response|unknown_buffer_response'
  [specstate]='dialect_by_name|spec_dialect|spec_of|state_of|state_field'
  [dispatch]='\bcall\b|\bcall_json\b|\bdispatch\b|\brespond\b|refusal_json|\bbad(_refusal)?\b'
  [catchall]=''
)

# Fleet gate predicate: a byte copy of the committed mutation_court.sh GATED
# regex (a00770b, line 329) so the fleet verdict IS the court verdict applied
# to the merged misses. The preflight asserts this copy still matches the
# court's live predicate; drift is a typed refusal, not a silent divergence.
GATE_RE='(^|/)src/registry\.rs:|(^|/)src/registry/[^:]*\.rs:|^src/abi\.rs:.*(\bcapabilities\b|\bdispatch\b)'

for f in "${FAMILIES_SEL[@]+"${FAMILIES_SEL[@]}"}"; do
  case " ${ALL_FAMILIES[*]} " in *" $f "*) ;; *)
    echo "mutation_families: unknown family: $f (known: ${ALL_FAMILIES[*]})" >&2; exit 2 ;;
  esac
done

# ---- concurrent-court guard (typed refusal, exit 8) ---------------------------
guard_courts() {
  local hits
  hits="$(pgrep -f 'cargo-mutants|mutation_court\.sh|mutation_families\.sh' 2>/dev/null \
            | grep -v -e "^$$\$" -e "^$PPID\$" || true)"
  if [ -n "$hits" ]; then
    echo "mutation_families: REFUSED [MUTATION-COURT-CONCURRENT]: another court process is live (pid(s): $(echo "$hits" | tr '\n' ' '))." >&2
    echo "  The court mutates src/ in place; concurrent courts parse each other's mutants" >&2
    echo "  (witnessed 2026-10-24: --list probes hit 'failed to parse src/law.rs' mid-mutant)." >&2
    echo "  Run the fleet solo, or wait for the live court to finish." >&2
    exit 8
  fi
}

# ---- listing helpers (pure --list; never mutate) ------------------------------
list_family_universe() { # $1 = family
  local cfg="$ROOT/.cargo/mutants-court-$1.toml"
  [ -f "$cfg" ] || { echo "mutation_families: missing family config: $cfg" >&2; exit 2; }
  ( export MUTANTS_EXTRA="--config $cfg"
    "$COURT" --list ) > "$1.universe.txt"
}

list_default_universe() {
  ( unset MUTANTS_EXTRA
    "$COURT" --list ) > default.universe.txt
}

list_family_selection() { # $1 = family, $2 = regex
  local cfg="$ROOT/.cargo/mutants-court-$1.toml"
  ( export MUTANTS_EXTRA="--config $cfg"
    "$COURT" --list --case "$2" ) > "$1.selected.txt"
}

# ---- preflight ----------------------------------------------------------------
WORK="$(mktemp -d "${TMPDIR:-/tmp}/mutation-families.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
cd "$WORK"

guard_courts

echo "== mutation_families preflight: per-family --list counts =="
for f in "${ALL_FAMILIES[@]}"; do
  list_family_universe "$f"
  printf '  %-10s universe: %4d\n' "$f" "$(wc -l < "$f.universe.txt" | tr -d ' ')"
  if [ -n "${REGEX[$f]}" ]; then
    list_family_selection "$f" "${REGEX[$f]}"
    printf '  %-10s selected: %4d  (--case %s)\n' "$f" "$(wc -l < "$f.selected.txt" | tr -d ' ')" "${REGEX[$f]}"
  fi
done
list_default_universe

# All four family configs must enumerate the SAME universe.
for f in builders specstate dispatch; do
  if ! cmp -s <(LC_ALL=C sort -u catchall.universe.txt) <(LC_ALL=C sort -u "$f.universe.txt"); then
    echo "mutation_families: REFUSED [MUTATION-FLEET-UNIVERSE-MISMATCH]: .cargo/mutants-court-$f.toml" >&2
    echo "  enumerates a different inventory than the catchall config (examine_globs/exclude_re skew)." >&2
    diff <(LC_ALL=C sort -u catchall.universe.txt) <(LC_ALL=C sort -u "$f.universe.txt") | head -20 >&2
    exit 8
  fi
done

UNIVERSE="$WORK/universe.sorted"
LC_ALL=C sort -u catchall.universe.txt > "$UNIVERSE"

# The fleet gate must be the court gate: compare the copied predicate against
# the live mutation_court.sh GATED= line. A loud SKIP if unparseable, a typed
# refusal on drift — never a silent divergence.
court_ere="$(sed -n "s/^GATED=\"\$(grep -E '\([^']*\)' \"\$MISSED\" || true)\"$/\1/p" "$ROOT/scripts/mutation_court.sh" || true)"
if [ -z "$court_ere" ]; then
  echo "  [SKIP] gate-predicate drift check: could not parse GATED= from scripts/mutation_court.sh (format changed; re-pin the check)" >&2
elif [ "$court_ere" != "$GATE_RE" ]; then
  echo "mutation_families: REFUSED [MUTATION-FLEET-GATE-PREDICATE-DRIFT]: fleet GATE_RE != mutation_court.sh GATED predicate." >&2
  diff <(printf '%s\n' "$GATE_RE") <(printf '%s\n' "$court_ere") >&2
  echo "  Re-pin GATE_RE in scripts/mutation_families.sh to the committed court predicate." >&2
  exit 8
else
  echo "  fleet GATE_RE == mutation_court.sh GATED predicate: MATCH"
fi

# Pairwise disjointness of the three explicit families.
OVERLAP=0
check_disjoint() { # $1 $2 = families
  local n
  n="$(comm -12 "$1.selset.txt" "$2.selset.txt" | wc -l | tr -d ' ')"
  if [ "$n" -ne 0 ]; then
    echo "mutation_families: REFUSED [MUTATION-FLEET-DISJOINT-COVER]: $1 and $2 share $n mutant(s):" >&2
    comm -12 "$1.selset.txt" "$2.selset.txt" | head -10 >&2
    OVERLAP=1
  fi
}
for f in builders specstate dispatch; do
  LC_ALL=C sort -u "$f.selected.txt" > "$f.selset.txt"
done
check_disjoint builders specstate
check_disjoint builders dispatch
check_disjoint specstate dispatch

# Cover: union of the three selections + catchall must equal the universe;
# the residual is what only the catchall (no --case) carries.
LC_ALL=C sort -u builders.selset.txt specstate.selset.txt dispatch.selset.txt > union3.sorted
RESIDUAL_N="$(comm -23 "$UNIVERSE" union3.sorted | wc -l | tr -d ' ')"
comm -23 "$UNIVERSE" union3.sorted > residual.sorted
UNIVERSE_N="$(wc -l < "$UNIVERSE" | tr -d ' ')"

# Skew gate: fleet universe must equal the DEFAULT committed config inventory.
# This is the precondition for the kill-set-equality falsifier: the families
# decompose exactly the inventory the default court mutates. Remediation on
# skew: re-pin examine_globs in the four .cargo/mutants-court-*.toml files (or
# drop the glob from .cargo/mutants.toml) — one line per file, coordinator call.
SKEW=0
if ! cmp -s "$UNIVERSE" <(LC_ALL=C sort -u default.universe.txt); then
  SKEW=1
  FLEET_N="$UNIVERSE_N"
  DEFAULT_N="$(wc -l < default.universe.txt | tr -d ' ')"
  UNCOVERED_N="$(comm -23 <(LC_ALL=C sort -u default.universe.txt) "$UNIVERSE" | wc -l | tr -d ' ')"
  EXTRA_N="$(comm -13 <(LC_ALL=C sort -u default.universe.txt) "$UNIVERSE" | wc -l | tr -d ' ')"
fi

echo ""
echo "== mutation_families preflight: cover table =="
printf '  %-10s %10s %10s\n' family universe selected
for f in builders specstate dispatch; do
  printf '  %-10s %10s %10s\n' "$f" "$UNIVERSE_N" "$(wc -l < "$f.selset.txt" | tr -d ' ')"
done
printf '  %-10s %10s %10s   (no --case)\n' catchall "$UNIVERSE_N" "$UNIVERSE_N"
printf '  residual carried only by catchall: %s\n' "$RESIDUAL_N"
printf '  pairwise overlap builders+specstate / builders+dispatch / specstate+dispatch: '
printf '%s / %s / %s\n' \
  "$(comm -12 builders.selset.txt specstate.selset.txt | wc -l | tr -d ' ')" \
  "$(comm -12 builders.selset.txt dispatch.selset.txt | wc -l | tr -d ' ')" \
  "$(comm -12 specstate.selset.txt dispatch.selset.txt | wc -l | tr -d ' ')"

if [ "$OVERLAP" -ne 0 ]; then exit 8; fi

if [ "$SKEW" -ne 0 ]; then
  echo "" >&2
  echo "mutation_families: REFUSED [MUTATION-FLEET-INVENTORY-SKEW]: fleet universe ($FLEET_N) != default config inventory ($DEFAULT_N)." >&2
  echo "  $UNCOVERED_N mutant(s) in the default inventory are outside the fleet universe; $EXTRA_N fleet-only." >&2
  echo "  First uncovered mutants:" >&2
  comm -23 <(LC_ALL=C sort -u default.universe.txt) "$UNIVERSE" | head -10 >&2
  echo "  Remedy (coordinator): re-pin examine_globs in the four .cargo/mutants-court-*.toml files" >&2
  echo "  to the default .cargo/mutants.toml set, or drop the extra glob from .cargo/mutants.toml." >&2
  echo "  The fleet refuses to sweep a skewed universe: kill-set equality would be undefined." >&2
  exit 8
fi

echo "  fleet universe == default .cargo/mutants.toml inventory: MATCH ($UNIVERSE_N)"
echo "  DISJOINT-COVER: OK (union == inventory, no overlap)"

if [ "$PREFLIGHT_ONLY" -eq 1 ]; then
  echo "mutation_families: preflight OK"
  exit 0
fi

# ---- sweep phase (never run by the preflight; serial by construction) ----------
RUN_FAMILIES=()
if [ "${#FAMILIES_SEL[@]}" -gt 0 ]; then
  RUN_FAMILIES=("${FAMILIES_SEL[@]}")
else
  RUN_FAMILIES=(builders specstate dispatch catchall)
fi

mkdir -p "$ROOT/$OUTROOT"
: > "$ROOT/$OUTROOT/merged_missed.txt"

TABLE=""
for f in "${RUN_FAMILIES[@]}"; do
  cfg=".cargo/mutants-court-$f.toml"
  out="$ROOT/$OUTROOT/$f"
  mkdir -p "$out"

  echo ""
  echo "== mutation_families: family run: $f =="
  court_args=(--log "$out/court.log")
  if [ -n "${REGEX[$f]}" ]; then court_args+=(--case "${REGEX[$f]}"); fi
  rc=0
  MUTANTS_EXTRA="--config $cfg" MUTANTS_OUTPUT="$out" "$COURT" "${court_args[@]}" || rc=$?
  case "$rc" in 0|2|3) ;; *)
    echo "mutation_families: FAMILY-RUN-FAILED: $f exited $rc (court log: $out/court.log)" >&2
    exit 4 ;;
  esac

  m="$out/mutants.out/missed.txt"
  c="$out/mutants.out/caught.txt"
  [ -f "$m" ] || { echo "mutation_families: cargo-mutants wrote no missed.txt at $m" >&2; exit 4; }
  [ -f "$c" ] || { echo "mutation_families: cargo-mutants wrote no caught.txt at $c" >&2; exit 4; }

  missed_n="$(wc -l < "$m" | tr -d ' ')"
  caught_n="$(wc -l < "$c" | tr -d ' ')"
  esc_n=0
  if [ "$missed_n" -gt 0 ] && [ "$f" != "catchall" ]; then
    # ESCALATION: survivors only, under the DEFAULT committed config (full
    # killer set). The survivor regex is derived mechanically from missed.txt
    # lines; -F matches those exact description strings.
    esc_dir="$out/escalation"
    mkdir -p "$esc_dir"
    esc_re="$(sed -e 's/[][^$*+.?{}()|\]/\\&/g' "$m" | paste -sd '|' -)"
    echo "== mutation_families: escalation ($f): $missed_n survivor(s) under default config =="
    rc=0
    MUTANTS_OUTPUT="$esc_dir" "$COURT" --case "$esc_re" || rc=$?
    case "$rc" in 0|2|3) ;; *)
      echo "mutation_families: ESCALATION-RUN-FAILED: $f exited $rc (court log: $esc_dir/court.log)" >&2
      exit 4 ;;
    esac
    [ -f "$esc_dir/mutants.out/caught.txt" ] || {
      echo "mutation_families: escalation wrote no caught.txt at $esc_dir/mutants.out/caught.txt" >&2; exit 4; }
    esc_n="$(wc -l < "$esc_dir/mutants.out/caught.txt" | tr -d ' ')"
    # Final family misses = survivors not killed by the escalated full killer set.
    LC_ALL=C sort "$m" > "$out/m.sorted"
    LC_ALL=C sort "$esc_dir/mutants.out/caught.txt" > "$out/esc.caught.sorted"
    comm -23 "$out/m.sorted" "$out/esc.caught.sorted" > "$out/missed.final.txt"
  else
    LC_ALL=C sort "$m" > "$out/missed.final.txt"
  fi
  final_n="$(wc -l < "$out/missed.final.txt" | tr -d ' ')"

  printf -v row '%-10s caught=%-4s missed=%-4s escalatedCaught=%-4s finalMissed=%s' \
    "$f" "$caught_n" "$missed_n" "$esc_n" "$final_n"
  TABLE+="$row"$'\n'
  cat "$out/missed.final.txt" >> "$ROOT/$OUTROOT/merged_missed.txt"
done

# ---- verdict --------------------------------------------------------------------
echo ""
echo "== mutation_families verdict =="
printf '%s' "$TABLE"

MERGED="$ROOT/$OUTROOT/merged_missed.txt"
LC_ALL=C sort -u "$MERGED" -o "$MERGED"
echo "merged missed.txt: $MERGED ($(wc -l < "$MERGED" | tr -d ' ') mutant(s))"

# The committed court gate predicate, applied to the merged misses.
GATED="$(grep -E "$GATE_RE" "$MERGED" || true)"
if [ -n "$GATED" ]; then
  echo "mutation_families: FLEET-GATE-FAIL: MISSED gated mutants (mutation_court.sh GATED predicate, a00770b :329):" >&2
  printf '%s\n' "$GATED" >&2
  exit 3
fi
echo "gate (court GATED predicate on merged missed): PASS"

# Kill-set equality: the three explicit families' final misses must equal the
# catchall reference run's misses, mutant for mutant (full fleet runs only).
if printf '%s\n' "${RUN_FAMILIES[@]}" | grep -qx catchall; then
  ref="$ROOT/$OUTROOT/catchall/mutants.out/missed.txt"
  fam_union="$WORK/fam.union.missed"
  cat "$ROOT/$OUTROOT/builders/missed.final.txt" \
      "$ROOT/$OUTROOT/specstate/missed.final.txt" \
      "$ROOT/$OUTROOT/dispatch/missed.final.txt" > "$fam_union"
  LC_ALL=C sort -u "$fam_union" -o "$fam_union"
  LC_ALL=C sort -u "$ref" -o "$WORK/ref.sorted"
  if ! cmp -s "$fam_union" "$WORK/ref.sorted"; then
    echo "mutation_families: KILL-SET-INEQUALITY: family-merged missed set != catchall reference missed set" >&2
    echo "  family-only misses (escalation gap: killed under the full killer set in the reference):" >&2
    comm -23 "$fam_union" "$WORK/ref.sorted" | sed 's/^/    /' >&2
    echo "  reference-only misses (family caught what the full killer set missed: flake/timeout asymmetry):" >&2
    comm -13 "$fam_union" "$WORK/ref.sorted" | sed 's/^/    /' >&2
    exit 5
  fi
  echo "kill-set equality vs catchall reference: OK"
fi

echo "mutation_families: fleet verdict PASS"
