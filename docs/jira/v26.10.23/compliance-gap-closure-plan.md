# Compliance Gap Closure Plan — EU AI Act + NIST AI RMF → graphlaw v26.10.22

Consolidated from the four expected compliance matrices, which **do not exist
on disk** (`docs/jira/v26.10.23/compliance-*.md`: none found; repo-wide grep
for "AI Act" / "NIST AI RMF": zero hits). Gaps are therefore derived directly
from the release subject's honest-boundary records:

- `docs/jira/v26.10.22/RELEASE-NOTES-26.10.22.md` (pre-publish checklist items 2, 3, 5; "What is NOT in this release")
- `docs/jira/v26.10.22/designs/residual-unknowns.md` (rows 1-3, terminal update 2026-10-24)
- `docs/refusals.md`, `docs/archive/receipt-2026-10-11-publish-dod.md`

Status as-of 2026-10-23. This lane is read-only except this file.

---

## 0. Subject classification (the argument that sizes everything else)

**Claim**: graphlaw v26.10.22 is not an AI system within Reg. (EU) 2024/1689
Art. 3(1) and is not high-risk under Art. 6.

**Grounds** (each checkable against the code):

1. No inference/learning: admission is deterministic RDF/SHACL validation
   plus a SQL-in-RDF compiler over a closed 35-statement-class space
   (`src/sql_ast.rs`), typed refusals (`docs/refusals.md`, REFUSAL-01..04,
   append-only), Ed25519-signed receipts (`src/attest.rs`).
2. No autonomy or adaptivity: the same input graph admits or refuses
   identically every run — an executable deterministic law, not a trained or
   inference-bearing system. NIST AI RMF's own scope ("machine-based system
   that... can, for a given set of objectives, generate outputs... with
   varying levels of autonomy") excludes a fixed rule engine.
3. At most, graphlaw is an incidental component a downstream AI system may
   embed (evidence provenance). The Act's obligations attach to the
   provider/deployer of the AI system; the risk-based approach (Art. 6,
   Recitals 12-14) carries Art. 9-15 obligations only for high-risk AI
   systems, and those land on the AI-system provider — who would consume
   graphlaw's supply-chain artifacts (attest receipts, pinned digests, typed
   refusal register) as inputs to their compliance file, not as gaps in
   graphlaw.

**Consequence**: under the Act's proportionality principle, Art. 9-15
obligations (risk management, data governance, logging, transparency-to-
deployer, human oversight, accuracy/robustness) are **N/A** for graphlaw.
What survives and can genuinely block a publish: (a) honesty of the release's
own claims, (b) provenance/supply-chain integrity of what ships, (c) not
shipping with known-false documentation. Those three form the pre-tag
blocker set.

---

## 1. Gap register

Every row verified against the repo on disk, 2026-10-23 (this lane,
read-only).

### G-1 — No EU AI Act / NIST mapping document exists

- **Gap**: no document maps graphlaw controls to the Act or to NIST AI RMF 1.0
  families (ME/MP/MG/MS/GO). Verified: repo-wide grep, zero hits.
- **Closure action**: this file + an INDEX row; later, per-family profile
  docs if graphlaw is ever embedded in a high-risk AI system.
- **Owner class**: post-release follow-up (docs lane).
- **Closure evidence**: this file committed in the v26.10.23 window + INDEX
  row naming it.
- **Blocking?** No. An internal mapping doc is not an obligation the Act
  imposes on a non-high-risk, non-AI-system library. It is a transparency
  improvement for downstream AI-system providers.

### G-1b — No NIST AI RMF profile

- **Gap**: no AI RMF profile mapping graphlaw controls (attest receipts,
  refusal register, mutation/SPC courts, registry digests) to ME-2/ME-3,
  MS-2/MS-5, GO-1/GO-2.
- **Closure action**: author `docs/reference/ai-rmf-profile.md`.
- **Owner class**: post-release follow-up.
- **Closure evidence**: profile doc + INDEX row.
- **Blocking?** No — same reasoning as G-1.

### G-2 — wasm artifact/source correspondence

- **Gap (raw, 2026-10-22)**: `RELEASE-NOTES-26.10.22.md` checklist item 3:
  wasm32-wasip1 link RED lane-isolated (`frozen-duckdb-sys/build.rs:101`,
  WASI_SDK_PATH required). Publishing red would leave the pinned artifact
  predating the released source, making the release's attestation claims
  describe an artifact that does not correspond to the shipped source.
