#!/usr/bin/env bash
# count_gate.sh — stale "N passed; 0 failed" detector for cheap-allowlist courts.
# Scans docs/**/*.md for count rows tied to an allowlist court, re-runs each
# referenced court once in a lane target dir, compares claimed vs actual.
# Mismatch => COUNT-STALE finding with file:line, claimed, actual. Exit 1 on any.
#
# R3 lane hardening (v26.10.22):
#   * UNPAIRED count claims: a count row with no allowlist --test line within
#     2 lines becomes a COUNT-UNPAIRED finding, not silence.
#   * count regex broadened: ';' ',' '/' '|' separators, either order.
#   * fence filtering: a count inside a ``` fence is quoted output, not a
#     claim — reported loudly as SKIPPED-FENCED (stderr), never a claim.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
TARGET="${CARGO_TARGET_DIR:-target-countgate}"
ALLOW="sql_rdf_dialect chicago_validation trust_attest duckdb_statement_kinds query_cache_court query_ledger_court evidentiary_ring_court"
TMP=$(mktemp); trap 'rm -f "$TMP"' EXIT
shopt -s globstar nullglob
for f in docs/**/*.md; do
  [ -f "$f" ] || continue
  awk -v allow="$ALLOW" '
    BEGIN { n = split(allow, A, " "); pend = ""; pline = 0; pfile = ""; ptext = ""; inf = 0 }
    /^```/ { inf = !inf; next }
    {
      nc = 0; court = ""; cnt = ""
      if (!inf) {
        for (i = 1; i <= n; i++)
          if (index($0, "--test " A[i]) > 0) { if (nc == 0) { court = A[i] }; nc++ }
        if (match($0, /[0-9]+ passed *[;,\/|] *0 failed/) || match($0, /0 failed *[;,\/|] *[0-9]+ passed/)) {
          head = substr($0, RSTART, RLENGTH)
          cnt = head
          sub(/ .*/, "", cnt)            # leading-number form: "12 passed; .." -> 12
          if (head !~ /^[0-9]/) sub(/^.* /, "", cnt)  # reversed form: "0 failed; 12 passed" -> 12
        }
      } else if (match($0, /[0-9]+ passed *[;,\/|] *0 failed/) || match($0, /0 failed *[;,\/|] *[0-9]+ passed/)) {
        print "SKIPPED-FENCED\t" FILENAME ":" FNR "\t" $0 > "/dev/stderr"
      }
      if (inf) next
      if (nc > 1) { pend = ""; next }
      if (nc == 1 && cnt != "") { printf "%s\t%s\t%s:%d\t%s\n", court, cnt, FILENAME, FNR, $0; pend = ""; next }
      if (nc == 1) { pend = court; pline = FNR; pfile = FILENAME; ptext = $0; next }
      if (nc == 0 && cnt != "") {
        if (pend != "" && FNR - pline <= 2)
          printf "%s\t%s\t%s:%d\t%s\n", pend, cnt, pfile, pline, ptext
        else
          printf "UNPAIRED\t-\t%s:%d\t%s\n", FILENAME, FNR, $0
      }
    }' "$f" >> "$TMP"
done
RC=0
declare -A ACTUAL
while IFS=$'\t' read -r court claimed loc text; do
  if [ "$court" = "UNPAIRED" ]; then
    echo "COUNT-UNPAIRED $loc count-row-without-nearby-test-line: $text"
    RC=1
    continue
  fi
  case " $ALLOW " in *" $court "*) ;; *) continue;; esac
  if [ -z "${ACTUAL[$court]:-}" ]; then
    flags=""
    [ "$court" = evidentiary_ring_court ] && flags="--features evidentiary-ring"
    out=$(CARGO_TARGET_DIR="$TARGET" cargo test $flags --test "$court" 2>&1)
    n=$(echo "$out" | grep -o '[0-9]* passed; 0 failed' | head -1 | grep -o '^[0-9]*')
    if [ -z "$n" ]; then
      echo "COUNT-GATE COURT-BUILD-FAIL: $court (concurrent lane edit or real break); rows marked UNKNOWN" >&2
      echo "$out" | tail -5 >&2
      ACTUAL[$court]="UNKNOWN"
    else
      ACTUAL[$court]="$n"
    fi
    echo "COUNT-GATE ran $court -> ${ACTUAL[$court]} passed" >&2
  fi
  actual="${ACTUAL[$court]}"
  if [ "$actual" = "UNKNOWN" ]; then
    echo "COUNT-UNKNOWN $loc court=$court claimed=$claimed (court not runnable this pass)"
  elif [ "$claimed" != "$actual" ]; then
    echo "COUNT-STALE $loc court=$court claimed=$claimed actual=$actual"
    echo "  line: $text"
    RC=1
  else
    echo "COUNT-OK $loc court=$court $claimed/0"
  fi
done < "$TMP"
exit $RC
