# Risk Management Plan — graphlaw (kernel + release train)

Standalone risk-management plan for the graphlaw kernel and its release
train. Closes compliance gap **G-A9-1** (no standalone risk-management plan;
`compliance-eu-ai-act.md` §2) and files the Art. 9(6)-class keep-vs-abandon
determinations owed by **G-A9-8**. Closes the structural half of **R-26**
(consolidated gap register, `compliance-final-verdict.md` §4).

**Subject**: `/Users/sac/graphlaw` v26.10.22 line, v26.10.23 milestone
window. Authored 2026-10-24 by a documentation lane; every pointer below was
read from disk this pass, and the load-bearing status markers were
re-verified by the commands in §9 (hardening markers, mutation-sweep state,
court-file presence).

**Nature**: a process document over already-landed mechanisms. It is not a
certificate, attestation, or declaration of conformity. The Art. 9 / Art. 10
/ Art. 14 N/A determination for graphlaw as-shipped stands
(`compliance-gap-closure-plan.md` G-7/G-8; consolidated register R-34); this
plan is provider-side due-diligence evidence for the embedding scenario in
which a downstream high-risk AI system uses graphlaw as its admission/egress
gate. It asserts only that cited files exist and say what they say, and that
named gates exist and refuse as described per their own receipts.

## 1. Scope

**In scope — the kernel**: the deterministic law-admission kernel and its
shippable surfaces:

- admission law `src/law.rs`; SQL-in-RDF compiler `src/sql_ast.rs` +
  `src/sql_ast/` (closed statement-class space); frozen-duckdb evaluator
  `src/duckdb_eval.rs` and op substrate `src/duckdb_op.rs`;
  Ed25519 attestation `src/attest.rs`; op ABI `src/abi.rs`;
  admission tiers `src/tier.rs`; in-memory receipt chain
  `src/receipt_chain.rs`.
- the wasm unit `wasm/` (`graphlaw-wasm`), one artifact class linking the
  kernel including frozen-duckdb, hosted by `graphlaw-run` (wasmi) and
  `graphlaw-run-eh` (wasmtime 48).
- the canonical graphs and shapes (`ontologies/`, `ggen/` dialect packs),
  generated projections excluded (regenerated, never patched).

**In scope — the release train**: the machinery that turns the tree into
published, verifiable artifacts:

- per-window milestone directories (`docs/jira/v26.10.NN/`), ticket ledgers,
  wave receipts, INDEX registers.
- the durable receipt ledger `receipts/` (per-file blake3 digests,
  `CHAIN.blake3` + `--check` gate) and the registry pins
  (`registry/ARTIFACTS.blake3`, `registry/capability-registry.json`).
- the publish channels: `js/npm` (publish-npm.yml, byte-authority gate at
  `.github/workflows/publish-npm.yml:196-203`), crates.io (eyeron-first
  order), git tags.
- the gates: mutation court, SPC court, receipt-chain sync, doc-staleness /
  witness / count gates, refusal probe, test ladder rungs 1-6.

**Out of scope**: downstream embeddings (a downstream high-risk embedder
inherits graphlaw's disclosed residuals as controls to add on top — their
risk process is their own; `SECURITY.md` "do not over-assume"); upstream
dependency internals (PurRDF, Eyeron, DuckDB/frozen-duckdb — per
`SECURITY.md` Scope, vulnerabilities there belong upstream); other repos
(wasm4pm oracles are consumed as witnesses, not governed here); the
coordinator's git transitions.

## 2. Risk criteria (the gate thresholds)

A risk is judged **mitigated** only by a witnessed mechanism, and
**accepted** only by a dated decision or a disclosed residual with a named
closure condition. The numeric thresholds below are the repo's existing
gates, cited, not invented for this plan:

