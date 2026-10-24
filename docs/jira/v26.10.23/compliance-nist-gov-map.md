# NIST AI RMF Map — GOVERN + MAP Functions

Repo: `/Users/sac/graphlaw` · Date: 2026-10-23 · Lane: no-code compliance
(owns this file only; read-only elsewhere; no cargo/git). Scope: map existing
graphlaw governance artifacts to NIST AI RMF 1.0 GOVERN and MAP functions.
Every claim cites a file:line or a receipt with embedded output. Gaps are
honest, not aspirational.

---

## GOVERN 1 — Accountability

| NIST subcategory | graphlaw instantiation | Evidence (file:line / receipt) |
|---|---|---|
| GOVERN 1.1 (roles/responsibilities) | Operator decision register: each entry carries question, evidence, options with costs, coordinator recommendation, and an explicit fill-in operator decision line — nothing is decided by the authoring lane. | `docs/jira/v26.10.22/designs/OPERATOR-DECISIONS.md:4-6` ("DECISION REGISTER — nothing decided by this lane"), fill-in line at `:53` |
| GOVERN 1.1 | Lane file-ownership (lithography): every dispatch carries the absolute repo path + disjoint file lease; coordinator owns git. | `/Users/sac/graphlaw/CLAUDE.md` (Lanes convention); `docs/jira/v26.10.22/_LANES.md` |
| GOVERN 1.4 (transparency of decisions) | Receipt chain: per-file BLAKE3 digests over the receipt corpus, append-only. | `receipts/CHAIN.blake3:1-14`; `docs/whitepaper-v26.10.22.md:169-175` (§7: "Every performance figure and invariant cited in this specification maps to an entry in the `CHAIN.blake3` ledger") |
| GOVERN 1.4 | Status flips are receipt-gated and documented, not silent; rebaselines are written up as their own receipts. | `receipts/2026-10-23-chain-rebaseline-note.md` (chain-covered, CHAIN.blake3 line 2); whitepaper §7 "re-baselines documented, not silent" |
| GOVERN 1.6 (human oversight) | Observation is separated from verdict: "Pending is pending — nothing here is claimed ALIVE" — an unverified state is never promoted by prose. | `receipts/2026-10-23-terminal-witnesses.md:3-7` (embedded real output, honest pending state) |

## GOVERN 2 — Policy, Process, Procedures

| NIST subcategory | graphlaw instantiation | Evidence |
|---|---|---|
| GOVERN 2.1 | Executable policy conventions: Chicago classicist testing (real collaborators, assert on real state), no-sleep, refusal codes append-only, "a gate that cannot exit 1 is the failure". | `/Users/sac/graphlaw/CLAUDE.md` (Commands/Test-ladder/Conventions) |
| GOVERN 2.1 | Qualification algebra: qualification is an executable falsification relation Q=(S,C,A,O,P,E,H); a missing attempt is UNKNOWN, never PASS. | `docs/QUALIFICATION_ALGEBRA.md:7-25` (core object :7-15; standing lattice `UNQUALIFIED < OBSERVED < ATTACKED < QUALIFIED` :20-24) |
| GOVERN 2.3 | Anti-vacuity doctrine: a court must flip under mutation or it is FAIL; mutation court, SPC court, shape mutation court. | `/Users/sac/graphlaw/CLAUDE.md` (Chicago convention); `scripts/mutation_court.sh`, `scripts/spc_court.sh`, `scripts/shape_mutation_court.sh` |
| GOVERN 2.3 | Refusal register is append-only, never renumbered; each code has a stable trigger + recovery. | `docs/refusals.md:59-80` (REFUSAL-01..05 table; REFUSAL-05 appended 2026-10-23, "append-only per convention — never renumbered", at :80) |
| GOVERN 2.9/2.10 (incident process) | In-flight operational failures are filed as durable, chain-covered incident receipts. | `receipts/2026-10-23-staging-wipe-incident.md` (CHAIN.blake3 line 10) |
| GOVERN 2.11 (escalation) | Compile-freeze SLA + lane-unblock disclosure rules define escalation for shared-tree contention. | `/Users/sac/graphlaw/CLAUDE.md` (Lanes convention, Compile-freeze SLA paragraph) |

