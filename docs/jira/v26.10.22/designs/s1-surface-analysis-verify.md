# S1 skeptic verification of receipt-surface-post-switchover.md (N27) — 2026-10-22

READ-ONLY lane. All verdicts from on-disk reads + wc/grep/python exits 0.
Repo: /Users/sac/graphlaw. One report file (this one) written; zero src/test edits.

## Verdicts

### Claim 1: 35.8% pin gap — CONFIRMED (arithmetic re-derived)

Actual `wc -c` (exit 0):

| file | bytes | lines |
|---|---|---|
| src/sql_ast.rs (facade) | 100,074 | 2,338 |
| src/sql_ast/ast.rs | 19,098 | 520 |
| src/sql_ast/refusals.rs | 28,243 | 603 |
| src/sql_ast/serializer.rs | 8,348 | 232 |
| src/duckdb_eval.rs | 74,573 | 1,681 |

FILES pins: CONFIRMED as described. `src/registry/emit_json.rs:74-77` pins
exactly `src/duckdb_eval.rs` (74,573 B) and `src/sql_ast.rs` (100,074 B) via
`include_bytes!`. Registry JSON rows at
registry/capability-registry.json:2100-2114 match byte-for-byte (74,573 /
100,074). Court `tests/registry_artifacts.rs:890` asserts
`files.len() == 2` (line 907). All cites in N27 verified against disk.

Claimed split = 55,682 B; measured today = 55,689 B (ast.rs drifted 19,091 ->
19,098 mid-switchover; immaterial). Gap = 55,689 / (55,689 + 100,074) =
35.75% ≈ 35.8%. CONFIRMED.

### Claim 2: vacuity for refusals.rs — CONFIRMED (and worse than stated: it is
vacuous TODAY, not only post-switchover)

Code path (read, exit 0):
- pin = `blake3_of(surface_document_bytes)` where the document contains only
  the per-file {path, blake3, bytes} rows of the hard-coded FILES const
  (emit_json.rs:70-98).
- FILES contains only duckdb_eval.rs and sql_ast.rs include_bytes.
- Therefore a semantic edit to src/sql_ast/refusals.rs changes compiled
  behavior (`pub use refusals::*` at src/sql_ast.rs:24) but does not change
  any byte hashed into receipt_surface_blake3. Answer to the dispatch
  question: NO, it would not move the pin. CONFIRMED.
- Nuance the proposal understates: N27 says vacuity "arrives only when the
  facade collapses." Strictly, any semantics ALREADY moved into ast.rs /
  refusals.rs / serializer.rs is unpinned today; the interim state is vacuous
  for exactly the 55,689 B of already-migrated semantics, under-inclusive
  only for what remains in the facade. The practical conclusion (5-entry fix
  must be in the final regen window) is unchanged and correct.

### Claim 3: 5-entry FILES list completeness — REFUTED (one concrete miss:
ontologies/sql-ast.ttl)

The proposal misses a genuinely admission-semantic input on the same compile
path: `src/sql_ast.rs:1138-1141` embeds
`include_str!("../ontologies/sql-ast.ttl")` (19,562 B on disk) into the
shapes used by `validate_ast_shacl` inside `compile_turtle` (src/sql_ast.rs:1031,
1137). The .ttl is hashed nowhere in the pin; a semantic edit to it moves
admission outcomes without moving receipt_surface_blake3 — the same failure
class the proposal exists to close. Recommendation: 6 entries (add
("ontologies/sql-ast.ttl", include_bytes!("../ontologies/sql-ast.ttl")))
with path resolved from src/registry/: `../../ontologies/sql-ast.ttl`.
  Note the include_bytes! path from emit_json.rs would need
  `../../ontologies/sql-ast.ttl` (two levels up), not `../` — a detail the
  proposal's own path-resolution note gets wrong for this file.
- src/registry/emit_json.rs itself: correctly excluded — it is the pinner,
  not the pinned surface; the pin is over the document it emits, so pinning
  its own bytes would be self-referential (pin depends on bytes that depend
  on the pin's file list — no fixed point). Exclusion is principled, agree.
- Broader admission surface (src/law.rs, src/tier.rs, src/attest.rs,
  src/shacl_guard.rs, src/shacl engine): outside the const's own doc-comment
  scope ("compiler/eval sources"); pinning them is a scope expansion, not a
  defect in this proposal. Flag as open question for the coordinator, not a
  REFUTED claim.
- src/shacl_guard.rs — call it out: validate_ast_shacl (sql_ast.rs:1032)
  calls crate::shacl_guard::check_shape_depth; semantic edits there are
  admission-semantic too, same argument class as the .ttl but at one remove
  (guard, not grammar). Coordinator's call on scope.

### Claim 4: timing/window reasoning — CONFIRMED
Section 5 ordering (FILES edit + court len assert + regen in one quiescent,
compile-quiescent window) is correct: include_bytes! digests are computed at
crate compile time (verified by the const's type: `&[(&str, &[u8])]` from
include_bytes), the court reads live disk bytes and compares to the
committed registry (tests/registry_artifacts.rs:904-911), and regen is the
only writer of the committed pin. Either misordering reds the checkout —
reasoning verified against the court code, not accepted on assertion.

## Receipt
- Commands: `wc -c -l` over 7 files (exit 0); `sed -n` reads of
  src/registry/emit_json.rs (60-110), src/sql_ast.rs (1-40), tests/registry_artifacts.rs
  (880-935), registry/capability-registry.json (2095-2120); `grep` for
  include_bytes/receipt_surface/shacl/compile_turtle; python3 arithmetic
  (exit 0).
- No cargo run required (static-analysis lane); the pinner const, the drift
  court, and the committed pin were all read on disk and are mutually
  consistent with N27's description.
- Files touched: only this report.