| Criterion | Threshold | Locus | On breach |
|---|---|---|---|
| Hot-path capability (Cpk) | ≥ 1.33 (`SPC_CPK_MIN`) | `scripts/spc_court.sh:31-32` | gate exits 1 (honest FAIL standing since 2026-10-22: Cpk 0.0008 on 2026-10-23, `receipts/2026-10-23-spc-witness.md`; 0.001 on 2026-10-22, `receipts/spc-2026-10-22T191548.json`) |
| Hot-path latency | p99 ≤ 250 ns (`SPC_USL_NS`) | `scripts/spc_court.sh:16-17,31` | gate exits 1 (p99 leg PASS at 209 ns in the 2026-10-22 receipt; variance leg FAIL) |
| Batch variance | ≤ 5% batch-p50 | 3x33/3M split, `receipts/spc-2026-10-22T191548.json` | gate exits 1 (26.91% measured under shared-tree load) |
| Mutation kill | zero gate-relevant survivors | `scripts/mutation_court.sh`; bar stated `residual-unknowns.md:39-45` | sweep must reach a verdict; survivors killed or operator-accepted |
| Gate anti-vacuity | a gate that cannot exit 1 is VACUOUS GATE | `scripts/mutation_court.sh` self-check; `scripts/spc_court.sh:10-14` inject leg | exit 1 as VACUOUS GATE; loud SKIP is not a pass |
| Refusal-register health | probe exit-class MATCH on every code | `examples/refusal_probe.rs`; 12/12 as of 2026-10-22 (`docs/refusals.md:14-17`; re-read at use time) | register row + probe divergence = finding |
| Request wall | request > `MAX_REQUEST_BYTES` (16 MiB, `src/abi.rs:48`) refuses before allocation; host wall 64 MiB with coherence assert | `src/bin/graphlaw_run_eh.rs:34,40`; exit 92 | typed exit 92 with length+cap diagnostics |
| Fuel budget | guest fuel exhausted → typed exit 98 (`FUEL_PER_REQUEST = 100_000_000_000`) | `src/bin/graphlaw_run_eh.rs:45` | typed exit 98; the pinned-artifact court leg must expect it (wasm-gap-register addendum saw 98-vs-97 on the old 10M budget) |
| Receipt discipline | a receipt with no embedded real output is not a receipt | root CLAUDE.md Receipts convention | closure refused; flip held |
| Pin authority | published artifact digest = committed pin, cross-checked per job | byte-authority gate `.github/workflows/publish-npm.yml:196-203`; `registry/ARTIFACTS.blake3` | publish refuses |
| Disk headroom | ≥ 20 GiB free before full suites | root CLAUDE.md Disk discipline | DISK-REFUSED (witnessed: mutation sweep DISK-BLOCKED at 17.3 → 4.8 GiB, `residual-unknowns.md:50-53`) |
| Dependency advisories | every residual advisory carries a dated operator decision with expiry | `deny.toml` ignores + §5 of this plan | publish precondition fails (R-2) |

Tolerance note: `SPC_CPK_MIN` / `SPC_USL_NS` are env-overridable
(`scripts/spc_court.sh:22`) — an explicit tolerance, registered as R-29, not
an immutable bound. Any gate threshold change is a reviewed diff, never a
silent env override, and the change is recorded at the next review tick.

## 3. Identified risks

Fourteen identified risk classes: the 11-row table from the Art. 9 matrix
(`compliance-eu-ai-act.md` §4) plus three classes this plan adds (supply
chain, credential boundary, environmental). Status is as-of disk
2026-10-24; several §3-of-the-verdict rows landed since, so rows marked
"mechanism landed, receipt pending" cite the live grep counts from §9.

