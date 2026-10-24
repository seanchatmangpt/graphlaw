# GraphLaw v26.10.22 — EU AI Act Art. 9 (Risk Management) Evidence Matrix

**Scope**: GraphLaw v26.10.22 on `main` (commit `7e92175` at authoring).
**Nature of this document**: an evidence matrix mapping on-disk artifacts to
Art. 9 requirements, with UNWITNESSED items marked as gaps. It is not a
certificate, attestation, or declaration of conformity. It asserts only that
the cited files exist and say what they say.

## 1. System classification argument

GraphLaw is a deterministic law-admission kernel: RDF/SHACL graphs are
admitted through one executable admission law (`src/law.rs`), compiled to SQL
(`src/sql_ast.rs`), evaluated on frozen-duckdb (`src/duckdb_eval.rs`), and
receipted (`src/attest.rs`, Ed25519). It contains no model training, no
inference from trained weights, no autonomous goal selection. Its only
"decision-making" is enforcement of operator-authored policy: a graph either
passes the closed-world SHACL shapes (`ontologies/sql-ast.ttl`) and executes
deterministically, or it is refused with a typed, stable-coded refusal
(`docs/refusals.md`, codes `REFUSAL-01`..`REFUSAL-05`,
`docs/refusals.md:66-80`).

Honest boundaries:

- **Is**: deterministic compilation and evaluation; policy-admitted
  transitions only; typed fail-closed refusals; content-addressed receipts.
- **Is not**: an ML system, a learned model, or a product applying
  statistical approaches to infer behavior from data. On the Art. 3(1)
  definition it is not an AI system as such. The nearest Art. 2(1) surface is
  **general-purpose AI-adjacent infrastructure**: a third-party developer
  could embed GraphLaw as the execution/egress gate inside a high-risk AI
  system's data pipeline, so this matrix is filed as provider-side
  due-diligence evidence for that embedding scenario — not as a claim that
  GraphLaw itself is a high-risk AI system.
- **Why this matters for Art. 9**: even absent AI-system status, the Art. 9
  process shape (identify → mitigate → track residual risk, iteratively) is
  exactly the operating shape this repo already runs (ticket ledger → typed
  refusals/gates → residual registers). The mapping below demonstrates the
  process exists; it does not claim regulatory conformity.

## 2. Art. 9(1) — a risk-management process, planned and run iteratively

