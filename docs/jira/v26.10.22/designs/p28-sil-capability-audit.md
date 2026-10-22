# P28-SIL v2.1.0 capability audit — consolidated record (12 lanes)

Condensed from CAP-1..12 audit lanes. License text status at audit time:
v2.1.0 text supplied by operator in-session; landed LICENSE-P28-SIL.md was
v1.0.0 (CAP-12 filed v2.1.0 concurrently). Tree @ ed3be81 + working set.

| CAP | Capability | Verdict | Key findings |
|---|---|---|---|
| 1 | H_query formula | **FORMULA DISCREPANCY — operator ruling needed** | v2.1.0 §1.5 adds a third term ∮ω_context; all in-tree normative docs (license v1.0.0 App A, PRD 3.1.2, sql-rdf-rec §1.4) are two-term; landed code is two-term and faithful to them. ∮ω_context unimplemented AND unspecified. Filing v2.1.0 makes it a real gap AND likely breaks environment-invariance (the cosmetic court's own property) — the third term binds execution context into the receipt by design. Operator must rule: adopt 3-term (breaks env-invariance witness) or keep 2-term (license §1.5 amended). |
| 2 | ≤0.25µs p99 | **NOT MET (honest FAIL, witnessed across 3 independent runs)** | CAP-2 final: capability bench NOT-CAPABLE (μ=613ns, Cpk −0.00, p99 459ns under load); SPC court FAIL both legs (p50 208ns, p99 292ns — today's best, still over); least-contended min-of-batches floor 301.2ns > 250ns USL even before quiet-machine re-measure. Closure paths: (1) RefusalMessage::Static fast path (closes the 100ns refusal leg only), (2) shrink the ~2KB hasher-state clone memcpy (dominant fold cost), (3) quiet-machine re-measure, or (4) measurement-backed CTQ row correction. Also disclosed: bench [CMD] line hardcodes target-dflss3 (replay-receipt skew); SPC mktemp poisoned by stale literal-named file in ~/.cache/tmp (environment defect). |
| 3 | SPSC ring + ledger | **ALL 5 §6.2 REQUIREMENTS PASS** (audited: 8/0 + 9/0 courts + 72.25M rows/s scanned gate) | SPSC lock-free (rtrb 4096, try_push never blocks); persistence decoupled (dedicated drainer, batch ≤256, BLAKE3 NDJSON + joint ledger); saturating drop-newest with u64::MAX saturated flag; typed gap markers (BrokenChain/TornSlot/SequenceGap/DigestMismatch/Fork/DoubleGenesis); columnar throughput preserved (72.25M scanned rows/s > 12M gate). Conservation-mismatch "998" not-found-on-disk — root-caused as court arithmetic (paired rounds push 2N; joint court's three-way exact-equality asserts at :323-341 passed this session). |
| 4 | No ambient authority | **3 PASS / 2 FAIL** | PASS: no bypass flags/env/cfg around validate_ast_shacl; mount_view validated; no debug backdoors. FAIL: duckdb_op ast.text raw-SQL admission (SQLRDF-17, live); raw_connection() pub + ungated + undocumented per-site. Residual: mount_view glob → ambient FS read (unaudited axis). |
| 5 | ASC 450 accrual | **PARTIAL** | Σ-accumulator per-epoch with saturation flag: court-ready. UnitCost NOT representable (execution_ms dropped at ring→ledger boundary; no tariff concept); epoch always 0 in ledger path (degenerate per-epoch breakdown); retention/GC design-only. |
| 6 | Σ_regularized certificate | **NOT CAPABLE** | sign_bytes primitive sufficient; no certificate schema, no lease↔fee linkage, no accrued balance to extinguish, no issuance orchestration, no append-only cert store. License v1.0.0 Appendix A already concedes Ed25519-only. |
| 7 | Circumvention detection | **1/4 requirements** | Receipt-surface digest pin: PRESENT + fail-closed (3-way enforcement). MISSING: CI fail-closed on suppression; static-analysis flagging of receipt-nullifying configs; pin omits src/duckdb_op.rs (where the hole lives). SQLRDF-17 = live §3.2 channel, ungated. |
| 8 | Typed refusal no-bypass | **ALL FOUR ENFORCED AT CONSTRUCTION** | 25/0 + 20/0 courts post-restructure. One dead variant (RefusalCode::Capability, render-only — REFUSAL-04 lives in tier.rs); one classified residual (ShaclViolation compat shim classifies by substring of a pub const — not a bypass witness). |
| 9 | SBOM classification | **FAIL all machine surfaces** | All 3 manifests emit plain MIT; P28-SIL not packaged in crate/npm. Fix in flight: `MIT AND LicenseRef-P28-SIL-2.1.0` + include/files packaging. |
| 10 | Canon invariance | **WITNESSED (graph-backed); VIOLATED on legacy paths** | 5/5 cosmetic properties witnessed on eval surface; M3 clone bit-identity property-tested (256 fuzz cases). Legacy SQL-string hash arm reachable via duckdb_op exec + rules/builtins — non-canonical bytes enter the hash on those paths (SQLRDF-17 class). |
| 11 | Evidentiary continuity | **HELD (restart); UNCOURTED (upgrade)** | Restart watermark reconciliation landed + courted (deviation accepted: restart_seq not live counter — prescribed form double-covered 911). Hot-upgrade structurally held but uncourted; SQLRDF-20 spec gap: 64B descriptor has no admission-table-version field (implementability gap the ticket missed). |
| 12 | License filing | **FILED (v2.1.0)** | CAP-12 replaces LICENSE-P28-SIL.md v1.0.0 body; MIT pointer retained; implementation-status appendix updated (refusal generator LANDED; ledger LANDED; Σ_PQC UNMET; revenue honor-system; circumvention witness = receipt-surface pin LANDED). |

## Cross-cutting
- The single deepest finding: **the legacy (None,None,None) hash arm is the
  convergence point of 5 audits** (CAP-4 bypass, CAP-7 circumvention,
  CAP-10 non-canonical bytes, CAP-11 upgrade-record gap, SQLRDF-17) —
  closing it closes findings in 5 lanes at once. Post-integration priority 1.
- License v2.1.0's three-term H_query formula needs an operator ruling
  before any implementation (it conflicts with the environment-invariance
  court).
- Feature-matrix: evidentiary-ring is release-only-tested; runner bin has
  zero coverage (SQLRDF-19).