| # | Risk class | Evidence locus | Status (2026-10-24) | Register linkage |
|---|---|---|---|---|
| 1 | Unbounded egress (COPY/ATTACH) | `SECURITY.md:33-42` (statement graph is the egress authority; scalar fallback hardcodes `/tmp/graphlaw-rdf-<table>.csv`, `src/sql_ast.rs:2255-2258`); SQLRDF-22 | Identified; disclosed lawful hole; ATTACH allowlist fail-closed on host (`src/duckdb_op.rs:225-230`), court 13/0 (W15) + hardened scanner `leading_sql_token` landed, green receipt pending (SEC1) | R-14, R-13, SEC1 |
| 2 | Sandbox escape | `SECURITY.md:55-92` (fail-closed 28-import allowlist, zero preopen, import inventory); hardening landed: frame bound, sterile WasiCtx, fuel — live greps in §9 | Mechanism landed; green-court receipts pending (G4/SEC1/INC1 held flips) | R-12 (residual half), G4, SEC1, INC1 |
| 3 | Resource exhaustion (host ABI) | `ResourceLimit` row `docs/refusals.md:49`; native Rust API enforces no caps (`SECURITY.md:53`) | Identified; disclosed residual | R-12 |
| 4 | Unbounded mutation (no WHERE) | `REFUSAL-05` `UnboundedMutationRefusal`, `docs/refusals.md:80`, refused at compile before engine contact | Mitigated (typed compile refusal; pins re-landed 36/0, W11/W18) | — |
| 5 | Scope-integrity laundering | `REFUSAL-03` arms incl. aggregate-arg laundering marker (arm 6) and MERGE constant-ON unbounded fan-out (arm 7) | Mitigated (typed compile refusals, named arms) | — |
| 6 | WASI ambient file-I/O import surface | 28-import inventory `SECURITY.md:68-92`; containment = zero-preopen; reach granted only by host embedding | Identified; documented mitigation; reach depends on host | R-12 (adjacent) |
| 7 | Mutation-surviving gate defect | `src/abi.rs:152:5` MISSED; sweep 88 mutants, 18/88 resolved, `end_time: None` (re-read §9), exit-137 kill + post-restore sanity typed exit 4 (mc4/mc5, GO-NO-GO §1.3) | OPEN — no completed sweep verdict; zero missed among resolved; gate-relevant dispatch mutants UNDETERMINED | R-7, G-A9-4-adjacent |
| 8 | Cross-crate limits drift (WALL vs MAX_REQUEST) | G5, `docs/reference/wasm-gap-register.md:15`; now guarded by the coherence assert `src/bin/graphlaw_run_eh.rs:40` | Partially mitigated on EH host; limits-table court still owed (G5 OPEN) | G5 |
| 9 | Conftest env routing | G3/N10, `conftest.py:35`; `GRAPHLAW_WASM` ignored | OPEN | R-24-adjacent, N10 |
| 10 | Blackbox suite pinned to legacy host | N11, `residual-unknowns.md:212-215` (12 fails vs legacy path) | OPEN; fix path = pin flip, not per-test patches | N11 |
| 11 | Non-reproducible wasm builds (pins copy-only) | N9, `residual-unknowns.md:199-206`; pin `472d5557…` → re-mint `58a44095…` (GO-NO-GO §1.4, uncommitted at that window) | OPEN — copy-only pins unless operator acceptance or rebuild witness lands | N9, R-1 |
| 12 | Supply-chain integrity (NEW) | Advisories RUSTSEC-2026-0258 / 2025-0134 / 2024-0449 (§6 dated decisions); unsigned receipt telemetry 1,510/1,515 `UNSIGNED-NO-KEY` (R-9); trusted keys out-of-band (R-10); receipt_v2 no emission site (R-8); tree-revert incident INC1 (verified fixes present→absent→present, re-landed `679299d` + `907e663`); eyeron version collision 0.7.8→0.7.9 (GO-NO-GO §1.6) | Identified; the advisory half is decided (§6); identity/chaining halves are registered backlog | R-2 (closed), R-8, R-9, R-10, R-11, INC1 |
| 13 | Credential boundary (NEW) | Publish is typed-blocked on operator credentials by design: npm Trusted Publisher unset (run `40084544193` E404), crates.io 403 no token — GO-NO-GO v4 terminal state; `PUBLISH-STATUS.md`; key custody/distribution/revocation outside the library (`SECURITY.md:50`); `unverified_lease: true` explicit opt-out (`SECURITY.md:52`) | Recorded typed block; machine-impossible by design, completes with two operator acts | R-10, R-22 (partial: validate-and-pack leg green in CI) |
| 14 | Environmental (NEW) | SPC gate FAIL under shared-tree load (Cpk 0.0008 vs 1.33); `generated_kinds` fails on hosts lacking the fixture path (R-24/G-5); disk pressure killed the mutation sweep (DISK-REFUSED, exit 137 kill); moving-tree reads (GO-NO-GO §1.1 present→absent→present) | Identified; gates refuse (working), solo-machine SPC rerun owed (R-5); portability-or-typed-SKIP owed (G-5); sweep needs solo window + ≥20 GiB | R-5, R-7, R-24, G-5 |