## GOVERN 3 — Risk tolerance

| NIST subcategory | graphlaw instantiation | Evidence |
|---|---|---|
| GOVERN 3.2 (explicit thresholds) | SPC gate with numeric thresholds: USL=250ns, CPK_MIN=1.33, exit 1 on breach. | `scripts/spc_court.sh:16,22,32` (USL 250ns; default CPK_MIN 1.33 at :32; "exits 1 when Cpk < 1.33" at :18); witness `receipts/2026-10-23-spc-witness.md:1-25` (exit **1**, Cpk 0.0008 < 1.33, verdict FAIL honestly receipted) |
| GOVERN 3.2 | Memory wall: 64 MiB `WALL_BYTES` linear-memory ceiling; DuckDB capped at 32 MiB inside it. | `docs/WASM.md:83,112`; `docs/wasm-duckdb-integration.md:39` |
| GOVERN 3.2 | Uniform 16 MiB outstanding-alloc cap across host/wasm with compile-time equality assertion — drift is refused at compile. | `src/abi.rs:73`, `wasm/src/abi_meta.rs:29`, `wasm/src/lib.rs:32-33` (via `docs/wasm-duckdb-op15-design.md:16`) |
| GOVERN 3.2 | Negative acceptance enforced: "a loud SKIP is not a pass"; typed BUILD_BROKEN/UNKNOWN/BLOCKED standing vocabulary; REFUSED and UNKNOWN are not weaker PASS states. | `/Users/sac/graphlaw/CLAUDE.md` (Lithography); `docs/QUALIFICATION_ALGEBRA.md:20-25` |
| GOVERN 3.3 (deployment risk) | Scope boundaries stated up front: not a distributed lakehouse, not zero-parse, parametric latency with a disclosed 23.13 ms first-contact admission fee. | `docs/whitepaper-v26.10.22.md:157-167` (§6) |
| GOVERN 3.4 (honest FAIL) | The SPC gate currently FAILs and the FAIL is receipted, not suppressed — risk tolerance is enforced in the direction of disclosing bad news. | `receipts/2026-10-23-spc-witness.md` (exit 1, Cpk 0.0008); `docs/whitepaper-v26.10.22.md:85` |

## GOVERN 4 — Transparency

| NIST subcategory | graphlaw instantiation | Evidence |
|---|---|---|
| GOVERN 4.1 (honest communication) | Append-only RECEIPTS.md ledger with honest-FAIL entries embedded. | `docs/jira/v26.10.22/RECEIPTS.md:36` (anti-vacuity full sweep UNMET — aborts pre-mutation, cargo-mutants exit 4, zero mutants tested; disclosed, not hidden) |
| GOVERN 4.1 | Receipt validity law: "A receipt with no embedded output is not a receipt." | `/Users/sac/graphlaw/CLAUDE.md` (Receipts convention) |
| GOVERN 4.2 (content transparency) | Chain integrity: per-file BLAKE3 digests make receipt tampering detectable. | `receipts/CHAIN.blake3:1-14`; whitepaper §7 `docs/whitepaper-v26.10.22.md:169-175` |
| GOVERN 4.3 | Generated projections are regenerated, never patched — provenance of reference docs is structural. | `/Users/sac/graphlaw/CLAUDE.md` (Generated artifacts table) |
| GOVERN 4.4 | Conformance-text honesty: W3C status claims in the sql-rdf Recommendation are labeled document text, not facts asserted by the repo. | `/Users/sac/graphlaw/CLAUDE.md` (Conventions, "Conformance text" bullet) |

## GOVERN 5 — Organizational policies into the AI-system lifecycle

