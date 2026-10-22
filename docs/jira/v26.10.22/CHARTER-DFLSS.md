# CHARTER-DFLSS.md

> **PENDING-VERBATIM**: The operator's charter ("Project Charter & Industrial
> Verification Dossier: The Invariant Data Substrate", DMEDI Track Phase 5)
> was dispatched to this milestone by summary only. No verbatim charter text
> is on disk in this repo. Every figure below is a **dispatch-carried
> summary** — it is UNVERIFIED until the verbatim charter text lands here and
> each number is re-verified against the DFLSS lane receipts it cites. Do not
> cite this file as a receipt.

Status: skeleton pending verbatim charter text (DMEDI Phase 5 — Implement).

## §1.2 CTQs (dispatch-carried summary)

| CTQ | Dispatch-carried value | Witness (DFLSS lane output) |
|---|---|---|
| Refusal latency | 0.10 µs | p28_hot_path_bench / evidentiary_ring courts — verify against real bench output before citing |
| Throughput | 12M rows/sec | p28_hot_path_bench — verify against real bench output before citing |
| Ledger write latency | 0.25 µs | query_ledger courts — verify against real bench output before citing |

## §2 Pareto (dispatch-carried summary)

Contribution split 48.2 / 27.4 / 16.1 / 8.3 across the four defect classes.
Lane Pareto output is the witness; the class labels were not carried in the
dispatch summary and must come from the verbatim charter.

## §3 p-FMEA (dispatch-carried summary)

| Failure mode | RPN before → after | Status on this tree |
|---|---|---|
| Row 1 | 420 → 10 | closed — verify against lane receipt |
| Row 2 | 240 → 8 | closed — verify against lane receipt |
| Row 3 | 270 → 9 | closed — verify against lane receipt |

**Open residual found during verification (this session, verified on disk)**:
the sql_ast expression-depth channel (RecursionLimitExceeded(32) at
`src/sql_ast.rs:719-720`; serialize_expr MAX_DEPTH(64) at `src/sql_ast.rs:573-574`)
had NO flipping court — deleting the depth guard passed the entire suite.
Closed this session: `deep_nested_expression_refuses_recursion_limit` in
`tests/sql_rdf_laundering_corpus.rs` (40-deep nested sql:leftOperand chain →
typed RecursionLimitExceeded(32); benign 4-level twin flips).

## §5.2 Capability (dispatch-carried summary)

μ = 0.198 µs, σ = 0.012 µs, Cpk = 1.44. Witness: the DFLSS capability-study
lane output; not on disk here yet.

## §6 OCAP (dispatch-carried summary)

OCAP control-plan rows were carried by dispatch summary only; row contents
(baseline → out-of-control action) must come from the verbatim charter.

## Verbatim-landing checklist

- [ ] Verbatim charter text filed in this directory
- [ ] §1.2 CTQ values re-verified against lane bench receipts
- [ ] §2 Pareto class labels + values re-verified
- [ ] §3 p-FMEA rows re-verified, residual register reconciled with
      `tests/sql_rdf_laundering_corpus.rs` depth court
- [ ] §5.2 capability stats re-verified against the capability-study receipt
- [ ] §6 OCAP rows transcribed verbatim
