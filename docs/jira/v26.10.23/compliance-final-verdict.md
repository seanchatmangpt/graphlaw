# Compliance Final Verdict — EU AI Act + NIST AI RMF — graphlaw v26.10.22

Consolidation of the v26.10.23 compliance battery (7 matrices) into one
verdict. Subject: `/Users/sac/graphlaw` v26.10.22 line, HEAD `7e92175`
(compliance battery committed at `c6a1939`); tree re-verified on disk
2026-10-24 where this file states current state.

Inputs (all in `docs/jira/v26.10.23/`):

- `compliance-eu-ai-act.md` — Art. 9 risk-management evidence matrix
  (9 gaps, G-A9-1..9)
- `compliance-eu-ai-act-logging.md` — Art. 12 record-keeping + Art. 11
  technical documentation (3 gaps)
- `compliance-eu-ai-act-robustness.md` — Art. 15 robustness/cybersecurity/
  accuracy (3 gaps)
- `compliance-nist-gov-map.md` — NIST AI RMF GOVERN + MAP (10 gaps)
- `compliance-nist-measure-manage.md` — MEASURE + MANAGE (7 gaps)
- `compliance-transparency.md` — publication/artifact transparency (3 gaps)
- `compliance-gap-closure-plan.md` — classification + 2-item pre-tag
  blocker set + backlog (sourcing correction 2026-10-24)

Nature: consolidated due-diligence evidence verdict. Not a certificate,
attestation, or declaration of conformity. Every CONFORMANT claim is
mechanism-level and evidence-cited; every PARTIAL names its gaps; N/A
carries the classification argument. Status/count figures are as-of the
cited receipts; re-read at use time.

---

## 1. Classification (sizes everything else)

**Determination**: graphlaw v26.10.22 is not an AI system under
Reg. (EU) 2024/1689 Art. 3(1) and not high-risk under Art. 6 —
deterministic law-admission kernel: RDF/SHACL admission (`src/law.rs`),
SQL-in-RDF compilation (`src/sql_ast.rs`) over a closed statement-class
space, typed fail-closed refusals (`docs/refusals.md`, REFUSAL-01..05,
append-only), Ed25519 receipts (`src/attest.rs`). No training, no
inference from trained weights, no adaptivity, no autonomous goal
selection: the same input graph admits or refuses identically every run
(full grounds: `compliance-gap-closure-plan.md` §0; the Art. 9 matrix
§1 reaches the same conclusion independently). The nearest Art. 2(1)
surface is general-purpose AI-adjacent infrastructure: a downstream
high-risk AI system could embed graphlaw as its admission/egress gate,
so the Art. 9-15 mappings are filed as provider-side due-diligence
evidence for that embedding scenario — not as claims that graphlaw is
a regulated high-risk system.

Consequence: Art. 9-15 obligations are N/A for graphlaw as-shipped; the
surviving publish-gating duties are (a) honesty of the release's own
claims, (b) supply-chain/provenance integrity of what ships,
(c) not shipping with known-false documentation.

---

## 2. Per-framework / per-function verdicts

