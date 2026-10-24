# s14-provenance-note-verify — skeptic verification of L176's staged provenance-gap note

Lane S14 (read-only + this file), /Users/sac/graphlaw, 2026-10-22.
Subject: `designs/commit-receipt-provenance-gap.md` (L176 STAGED note).

## Verdicts

### Check 1 — cited receipts exist + contain claimed content: PASS (2 minor gaps)
All 11 on-disk paths cited in the note were `test -f` exit 0:
switchover-provenance-census.md, s2-provenance-census-verify.md,
s8-unlogged-writer-watch.md, surface-six-entry-patch.md,
receipt-surface-post-switchover.md, eyeron-switchover-fixforward.md,
precommit-acceptance-provenance (precommit-acceptance-receipt.md),
commit-manifest.md, link-green-receipts.md, integration-gonogo.md,
wasm-duckdb-op-wired.md.

Content spot-checks (all confirmed verbatim at cited sections):
- N26 §1/§3: eight split trees WIRED via facade `mod`/`pub use`, mtimes
  19:32:38–19:57:34, ZERO Write/Edit in transcript corpus → UNDETERMINED
  (line 77) — matches.
- S2: independent python jsonl sweep, staging dirs gone at 20:08 (line 47),
  `~/.zsh_history`/Trash/fswatch ruled out (lines 41/43), UNDETERMINED
  CONFIRMED with escalation (line 64) — matches.
- S8: QUIESCED 20:14–20:20, 5×60 s bounded poll, sha256
  e1c5260259f18ddf5ae4bb3d23a7878fe813b90dfe4d35f6e5e347ce579f7854 (line 18), (historical)
  `lsof +D` zero handles, PID 23692 (line 44), commit-if-and-only-if
  recommendation (line 58) — matches.
- N25 (eyeron-switchover-fixforward.md): `cargo test -p graphlaw-eyeron`
  21/0, sql_rdf_dialect 33/0, chicago_validation 9/0, `cargo check -p
  graphlaw-eyeron` exit 0, target-lane-n25 (lines 7/43–45) — matches.
- N23/L147b (precommit-acceptance-receipt.md): 128 tests, 124 passed / 4
  failed, all 4 known registry-drift rows, deterministic across two runs,
  drift court ALIVE (lines 62–116) — matches.
- L1 (link-green-receipts.md:20): `cargo test -p graphlaw-wasm` 9/0 —
  matches.
- L125 (wasm-duckdb-op-wired.md:80–81 + integration-gonogo.md P3 row):
  `3 passed; 0 failed; 1 ignored` (typed BLOCKED-MODULE-EXEC) — matches.

Minor gaps (non-blocking):
a) The header claims "eight receipt paths … all `test -f` exit 0", but §3
   cites two more files (integration-gonogo.md, wasm-duckdb-op-wired.md)
   not in that list. Both exist and support their claims; only the header
   count is understated.
b) §1's "9 read-only greps/ls from skeptic lanes (S2 §2)" — S2 §2 records
   the sweep method but the precise "9 hits" figure is in N26's corpus, so
   the citation split is slightly loose (both receipts agree on zero logged
   writes, the load-bearing claim).

### Check 2 — UNRECEIPTED marking on V1's 110/0: PASS
`grep -rniE "110 ?(passed|ok|tests|= 110|/ *110)" docs/jira/v26.10.22/` →
exit 1, zero hits. No V1 receipt file exists anywhere in the milestone dir.
The note's UNRECEIPTED marking is accurate and it correctly declines to
claim 110/0.

### Check 3 — verbatim-ready (self-contained): PASS with 1 caveat
Self-contained: §1–§4 carry their own citations inline; no reference to
external lane state; §5 receipt is honest (doc-only lane, no target dir).
Caveat: the header "eight receipt paths" count vs actual citations (gap a
above) is a cosmetic mismatch a coordinator could fix in one word ("ten")
or by listing all cited paths before declaring the section final.

### Check 4 — scope: correctly scoped; 2 items noted
- The L172 six-entry patch is NOT missed: §4 item 2 covers it explicitly
  (STAGED, DO-NOT-APPLY, sequenced before `graphlaw-registry -- --write`),
  and N27's FILES-extension + `tests/registry_artifacts.rs:907` len assert
  2→5 requirement is correctly tied to the same regen window (verified
  against receipt-surface-post-switchover.md lines 111/120–121).
- The S13 A1.1 defect (magnitude-blind tail gate anti-vacuity arithmetic,
  2 µs vs 2500 ns; s13-spc-amendment-verify.md:40/61) is a separate
  court-amendment item, not a switchover commit-receipt item — correctly
  out of scope for THIS receipt. It should ride a different follow-up
  (SPC gate court), not this note.
- P7 final regen + 117/0 re-run is in §4 item 3, matching
  integration-gonogo.md P7 row (line 49) and GO gate (line 70) — matches.
- Live re-verification replicated: `ls -d src/*_split
  crates/graphlaw-eyeron/src/*_split` → zsh "no matches found", exit 1 —
  staging dirs confirmed deleted as of this lane's check time.

## Receipt
- Reads: note + 11 receipt files (all `test -f` exit 0).
- Greps: 110/0 sweep exit 1 (basis for Check 2); per-claim content greps
  above with line numbers.
- Live: split-dir glob exit 1 (deletion confirmed).
- Lane target dir: none (doc-only).
- Verdict: the staged note is accurate and verbatim-ready; fix the header
  count (eight→ten) or leave as a disclosed cosmetic gap.