| NIST subcategory | graphlaw instantiation | Evidence |
|---|---|---|
| GOVERN 5.1 | Policy-to-mechanism pairing: "A law with no mechanism is prose; a mechanism with no law is archaeology" — every doctrine has a court/script. | `~/.claude/rules/entropic-annealing-doctrine.md` §5; enforcement: `scripts/mutation_court.sh`, `scripts/spc_court.sh` |
| GOVERN 5.1 | Wave/receipt closure: milestone windows close only with terminal witnesses. | `receipts/2026-10-23-v26.10.22-terminal.md` + `receipts/2026-10-23-wave-close.md` (both chain-covered) |
| GOVERN 5.2 | Adversarial lanes grade, never build what they audit (skeptic-pairing). | `~/.claude/rules/entropic-annealing-doctrine.md` §3/§4 (skeptic-pairing row) |
| GOVERN 5.1 | Refusal codes append-only, never renumbered, across the lifecycle. | `docs/refusals.md:59-80` |

## MAP 1/2/3/4/5 — System definition, context, intended use

| NIST subcategory | graphlaw instantiation | Evidence |
|---|---|---|
| MAP 1.1 (system definition) | Whitepaper §2: pipeline (JSON envelope → Turtle parse → SHACL admission fail-closed → BLAKE3 query identity in-module → SQL codegen → frozen-duckdb → ExecutionReceipt → CHAIN.blake3) plus 4 named core invariants. | `docs/whitepaper-v26.10.22.md:41-66` (§2 + "Core System Invariants" 1-4) |
| MAP 1.1 | Qualification algebra pins exact-subject identity (S in Q) to repo/ref/SHA + load-bearing collaborator identities, so "the system" is per-court precise. | `docs/QUALIFICATION_ALGEBRA.md:7-15` |
| MAP 2.1/2.2 (context) | Banking-domain qualification courts: BCBS 239 risk aggregation (Petri-net conformance fitness 1.0 / precision 1.0), rehypothecation limits, UBO sanctions unrolling, DvP settlement. | `docs/whitepaper-v26.10.22.md:89-102` (§3 Banking Domain Verification); DvP witness `receipts/2026-10-23-banking-4of4-witness.md` (embedded output: `test result: ok. 4 passed; 0 failed` from `cargo test --test dvp_settlement_court`) |
| MAP 3.1/3.2 (intended use vs out-of-scope) | §6 scope boundaries: sovereign fixed-cost in-process substrate, NOT a distributed lakehouse; NOT zero-parse; parametric latency disclosed. | `docs/whitepaper-v26.10.22.md:157-167` (§6) |
| MAP 3.3 (benefits vs cost) | Performance envelope witnessed (Divan, 100 samples/leg) with the honest note that the TC-HOT-01 perf budget is genuinely unmet (SPC FAIL). | whitepaper §3 (`docs/whitepaper-v26.10.22.md:104-115`); `receipts/2026-10-23-spc-witness.md` |
| MAP 4.1 (risk beyond intended use) | Egress baseline + workload amortization receipts bound behavior beyond the qualification battery. | `receipts/2026-10-23-egress-baseline.md`, `receipts/2026-10-23-workload-amortization.md` (chain-covered) |
| MAP 4.x | Typed refusal vocabulary (REFUSAL-01..05) with per-code trigger + recovery, enforced before engine contact. | `docs/refusals.md:59-80`; whitepaper §2 invariant 2 |
| MAP 5.1 | Anti-vacuity lifecycle: "every qualification court is paired with an inversion twin to ensure that any relaxation of security, schema, or memory bounds forces an immediate exit code 1." | `docs/whitepaper-v26.10.22.md:169-175` (§7) |

---

## Honest Gaps