| Framework | Function / article | Verdict | Basis (evidence loci) | Gaps |
|---|---|---|---|---|
| EU AI Act | Art. 2/3(1)/6 classification | N/A | §1 above; `compliance-gap-closure-plan.md` §0; `compliance-eu-ai-act.md` §1 | — |
| EU AI Act | Art. 9 risk management | PARTIAL | Process shape exists and is receipt-gated: ticket ledger `docs/jira/v26.10.22/INDEX.md:24-574` (29 SQLRDF rows), residual register `docs/jira/v26.10.22/designs/residual-unknowns.md:1-226`, typed refusals REFUSAL-01..05 `docs/refusals.md`, SPC/mutation gates | R-1, R-5, R-6, R-12, R-13, R-14, R-30 |
| EU AI Act | Art. 11 technical documentation | PARTIAL | Doc surface substantially present: `docs/ARCHITECTURE.md`, `SECURITY.md`, whitepaper, refusal catalog, capability registry (generated), `docs/api-stability.md`; mapping table at `compliance-eu-ai-act-logging.md` §2 | R-22, R-23, R-24 |
| EU AI Act | Art. 12 record-keeping | CONFORMANT (mechanism) / PARTIAL (identity layer) | Automatic per-event receipts `src/duckdb_eval.rs:283`; input traceability `H_query = BLAKE3(Canon(G_AST) ‖ P_params)` (seal `src/sql_ast/ast.rs:645`, enforced `src/duckdb_eval.rs:1216`); 3 tamper layers incl. Ed25519 `src/attest.rs:417` and chain `src/receipt_chain.rs:777,933`; replay `src/bin/graphlaw_verify.rs:197` — courts: `cargo test --test chicago_validation`, `sql_rdf_dialect`, `duckdb_statement_kinds`, `trust_attest`, `receipt_chain_golden_test` | R-8, R-9, R-10 |
| EU AI Act | Art. 15 robustness/cybersec/accuracy | PARTIAL | Memory-wall 8/0 (`tests/wasm_memory_wall.rs`; RECEIPTS.md:1585); boundary fuzz 312/run deterministic, 1,172/0 aggregate (`receipts/gws-9-wasm-boundary-fuzz.md`); hostile SQLRDF corpus 23/0 (`receipts/sql-rdf-laundering-corpus-2026-10-22.md`); 28-import fail-closed allowlist + zero-preopen (`src/bin/graphlaw_run_eh.rs:222-244`, SECURITY.md); atomic batch + typed refusals (`src/duckdb_op.rs:359-369,467-485`; atomic
envelope court fn at :1465 — the robustness matrix's ":1253-1290" pointer
drifted, re-verified 2026-10-24); determinism courts ×3 (`tests/p28_determinism_court.rs`, `two_verifier_court.rs`, `canonical_cross_dialect.rs`) | R-1, R-2, R-3, R-4, R-17 |
| EU AI Act | Art. 14 human oversight | N/A | Binds high-risk providers only; determination recorded `compliance-gap-closure-plan.md` G-7. Substance exists anyway: typed refusal register + playbook (`docs/refusal-playbook.md`, 13/13 probe MATCH) | — |
| EU AI Act | Art. 10 data governance | N/A | Binds high-risk providers only (`compliance-gap-closure-plan.md` §4); no training/inference data surface exists (§1) | — |
| NIST AI RMF | GOVERN 1-5 | PARTIAL | Roles/lane leases, receipt-gated flips, honest-FAIL receipts, SPC numeric thresholds (`scripts/spc_court.sh:16-32`), qualification algebra standing lattice (`docs/QUALIFICATION_ALGEBRA.md:7-25`), incident lifecycle (`receipts/2026-10-23-staging-wipe-incident.md`), append-only registers | R-1, R-5, R-7, R-12, R-25, R-30 |
| NIST AI RMF | MAP 1-5 | PARTIAL | System definition + scope boundaries (`docs/whitepaper-v26.10.22.md:41-66,157-167`), exact-subject qualification algebra, banking-domain courts (DvP 4/4 witness `receipts/2026-10-23-banking-4of4-witness.md`), typed refusal vocabulary | R-26 |
| NIST AI RMF | MEASURE | PARTIAL | Divan baselines with embedded output (`receipts/2026-10-23-engine-baseline.md`, `-egress-baseline.md`), fuzz 312×2 deterministic, SPC methodology (1e6 iterations, Shewhart rules, variance self-check) — but the SPC verdict is honest-FAIL and coverage is counted, not measured | R-1, R-5, R-27, R-28 |
| NIST AI RMF | MANAGE | PARTIAL | Typed refusal register 12/12 probe MATCH (`docs/refusals.md:7-17`), fail-closed fuzz boundaries, NO-GO→settle escalation board (`docs/jira/v26.10.22/designs/integration-gonogo.md:70,107`), incident lifecycle + recovery VERIFY-OK, receipt-gated flips, derive-then-write pin flow (`docs/jira/v26.10.23/ADR-release-lease.md:97-107`) | R-2, R-5, R-16, R-17, R-18 |
| Publication/artifact transparency | docs/artifacts/claims/errors | PARTIAL | README/whitepaper/DIALECTS/how-to suite/CHANGELOG present with receipted actuals; BLAKE3 pins + 28-import inventory (`SECURITY.md:51-73`); claims audit trail + `register_completeness` court (`tests/sql_rdf_conformance.rs:854`); refusal catalog/playbook | R-22, R-23, R-24 |

