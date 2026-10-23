#!/usr/bin/env bash
# witness_gate.sh — M02 lane (v26.10.22)
# Law: a factual build/run witness in docs/**/*.md must cite an existing
# receipts/ row or an existing path. Scans blake3 digests (64-hex) and
# `target-lane*` citations; each must resolve to a receipts/ file containing
# it, or an existing dir/file on disk.
# Skips fenced code blocks (quoted court output) and lines carrying
# `witness-gate:allow`. Excludes generated docs/reference-generated/.
# Non-vacuity: flags docs/WASM.md:44/47/99 (target-lane114 + unregistered
# 9d299ed8…) today; a doc citing a registered digest passes.
# A gate that cannot exit 1 is the failure.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RECEIPTS="$ROOT/receipts"
status=0
[ -d "$RECEIPTS" ] || { echo "WITNESS-GATE REFUSED: no receipts/ dir at $RECEIPTS" >&2; exit 2; }
RECEIPT_DIGESTS=$(grep -rhoE '[0-9a-f]{64}' "$RECEIPTS" 2>/dev/null | sort -u)
if [ "$#" -eq 0 ]; then
  FILES=$(find "$ROOT/docs" -name '*.md' -type f -not -path "*/reference-generated/*")
else
  FILES="$*"
fi
FILTERED=$(mktemp)
trap 'rm -f "$FILTERED" "$FILTERED.hit"' EXIT
WITWARN=0
for f in $FILES; do
  awk -v F="$f" '
    BEGIN { inf = 0; held = "" }
    /^```/ {
      if (!inf) { inf = 1; held = held $0 "\n" }
      else { inf = 0; held = "" }
      next
    }
    {
      if (inf) { held = held F ":" FNR ":" $0 "\n"; next }
      if ($0 ~ /witness-gate:allow/ && $0 !~ /witness-gate:allow\([^)]*,[ ]*[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]\)/)
        print "BARE-ALLOW\t" F ":" FNR "\t" $0
      print F ":" FNR ":" $0
    }
    END {
      if (inf) {
        # unmatched fence: the exemption must not silently disable the rest of
        # the file. Warn loudly and check the held lines anyway.
        print "UNBALANCED-FENCE\t" F > "/dev/stderr"
        printf "%s", held
      }
    }
  ' "$f"
done > "$FILTERED"
NBARE=$(grep -cE '^BARE-ALLOW' "$FILTERED" || true)
if [ "${NBARE:-0}" -gt 0 ]; then
  grep -E '^BARE-ALLOW' "$FILTERED" | while IFS= read -r b; do
    echo "WITNESS-GATE FAIL: bare 'witness-gate:allow' without (reason,YYYY-MM-DD) — $b"
  done
  status=1
fi
grep -vE '^BARE-ALLOW' "$FILTERED" > "${FILTERED}.body"
mv "${FILTERED}.body" "$FILTERED"
grep -E '[0-9a-f]{64}|target-lane[A-Za-z0-9_-]*' "$FILTERED" > "$FILTERED.hit" || true
while IFS= read -r m; do
  [ -n "$m" ] || continue
  f=${m%%:*}; rest=${m#*:}; ln=${rest%%:*}; line=${rest#*:}
  for dig in $(printf '%s\n' "$line" | grep -oE '[0-9a-f]{64}'); do
    if ! printf '%s\n' "$RECEIPT_DIGESTS" | grep -qx "$dig"; then
      echo "WITNESS-GATE FAIL: $f:$ln unregistered blake3 digest $dig"
      status=1
    fi
  done
  for lane in $(printf '%s\n' "$line" | grep -oE 'target-lane[A-Za-z0-9_-]*' | sort -u); do
    if [ ! -e "$ROOT/$lane" ]; then
      echo "WITNESS-GATE FAIL: $f:$ln cites nonexistent lane dir $lane (build witness must cite receipts/)"
      status=1
    fi
  done
done < "$FILTERED.hit"
if [ "$status" -eq 0 ]; then
  echo "WITNESS-GATE PASS"
fi
exit "$status"
