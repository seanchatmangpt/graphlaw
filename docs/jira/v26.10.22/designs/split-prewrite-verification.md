# Split pre-write verification — lane L68 (skeptic)

Date: 2026-10-22. Lane: L68, read-only skeptic over /Users/sac/graphlaw (one canonical
checkout, no worktrees). Subject: the five landed pre-write split trees. Method:
independent re-derivation — my own Python checker (/tmp/l68/check_wt.py) slicing the
pinned source by the claimed ranges with `split('\n')` and testing byte-containment of
each range block in the split file, plus full-file unified diffs vs the union of claimed
ranges, plus per-crate rustfmt parse gates, unwired grep, and a mutation control on the
court itself. 186 range checks total; 174 byte-exact pass, 12 initially flagged; all 12
diagnosed to root cause below.

Base pin finding (applies to L44 and any split claiming "HEAD 615dbfa"): the sql_split
content matches the **modified working tree** of sql_ast.rs, not HEAD 615dbfa — the
split contains `JOIN_TYPE_IRI: u16 = 37` and `LiteralNotInt = 36 / LiteralNotFloat = 37`
(refusal_arm + RefusalCode arms added by the in-flight working-tree edits, +295 lines);
HEAD has `INDEX_NEGATIVE: 35` / `UNKNOWN_STATEMENT_PREDICATE: 36` in refusal_arm and no
`JOIN_TYPE_IRI`. Verified: `git show HEAD:src/sql_ast.rs` lacks JOIN_TYPE_IRI (False);
working tree has it (True); sql_split has it (True). The "HEAD 615dbfa" provenance
string in src/sql_split/*.rs headers and SPLIT-NOTES.md is therefore false as a base
pin; extraction base = working tree at extract time.

## Per-tree verdicts

### L43 reasoner_split — VERIFIED
- Provenance replay: all 131 ranges in provenance.json (limits 15, builtins/mod 29,
  math 22, string 13, list 24, log 28) byte-verified: 131/131 PASS.
- Source identity: `shasum -a 256 crates/graphlaw-eyeron/src/reasoner.rs` →
  `1fb8f0a4d5dad8fea77e93073404952d98579098aad360d5fe2d5dc11d4bad2c`, equals the (historical)
  SPLIT-NOTES pin (extract-time and post-write). reasoner.rs itself is unmodified in
  git (not in the modified list).
- Parse gate: rustfmt --emit stdout --edition 2021, all 7 files → exit 0 each
  (eyeron edition 2021; the 2021 gate is correct, the 2024 deviation disclosed in
  SPLIT-NOTES is the right call).
- Unwired: no mod decl references (grep exit 1).
- query_ledger_split (L65, skip per dispatch): present at src/query_ledger_split/
  (batch.rs 442 lines, storage.rs 534 lines, no SPLIT-NOTES), not verified.

### L45 parser_split — VERIFIED
- All 8 claimed ranges (cursor 6-17, 356-457, 1758-1809, 1811-1987, 2160-2236; grammar 243-247, 458-1757, 1989-2158; per SPLIT-NOTES)
  SPLIT-NOTES) byte-exact vs working parser.rs (unmodified in git; 2236 lines).
- Parse gate: edition 2021, exit 0 both files. Unwired: grep exit 1.

### L46 receipt_split + registry_split — VERIFIED
- receipt_split/frame.rs = receipt_chain.rs 47-426 byte-exact (380 lines contiguous);
  record.rs = 435-871 + 1597-1989 byte-exact; store.rs = 2264-2370 byte-exact.
- registry definitions 29-911, emit_json 913-1636, emit_ttl 1638-1964: byte-exact
  vs working tree.
- Invariance laws: `#[repr(transparent)]` ×2 (DenialPolarity, PackedObjRef) and
  `#[repr(C, align(64))]` (OcelCausalFrame) present at identical offsets; frame struct
  field order (instruction_id, fired_mask, denial, obj_refs[8], ts_ns, activity_idx,
  node_kind, pad, prior_hash) intact by byte-exactness. emit_json key order intact
  by byte-exactness.
- Note: no SPLIT-NOTES.md in either dir (contrary to dispatch expectation); provenance
  is carried in per-file header comments instead.
- Parse gate: edition 2024 (root crate), exit 0 all 6 files.
- Note: receipt_chain.rs and registry.rs working tree carry +243/+12-line concurrent
  edits from other lanes; the split ranges verify against the pre-shift coordinates,
  and the byte-exact containment holds on the current working tree (insertions land
  outside the claimed ranges — receipt_chain hunk starts at L19 within a region that
  was... actually insertions land in frame region 19-2370; containment held anyway,
  meaning ranges were taken post-insertion).

### L67 abi_split — VERIFIED
- governance/parse_convert/query_engines/meta: every claimed range byte-exact except
  the disclosed glue: inline dispatch arms (abi.rs 447-450, 453-457, 461-473, 451-452)
  rendered as named fns `op_sniff`/`op_canonical`/`op_n3` with body text byte-identical
  to the arm bodies; headers; blank lines between items. Union-diff vs claimed ranges
  shows only header/glue hunks — zero body edits.
- lease.rs is a new design draft (not an extraction), per its own header; not range-
  verified by construction.
- Parse gate: edition 2024, exit 0 all 6 files (incl. ops/mod.rs, lease.rs).

### L66 dialect_split — DISCREPANCY (semantics hold; "verbatim" claim false)
- refusal.rs and sniff.rs bodies are semantically identical to src/dialect.rs but
  rustfmt-reformatted at 4 sites, contradicting the header claim "Verbatim slices;
  provenance = original line ranges":
  1. refusal.rs:163-166 `if !matches!(\n dialect,\n ...)` (3 lines) vs source
     dialect.rs:510 one-liner `if !matches!(dialect, Dialect::Turtle | Dialect::NTriples | Dialect::NQuads) {`.
  2. sniff.rs:289 `let rest = tok.strip_prefix(SQL).or_else(...)` one line vs source
     dialect.rs:796-798 3-line chain.
  3. sniff.rs:296-300 `has_select` closure reformatted (4-line closure body joined).
  4. sniff.rs:314 `object_start` closure body joined to one line vs source 317-319.
  Plus blank lines inserted between items. No token-level semantic change at any site
  (whitespace/wrap only); refusal codes, variants, and discriminants untouched.
- Parse gate: edition 2024, exit 0 both files. Unwired: grep exit 1.

### L44 sql_split — DISCREPANCY (2 findings)
1. **Missing extracted items**: SPLIT-NOTES.md and the ast.rs header (line 5) claim
   RefusalCode, RefusalContext, AstError are in ast.rs; the claimed ranges 89-122
   (RefusalCode) and 202-243 (RefuralContext/AstError) were never extracted into ANY
   sql_split file. Current source spans (working tree): RefusalCode src/sql_ast.rs:94-122,
   RefusalContext 206-219, `impl Default for RefusalContext` 221-232, AstError 238-243.
   refusals.rs:9 does `use super::ast::{AstError, RefusalCode}` — the split tree cannot
   compile at switchover without these; switchover lane must extract them or the
   provenance table corrected.
2. **False base pin**: headers/SPLIT-NOTES say "HEAD 615dbfa" but content matches the
   modified working tree (see Base pin finding above).
- What IS verified: refusals.rs and serializer.rs bodies byte-exact vs working tree
  except the disclosed `pub(crate)` widenings (refusals.rs:168/172/182/187;
  serializer.rs serialize_expr) and glue headers; ast.rs remaining items (Operator,
  Scope/ScopePos with the two disclosed pub(crate) widenings, AggregateFunction,
  SqlLiteral, SqlExpr, TableRef, validators, OrderTerm, JoinType, JoinClause,
  SelectStatement, CompiledQuery incl. seal field, PartialEq impl, CANONICALIZATION_COUNT,
  canonical_graph_bytes, MergeStatement) byte-exact. Parse gate: edition 2024,
  exit 0 all three files.

## Invariance-law receipts
- receipt frame repr: 3 repr attrs byte-identical positions; field order byte-exact.
- registry emit_json key order: byte-exact slice 913-1636 — key order trivially intact.
- dialect codes: variant names/discriminants unchanged (only whitespace rewraps; no
  code value touched).
- Unwired: `grep -rn "_split" src crates/graphlaw-eyeron/src --include='*.rs'` outside
  the split dirs → no matches (exit 1); `grep -rn "mod .*split"` → exit 1. Confirmed
  UNWIRED for all seven trees.

## Court commands + exits
- /tmp/l68/check.py (vs HEAD) and /tmp/l68/check_wt.py (vs working tree): 186 range
  containment checks; final state 174 pass / 12 diagnosed (5 abi arm-glue,
  2 sql missing-item + stale-range, 4 dialect reformat, 1 sql pub(crate) widening).
- /tmp/l68/diag.py + diag_wt.py: alignment diagnostics producing the quoted diffs.
- Mutation control: corrupting `pub struct OcelCausalFrame` → `OcelCausalFrameX` in a
  frame.rs copy flips the court to False (court is non-vacuous; the real file contains
  the block, the mutant does not).
- rustfmt gates: eyeron splits @2021 → all exit 0; root-crate splits @2021 → record.rs
  FAILS (exit 1, `let chains are only allowed in Rust 2024`, record.rs:805 — expected:
  root crate is edition 2024); @2024 → all root-crate split files exit 0. Total: 28
  split .rs files, all pass at their crate's edition.
- git diff --stat of sources (concurrent-lane edits, not split-lane violations):
  dialect.rs 7±, registry.rs 12±, receipt_chain.rs +243, sql_ast.rs +295/-19;
  abi.rs, reasoner.rs, parser.rs unmodified.

## Bottom line
- L43 VERIFIED, L45 VERIFIED, L46 VERIFIED, L67 VERIFIED, L66 DISCREPANCY (reformat vs
  verbatim claim, semantics intact), L44 DISCREPANCY (RefusalCode/RefusalContext/
  AstError never extracted despite provenance-table claims; false HEAD pin).
- Discrepancy count: 3 findings across 2 trees (sql: missing items + false base pin;
  dialect: 4 reformat sites vs "verbatim" claim). 131+ ranges otherwise byte-exact.