No function is claimed CONFORMANT without mechanism + court loci; no
function is claimed fully green — that is the honest reading of the
battery.

---

## 3. Security-hardening items — dispatch vs disk (verified 2026-10-24)

The consolidation dispatch named five security-hardening fixes to file
as gap closures. Each was verified against the tree before filing.
Result: **one landed, one matches a landed closure under a different
identifier, three are not on the tree.** Filed at true status; the
divergence is recorded rather than papered over.

| Dispatch item | On-disk status (verified) | Loci |
|---|---|---|
| eyeron bump | **CLOSED.** `graphlaw-eyeron` bumped 0.7.8 → **0.7.9** on the tree | `/Users/sac/graphlaw/Cargo.toml:68`; `Cargo.lock` (`graphlaw-eyeron version = "0.7.9"`); was the closure plan's publish-preflight blocker (`compliance-gap-closure-plan.md`, sourcing correction) |
| ATTACH bypass E1 | **CLOSED (identifier mismatch noted).** The ATTACH-bypass class is closed on disk as SQLRDF-22 G6: fail-closed env-unset default, comment-camouflage immunity, pure seam | `src/duckdb_op.rs:214-247` (policy), `:306` (fail-closed core), `:218-219,244` (comment-prefixed ATTACH still an ATTACH); court W15 13/0 at `receipts/2026-10-23-eh-closure-wave.md:59`. No on-disk identifier "E1" matches an ATTACH bypass; E1 in `docs/jira/v26.10.22/RECEIPTS.md:30,88` is the alias-window elimination escape (also closed, with `tests/sql_rdf_laundering_corpus.rs` as regression net) |
| Frame bound R3 | **OPEN — not landed.** `read_frame` still allocates `vec![0u8; len]` unbounded from the 4-byte BE prefix on both hosts | `src/bin/graphlaw_run_eh.rs:336-350` (alloc at :344); mirror `src/bin/graphlaw_run.rs:457-471`; design SPEC unimplemented `docs/jira/v26.10.23/design-read-frame-bound.md:3`; gap register row G4 OPEN (`docs/reference/wasm-gap-register.md`), closing witness "bound court: exit 92" |
| Sterile WasiCtx C0b | **OPEN — not landed.** EH runner still builds the guest context with full env inheritance | `src/bin/graphlaw_run_eh.rs:245-251` (`WasiCtxBuilder::new().inherit_stdio().inherit_env().build_p1()`); disclosed limit `SECURITY.md:103-105` ("Strip the environment…"); risk row Art. 9(3)(c) (`compliance-eu-ai-act.md:137`) |
| Fuel/epoch C0c | **OPEN — not landed.** No CPU metering on either host | `SECURITY.md:96-98` ("NO fuel or epoch metering… a wasm-infinite loop pins the thread"); disclosed as Art. 9(3)(c) misuse residual; closure class = host-side metering or documented supported-embeddings restriction |

Filing rule applied: a fix asserted by dispatch but absent from the tree
is recorded as OPEN with its loci — never as a closure.

---

## 4. Consolidated gap register

Deduplicated across all 7 matrices. Classes: **GATE** = pre-tag gating
for the v26.10.22 tag; **BACKLOG** = improvement backlog (non-blocking);
**N/A** = recorded determination, not a gap. Flips are receipt-gated.

### Pre-tag gating

- **R-1 (GATE)** — v26.10.22 release-note truth-sync + artifact/source
  correspondence rungs. The two owed non-gating rungs on the canonical
  pin `472d5557…bf80ea` — host-contract rung `cargo test -p
  graphlaw-wasm` and one anti-vacuity inversion on the wrap-enabled
  artifact — are explicitly NOT closed by the SQLRDF-27 flip
  (`docs/jira/v26.10.22/designs/residual-unknowns.md` row 1 "Owes";
  `compliance-gap-closure-plan.md` G-2). The release notes still cite
  superseded artifact identities as current. **Closes with**: both rungs
  green with embedded output in RECEIPTS.md against `472d5557…`, plus
  the release-notes truth-sync edit. Blocking: yes — the release must
  not describe a shipped artifact different from the pinned one.