Gap count: 14 identified classes; rows 4, 5 fully mitigated by typed compile
refusals; rows 2, 8 partially (mechanism landed, courts/receipts pending);
rows 1, 3, 6 disclosed-with-mitigation; rows 7, 9, 10, 11 open with named
closures; rows 12-14 the new classes with mixed state as tabulated.

## 4. Mitigation mapping

How the risk classes above map onto the three mitigation families. The
mapping is by mechanism, not by intention — each row names the artifact
that refuses or witnesses.

**Typed refusals** (append-only codes, never renumbered;
`docs/refusals.md`):

- Risk 1 (egress): `DuckDb` refusal family + ATTACH policy refusal via
  `enforce_attach_policy` (`src/duckdb_op.rs:225-230`); document-sequence
  `BatchMemberFailed` keeps partial-execution visible.
- Risk 3 (host ABI): `ResourceLimit` (`limit`/`observed`/`max`), limits
  `request_bytes`, `json_depth`, `atoms_per_field`, `plan_actions`,
  `policy_entries`, plus N3 and duckdb-chunk/session limits.
- Risk 4 (unbounded mutation): `REFUSAL-05` at compile, zero engine contact.
- Risk 5 (laundering): `REFUSAL-03` named arms (5, 6, 7) for scope
  discrimination, aggregate-arg re-scope, MERGE constant-ON.
- Risk 2 (sandbox): host refusals `GRAPHLAW-RUN-EH` exit trio 95/96/97,
  `GRAPHLAW-RUN-EH-REQUIRED` exit 96 (wasmi vs EH artifact), frame
  `FrameOversize` exit 92, fuel exhaustion exit 98.
- Risk 12 (supply chain, identity half): `ATTEST-REFUSED`
  (`BadSignature` / `UnknownKey` / `Malformed`, court
  `tests/trust_attest.rs`); `ReceiptRefused` reasons
  `unattested`/`bad_signature`/`untrusted_key`.

**Courts** (real collaborators, anti-vacuity-gated):

- Risk 1/2: `tests/attach_bypass_court.rs` + seam courts (`src/duckdb_op.rs`
  tests at :1336+); `tests/wasm_frame_bound_court.rs` (at-cap admitted /
  cap+1 exit 92 / 0xFFFFFFFF exit 92 / zero-len exit 0, both hosts) — court
  files landed, green receipts pending (§9).
- Risk 7: `scripts/mutation_court.sh` full sweep to verdict; per-court
  inversions as local evidence meanwhile.
- Risks 1-14 (whole surface): sql-rdf acceptance set `chicago_validation`,
  `sql_rdf_dialect`, `duckdb_statement_kinds`, `trust_attest`,
  `receipt_chain_golden_test`; determinism courts ×3 (`p28_determinism_court`,
  `two_verifier_court`, `canonical_cross_dialect`); boundary fuzz
  1,172 frames / 0 defects and memory-wall 8/0 on pin `472d5557…`
  (transcript-class, durable receipts owed — R-21); SPC court with
  `SPC_INJECT_OUTLIER=1` anti-vacuity leg.
- Risk 14 (environmental): gates exiting 1 is the designed answer to
  environmental drift — doc-staleness/witness/count gates run and refuse;
  the FAIL verdict is the gate working, not a hidden red.

**Pins** (byte authority):

- Risk 11: artifact pin chain — `priv/graphlaw.wasm` + `priv/graphlaw.wasm.blake3`
  + `registry/ARTIFACTS.blake3:13` + `js/npm/src/generated/artifact-meta.ts:3`
  (4/4 surfaces converged on `58a44095…` at GO-NO-GO close, uncommitted in
  that window; re-read at use time). Copy-only until a rebuild witness lands.
- Risk 12: per-file blake3 receipt ledger `receipts/CHAIN.blake3` +
  `scripts/receipt_chain_sync.sh --check`; query identity
  `H_query = BLAKE3(Canon(G_AST) ‖ P_params)` with admission seal verified
  pre-execution.
- Risk 13: publish channels refuse without operator credentials (npm OIDC
  404, crates.io 403) — the credential boundary is itself a pin-shaped
  control: nothing publishes by machine without a human act.

