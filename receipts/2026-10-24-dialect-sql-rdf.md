# Dialect Verification Receipt — SQL-in-RDF surface (2026-10-24)

Lane: DIALECT VERIFICATION (read-only except this file; no git mutations).
Deliverable: witnessed pass state of the SQL-in-RDF dialect surface on this
tree, as-is, pre-commit.

## Subject identity

- Repo: `/Users/sac/graphlaw` (canonical checkout, one lane on the tree).
- HEAD at run time and re-verified at receipt-write time:
  `6da1d33a93f61068fa9b033bec353db78d335460`
- Working tree: dirty, closeout lanes mid-flight. `git status --porcelain | wc -l`
  was **466** at run start (10:33 local) and **487** at receipt write — the tree
  moved during verification while HEAD stayed fixed; this receipt witnesses the
  tree as-is at HEAD `6da1d33`, not a clean snapshot.
- Engine pin: `/opt/homebrew/bin/b3sum priv/graphlaw.wasm` →
  `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a`
  (matches the expected pin; re-verified at write time).
- Every invocation ran with `CARGO_TARGET_DIR=/Users/sac/graphlaw/target-lane-dial-sqlrdf`
  and `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm` (cold lane target;
  no sleeps anywhere in the run — bounded polls on log observables only).
- b3sum binary: `/opt/homebrew/bin/b3sum`.

## Per-court witnessed results

Exact command shape (cwd `/Users/sac/graphlaw`):

```
CARGO_TARGET_DIR=/Users/sac/graphlaw/target-lane-dial-sqlrdf \
GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm \
cargo test --test <target>
```

All 13 courts exited 0. Verbatim `test result:` lines:

| # | court (`--test`) | exit | verbatim `test result:` line |
|---|---|---|---|
| 1 | sql_rdf_dialect | 0 | `test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s` |
| 2 | duckdb_statement_kinds | 0 | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.39s` |
| 3 | sql_rdf_conformance | 0 | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s` |
| 4 | sql_rdf_laundering_corpus | 0 | `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s` |
| 5 | k5_hostile_sqlrdf | 0 | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s` |
| 6 | canonical_cross_dialect | 0 | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` |
| 7 | refusal_details | 0 | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 8 | refusal_crosswalk_differential | 0 | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.69s` |
| 9 | docs_release_claims | 0 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 10 | set_operations_court | 0 | `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s` |
| 11 | chicago_validation | 0 | `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.72s` |
| 12 | parse_refusal_court | 0 | `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 13 | sniff_court | 0 | `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |

(A court with no embedded output is not a receipt: raw logs retained at
`/tmp/dial-sqlrdf-logs/<court>.log` for the session.)

## AGGREGATE

- **Witnessed total: 157 passed / 0 failed across 13 courts** (36+8+8+23+8+6+7+6+2+16+12+12+13).
- **Census statement:** the durable dialect number witnessed on this tree is
  **36/0 for `sql_rdf_dialect`** (test result line above). The historical
  "69/0" figure has no durable locus — no `69/0` string exists in `README.md`,
  `CHANGELOG.md`, or `docs/reference/sql-rdf-rec.md` (grepped at write time) —
  and is not claimed by this receipt.
- Reconciliation note: `sql_rdf_conformance` exposes 8 real tests; a raw
  `grep -c '#\[test\]'` reports 10 because two doc-comment lines contain the
  literal `#[test]` (file head + §PROSE-ONLY comment). Executed test names were
  read from the log; nothing was silently skipped.
- The 35-statement-kind inventory is witnessed green via
  `section3_2_inventory_is_35_classes_with_typed_engine_refusals` (rec §3.2)
  inside `sql_rdf_conformance`, and via `duckdb_statement_kinds` (8 tests over
  the 35 kinds on real frozen-duckdb).

## Classification of every non-green item

**Zero test failures occurred.** Non-green-adjacent items, classified:

1. **Typed refusals (by design, not failures).** `ConnectStatement` /
   `DisconnectStatement` are recognized-but-unimplemented statement kinds:
   `docs/refusals.md` line 77, `REFUSAL-02` `UnimplementedStatementRefusal`,
   surface `sql_ast::AstError::UnsupportedTerm` carrying
   `compile refusal: http://graphlaw.org/sql#<Kind> [REFUSAL-02]
   UnimplementedStatementRefusal: not implemented by this DuckDB build`.
   Witnessed as typed behavior by passing courts `connect_disconnect_carry_refusal_02`
   and `l163_msg_connect_statement_is_verbatim` (sql_rdf_dialect).