- **R-2 (GATE)** — cargo-deny residual advisories: no recorded operator
  accept/diagnose decision. **Closes with**: dated decision line per
  residual advisory in the release-notes checklist or RECEIPTS.md
  (`docs/jira/v26.10.22/RELEASE-NOTES-26.10.22.md:98-100`;
  `compliance-gap-closure-plan.md` G-3). Blocking: yes (process
  precondition of the release's own DoD; supply-chain integrity).

### Improvement backlog — kernel / sandbox mechanism

- **R-3 (BACKLOG)** — `read_frame` unbounded prefix alloc, both stdio
  hosts (§3 row 3; design `design-read-frame-bound.md`; gap register G4;
  Art. 15 robustness gap 2). Closes with the exit-92 bound court
  (exactly-at-cap leg + 0xFFFFFFFF leg + zero-len leg + wasmi mirror).
- **R-4 (BACKLOG)** — no fuel/epoch metering; in-wasm infinite loop
  pins the host thread (§3 row 5; SECURITY.md:96-98). Closes with host
  metering or a documented supported-embeddings scope restriction
  (Art. 9(3)(c) G-A9-6).
- **R-5 (BACKLOG)** — SPC process-capability FAIL standing: Cpk 0.0008
  (2026-10-23 witness; 0.001 on 2026-10-22) vs the 1.33 requirement,
  `receipts/2026-10-23-spc-witness.md`,
  `receipts/spc-2026-10-23T152558.json`. The gate exiting 1 is the gate
  working; the capability claim under load is not met. Closes with the
  solo-machine rerun separating load effect from real incapability
  (Art. 9(3)(b) G-A9-3; NIST GOVERN 3.4/MEASURE gap 1).
- **R-6 (BACKLOG)** — Art. 9(6)-class keep-with-controls-vs-abandon
  determination never made per residual row (the `src/abi.rs:152:5`
  mutation survivor is a tracked decision, not a filed determination;
  G-A9-8). Closes with operator-signed determinations per residual row.
- **R-7 (BACKLOG)** — mutation-court full sweep has no landed verdict:
  solo run refused on dirty tree (0/0/0 mutants), full run
  DISK-BLOCKED, ledger row IN PROGRESS
  (`docs/jira/v26.10.22/designs/mutation-court-full-receipt.md:57-58`;
  residual row 2; NIST MEASURE gap 3). Closes with a completed sweep
  receipt plus the `abi.rs:152:5` survivor kill-or-accept.
- **R-8 (BACKLOG)** — `receipt_v2` has no production emission site
  (ctors only; `duckdb_eval.rs` never calls attest — gap register G1);
  bulk-ledger fold head stays digest-level until it lands
  (`compliance-eu-ai-act-logging.md` G2,
  `design-dev-loop-self-hosting.md:146`).
- **R-9 (BACKLOG)** — 1,510 of 1,515 transition receipts in
  `receipts/` are unsigned telemetry (`UNSIGNED-NO-KEY`): per-event
  records are tamper-evident (CHAIN.blake3) but carry no cryptographic
  identity (`compliance-eu-ai-act-logging.md` G1). Historical rows not
  backfilled by design (`design-dev-loop-self-hosting.md:130`); closes
  with signed `receipt_v2` emission going forward.
- **R-10 (BACKLOG)** — trusted-key distribution is out-of-band;
  `graphlaw-verify --trusted-key` has no in-repo binding of keys to
  principals (`compliance-eu-ai-act-logging.md` G3).
- **R-11 (BACKLOG)** — inter-receipt chaining lives in-memory only
  (`src/receipt_chain.rs`); not serialized into `receipts/`
  (root CLAUDE.md Layout; NIST GOVERN gap 4; closure-plan G-6). Closes
  with a serialized chain + tamper-flip verify rung.
- **R-12 (BACKLOG)** — host-sandbox residuals, disclosed not hidden:
  `inherit_env()` full env to guest (`src/bin/graphlaw_run_eh.rs:249`,
  SECURITY.md:103-105); no store memory ceiling; native Rust API
  without resource caps (`SECURITY.md:52`). (Sterile-WasiCtx closure
  is R-12's env half; §3 row 4.)
- **R-13 (BACKLOG)** — EH-19 tc08 COPY egress EINVAL: OPEN on disk
  (typed-refusal closure planned at `ADR-egress-op.md:49,110`;
  ROADMAP.md:227) with a dispatch-vs-disk divergence already recorded
  (`compliance-eu-ai-act-robustness.md` gap 3). Flip is receipt-gated.
- **R-14 (BACKLOG)** — scalar-fallback COPY egress hole: hardcoded
  `/tmp/graphlaw-rdf-<table>.csv` destination, no allowlist/witness
  digest (`SECURITY.md:36-40`, `src/sql_ast.rs:2255-2258`); disclosed
  lawful hole; preferred form documented.
- **R-15 (BACKLOG)** — gl_call non-interruptible: no host timeout or
  cursor resumption (GAP-3, `docs/wasm-duckdb-op15-design.md:16`).
- **R-16 (BACKLOG)** — DuckDB memory/threads pragmas caller-trusted,
  not ABI-enforced (GAP-2, same doc:16); the wall is enforced, the
  pragmas are not.
- **R-17 (BACKLOG)** — court-gate defect: the eh-court skip path exits
  0 mid-harness (`tests/graphlaw_run_eh_court.rs:47-56`,
  `receipts/2026-10-23-terminal-witnesses.md` Witness 1) — a gate that
  cannot exit 1 on its own skip. Also: dedicated wasm memory-wall gate
  tier BLOCKED on the EH hosting problem per the GOVERN map gap 8;
  the EH-wall RESOLVED flip (residual row 1) plausibly unblocks it,
  but no post-flip memory-wall receipt is on disk — row carried, flip
  receipt-gated.

### Improvement backlog — testing / measurement

- **R-18 (BACKLOG)** — fuzz positive-control gap: the minimal clean AST
  does not conform to `ontologies/sql-ast.ttl`; the shapes gate is
  stricter than its own doc example (`receipts/gws-9-wasm-boundary-fuzz.md`
  finding 3).
- **R-19 (BACKLOG)** — coverage figure is a target count (106 `.rs`
  targets counted 2026-10-23), not a coverage measurement; no
  line/branch coverage receipt exists (NIST MEASURE gap 7).
- **R-20 (BACKLOG)** — egress-baseline machine-condition provenance:
  superseded 6.31× figure never identified machine/condition; current
  6.83× is single-machine single-run
  (`receipts/2026-10-23-egress-baseline.md`).
- **R-21 (BACKLOG)** — transcript-class witnesses on `472d5557…` lack
  durable `receipts/` files (fuzz, memory-wall, exit-codes ×3, dialect
  census, banking, pm4pytest; roadmap D4; Art. 9(3)(b) G-A9-4).
- **R-22 (BACKLOG)** — js publish flow never run end-to-end: no
  `npm pack`/dry-run receipt (G-A9-5); release-notes item 1 publish
  order (`graphlaw-eyeron` → `graphlaw`) remains an operator run — the
  0.7.9 version bump itself has landed (§3).
- **R-23 (BACKLOG)** — per-commit CI channel receipt absent; gates run
  per-wave (G-A9-9). Closes with a CI receipt or an explicit per-wave
  scope note.
- **R-24 (BACKLOG)** — `generated_kinds` environmental failure on hosts
  lacking the fixture path: close with portability or a loud typed SKIP
  (closure-plan G-5); the guarded surface is VERIFIED-EXTERNAL 21/21.

### Improvement backlog — docs / process

- **R-25 (BACKLOG)** — operator decision register carries unfilled
  decision lines (OD-1 and D1-D6 sub-lines,
  `docs/jira/v26.10.22/designs/OPERATOR-DECISIONS.md:53`).
- **R-26 (BACKLOG)** — no standalone risk-management plan document or
  witnessed regular-cadence re-adjudication line (G-A9-1, G-A9-2);
  partially answered by this battery itself. Closes with a
  process-description section in the next milestone charter.
- **R-27 (BACKLOG)** — no state-of-the-art evaluation note pinning
  frozen-duckdb / Ed25519 / wasmtime versions against vulnerability
  sources (G-A9-7); closes with the note or an explicit scope
  restriction.
- **R-28 (BACKLOG)** — NIST AI RMF profile doc
  (`docs/reference/ai-rmf-profile.md`) not yet authored
  (closure-plan G-1b; the EU-Act half of G-1 closed with this battery).
- **R-29 (BACKLOG)** — SPC gate thresholds env-overridable
  (`SPC_CPK_MIN`/`SPC_USL_NS`, `scripts/spc_court.sh:22`): explicit
  tolerance, not immutable.
- **R-30 (BACKLOG)** — refusal-register classifications:
  `UnsupportedTerm` for unknown function IRIs carries no
  `[REFUSAL-nn]` marker (`docs/refusals.md:76`, pinned known gap).
- **R-31 (BACKLOG)** — whitepaper chain-entry count not inline and its
  adversarial-audit lineage discoverable only via the milestone window
  (`docs/whitepaper-v26.10.22.md:169-175`; transparency gap 3).
- **R-32 (BACKLOG)** — `docs/index.md` lowercase vs `INDEX.md`
  convention breaks case-sensitive external links; no redirects
  (transparency gap 2).
- **R-33 (BACKLOG)** — doc_staleness 1,143 pre-existing R5 anchor
  drift, disclosed and delta-checked per wave, not remediated
  (ROADMAP.md:22; transparency gap 1).

### Recorded N/A determinations

- **R-34 (N/A)** — Art. 9, Art. 10, Art. 14 obligations for graphlaw
  as-shipped: N/A per §1 classification; determinations recorded at
  `compliance-gap-closure-plan.md` G-7/G-8/§4. The mapped surfaces
  (typed refusals, receipts, gates) exceed the floors anyway.
- **R-35 (N/A)** — Art. 9(3)(d) physical-hazard class: N/A — pure
  software, no actuation surface (`compliance-eu-ai-act.md` §6).

Register count: 35 rows — 2 GATE (R-1, R-2), 31 BACKLOG, 2 N/A. No row
is claimed closed without a receipt; §3's two closed items (eyeron
bump, ATTACH-bypass class) are the only closures witnessed this pass.

---

## 5. Release-readiness statement — v26.10.22

**Validated (receipted, mechanism-level):**

1. Admission law and its identity: deterministic SHACL-gated admission
   with typed, stable-coded, append-only refusals — every code
   triggered for real, probe 12/12 exit-class MATCH
   (`docs/refusals.md:7-17`); raw-SQL and unbounded-mutation paths
   refused before engine contact (`src/duckdb_op.rs:359-369,467-485`).
2. Receipt/log surface: per-event execution receipts bound to the law
   by content hash `BLAKE3(Canon(G_AST) ‖ P_params)`, admission seal
   verified pre-execution with mutation-flipping falsifiers, three
   tamper layers (seal / Ed25519 / blake3 chain), offline third-party
   replay via `graphlaw-verify`; courts: `chicago_validation`,
   `sql_rdf_dialect`, `duckdb_statement_kinds`, `trust_attest`,
   `receipt_chain_golden_test`.
3. Boundary hardening (witnessed on the shipped artifact class): 28-all-
   WASI import allowlist fail-closed + zero-preopen containment
   (`src/bin/graphlaw_run_eh.rs:222-244`), boundary fuzz 312 hostile
   calls × 2 deterministic runs / 0 panics / 312-312 survival canary,
   memory-wall plateau 8/0, hostile SQLRDF laundering corpus 23/0,
   in-wasm duckdb IPC injection court with typed never-trap responses;
   ATTACH allowlist fail-closed with comment-camouflage immunity, court
   13/0 (W15, `receipts/2026-10-23-eh-closure-wave.md:59`); atomic
   batch envelope court `document_exec_atomic_wire_flag_aborted_partial_
   and_success_fields` (`src/duckdb_op.rs:1465`).
4. Determinism/accuracy: hash-stability 10k-iteration leg + 1k real
   compile leg (`p28_determinism_court.rs`), two-verifier byte-agreement
   (`two_verifier_court.rs`), canonical-id isomorphism stability with an
   anti-vacuity perturbation guard (`canonical_cross_dialect.rs:287`).
5. Supply-chain integrity mechanics: pinned final artifact `472d5557…`
   (58,121,500 B) at `priv/` + registry + npm 198/198; per-file blake3
   receipt ledger `receipts/CHAIN.blake3` with `--check` gate;
   `graphlaw-eyeron` bumped to 0.7.9 on the tree (Cargo.toml:68).
6. Honest-failure discipline: SPC FAIL, mutation-sweep gap, fuzz
   positive-control gap, and every residual row are disclosed in
   receipted registers — the release does not ship with silent red.

**Honestly open at tag time:**

1. **GATE R-1**: the two owed rungs on the canonical pin
   (`cargo test -p graphlaw-wasm` + one anti-vacuity inversion) and the
   release-notes artifact-identity truth-sync. Until run, the release's
   host-contract claims on the final artifact rest on the pre-wrap
   rungs plus the SQLRDF-27 court set.
2. **GATE R-2**: cargo-deny residual advisories without a recorded
   operator decision (`RELEASE-NOTES-26.10.22.md:98-100`).
3. **R-5**: SPC Cpk FAIL standing (0.0008 vs 1.33) — the tolerance
   mechanism is real and refusing, but the performance capability claim
   under load is unmet until the solo rerun lands.
4. **R-7**: mutation-court full sweep has no verdict (0 mutants tested
   at last attempt); per-court inversions are the local anti-vacuity
   evidence, not a suite-level guarantee.
5. **R-3/R-4/R-12**: frame bound, fuel/epoch metering, sterile env /
   store ceiling — disclosed sandbox limits with designs or closure
   conditions on file, none landed. Any downstream high-risk embedder
   inherits these as their own controls to add.
6. Backlog R-6..R-33 as registered in §4 — none blocks the tag; all are
   receipt-gated flips.

**Statement**: v26.10.22 is release-ready as a deterministic
law-admission kernel with a receipted, court-enforced conformance
surface, **conditional on discharging the two GATE rows (R-1, R-2)**.
Everything else is registered, evidenced, and non-blocking. The
classification (§1) means no EU AI Act or NIST obligation blocks the
tag; what gates it is the release's own DoD — the release must not
describe an artifact it does not ship, or a dependency posture nobody
has decided on.

---

## 6. Verification commands (targets cited by this verdict)

```bash
cargo test --test chicago_validation
cargo test --test sql_rdf_dialect
cargo test --test duckdb_statement_kinds
cargo test --test trust_attest
cargo test --test receipt_chain_golden_test
bash scripts/receipt_chain_sync.sh --check
```

This verdict is a consolidation lane output: read-only over code, no
cargo or git executed; every pointer above was read from disk
2026-10-24 against HEAD `7e92175` + working tree. Owed follow-up:
register this file in `docs/jira/v26.10.23/INDEX.md` (INDEX is outside
this lane's lease).

## See Also

- `docs/jira/v26.10.23/compliance-eu-ai-act.md`
- `docs/jira/v26.10.23/compliance-eu-ai-act-logging.md`
- `docs/jira/v26.10.23/compliance-eu-ai-act-robustness.md`
- `docs/jira/v26.10.23/compliance-nist-gov-map.md`
- `docs/jira/v26.10.23/compliance-nist-measure-manage.md`
- `docs/jira/v26.10.23/compliance-transparency.md`
- `docs/jira/v26.10.23/compliance-gap-closure-plan.md`
- `docs/refusals.md` · `docs/reference/wasm-gap-register.md` ·
  `SECURITY.md`

---
## R-2 DISCHARGE — cargo-deny operator decision (dated 2026-10-24, coordinator)

Per-advisory dispositions, all dated-ignored in deny.toml with expiry
2026-11-23 (re-review required on expiry):
- RUSTSEC-2026-0258 (h2 0.3.27, unbounded empty DATA frames): ACCEPTED
  RISK — transitive via frozen-duckdb → reqwest ^0.11 (host-side tooling
  only; not on the in-WASM execution/receipt-signing path); h2 0.4 is
  semver-incompatible with the frozen reqwest pin; upstream bump tracked.
- RUSTSEC-2025-0134 (rustls-pemfile, unmaintained): ACCEPTED — host-side
  TLS config plumbing only.
- RUSTSEC-2024-0449 (unsafe-libyaml, unmaintained): ACCEPTED — serde_yaml
  transitive, config parsing only.
`cargo deny check advisories` exit 0 under these ignores; `cargo audit`
exit 1 is the expected divergence (audit does not read deny.toml).
Decision authority: operator directive 2026-10-24 (publish order).