## 5. Residual acceptance — dated decisions

The register's acceptance posture: residuals are accepted only by dated
decision or disclosed-with-closure-condition; a silent pass is not a
decision. The dated deny decisions on file:

- **2026-10-24, coordinator, per-advisory (R-2 discharged)** — appended to
  `compliance-final-verdict.md` (R-2 DISCHARGE section): all three residual
  advisories dated-ignored in `deny.toml` with expiry **2026-11-23**
  (re-review required on expiry):
  - RUSTSEC-2026-0258 (h2 0.3.27, unbounded empty DATA frames) — ACCEPTED
    RISK: transitive via frozen-duckdb → reqwest ^0.11, host-side tooling
    only, not on the in-WASM execution/receipt-signing path; h2 0.4 is
    semver-incompatible with the frozen reqwest pin; upstream bump tracked.
  - RUSTSEC-2025-0134 (rustls-pemfile, unmaintained) — ACCEPTED: host-side
    TLS config plumbing only.
  - RUSTSEC-2024-0449 (unsafe-libyaml, unmaintained) — ACCEPTED: serde_yaml
    transitive, config parsing only.
  `cargo deny check advisories` exit 0 under these ignores; `cargo audit`
  exit 1 is the expected divergence (audit does not read `deny.toml`).
  Decision authority: operator directive 2026-10-24 (publish order).
- **2026-10-24, coordinator, publish terminal state (GO-NO-GO v4)** — npm
  and crates.io publish are typed-blocked on operator credentials
  (machine-impossible by design): npm Trusted Publisher console act, then
  `gh run rerun 40084544193`; `cargo login` then eyeron-then-graphlaw
  publish in dependency order. Recorded, not worked around.

No other residual carries a dated operator acceptance yet; every other
residual stands as disclosed-with-closure-condition (the residual register's
own standing line: CONFORMANT-AS-A-DISCLOSURE, falsifier = any row closes
silently and stays listed, `residual-unknowns.md:223-225`).

## 6. Review cadence

Two fixed ticks, plus expiry triggers. This closes the cadence half of
**G-A9-2** (iteration is per-window and event-driven, not a documented
regular cadence) at the plan level; the witnessed-run obligation travels
with each tick.

**Per-wave** (each milestone wave, already the operating pattern — the
residual register was re-adjudicated in the 2026-10-24 terminal wave,
`residual-unknowns.md:3-6`):

1. re-adjudicate `residual-unknowns.md` rows against the wave's receipts;
   a row flips only on a landed receipt;
2. re-read gap-register counts at use time (consolidated R-1..R-35;
   wasm-gap-register 17 rows) — no count is recalled from session memory;
3. wave receipt lands in `receipts/` with embedded output; INDEX statuses
   refreshed by the INDEX owner.

**Per-release** (pre-tag, pre-publish):

1. R-1-class truth-sync: artifact/source correspondence rungs on the exact
   pin (host-contract `cargo test -p graphlaw-wasm` + one anti-vacuity
   inversion), release notes citing the shipped digest, TAG-PREP §3/§4 in
   agreement;
2. R-2-class advisory sweep: `cargo deny check advisories`, per-advisory
   dated decision with expiry;
3. publish preflight: eyeron version vs crates.io max_version, byte-authority
   gate, npm pack/dry-run receipt (R-22's owed leg);
4. this plan re-read: §3 statuses re-verified against the tree by the §9
   commands, §8 determinations re-affirmed or flipped, and the plan's
   changes receipted in the window's RECEIPTS.

**Expiry triggers** (re-review before the named date, not at it):

- **2026-11-23**: the three cargo-deny advisory acceptances expire —
  re-decide per advisory (§5).
- Mutation sweep: any landed green-court receipt for G4/SEC1/INC1 flips
  those rows; the sweep verdict (88/88 with `end_time`) flips R-7.
- SPC solo rerun: a quiet-machine verdict supersedes the load-contaminated
  standing (R-5's named closure).

Owed cross-leak: the ROADMAP cadence line G-A9-2 names is outside this
lane's lease; the INDEX row for this file likewise (final-verdict precedent:
"register this file in INDEX — INDEX is outside this lane's lease").

## 7. Keep-vs-abandon determinations (Art. 9(6)-class)

The Art. 9(6)-style question: can the risk be reduced or eliminated without
abandoning the approach? Applied per residual row. Determinations here are
**lane-proposed** with rationale; where an operator decision already exists
on disk it is cited; the rest carry ratification status
**PROPOSED — operator ratification owed at the next review tick**. This
mirrors the repo's own lesson on decision lines (`OPERATOR-DECISIONS.md`
OD-1: a lane writing a decision line decided nothing) — a determination
without a signature is filed as proposed, never as decided.

