# Moonshot design records — v26.10.22 (condensed)

Full designs were produced by read-only spec lanes; these files are the
coordinator's condensed records. Anchors verified against the tree at
filing. Rows map to SQLRDF-16.

- [E1-alias-elimination.md](E1-alias-elimination.md) — alias-window elimination (census + one-commit rewrite)
- [A1-scope-as-type.md](A1-scope-as-type.md) — entailment closure + positional TripleRules (= SQLRDF-13 plan)
- [C1-cache.md](C1-cache.md) — H_query content-addressed eval cache
- [E2-shapes-reader.md](E2-shapes-reader.md) — shapes as single grammar description (full spec, written by its lane)
- [R2-capability-routes.md](R2-capability-routes.md) — capability route table
- [E3-conformance.md](E3-conformance.md) — executable conformance rung
- [SKEPTIC-thesis-review.md](SKEPTIC-thesis-review.md) — adversarial grading
- [TC-HOT-benchmarks.md](TC-HOT-benchmarks.md) — TC-HOT-01/02 harness design
- [P28-SIL-specs.md](P28-SIL-specs.md) — §3.1 incremental fold, §3.2 SPSC ring, enforceability map, phase sequencing
- [RECEIPTS.md](RECEIPTS.md) — moonshot-layer manufacturing receipt (SQLRDF-16/17 family; companion to the window-level [RECEIPTS.md](../RECEIPTS.md))
- [cold-path-ledger.md](cold-path-ledger.md) — cold-path query-receipt ledger: 64B descriptor + SPSC-ring contract (PRD §3.2.2; companion to C1-cache)
- [hot-path-baseline.md](hot-path-baseline.md) — DMEDI Measure: release-mode `tests/p28_hot_path_bench.rs` baseline vs budget (ratios are the signal)
- [PHASE1-generated-gate.md](PHASE1-generated-gate.md) — Phase 1: shapes → compiled SHACL gate + canon-bytes wire (extends E2; third consumer of embedded shapes)
- [receipt-surface-pin.md](receipt-surface-pin.md) — receipt-surface digest pin: upgrades P28-SIL §4 circumvention from NOT PRESENT to technical witness
- [TC-MUT-01-mapping.md](TC-MUT-01-mapping.md) — TC-MUT/TC-MOCK audit mapping: TC-MOCK-00 landed as `tests/no_mock_markers.rs` + TC-MUT-01 targets
- [sql-ast-decomposition.md](sql-ast-decomposition.md) — `sql_ast` split on disk (`ast` / `refusals` / `serializer` beside the parent); the prep-lane `mod.rs` + reader/dataset/compile cut is unexecuted
- [core-team-decomposition.md](core-team-decomposition.md) — operator brief for the primary and secondary file splits, reconciled to the trees on disk
- [deterministic-agent-admission.md](deterministic-agent-admission.md) — operator brief: admitted code deltas, semantic work graphs, axiomatic / projection / delta partitions
- [session-lithography.md](session-lithography.md) — session card, PreToolUse refusals, `shacl` admission, and the PostToolUse receipt fold
- [graphlaw-admission-blueprint.md](graphlaw-admission-blueprint.md) — PreToolUse `act:ToolCall` admitted by `graphlaw-run` against the seed shapes; receipt hook registered
- [v2030-receipt-pipeline.md](v2030-receipt-pipeline.md) — PostToolUse transition receipt and `CHAIN.blake3` fold; compaction still open
- [c4-architecture.md](c4-architecture.md) — operator C4 diagrams for the agent-governance platform
- [c4-reconciliation.md](c4-reconciliation.md) — L180 snapshot of those diagrams against the tree, plus the later registration correction
- [twenty-agent-scaling-thesis.md](twenty-agent-scaling-thesis.md) — 20-agent leverage claim, reconciled; non-bypassable coverage is the hooked surface only
- [code-first-refusals.md](code-first-refusals.md) — P29-RFL: pure-data 64B RefusalCode, ggen-generated code space, binary-first ledger (implements the operator's "no strings or json" ruling)
- [defect-pareto-verified.md](defect-pareto-verified.md) — witnessed defect Pareto vs charter: injection 45.5% / schema drift 18.2% / parser 13.6% / audit 4.5% / test-infra 18.2% (n=22); two charter claims REFUTED
- [ctq-traceability.md](ctq-traceability.md) — CTQ → court traceability matrix: 6 budgets, verdicts MET(smoke)/PARTIAL/UNCOURTED per row
- [pce-baseline.md](pce-baseline.md) — PCE baseline at 25M rows: 70.36% PCE, substrate overhead 42.12% — charter "<2.8%" REFUTED at this operating point
- [cpk-correction.md](cpk-correction.md) — Cpk capability correction: charter 1.44 NOT witnessed; measured Cpk ≤ 0 under load, fold 250-300ns intrinsic (budget-adjacent)
- [benchmark-census.md](benchmark-census.md) — every hot-path, SPC, PCE, and ring number re-read from disk; six SPC receipts all FAIL; prose that quotes cpk −0.0018 still names an overwritten file