2. **Known limitation — tc08 COPY egress (EH-19, OPEN).** Not run per lane
   instruction (`tests/wasm_fibo_egress_eh.rs` is a disclosed red). Status:
   fuel-independent typed EINVAL (musl↔wasi-libc fcntl ABI mismatch), ticket
   `docs/jira/v26.10.22/SQLRDF-27.md`; standing citations:
   `docs/jira/v26.10.22/RECEIPTS.md` line 2045 ("tc08 egress stays OPEN on
   SQLRDF-27") and `docs/jira/v26.10.23/compliance-final-verdict.md` R-13
   (BACKLOG, "OPEN on disk").
3. **Typed skips: none observed.** `refusal_crosswalk_differential`'s
   `refusal_crosswalk_pin_matches_live_sibling` did NOT take the BLOCKED/skip
   path: the sibling pack exists at
   `/Users/sac/ggen-marketplace/packs/refusal-vocabulary-pack` and the
   pin-vs-live-sibling byte drift check executed for real (log shows
   `test refusal_crosswalk_pin_matches_live_sibling ... ok`, no
   "BLOCKED: sibling pack absent" line). No `0 ignored` test carries a skip
   witness in any of the 13 logs.

## Discovery beyond the dispatched 9

`ls tests/ | grep -i sql` plus header inspection added 4 courts to the witness:

- `set_operations_court` (16) — SQLRDF-27 dialect gap #1: UNION [ALL] [BY NAME]
  / INTERSECT / EXCEPT as sql-ast nodes; landed in HEAD commit `6da1d33`.
- `chicago_validation` (12) — named in `CLAUDE.md` as part of the sql-rdf
  acceptance set; real frozen-duckdb per statement class.
- `parse_refusal_court` (12) — typed `dialect::ParseRefusal` surface adjacent
  to the SQL-in-RDF compile path (ReasonClass/RefusalKind).
- `sniff_court` (13) — sniff routing of SQL-in-RDF bytes vs plain Turtle
  (design-sniff-fixes G8 rows).

Considered and left out of scope: `w3c_conformance` (RDF-side, not sql-rdf),
`k6_hostile_lease` (lease domain), `wasm_*` courts (wasm lane's lease),
`wasm_fibo_egress_eh` (excluded by dispatch, classified above).

## Falsifiers (what would falsify "all courted SQL-RDF green")

1. Any of the 13 courts returning a non-`ok` `test result:` line or nonzero
   exit on this HEAD with pin `58a44095…`.
2. Refusal-arm inversion: making `src/sql_ast.rs` accept CONNECT/DISCONNECT
   must flip `connect_disconnect_carry_refusal_02` +
   `l163_msg_connect_statement_is_verbatim` red; if it still passes, the
   REFUSAL-02 court is vacuous.
3. Canon-hash inversion: any change to the receipt hash law
   (BLAKE3(Canon(G_AST) || P_params) via `purrdf::try_canonicalize`) must flip
   `five_cosmetic_variants_hash_identically_through_public_path`,
   `parameter_change_moves_the_digest`, `semantic_literal_change_moves_the_digest`,
   or `receipt_equals_recomputed_canon_plus_params_hash` red.
4. Census drift: the `sql_rdf_dialect` executed count is 36 on this tree; a
   different witnessed count (without a receipted test addition) invalidates
   the "36/0 durable" claim.
5. Sibling drift: byte-difference between
   `tests/fixtures/refusal_crosswalk/*` and the live sibling pack must turn
   `refusal_crosswalk_pin_matches_live_sibling` red; passing after a sibling
   edit means the pin court is dead.
6. Pin drift: rebuilding `priv/graphlaw.wasm` to any other b3 digest means this
   receipt no longer witnesses the current artifact; re-run required.
7. Dirty-tree caveat: HEAD stayed fixed during the run, but 21 dirty entries
   landed mid-verification (466→487). A court re-run after the closeout lanes
   integrate is the only observation that would extend this receipt to the
   post-closeout tree.

## Court rung used

`cargo test --test <target>` (single-court rung, cheapest that answers the
question). Escalation to full `cargo test` / mutation court / SPC court was
not required for this lane's deliverable and was not run.

## Pre-existing compile warnings (disclosure, nothing fixed)

The lane is a verification court: zero fixes applied, zero src edits. Compiles
witnessed (per `cargo test` logs): `graphlaw` (lib) 6 warnings — including 4 in
`src/sql_ast.rs` (1592 `unused variable: params`; 2712 `unused variable: c`;
2729/2730 `value assigned to iniri is never read` / `iniri assigned but never
used`), plus 2 in `src/sql_ast/serializer.rs:569` and `src/egress/arrow.rs:180`
(not lane files); `graphlaw_ocel_export` bin 7 warnings; cargo-level
multiple-build-target warnings on 7 `examples/*.rs` files and a non-root
profile notice. All pre-existing on this tree; none are dialect-court
failures. Left in place for the owning lanes/coordinator.

---

receipt-b3sum: `bc081ca921507b5e2121f0c19f7781dadc89f9e5b5d12a74af7c526ee306fad0`

self-hash protocol: b3sum over every byte of this file as it stood
BEFORE this line was appended (a file cannot contain its own hash).
Verify with:

```bash
awk '{a[NR]=$0} END {for(i=1;i<=NR-12;i++) print a[i]}' /Users/sac/graphlaw/receipts/2026-10-24-dialect-sql-rdf.md | /opt/homebrew/bin/b3sum
```