The process is instantiated per-milestone-window. The active window ledger
`docs/jira/v26.10.22/INDEX.md` carries 29 SQLRDF ticket rows
(header at `docs/jira/v26.10.22/INDEX.md:24`; 29 `| SQLRDF` rows counted at
authoring — re-read at use time per the no-overclaiming rule). Each ticket
row carries Id / Title / Status / Gap / closure condition / court target, and
closure is receipt-gated (e.g. SQLRDF-27 at
`docs/jira/v26.10.22/INDEX.md:574`, "Closed (2026-10-23, receipt-gated:
FIBO 4/0 + lifecycle 3/0 … owed non-gating: host-contract rung +
anti-vacuity inversion").

**Gap G-A9-1 (UNWITNESSED)**: Art. 9(1) also requires the process to be
*planned and documented as a process*, not just enacted. No standalone
risk-management plan document exists on disk. Closure condition: a
process-description section in the next milestone charter
(`docs/jira/v26.10.23/ROADMAP.md` candidate) naming the plan-review cycle
cadence, referencing this matrix.

## 3. Art. 9(2) — regular iteration and coordination

Iteration cadence: daily milestone windows with per-wave receipts
(`docs/jira/v26.10.22/RECEIPTS.md`, wave sections; SPC leg verdicts at
RECEIPTS.md:1471, 1597, 1719). Coordination: lane leases + shared seams in
`docs/jira/v26.10.22/_LANES.md`, coordinator-owned integration,
compile-freeze SLA (`CLAUDE.md` "Lanes" conventions). The residual-unknowns
register is explicitly re-adjudicated ("Terminal-wave update 2026-10-24:
rows 1-7 re-adjudicated…",
`docs/jira/v26.10.22/designs/residual-unknowns.md:3-6`).

**Gap G-A9-2 (UNWITNESSED)**: iteration is per-window and event-driven, not a
documented *regular planned cadence*. Closure: a cadence line in the roadmap
with a witnessed re-adjudication run per tick.

## 4. Art. 9(3)(a) — identification and analysis of foreseeable risks

On-disk identification channels:

- **Ticket ledger** (foreseeable-risk register of record):
  `docs/jira/v26.10.22/INDEX.md:24-574`, 29 tickets each with Gap and
  closure condition, including safety-adjacent gaps (SQLRDF-22 egress row;
  SQLRDF-23 sequence partial-commit semantics row).
- **Residual-unknowns register**:
  `docs/jira/v26.10.22/designs/residual-unknowns.md:1-226` — rows 1-9 plus
  gate backlogs, each row: claim / why open / what closes it; terminal-wave
  re-adjudicated.
- **Wasm gap register**: `docs/reference/wasm-gap-register.md:9-25` — 15 rows
  (G1-G8, D1-D4, BB1, UU1, W08; 13 OPEN / 2 PENDING), each with an evidence
  locus and a closing witness.

### Identified risk classes (with evidence)

| Risk class | Evidence | Status |
|---|---|---|
| Unbounded egress (COPY/ATTACH) | `SECURITY.md:36-40` (scalar-fallback COPY hardcodes `/tmp/graphlaw-rdf-<table>.csv`, `src/sql_ast.rs:2255-2258`, no allowlist, no witness digest); SQLRDF-22 (INDEX) | Identified; disclosed as lawful hole, tracked under ticket |
| Sandbox escape | `SECURITY.md:55-112` (fail-closed import allowlist `src/bin/graphlaw_run_eh.rs:222-231`; zero WASI preopen; NO fuel/epoch metering; `inherit_env()` leaks full host env at `graphlaw_run_eh.rs:236`) | Identified + partially mitigated; metering absent = disclosed residual |
| Resource exhaustion (host ABI) | `ResourceLimit` row `docs/refusals.md:49` (`request_bytes`, `json_depth`, `atoms_per_field`, `plan_actions`, `policy_entries`); native Rust API does not enforce the caps (`SECURITY.md:52`) | Identified; native-API cap absence = residual |
| Unbounded mutation (no WHERE) | `REFUSAL-05` `UnboundedMutationRefusal`, `docs/refusals.md:80` — refused at compile, zero engine contact, receipt `receipts/2026-10-23-eh-closure-wave.md` | Mitigated (typed compile refusal) |
| Scope-integrity laundering | `REFUSAL-03` arms incl. laundering-marker analysis, `docs/refusals.md:78` (aggregate-arg re-scope; constant-only MERGE ON unbounded fan-out) | Mitigated (typed compile refusal, named arms) |
| WASI ambient file-I/O import surface | `SECURITY.md:68-92` full 28-import inventory (`path_open`/`rename`/`unlink`/`environ_get`); containment = zero-preopen posture | Identified + documented mitigation; reach depends on host embedding |
| Mutation-surviving gate defect | MISSED at `src/abi.rs:152:5` (survives suite); full sweep DISK-BLOCKED, OPEN (`residual-unknowns.md:37-72`) | Identified, OPEN — closure = completed sweep + kill-or-accept |
| Cross-crate limits drift (WALL vs MAX_REQUEST) | G5 (`docs/reference/wasm-gap-register.md:15`, `src/abi.rs:48`) | Identified, OPEN |
| Conftest env routing (wasm-vs-host test selection) | G3 / N10 (`wasm-gap-register.md:13`, `residual-unknowns.md:207-211`, `conftest.py:35`) | Identified, OPEN |
| Blackbox suite pinned to legacy host (12 fails) | N11 (`residual-unknowns.md:212-215`) | Identified, OPEN |
| Non-reproducible wasm builds (pins copy-only) | N9 (`residual-unknowns.md:199-206`) | Identified, OPEN |

## 5. Art. 9(3)(b) — risks known/foreseeable after development under normal use

- **SPC gate standing FAIL**: Cpk 0.001 vs the 1.33 requirement on the
  receipt fold path — disclosed, not hidden.
  `receipts/spc-2026-10-22-inject.json:12,34,36` (`"cpk": -0.0011` injected
  leg; verdict `"FAIL"`, reason "cpk … < 1.33") and
  `receipts/spc-2026-10-22T191548.json` (fresh honest-FAIL: Cpk 0.001 vs
  1.33, batch variance 26.91% vs 5% budget, p99 209 ns PASS)
  (`residual-unknowns.md:74-99`, row 4). **Gap G-A9-3**: the FAIL is
  attributed to shared-tree load variance; the quiet-machine solo rerun is
  owed — closure is exactly that rerun with SPC refresh
  (`residual-unknowns.md:97-99`).
- **Gate backlogs live, not green**: `doc_staleness_gate.sh` exit 1 (76
  findings post rule-change), `witness_gate.sh` exit 1 (190 findings),
  `count_gate.sh` exit 1 (106 stale rows) — `residual-unknowns.md:180-183`.
  Gates run and refuse; the backlogs are the tracked residual. Closure:
  remediate rows while gates still refuse on injected defects.
- **wasm4pm oracle legs**: executed for real on pin `472d5557…bf80ea`
  (fuzz 1,172 frames / 0 defects; memory-wall 8/0; banking 4/0 after DvP
  fix) but stand as witnessed-transcript claims with durable `receipts/`
  files owed — `residual-unknowns.md:117-134` (row 6). **Gap G-A9-4**:
  closure = the durable receipts land.
- **js publish flow never run end-to-end** (`residual-unknowns.md:101-115`,
  row 5): no `npm pack` / dry-run publish receipt exists. **Gap G-A9-5**:
  closure = dry-run publish receipted against the real package.json.

## 6. Art. 9(3)(c) — reasonably foreseeable misuse

| Foreseeable misuse | Evidence | Disposition |
|---|---|---|
| Running untrusted graphs with env inherited | `SECURITY.md:104-106` | Disclosed residual — strip env before untrusted graphs |
| Preopen into guest; attach allowlist not followed in-sandbox | `SECURITY.md:107-110` (`GRAPHLAW_ATTACH_ROOTS` is a host-library control, NOT enforced in-sandbox) | Disclosed residual |
| Treating plain receipts as authenticated | `SECURITY.md:44-46` (presence-only; anyone who can write a state can record one) | Mitigated by the signed API (`src/attest.rs`, Ed25519); documented |
| `unverified_lease: true` opt-out | `SECURITY.md:48-50`; typed refusal `UnverifiedLeaseRefused`, `docs/refusals.md:47` | Documented explicit opt-out |
| Admitted COPY as host-user egress | `SECURITY.md:31-40` ("treat the statement graph as the egress authority") | Disclosed; scalar-fallback hole named with file:line |
| No CPU metering → in-wasm loop pins the thread | `SECURITY.md:96-98` | Disclosed residual; closure = host-side metering |
| Native API without resource caps | `SECURITY.md:52` | Disclosed residual |
| Embedding as a high-risk-AI pipeline gate while metering/attach enforcement absent | `SECURITY.md:96-110` | **Gap G-A9-6**: closure = metering + in-sandbox attach-policy enforcement, or a documented supported-embeddings scope restriction |

GraphLaw is pure software with no actuation surface; no Art. 9(3)(d) physical
hazard class applies. The technical-hazard surface is the misuse table above.

## 7. Art. 9(4) — measures that reduce risk as far as possible without other considerations

The typed-refusal doctrine states the shape directly: "Every refusal is typed
and leaves no child state" (`docs/refusals.md:1-3`); codes are append-only,
never renumbered (CLAUDE.md conventions). Concretely:

- Typed refusal carriers `REFUSAL-01`..`REFUSAL-05` with trigger / carrier /
  recovery per row; every code triggered for real, probe 12/12 exit-class
  MATCH exit 0 (`docs/refusals.md:7-17`); register re-audit with zero anchor
  drift (lane T4, `docs/refusals.md:19-24`,
  `docs/jira/v26.10.22/designs/t4-refusal-reaudit.md`).
- Fail-closed gates: import allowlist (`SECURITY.md:57-59`); wasm boundary
  contract — in-wasm `{"op":"duckdb"}` fail-closes with typed
  `Unsupported`/`unknown op` refusal (`residual-unknowns.md:146-156`,
  `tests/gws4_ipc_injection_court.rs`); loud typed SKIP is not a pass
  (CLAUDE.md Lithography).
- Adequacy testing: per-ticket DoD courts plus cross-ticket anti-vacuity
  (`scripts/mutation_court.sh`, CLAUDE.md:24); Chicago-style real-collaborator
  courts (`tests/chicago_validation.rs`, real frozen-duckdb connections,
  readback-asserted).

**Gap G-A9-7 (UNWITNESSED)**: Art. 9(4) requires measures appropriate to
generally acknowledged state of the art. No on-disk state-of-the-art
evaluation exists. Closure: a short note pinning frozen-duckdb / Ed25519 /
wasmtime versions against known vulnerability sources, or an explicit
scope restriction.

## 8. Art. 9(5)-(6) — proportionate selection, residual-risk evaluation, and the abandon decision

Art. 9(5) selection is witnessed by per-row closure conditions with named
receipts. Art. 9(6) residual-risk *evaluation* (not assumption) has its
strongest on-disk evidence in the residual register's own standing line:
"CONFORMANT-AS-A-DISCLOSURE — this file enumerates unknowns; it proves
nothing beyond that the unknowns were stated. Falsifier: any row above closes
silently (no receipt, no re-run) and stays listed."
(`residual-unknowns.md:223-225`).

**Gap G-A9-8**: no on-disk Art. 9(6)-style determination ("risk cannot be
reduced without abandoning the approach") has ever been considered per
residual row. The mutation-court survivor (`src/abi.rs:152:5` MISSED, kept
with operator sign-off as the named closure condition,
`residual-unknowns.md:43-45`) is a tracked decision, not a filed Art. 9(6)
determination. Closure: an explicit keep-with-controls vs abandon
determination per residual row, operator-signed.

## 9. Art. 9(7)(a)-(b) — testing

**(a) pre-defined testing against the identified risks**: the risk classes in
§4-§6 define the objectives; testing is real-collaborator Chicago courts
(`tests/chicago_validation.rs`), anti-vacuity mutation courts
(`scripts/mutation_court.sh`), adversarial courts (gws4 IPC injection 3/0
green, `residual-unknowns.md:153-156`), and the SPC gate
(`scripts/spc_court.sh`, Cpk ≥ 1.33, CLAUDE.md:25) — whose current FAIL
verdict is the gate doing its job (§5).

**(b) pre-defined metrics/indicators throughout development**: Cpk ≥ 1.33,
5% variance budget, p99 250 ns bound (TC-HOT: p99 209 ns PASS, variance
26.91% FAIL, Cpk 0.001 FAIL — `residual-unknowns.md:85-96`); mutation-kill
requirement; refusal-probe 12/12 exit-class MATCH (`docs/refusals.md:7-17`);
gate ladder rungs 1-6 per window (CLAUDE.md test ladder); receipts land into
`receipts/` with embedded real output.

**Gap G-A9-9**: "at all times" — testing is per-window/wave, not continuous;
no per-commit CI channel receipt is cited in this matrix. Closure: a
per-commit CI channel receipt, or an explicit scope note that gates run
per-wave.

**Gap count**: 9 (G-A9-1 … G-A9-9), each with an inline closure condition.
No claim in this file is UNWITNESSED unless marked as a gap.

## 10. What this file does not claim

No conformity assessment, notified-body engagement, Annex support,
presumption of conformity, CE-marking basis, or declaration. The mapping
shows a working, receipt-gated process that structurally satisfies the
Art. 9 process shape; whether GraphLaw is in Art. 2 scope at all is a
classification question the provider answers in the context of each
embedding. This file is a due-diligence evidence matrix, not a certificate.
