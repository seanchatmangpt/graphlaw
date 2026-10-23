# S10 skeptic — adversarial verification of L174's 3 INDEX rows

Lane S10 (read-only), repo /Users/sac/graphlaw, 2026-10-22.
Subject: docs/jira/v26.10.22/INDEX.md lines 303-305 (lane L174 block, "post-L171").
Method: `test -f` per linked file, first-lines read vs row text, grep dedupe
anchors, source-tree cross-check of the staged-patch claim.

## Row verdicts

### Row 1 — designs/s2-provenance-census-verify.md — CONFIRMED
- File exists (`test -f` exit 0). Header: "S2 skeptic verification of N26
  switchover provenance census" — matches the row.
- Row claims independently confirmed in-file: facade `mod`/`pub use` evidence
  per tree (grep table over 8 trees); its own python jsonl sweep showing
  **0 Write/Edit hits** to any wired split dir in the wiring window; the
  correction "abi/ops and reasoner/builtins are DIRECTORIES... actual split
  surface is 12 files + 2 nested dirs, not 8 flat trees"; verdict "out-of-band
  actor not identified". Row's "advises do NOT commit until the unlogged
  writer is found" matches the file's bottom line.

### Row 2 — designs/s4-spc-design-verify.md — CONFIRMED
- File exists. Header: "S4 Skeptic — adversarial verification of
  spc-red-remediation-design.md" — matches (row's "(L169)" anchor: the design
  row sits in the L169-cited section, verified present in INDEX).
- Receipt table re-parses all 5 spc receipts (exit 0 parses per its own text);
  "p999=292 in FOUR receipts... stronger than claimed"; 3 arithmetic
  corrections (rule1 counts wrong as n/a, p99 range claim, tail-fraction
  range) — the row's "3 arithmetic corrections" is accurate; "composite
  tail-magnitude loophole found" matches the bottom line ("composite gate
  needs one more term (tail-magnitude fence)").

### Row 3 — designs/surface-six-entry-patch.md — CONFIRMED
- File exists. Header: "Surface six-entry patch — staged, DO NOT APPLY
  (lane L172)" — matches the row's lane and STAGED status.
- Source cross-check (run this session): `grep -c include_bytes
  src/registry/emit_json.rs` = **2**; `const FILES` at emit_json.rs:74 —
  the patch is indeed NOT APPLIED, so the row's STAGED status is true, and
  "2 to 6 entries" is internally consistent (2 existing + 4 new target files,
  all 4 verified present in the patch's own read-only check).
- "closes the S1-confirmed 35.8% pin gap": consistent with the L171-block
  s1 row (35.8% re-derived) and receipt-surface-post-switchover (N27).

## Dedupe claim (L174, INDEX.md:294-297) — CONFIRMED
- `commit-manifest.md` is already indexed in the L165 block (INDEX.md:243);
  `s1-surface-analysis-verify.md` and `s7-chain-board-check.md` in the L171
  block (verified rows present). All 3 dedupe-base files exist on disk
  (`test -f` chain exit 0, echoed DEDUPE-BASES-EXIST). Skipping them from
  L174's table is correct — no duplicate rows exist (grep shows one row each).

## L171 stale loud-skips correction — CONFIRMED
- INDEX.md:280-281 still carries L171's text "Loud skips: s2-... and s4-...
  NOT landed"; both files now exist on disk (`test -f` exit 0 each), so
  L174's "have since landed and are indexed below" is a correct correction
  of a stale claim, not an overclaim. L174 additionally reports "no new loud
  skips" — consistent with this verification: no expected candidate was
  absent from `designs/`.

## Receipt
- Reads: head of each of the 3 subject files; INDEX.md 235-245 / 270-305;
  grep of INDEX for dedupe anchors; grep of src/registry/emit_json.rs.
- Exit codes: all `test -f` exit 0 (row files + 3 dedupe bases);
  `grep -c include_bytes` = 2 (exit 0); no failing command in this check.
- Zero defects found. No row summary materially misstates its file.