| Residual row | Determination | Rationale | Ratification |
|---|---|---|---|
| Row 1 EH wall / in-wasm execution | KEEP (substantially closed) | RESOLVED 2026-10-23/24 by uniform new-EH rebuild + wasmtime 48 + wrap shim; SQLRDF-27 flip receipt-gated; owes two non-gating rungs on the pin | Operator ratification folded into R-1 discharge |
| Row 2/3 mutation sweep incomplete | KEEP-WITH-CONTROLS | The sweep is a real channel (baseline ran, 18/88 resolved, 0 missed among resolved); closure is a completed solo run, not a redesign. Abandoning mutation testing would be abandoning the anti-vacuity evidence class itself | PROPOSED; sweep verdict is the flip gate |
| Row 4 SPC FAIL under load | KEEP-WITH-CONTROLS | Gate refuses as designed; miss attributed to shared-tree load with a named closure (solo rerun). Abandoning the Cpk gate would remove the only capability control on the hot path | PROPOSED; solo rerun verdict governs |
| Row 5 js publish flow untested | KEEP-WITH-CONTROLS | Validate-and-pack leg witnessed green in CI (run `40084544193`); remaining legs are operator credential acts, not engineering. Abandoning the flow is impossible — it is the publish path | PROPOSED; publish completion closes it |
| Row 6 wasm4pm oracle receipts owed | KEEP-WITH-CONTROLS | Execution witnessed on the pin (fuzz 1,172/0; wall 8/0; banking 4/0); the gap is durable files, not capability | PROPOSED; receipt landing flips |
| Row 7 gws4 contract (in-wasm eval sub-claim) | KEEP (closed by row 1) | In-wasm execution witnessed (`0d11ba`-class commit series, SQLRDF-21..25); court re-run on post-EH tree implied by wave receipts | Folded into row 1's closure |
| N9 copy-only pins | KEEP-WITH-CONTROLS (this cycle) | Rebuild-reproducibility is environment-bound (wrap-shim RUSTFLAGS, frozen-duckdb cache, link order); the honest state is operator acceptance of copy-only pins this cycle or a rebuild witness | PROPOSED — this is the row closest to needing a real operator signature |
| N10 conftest env routing | KEEP-WITH-CONTROLS | Test-routing convenience gap; closure = env-driven routing with flip witness both directions | PROPOSED |
| N11 blackbox legacy-host pins | KEEP-WITH-CONTROLS | Fix path is the pin flip + 0-fail rerun, not per-test patches | PROPOSED |
| Gate backlogs (doc-staleness 1,143 / witness 190 / count 106) | KEEP-WITH-CONTROLS | Gates run and refuse (working as designed); backlogs are remediation queues; R5 anchor drift 1,143 is pre-existing and delta-checked per wave (R-33) | PROPOSED |
| `abi.rs:152:5` mutation survivor | KEEP-WITH-CONTROLS | Named gate-relevant survivor; bar: killed by a test or accepted with operator sign-off before the register row flips | PROPOSED — operator sign-off is the named closure condition itself |
| Supply-chain class (§3 row 12) | KEEP-WITH-CONTROLS | Advisories decided (§5); identity halves (receipt_v2 emission, signed telemetry, serialized chain) are receipt-gated backlog; INC1 shows the control that worked (incident recorded, re-land witnessed) | Advisory half: decided 2026-10-24. Identity half: PROPOSED |
| Credential boundary class (§3 row 13) | KEEP (by design) | The boundary is the control: publish cannot complete without operator acts; key custody stays outside the library by documented scope | Decided by the v4 terminal state |
| Environmental class (§3 row 14) | KEEP-WITH-CONTROLS | The failure mode (gates refuse under drift) is the designed behavior; closures are per-row (solo rerun, fixture portability, solo sweep window) | PROPOSED |

