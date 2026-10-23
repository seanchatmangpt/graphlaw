# P28-SIL spec records — §3.1 fold, §3.2 ring, enforceability, sequencing

Condensed from four spec lanes (full text in wave transcripts).

## Deployment-epoch term ω_context (SQLRDF-18, operator ruling 2026-10-22)

The license's three-term H_query identity
(H = BLAKE3(Canon ‖ P_params ‖ ∮ω_context)) is formalized: **ω_context is the
deployment epoch** — a `u64` assigned exactly once at substrate initialization
(ggen build stamp / deployment id), immutable for the deployment lifetime.
NOT ambient hardware state. Same epoch → same receipt (deterministic,
replayable within a deployment); cross-deployment receipts are
domain-separated. The landed two-term formula is the ω_context = 0
(single-deployment) special case — the term is **reserved**, zero-valued, no
code change required now. Reserved derivation pinned by the epoch court:
H³(epoch) = BLAKE3(H² ‖ epoch.to_le_bytes()) with H² the landed two-term
hash; epoch 0 gives the reserved single-deployment value, any nonzero epoch
domain-separates. Harmonization notes filed in LICENSE-P28-SIL.md §1.5,
PRD-P28-SIL.md §3.1.2, sql-rdf-rec.md §1.4.

## §3.1 incremental receipt fold (0.15µs path)
Landed: `CompiledQuery.receipt_state` is a 32-byte `h_ast`, not a
`blake3::Hasher`. `fold_params_anchored` copies `h_ast` and the inline param
bytes into one `[u8; 96]` and calls `Hasher::update` once. Bit-identical to
`admitted_statement_hash`
(`receipt_state_clone_path_bit_identical_to_reference`, exit 0,
2026-10-22). `admitted_statement_hash` still re-absorbs the canon; that is
the baseline leg, not the receipt path. The 150 ns §2 row stays a phase-2
target with no gate. TC-HOT-01 full 10M (exit 1): clone-path p99 209 ns
(inside 250), variance 26.91% FAIL. The timed refusal leg is two `write_str`
copies of a static class label and `RefusalMessage::Static` (p50 41 ns /
p99 42 ns, PASS). `Display` stays outside that leg. SPC smoke Cpk 0.001
FAIL (`receipts/spc-2026-10-22T191548.json`); p99 209 ns is inside the
250 ns USL. The miss is the scheduler tail (max 21,413,887 ns).
Receipt: `designs/tc-hot-01-errc-receipt.md`.

## §3.2 SPSC evidentiary ring (Phase 2)
- `RingDescriptor` — exactly 64B `#[repr(C)]`: h_query[32], rows_emitted u64,
  execution_ms u32 (saturating), cached u8, pad, timestamp_ms u64, pad
  (drainer validates zero pads = torn-read detector).
- Ring: recommend `rtrb` (structurally SPSC, wait-free, no_std, wasm-clean);
  hand-rolled Lamport fallback ~80 lines if dep review refuses. 4096 slots
  (256 KiB).
- Drainer: batch-to-empty ≤256, 2ms sleep when empty (deliberately no condvar
  — producer never takes a lock even indirectly), fsync every 16 flushes
  (~1s crash window, typed: torn tail detected via chain, never accepted).
- Ledger: NDJSON per subject mirroring receipt_store.rs conventions
  (canonical JSON, BLAKE3 parent→child chain, atomic writes, verify law).
- Overflow: DROP-NEWEST (deliberately superseding drop-oldest — drop-oldest
  breaks SPSC purity and is structurally incompatible with never-block under
  a stalled drainer); `DroppedEvidence { dropped_count, refused_h_query,
  refused_at_ms }`, overflow events chained as ledger lines, counter surfaced
  in capabilities.
- Seam: end of `eval_ast_query`, `#[cfg(feature = "evidentiary-ring")]`
  (default OFF until TC-HOT-01/02 bench it; the drainer is native-side
  (std fsync) and not wired into the wasm32 path — frozen-duckdb itself is a
  normal dependency of the `graphlaw` crate with no feature gate, dispatched
  by the wasm module itself, so no separate duckdb target exists; explicit
  init(dir), no surprise-I/O OnceLock).
- Courts: chain integrity under 100k push/drain, zero-loss happy path,
  torn-slot typed refusal, pad-zero detector, THE no-blocking property test
  (1M pushes against a stalled drainer: max push latency <10µs, drops
  observed>0 — anti-vacuity), off-by-default compile check.
  Superseded 2026-10-22: the no-blocking court is now a two-leg fence (dev
  ratio fence + release absolute <10µs leg); see
  `cold-path-ledger.md` § "Restart watermark reconciliation (landed)" —
  which also records the landed fix for the fuzz case-4 crash-restart
  silent-renumbering defect.

## P28-SIL enforceability map
| Mechanism | Substrate | Status |
|---|---|---|
| Receipt generator R_receipt | admitted_statement_hash / H_query | LANDED |
| Cryptographic ledger of operational receipts | receipt_chain.rs + receipt_store.rs cover LAW receipts only; query-receipt ledger = PRD §3.2 design | PARTIAL |
| Σ_regularized certificate | attest.rs Ed25519 sign_bytes | LANDED (Ed25519); Σ_PQC clause UNMET (docs/refusals.md:47) — no PQ claim may be made |
| $50M revenue threshold | none in-code; honor-system + ex-post ledger evidence | HONOR-SYSTEM |
| Circumvention detection | no technical signal today; cheapest witness = surface_blake3-style digest row pinning the receipt-generation surface (duckdb_eval.rs) in capability-registry + ARTIFACTS.blake3, re-pinned per release (gws-16 workflow) | NOT PRESENT (legal-only) |
Honest phrasing for LICENSE.md implementer: generator exists and is
content-bound; ledger designed-not-built for the query path; signatures don't
cover query receipts; ratchet is legal-only.

## Phase sequencing (dependency-true)
M0 integration commit → M1 TC-HOT bench court (independent) · M2 Phase-3
residue (mutation sweep re-run + TC-MOCK gate + e4 receipt) · M3 blake3
state-clone (needs M1) · M4 E1 → M5 A1 → M6 E2 → M11 E3 · M7 R2 → M8
SQLRDF-17 → M9 C2 wire decision → M12 Phase-1 close · M10 C1 (R1+E1) · M13
Phase-2 ring (M1 only) · M14 Phase-4 close (independent).
**Critical path: M0 → M7 → M8 → M9 → M12** (the C-chain, not the E-chain —
more independent work per hop and the only external coupling via the census
court).