- **Update (verified on disk)**: `designs/residual-unknowns.md` row 1 records
  the EH wall RESOLVED (2026-10-23/24 terminal wave): uniform new-EH rebuild +
  wasmtime 48 host, fcntl-ABI translation shim, canonical pin
  `472d5557…bf80ea` at `d9cab2f`; `4d9083af…`/`332ea218…` marked
  historical-superseded. **Explicitly owed and NOT closed by that flip**:
  (a) host-contract rung `cargo test -p graphlaw-wasm` on the wrap-enabled
  artifact, (b) anti-vacuity inversion on the same artifact.
- **Closure action**: run the two owed rungs on the canonical-pin artifact;
  re-pin and re-run if anything is red; then the release-notes checklist
  item 3 must be rewritten to cite the new pin (the notes still cite
  `332ea218…` as current — that text is now stale against RECEIPTS.md).
- **Owner class**: pre-tag gating (engineer lane).
- **Closure evidence**: embedded real output of `cargo test -p
  graphlaw-wasm` (exit 0, counts) in RECEIPTS.md against pin `472d5557…`, +
  one anti-vacuity inversion receipt on the same artifact + a release-notes
  truth-sync edit.
- **Blocking?** **Yes.** This is the one genuine Act-adjacent blocker: a
  publish whose attestation/pin claims describe a superseded artifact is a
  false documentation claim about what is shipped. Under the risk-based
  approach this is not an Art. 9-15 obligation, but it *is* the release
  making an untrue statement about itself, and the repo's own DoD makes it a
  publish precondition. Cheap to close (two test rungs + a doc edit).

### G-3 — cargo-deny residual advisories without a recorded operator decision

- **Gap**: release-notes checklist item 2: deny.toml residual advisories
  need an explicit operator accept/diagnose decision recorded before
  publish; a silent pass is not a decision.
- **Closure action**: operator records the accept/diagnose decision line
  (in the release-notes checklist or RECEIPTS.md), one row per residual
  advisory.
- **Owner class**: pre-tag gating (operator decision, not engineering).
- **Closure evidence**: recorded decision with date + advisory IDs.
- **Blocking?** **Yes**, as a process blocker: it is a recorded precondition
  of this release's own DoD, and shipping a dependency graph with unknown,
  undecided advisories is a supply-chain integrity gap (NIST AI RMF GO/MS
  family; Act Art. 15 cybersecurity, by analogy for any downstream high-risk
  embedder). Trivially cheap to close.

### G-4 — Anti-vacuity sweep UNMET / mutation-court full sweep incomplete

- **Gap**: release notes "What is NOT in this release": full anti-vacuity
  sweep UNMET; residual-unknowns row 2: 88-mutant sweep DISK-BLOCKED
  mid-run; gate-relevant survivor named at `src/abi.rs:152:5`; row 2 notes a
  post-settle re-attempt IN PROGRESS at last writing.
- **Closure action**: complete the sweep to a verdict; kill or
  operator-accept the `abi.rs:152:5` survivor.
- **Owner class**: post-release follow-up (v26.10.23 wave already owns this
  per ROADMAP).
- **Closure evidence**: completed sweep receipt with kill counts, runtime,
  and the survivor's disposition.
- **Blocking?** No. Per-court inversions remain as local evidence; a missing
  full-sweep verdict weakens test-suite confidence but falsifies no receipt
  and imposes no Act obligation on a non-high-risk system. Improvement item.

### G-5 — `generated_kinds` environmental test failure

- **Gap**: release notes: the `duckdb_statement_kinds::generated_kinds` test
  fails on any host lacking `/Users/sac/frozen-duckdb/target/wasm-build/` —
  a typed environmental failure, not a silent pass; parse-parity surface is
  VERIFIED-EXTERNAL 21/21.
- **Closure action**: make fixture-path resolution portable or emit a loud
  typed SKIP with reason on hosts lacking the fixture.
- **Owner class**: post-release follow-up.
- **Closure evidence**: green run on a fixture-less host, or a loud typed
  skip with reason embedded in output.