1. **SPC gate is currently FAIL, not PASS.** Latest witness: Cpk 0.0008 < 1.33
   (`receipts/2026-10-23-spc-witness.md`; corroborated at
   `docs/whitepaper-v26.10.22.md:85`: "latest witness FAIL Cpk 0.0008 under
   fleet load — solo-machine rerun owed; prior PASS 1.7573"). Disclosed by the
   whitepaper; no closed remediation receipt exists yet.
2. **Mutation-court full sweep is UNMET** at last full attempt: aborts
   pre-mutation at `tests/test_cold_chain_ocel.rs:61`, cargo-mutants exit 4,
   zero mutants tested (`docs/jira/v26.10.22/RECEIPTS.md:36`). Per-court
   inversions are the local anti-vacuity evidence; the sweep-level guarantee is
   not manufactured.
3. **Operator decision register has unfilled decision lines.**
   `docs/jira/v26.10.22/designs/OPERATOR-DECISIONS.md:53` (OD-1 wasm EH route)
   and OD-2 sub-decisions D1–D6 carry blank `DECISION (operator):` lines — the
   accountability surface exists but several operator sign-offs are outstanding.
4. **Receipt chain is per-file only.** `receipts/CHAIN.blake3` carries per-file
   digests; inter-receipt chaining lives only in-memory (`src/receipt_chain.rs`)
   and is not yet serialized into `receipts/` (stated in
   `/Users/sac/graphlaw/CLAUDE.md`, Layout, receipts bullet). Tamper-evidence is
   per-file, not transitive, until the chain serializes.
5. **DuckDB memory/threads pragmas are caller-trusted, not ABI-enforced**
   (GAP-2, `docs/wasm-duckdb-op15-design.md:16`). The wall is enforced; the
   pragmas are not.
6. **`gl_call` is non-interruptible** (GAP-3, `docs/wasm-duckdb-op15-design.md:16`)
   — no host timeout or cursor resumption on long scans; wall-clock risk
   tolerance is not mechanically backed on this path.
7. **SPC gate thresholds are env-overridable** (`SPC_CPK_MIN`/`SPC_USL_NS`,
   `scripts/spc_court.sh:22`) — the risk tolerance is explicit but not
   immutable; a weaker gate can be configured without a code change.
8. **Wasm memory-wall court is BLOCKED on the EH wall.** The dedicated
   memory-wall gate is typed BLOCKED on the EH hosting problem (third
   independent corroboration, `docs/jira/v26.10.22/RECEIPTS.md:710`), so the
   64 MiB bound's hosted-module court tier is not currently green.
9. **Unclassified refusal codes exist.** Unrecognized *function* IRIs use
   `UnsupportedTerm` with no `[REFUSAL-nn]` marker — "pinned as a known gap"
   at `docs/refusals.md:76`.
10. **Whitepaper §7 chain-entry count is not inline** — "entry count is
    chain-extension-live and cited at the pinned release commit, not inline"
    (`docs/whitepaper-v26.10.22.md:169-175`); ledger size is not verifiable
    from the whitepaper alone.

**Gap count: 10.**

---

## Section inventory

Sections: GOVERN 1 (Accountability), GOVERN 2 (Policy), GOVERN 3 (Risk
tolerance), GOVERN 4 (Transparency), GOVERN 5 (Policy→lifecycle),
MAP 1/2/3/4/5, Honest Gaps (10). Receipts cited:
`receipts/2026-10-23-spc-witness.md`,
`receipts/2026-10-23-banking-4of4-witness.md`,
`receipts/2026-10-23-terminal-witnesses.md`,
`receipts/2026-10-23-chain-rebaseline-note.md`,
`receipts/2026-10-23-staging-wipe-incident.md`,
`receipts/2026-10-23-wave-close.md`,
`receipts/2026-10-23-v26.10.22-terminal.md`,
`receipts/2026-10-23-egress-baseline.md`,
`receipts/2026-10-23-workload-amortization.md`,
`docs/jira/v26.10.22/RECEIPTS.md:36,710`, `receipts/CHAIN.blake3`.

See Also: `docs/QUALIFICATION_ALGEBRA.md`, `docs/refusals.md`,
`docs/whitepaper-v26.10.22.md`, `docs/jira/v26.10.22/RECEIPTS.md`