**No ABANDON determination arises**: no residual row makes the kernel or the
release-train approach the hazard — the residuals are control gaps with
named closures, not properties of the approach. The one candidate (the
scalar-fallback COPY form, R-14) is keep-with-controls: the preferred
expression form is documented (`docs/how-to/egress-and-attach-policy.md`),
and removing the fallback form is the named alternative if the
allowlist+witness-digest closure slips past a review tick.

## 8. What this closes and what stays open

**Closed by this file**: G-A9-1 (standalone plan exists, with cadence,
criteria, risks, mitigations, dated acceptances, and per-row
determinations); the structural half of R-26; the plan-side of G-A9-2
(witnessed-run obligation per tick remains event-owned).

**Contributed to, not closed**: G-A9-8 (determinations are PROPOSED;
operator ratification per row is the flip gate); G-A9-3/R-5 (solo rerun
owed); G-A9-4/R-21 (durable receipts owed); G-A9-5/R-22 (publish dry-run
receipt owed); G-A9-9/R-23 (per-commit CI receipt or explicit per-wave scope
note — this plan's §6 is that scope note at plan level; the receipt is still
owed).

**Explicitly not claimed**: no conformity assessment, notified-body
engagement, CE-marking basis, or declaration of conformity; no claim that
any CONFORMANT-AS-A-DISCLOSURE register became a pass; no claim that
PROPOSED determinations are decided. Counts cited from registers (35-row
consolidated register; 17-row wasm gap register; 12/12 probe; 18/88 sweep)
are as-of their cited files and re-read at use time.

## 9. Verification commands (run by this lane, 2026-10-24)

Real output backing the §3/§4 status markers (all against the working tree
at authoring):

```text
$ grep -c "leading_sql_token" src/duckdb_op.rs
14
$ grep -c "FrameOversize" src/bin/graphlaw_run_eh.rs src/bin/graphlaw_run.rs
src/bin/graphlaw_run.rs:4
src/bin/graphlaw_run_eh.rs:4
$ grep -n "\.inherit_env()" src/bin/graphlaw_run_eh.rs
295:            // empty — the builder default, no `.inherit_env()` / `.env()` /
$ grep -c "consume_fuel\|FUEL_PER_REQUEST" src/bin/graphlaw_run_eh.rs
8
$ python3 -c "...outcomes.json"   # caught/missed/unviable/end_time
caught: 9 missed: 0 unviable: 9 end_time: None
$ ls tests/ | grep -i "frame_bound\|attach"
attach_bypass_court.rs
wasm_frame_bound_court.rs
$ grep -n "WALL_BYTES\|FUEL_PER_REQUEST" src/bin/graphlaw_run_eh.rs
34:const WALL_BYTES: u32 = 67_108_864;
45:const FUEL_PER_REQUEST: u64 = 100_000_000_000
```

Court targets this plan's criteria cite (run at use time; this lane ran no
cargo):

```bash
cargo test --test chicago_validation
cargo test --test sql_rdf_dialect
cargo test --test trust_attest
cargo test --test receipt_chain_golden_test
bash scripts/receipt_chain_sync.sh --check
```

## See Also

- `docs/jira/v26.10.23/compliance-eu-ai-act.md` (Art. 9 matrix, G-A9-1..9)
- `docs/jira/v26.10.23/compliance-final-verdict.md` (consolidated register
  R-1..R-35; R-2 discharge appendix)
- `docs/jira/v26.10.23/compliance-gap-closure-plan.md` (classification +
  blockers)
- `docs/jira/v26.10.22/designs/residual-unknowns.md` (residual register)
- `docs/reference/wasm-gap-register.md` (17-row gap register)
- `docs/refusals.md` (typed refusal register) · `SECURITY.md` (disclosed
  limits) · `docs/jira/v26.10.23/GO-NO-GO-v26.10.22.md` (v3 + v4 terminal
  state) · `docs/jira/v26.10.23/PUBLISH-STATUS.md`