- **Blocking?** No. The failure is loud and typed, the guarded surface is
  externally verified; the risk-based approach does not gate a publish on
  host-convenience polish.

### G-6 — Inter-receipt chaining not serialized into `receipts/`

- **Gap**: per root `CLAUDE.md` layout note: receipt chain rule lives in
  `src/receipt_chain.rs` for in-memory OCEL frames; not yet serialized into
  `receipts/`.
- **Closure action**: serialize the chain (per-file blake3 + parent link)
  into `receipts/`; add a verify rung.
- **Owner class**: post-release follow-up.
- **Closure evidence**: chained receipts directory + a verify command whose
  tamper mutation flips it.
- **Blocking?** No. Individual receipts already carry blake3 digests;
  chaining is a traceability strengthening (NIST AI RMF ME/MS), valuable for
  any downstream high-risk embedder, not an Act obligation here.

### G-7 — Human oversight (Act Art. 14) — N/A, recorded

- **Determination**: Art. 14 applies to high-risk AI systems. graphlaw's
  typed-refusal surface (REFUSAL-01..04, `docs/refusals.md`) already gives
  human operators machine-checkable refuse reasons — the oversight-friendly
  property — but no obligation attaches. Recorded as N/A, not a gap.

### G-8 — Risk management system (Act Art. 9) — N/A, recorded

- **Determination**: Art. 9 binds providers of high-risk AI systems.
  graphlaw is not one (§0). The repo's own court/SPC/mutation machinery
  exceeds any risk-management floor the Act would impose on such a
  component. Recorded as N/A, not a gap.

---

## 2. PRE-TAG BLOCKER list

Exactly two, both cheap, both dischargeable before tagging:

1. **G-2 — wasm artifact/source correspondence.** Run `cargo test -p
   graphlaw-wasm` + one anti-vacuity inversion against the canonical pin
   `472d5557…bf80ea`, embed real output in RECEIPTS.md, and truth-sync the
   release notes (they still cite `332ea218…` as current). Rationale: the
   release must not describe a shipped artifact different from the pinned
   one — false self-description is the only gap here that touches the Act's
   honesty/transparency register, and the repo's own DoD gates on it.
2. **G-3 — cargo-deny residual advisory decision record.** Operator
   accept/diagnose line per residual advisory, dated, before tag. Rationale:
   recorded precondition of the release's own DoD; supply-chain integrity.

Expected-empty held: nothing else rises to blocker. G-4, G-5, G-6 are
improvement items; G-1/G-1b are documentation improvements; G-7/G-8 are
recorded N/A determinations, not gaps.

## 3. Improvement backlog (post-release)

| Id | Item | Class | Closes with |
|---|---|---|---|
| G-1 | EU AI Act mapping doc | docs | this file + INDEX row (done on write) |
| G-1b | NIST AI RMF profile (`docs/reference/ai-rmf-profile.md`) | docs | profile + INDEX row |
| G-4 | Mutation-court full sweep verdict + `abi.rs:152:5` disposition | test infra | completed sweep receipt |
| G-5 | `generated_kinds` fixture portability / loud typed skip | test infra | green-or-typed-skip output on fixture-less host |
| G-6 | Serialize inter-receipt chain into `receipts/` | kernel | chained receipts + tamper-flip verify rung |

## 4. N/A determinations (recorded, not gaps)

- Art. 9 risk management — N/A (§0).
- Art. 14 human oversight — N/A (§0); typed refusals already provide the
  substance.
- Art. 10 data governance, Art. 11 record-keeping/logging, Art. 12
  transparency-to-deployer, Art. 13 instructions — N/A for non-high-risk;
  graphlaw's receipt/refusal/registry surfaces exceed them anyway.


---
**SOURCING CORRECTION (2026-10-24):** the four compliance matrices cited as
"nonexistent" above landed AFTER this plan's grep ran — all seven now exist
in this directory (compliance-eu-ai-act{,-logging,-robustness}.md,
compliance-nist-gov-map.md, -measure-manage.md, compliance-transparency.md,
11-15 KB each, verified on disk). Their consolidated gap sets supersede the
derived-from-boundary-records gaps where they overlap. The 2-blocker /
backlog split stands. One additional pre-tag blocker from the publish
preflight: `graphlaw-eyeron 0.7.8` is already on crates.io — bump to 0.7.9
before publish.
