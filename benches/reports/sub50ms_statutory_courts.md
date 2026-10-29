# sub50ms_statutory_courts

Statutory boundary benchmark report: the 80/20 EU-AI-Act operational envelope
(Art. 12 logging, Art. 14 fail-closed oversight, Art. 15 deductive
traceability), measured on the real `graphlaw` pipeline — real BLAKE3 receipt
folds, real SHACL validation over a real candidate state, real bounded N3
derivation with proofs over quarantined `{ ... }` formulas. No mocks, no
sleeps; every leg asserts on real output state and every gate can fail
(incomplete fixpoints fail the gate by construction).

## Method

- Harness: `benches/statutory_boundary_50ms.rs` (harness = false, nearest-rank
  quantiles over per-iteration `Instant` samples).
- Fixtures sized to the directive's operational sweet spot: receipt legs over
  a 4 KiB canonical statement + 3 params; SHACL over a ≤ 64-quad candidate
  state (degree ≤ 8) against a depth ≤ 2 closed shape (id/acct/qty cardinality
  + datatype + IRI nodeKind); N3 over a policy program with two formula
  rules (threshold + halt) and a `math:greaterThan` builtin gate, budgeted
  exactly as `src/law.rs` wires `Step::DeriveN3`.
- Fail-closed legs assert the invariant, not the timing: the conforming
  candidate must conform 400/400 (2,000/2,000 full) and the violating
  candidate (undeclared `ex:ghost` property under `sh:closed`) must be
  halted 100% of runs. An incomplete N3 fixpoint panics the bench.
- Machine disclosure: 100,000 receipt iters, 2,000 SHACL iters, 400 N3
  iters, macOS aarch64, `cargo bench --bench statutory_boundary_50ms`
  (profile `bench`, optimized).

## Measured results (this tree, this run)

| Gate | Leg | p50 | p95 | p99 | Ceiling | Verdict |
|---|---|---|---|---|---|---|
| Art. 12 receipt issuance | `admitted_statement_hash` (full re-absorb) | 2,750 ns | 2,834 ns | 3,041 ns | 500 µs | PASS (164× headroom) |
| Art. 12 receipt issuance | `fold_params_anchored` | **125 ns** | 125 ns | 125 ns | 500 µs | PASS (flat across 100k iters) |
| Art. 14 SHACL boundary | `validate_candidate` (conforming) | 63,334 ns | 69,667 ns | 74,666 ns | 5 ms | PASS (67× headroom) |
| Art. 14 SHACL boundary | `failclosed_halt` (violating → halt) | 70,375 ns | 73,375 ns | 81,917 ns | 5 ms | PASS |
| Art. 15 bounded N3 proof | `n3_derive_proof` (formula rules + `math:*`, proofs on) | 30,292 ns | 31,125 ns | 38,375 ns | 30 ms | PASS (780× headroom) |

**End-to-end: sum of gate p99s = 195.1 µs against the 40 ms budget — 205×
headroom under the strict 50 ms ceiling.** `overall: ALL GATES PASS`.

## Reproduce

```sh
cargo bench -p graphlaw --bench statutory_boundary_50ms -- --quick
```

(The full run drops `-- --quick`; iteration counts are printed in the header.
Gate verdicts are recomputed per run — a gate that cannot fail is not a
gate.)

## Structural comparison (incumbent columns are claims, not benchmarks)

The incumbent columns are **structural claims drawn from vendor-documented
architecture**, not numbers measured here. The Sovereign column carries this
run's measured numbers.

| Statutory invariant | Incumbent graph DBs (Stardog, GraphDB) | Frontier-AI / wrapper stacks | Sovereign substrate (`graphlaw_wasm`) |
|---|---|---|---|
| Art. 12 wire receipt | Narrative `explain` diagnostics; no sealed artifact | Ephemeral prompt logs; LLM-as-judge (probabilistic) | **Measured 125 ns p99** anchored BLAKE3 fold; seal at the boundary |
| Art. 14 oversight halt | Server-side; no in-process fail-closed sandbox halt | Cloud round-trip; async tool-call timeouts | **Measured 82 µs p99** halt: candidate state → non-conformance → stop, in-process |
| Art. 15 proof search | Cost-based planners; derivation path not sealed | Stochastic steps; no closed derivation object | **Measured 38 µs p99** bounded derivation, proofs emitted, fail-closed on incompleteness |
| Portability & footprint | JVM/Lisp server runtimes; server-only | Cloud REST; data leaves the perimeter | **Measured 60,880,445 B single wasm32-wasip1 module**, zero network |

## Witnesses (all from this tree)

- `graphlaw_wasm.wasm` — 60,880,445 B, links and executes N3 inside the
  module (`wasm_abi` 71 passed, `chicago_cross_authority` 25 passed,
  `wasm_n3_admission` 3 passed, `wasm_size_court` 4/4).
- Receipt hot path: `benches/tc_hot_01.rs` (anchored fold ≤ 250 ns p99
  reference, consistent with this run's flat 125 ns).
- Migration + audit receipts: `receipts/2026-10-25-n3-upstreaming-purrdf.md`.

## Honest caveats (binding)

1. Incumbent columns are structural claims from vendor-documented
   architecture — not benchmarked here, and Stardog's `explain` does emit
   proof trees; the defensible differentiator is sealed, machine-checkable
   wire artifacts vs. narrative diagnostics, plus in-sandbox execution.
2. The 2026-10-25 audit
   (`docs/jira/v26.10.23/n3-audit-100-report.md`) holds verdict
   **conditional**: the N3 proof pipeline's court coverage is being closed in
   the current fix wave. This bench is now a positive witness for Art. 15
   latency and completeness (incomplete fixpoints fail the gate), but it does
   not substitute for those courts.
3. Numbers are from one host (macOS aarch64), bench profile, iteration
   counts as disclosed. Reruns recompute verdicts.
