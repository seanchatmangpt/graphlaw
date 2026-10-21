# GWS-10 — Cold-Chain E2E Court (Phases 1-6 in one wasm call sequence)

Standing: PASS (integration crown: all case-study phases in one wasm sequence)
Context: pair · Date: 2026-10-21 · Lane: GWS-10 · Milestone: v26.10.21 (smalt) / graphlaw
Court file: `/Users/sac/graphlaw/tests/wasm_coldchain_e2e.rs` (3 tests)
Corpus fixtures: `/Users/sac/graphlaw/tests/fixtures/coldchain_e2e/*.csv` — byte-identical to COLD-1's corpus (`sha256` per-file equality verified against `/Users/sac/smalt/tmp/coldchain/`):
```
ocel_events.csv           1b2db9dcdd03bbb61a8a7117755301c250f991907c21826f053efab866c449d8
ocel_objects.csv          473f19f4d39a75f412abc379e9a069ce36e206ef466c57b346d81135a97c0b18
ocel_event_object_rel.csv b6dbd631571a7d7c82ac1125043a56f15865090ee26da34d3f1644dd4fc6c8d5
```

## What ran inside the wasm guest (real graphlaw-wasm module via tests/common/mod.rs wasmi host)

1. **Phase 2 — SHACL preflight (in-wasm, `op: shacl`)**: the cold-chain AST
   (`tests/fixtures/ast_governance/cold_chain_ast.ttl`) against
   `ontologies/shapes/ast_governance.ttl` ADMITS (`conforms: true`, zero
   violations).
2. **Phases 3-5 — typed stub (PENDING-GWS-3 / PENDING-GWS-4)**: the ABI has no
   duckdb op yet, so the aggregate pushdown (JOIN events↔rel on event_id, WHERE
   temp > -20.0 AND qualifier='contains_product', GROUP BY object_id, MAX,
   SUM — case-study §3) is computed by the host stub from the same injected
   corpus and re-injected as materialized triples. Mechanism evidence pointer:
   COLD-1 PASS receipt
   (`/Users/sac/smalt/docs/jira/v26.10.21/receipts/cold1-pipeline-court.md`)
   covers the pushdown natively with the real duckdb CLI on this exact corpus.
3. **Phases 5-6 — materialized triples + N3 fixed-point (in-wasm, `op: n3`)**:
   one gl_call carries corpus (17 events / 17 rels / 10 objects) + aggregate
   triples + ruleset (case-study §5 on the SWAP math builtin family).
4. **Phase 7 — derived-state assertions**: in-wasm partition equals COLD-1's
   independent partition exactly —
   `released {R1..R5}`, `quarantined {A1, Q1, Q2, Q3}` derived in-guest;
   `A2` quarantined through the FAIL-CLOSED missing-premise path (strict
   WHERE temp > -20.0 eliminates all its events → no aggregate row → derives
   NOTHING in-guest; asserted absent from derived output). A1 (max = -15.0
   exactly, duration = 300 exactly) is value-quarantined via RULE 1b
   (math:notLessThan -15). No cross-fire (partitions disjoint).
5. **Non-vacuity legs**: (a) stripping `gl:auditProof` from R1 collapses
   exactly R1's release (no proof → no release); (b) tampered digest leg:
   a wrong proof never changes the partition and fails the host identity check.
6. **Differential leg**: native `graphlaw::abi` and the compiled wasm module
   agree byte-identically on the full admission request.

## Command + output

```
$ cd /Users/sac/graphlaw && cargo test --test wasm_coldchain_e2e
test gws10_tampered_proof_is_not_a_release_premise ... ok
test gws10_native_wasm_differential_on_admission_request ... ok
test gws10_full_pipeline_in_one_wasm_sequence ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s
```
Second consecutive run (determinism): `test result: ok. 3 passed; 0 failed; ... finished in 0.44s`

Materialized aggregates observed in the request (must equal COLD-1's): A1
300 / -15.0 · Q1 400 / -12.5 · Q2 140 / -14.0 · Q3 400 / -16.0 · R1 120 / -17.0
· R2 250 / -18.5 · R3 300 / -16.0 · R4 250 / -16.5 (heartbeat event eliminated)
· R5 200 / -19.5 · A2 absent (missing premise). Match.

## Bounded-poll of GWS-2 / GWS-5 / GWS-6 (as dispatched)

- **GWS-2** (build outcome): the wasm32-wasip1 build under
  `target/wasm-abi/wasm32-wasip1/wasm/graphlaw_wasm.wasm` is produced by the
  harness (`wasm_path()`), including two full rebuilds during this lane's
  fix cycle. Build ALIVE; a dedicated build receipt remains the GWS-2 lane's
  deliverable.
- **GWS-5 / GWS-6** (`cargo test --test wasm_governance_preflight
  --test wasm_n3_admission`, ~67 s wasm build + run): **GWS-6 green; GWS-5's
  two hostile-AST assertions failed in their then-current in-flight state**
  (violation `shape` reported as a blank node
  `_:gts_00000000000000000000000002`, not the named shape IRI). Disclosed as
  PRE-EXISTING to the GWS-5 lane, whose files
  (`tests/wasm_governance_preflight.rs`, `ontologies/shapes/ast_governance.ttl`,
  mid-flight diff renaming expected shapes to `sql:HasWhereClause` /
  `sql:TableNamePattern` property shapes) are owned and being edited by that
  lane — this lane did not touch them. The GWS-10 conforming leg (the only
  shared surface this court depends on) passed even against the in-flight
  shapes file.

## Typed limitations (honest)

- Phases 3-5 run as a typed host stub, NOT in-guest: `PENDING-GWS-3`
  (duckdb-op ABI) / `PENDING-GWS-4` (data injection). No in-guest duckdb
  exists; the court types this explicitly rather than claiming in-guest
  pushdown.
- Audit digest is an FNV-1a 64 court identity, not a GxP BLAKE3 claim
  (case-study numbers remain OPERATOR-CHARTER design targets).
- Eyeron's derived closure omits already-explicit facts
  (`include_explicit: false`), so the court proves proof-binding through the
  strip/tamper legs, not by echoing the digest in derived output.

## Machine

macOS 26.2 (25C56), arm64 (Apple Silicon); cargo test driving the compiled
`graphlaw-wasm` (wasm32-wasip1, `--profile wasm`) under wasmi. (`just
bench-machine` not invoked — this is a court lane, not a bench lane; no jsonl
emitted.)

## Replay

```
cd /Users/sac/graphlaw
cargo test --test wasm_coldchain_e2e
```

Exit 0. No `Process.sleep` anywhere; the court polls nothing — single
deterministic call sequence per test.

## Standing

PASS — the full cold-chain pipeline (corpus injection → SHACL preflight admit →
aggregate → materialized triples → N3 fixed-point → COLD-1-partition equality +
fail-closed missing premise + proof-strip/tamper non-vacuity) is ALIVE inside
one wasm call sequence, with a native-vs-wasm byte-identical differential.
In-guest SQL pushdown remains PENDING-GWS-3/GWS-4; those tokens stay open in
`docs/wasm-duckdb-integration.md` (no flip without their own receipts).
