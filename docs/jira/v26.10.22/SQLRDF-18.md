# SQLRDF-18 — P28-SIL program: zero-overhead execution architecture

- **Id:** SQLRDF-18
- **Title:** P28-SIL program — substrate + zero-overhead execution
  (PRD: [PRD-P28-SIL.md](PRD-P28-SIL.md))
- **Status:** Open (successor program, not part of the v26.10.22 acceptance
  set)
- **Standing:** **BLOCKED-BY: v26.10.22 integration commit** — with exactly
  three exceptions that **proceed now** (from the dispatch): (1) `LICENSE.md`,
  (2) the TC-LEGAL-01 court, (3) the baseline measurement. Everything else in
  this ticket waits for the coordinator's integration commit of the v26.10.22
  sql-rdf wave.
- **PRD:** "GraphLaw Substrate & P28-SIL Zero-Overhead Execution
  Architecture" v1.0.0
  ([PRD-P28-SIL.md](PRD-P28-SIL.md)). **Note:** the PRD verbatim prose was
  not delivered in the dispatch payload;
  [PRD-P28-SIL.md](PRD-P28-SIL.md) carries the structure; this ticket
  carries the operative content actually received. Falsifiers below are
  **inferred** from the dispatch-carried mandates (≤ 0.25 µs evidentiary
  overhead; > 12M rows/sec, p95 < 12 ms hot path; P28-SIL $50M safe harbor)
  and from the repo's existing court discipline — not verbatim PRD text. The
  coordinator should correct them at PRD-body landing.

## Dependency notes (dispatch-carried)

- Phase 1 intersects moonshot **E2** (generate the compiler surface from the
  shapes — see [SQLRDF-16.md](SQLRDF-16.md) row E2). Concurrency on the
  shapes/ggen surface must be sequenced through the coordinator.
- **§3.1 builds on landed R1 canon-once** — verified on this tree:
  canonicalization happens once at compile time
  (`src/sql_ast.rs:1447-1449`), and the receipt path consumes
  `CompiledQuery::canon_graph_bytes` with a fallback re-canonicalization
  (`src/duckdb_eval.rs:178-181`). The 0.25 µs evidentiary-overhead budget
  (§3.1 / TC-HOT rows) is therefore a measurement against a canon-once
  baseline, not a from-scratch build.

## Test-court rows (PRD §5)

| Test id | Claim under test | Falsifier (a real observation that kills the claim) | Standing |
| --- | --- | --- | --- |
| TC-HOT-01 | Hot-path invariant overhead ≤ 0.25 µs per operation (p99) | A latency benchmark court over the real frozen-duckdb hot path measures > 0.25 µs per operation at p99 (court must first be built — a latency court that cannot produce a per-op/p99 number is itself FAIL) | BLOCKED-BY integration commit |
| TC-HOT-02 | Sustained columnar throughput > 12M rows/sec | A bulk-ingestion benchmark court over the real frozen-duckdb hot path measures ≤ 12M rows/sec sustained (a throughput court that cannot produce a rows/sec number is itself FAIL) | BLOCKED-BY integration commit |
| TC-MUT-01 | The program's gates refuse on mutation (anti-vacuity) | `bash scripts/mutation_court.sh` (or the program's equivalent sweep) accepts a mutated/perturbed gate without exiting 1 — a gate that cannot flip is vacuous | BLOCKED-BY integration commit |
| TC-MOCK-00 | Zero mock-based tests on the program surface (Chicago classicist: real collaborators, no `mockall`/stubs of owned collaborators) | A real grep of the program's test surface for `mockall`/`Mock`/`patch`-style doubles returns a match and the gate still passes — i.e. the mock-ban gate cannot exit 1 on a violating tree | BLOCKED-BY integration commit |
| TC-LEGAL-01 | P28-SIL legal posture is witnessed, not asserted (safe-harbor claims exist only as receipted text) | A legal claim (P28-SIL / $50M safe harbor) is relied on by the program with no court-pinned document behind it — the court cannot name the exact filed text at `LICENSE.md` / `docs/reference/` backing the claim | **Proceeds now** (LICENSE.md + this court) |
| TC-LEGAL-02 | The legal boundary is typed: repo behavior claims vs. P28-SIL document-text claims are asserted separately (pattern: the sql-rdf rec boundary — `docs/reference/sql-rdf-rec.md` status claims are document text, not repo facts) | A court (or doc) asserts a P28-SIL/regulatory status claim as a repo-executed fact — the separation court cannot distinguish the two registers | BLOCKED-BY integration commit |

## Phase rows (PRD §6)

Falsifiers per phase; phases carry the ticket's standing unless noted.

| Phase | Scope (as inferable from dispatch) | Falsifier | Standing |
| --- | --- | --- | --- |
| Phase 1 | Substrate phase — intersects moonshot E2 (compiler surface generated from shapes); builds on landed R1 canon-once. Designs complete: [designs/PHASE1-generated-gate.md](designs/PHASE1-generated-gate.md) | Phase 1 is declared done while the E2 falsifier (deleting a shapes property does not change the generated reader / fails no coverage court) still kills E2 — i.e. Phase 1 claimed the shared surface without landing the shared work; or Phase 1's overhead numbers are asserted without a canon-once-baseline measurement receipt | Designs COMPLETE; implementation BLOCKED-BY the E1→A1→E2 chain (E2 owns the shared compiler surface; see [SQLRDF-16.md](SQLRDF-16.md)) |
| Phase 2 | Execution-architecture phase (hot-path rows TC-HOT-01/02 live here). SPSC ring skeleton IMPLEMENTED: `src/evidence_ring.rs` + `src/query_ledger.rs` + ring↔ledger join; court `tests/evidentiary_ring_court.rs` green. TC-HOT-01/02 embedded benchmark receipt still pending the full-corpus run | Any Phase 2 exit claim without an embedded TC-HOT-01/02 benchmark receipt (real output, exit code, rows/sec, p95) is FAIL — a phase closed on assertion alone | Open (skeleton-done: ring + ledger + join landed, courts green); TC-HOT receipt BLOCKED-BY integration commit |
| Phase 3 | Anti-vacuity + testing discipline (TC-MUT-01, TC-MOCK-00 live here). Residue CLOSED: laundering corpus `tests/sql_rdf_laundering_corpus.rs` 14+ cases / 0 failures with inline mutations flipped; `scripts/shape_mutation_court.sh` M1/M2; mock-ban gate `tests/no_mock_markers.rs` green (zero allowlisted source files). Full mutation sweep pending final solo run | The Phase 3 sweep passes on a tree where a known mutation (operator inversion) is injected and the sweep does not flip — vacuous gate, phase FAIL | Residue CLOSED (corpus + M1/M2 + mock gate landed, gates green); TC-MUT-01 full-sweep receipt BLOCKED-BY integration commit |
| Phase 4 | Legal / P28-SIL closeout (TC-LEGAL-01/02 live here; LICENSE.md precedes). Landed: [LICENSE-P28-SIL.md](../../../LICENSE-P28-SIL.md) + LICENSE pointer + TC-LEGAL-01/02 courts | The safe-harbor posture is declared closed with TC-LEGAL-01 or TC-LEGAL-02 unfilpped/unfiled — a legal closeout with no court-pinned filed text is FAIL | Essentially DONE pending integration (both legal courts + filed text landed); final standing flips at the integration commit |

## Acceptance

- Each row's falsifier becomes a runnable court when spun out; a row with no
  executable falsifier at spin-out is FAIL for that spin-out.
- The three proceed-now items (LICENSE.md, TC-LEGAL-01, baseline measurement)
  land with embedded real command output; a receipt with no embedded output
  is not a receipt.
- Anti-vacuity inherits the window law: `bash scripts/mutation_court.sh`
  must flip any court built from these rows.

## Cross-check findings (PRD vs moonshot designs, 2026-10-22)

Three divergences found by the cross-check lane; each is filed here as a
follow-up with its PRD / design anchors. None blocks the v26.10.22
integration commit; all gate the §3.1.1 / Phase 1 claims they name.

### Follow-up 1 — runtime re-canonicalization fallback contradicts §3.1.1

- **Divergence:** `src/duckdb_eval.rs:178-188` — when a hand-built
  `CompiledQuery` carries `ast_graph_turtle` but no `canon_graph_bytes`,
  `eval_ast_query` re-canonicalizes the AST graph at execution time
  (`canonical_ast_graph_bytes(turtle)?` at line 181) to compute the receipt
  hash. PRD Requirement 3.1.1 (PRD-P28-SIL.md:56) states: "The runtime
  shall NOT re-canonicalize the entire RDF AST graph at execution time…
  computed at compile time or initial shape admission and stored as a
  static byte slice."
- **Required behavior:** remove the execution-time re-canonicalization path
  or promote it to compile time (carry `canon_graph_bytes` on every
  `CompiledQuery` constructor), before §3.1.1 is claimable. The
  `(None, None)` SQL-string fallback (lines 183-188) is a separate legacy
  path and needs its own disposition in the same change.
- **Acceptance:** a court asserting that no `eval_ast_query` input path
  canonicalizes at execution time, flipping on a mutation that restores the
  fallback (`bash scripts/mutation_court.sh` discipline); plus
  `cargo test --test chicago_validation`.

### Follow-up 2 — A1 per-admission closure cost asserted, not measured

- **Divergence:** [designs/A1-scope-as-type.md](designs/A1-scope-as-type.md)
  (rows at lines 6-13, 45) asserts the validation-scoped RDFS closure with
  a process-wide axioms-closure cache fits the admission budget, but no
  measured derivation exists against the PRD §2 gate budget of
  **≤ 0.25 µs** per admission step (PRD-P28-SIL.md:18, :44) — and the
  ≤ 0.15 µs H_query row (:45) is tighter still. The closure cache +
  subClassOf-only regime (design line 36: "closure needed" scoped to
  subClassOf axioms) is the only plausible path to that budget; it is
  currently an assertion.
- **Required behavior:** a microbenchmark court over the real
  closure-cache hit path deriving per-admission cost from measurement
  (paired rounds, min-of-N — no fixed-sleep methodology), compared against
  the 0.25 µs gate budget; cache-miss cost reported separately.
- **Acceptance:** the benchmark lands with embedded real output (per-op
  ns, p99, rounds) and either meets the budget or files a typed
  budget-exceeded refusal; `cargo test --test p28_hot_path_bench` is the
  nearest existing rung to extend.

### Follow-up 3 — Phase 1 "strip all runtime text parsers" over-scoped

- **Divergence:** PRD Phase 1 (PRD-P28-SIL.md:111) says "Strip all runtime
  text parsers from release builds." Admission-phase Turtle parsing
  survives by design: shapes arrive as TTL and are admitted at shape
  admission (per §3.1.1's own "compile time or initial shape admission"
  carve-out), so a literal reading of Phase 1 is self-contradictory.
  Phase 1 must be scoped to SQL / text-query parsers — i.e. it subsumes
  moonshot **C2** (runtime SQL-text parsing elimination; see
  [SQLRDF-17.md](SQLRDF-17.md), [designs/C1-cache.md](designs/C1-cache.md)
  family) rather than equalling the whole parser-stripping program.
- **Required behavior:** PRD Phase 1 text corrected to "strip runtime
  SQL/text-query parsers"; Phase 1 scope restated as C2 + E2 compiler
  generation, with the admission-phase TTL reader explicitly in scope as
  retained surface.
- **Acceptance:** PRD-P28-SIL.md diff carrying the corrected Phase 1
  sentence and scope note; no code change required for this follow-up.

## TC-MOCK-00 adjudicated criterion (mapping lane, landed 2026-10-22)

**Criterion (as adjudicated):** Zero interaction-recording test doubles on
the program surface: no `mockall`/`automock!`, no
`jest.mock`/`vi.mock`/`sinon.stub`, and no `struct Mock*`/`impl Mock*` type
that records calls or scripts expectations against an owned collaborator.
Hand-written implementations of production traits over real data are lawful
collaborators regardless of identifier; such identifiers must carry an
allowlist entry with justification, never a silent exemption.

**Enforcement:** `tests/no_mock_markers.rs` — the TC-MOCK-00 build gate.
It scans `src/`, `tests/`, `wasm/`, `crates/` (`.rs` files, skipping
`target*`/`vendor`/`.git`/`receipts`) for the split marker pairs
(`mock`+`all`, `auto`+`mock!`, `jest`+`.mock`, `vi`+`.mock`,
`sinon`+`.stub`, `struct Mock`) and fails on any hit without an
`ALLOWLIST` entry. The allowlist currently carries exactly one entry —
the gate file itself (self-match exclusion), i.e. **zero allowlisted
source files** after the fixture renames. Hand-written real-interface
implementations are documented in the gate header as lawful collaborators;
only call-recording / expectation-scripting types are doubles.

**Cargo.lock note:** `grep -in mockall Cargo.lock` returns no matches
(exit 1) — the dev-dependency was removed in the fixture renames; no
`Mock` substring survives in the lockfile (`grep -c Mock Cargo.lock` = 0).

**Gate receipt (real output, this tree):**

```
cargo test --test no_mock_markers
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.57s
     Running tests/no_mock_markers.rs (target/debug/deps/no_mock_markers-3873c5120be9de2a)

running 1 test
test no_banned_mock_markers_in_source ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
```

## Scope decision (operator, 2026-10-22): the client edge owns serialization

**Operator constraint (verbatim-faithful to the dispatch):** the contract is
"the caller hands me an already-admitted graph, period." Serialization is
the client edge's job, not the kernel's. This is a scope decision, not an
optimization: the kernel stops owning wire/prose formats entirely.

**Drops out entirely:**

1. **Runtime Turtle/JSON text path at admission.** `compile_turtle(&str)`
   becomes dev/test-only behind a cargo feature; `compile_from_bytes(&[u8])`
   is the release entry point. The PRD's "no runtime lexer" row becomes
   trivially true rather than something to enforce by court.
2. **`Refusal.message` String.** The refusal becomes pure data —
   approximately 64 bytes: a `refusal_code` u8 discriminant,
   shape_id/predicate table indices, and an offending-node digest. Zero
   heap, zero `format!`. The 0.10 µs refusal budget is met **by
   construction** (CTQ #1 and OCAP row 1 convert from measured-miss to
   by-construction pass; the `RefusalMessage::Static` intermediate design is
   superseded). Prose rendering of refusals is the client edge's job.
3. **Capabilities/registry JSON.** Canonical binary/const tables internally
   (`ENGINE_ROUTES`); wire rendering moves to the client edge.

**Preserved:**

- `H_query` is already 32 bytes — no change required.
- `Canon(G_AST)` as wire format is the C2/Phase-1 endgame, now promoted to
  **DEFAULT** rather than target.
- The evidentiary ledger is already binary-chained.

**Stays:** tests and docs keep text formats — trusted-interior dev
authoring, not wire surface.

**Sharpens (existing filings re-scoped by this decision):**

- SQLRDF-17's `ast.text` field is **REPLACED by canon bytes, not gated**.
- §3.1.1's hand-built-string fallback arm (`src/duckdb_eval.rs:183-188`,
  Follow-up 1's `(None, None)` legacy path) is **DELETED, not gated**.

**Net effect:** converts three measured-miss items into by-construction
passes — CTQ #1 (refusal budget), OCAP row 1, and the §3.1.1 fallback arm.
No court needs to measure them; the architecture makes the violation
unrepresentable.

## Scope decision 2 (operator): 80/20 — prod admission is ggen-stamped byte constants

**Operator constraint (verbatim-faithful to the dispatch):** production RDF
graphs are finite and known at build time; ggen holds the vocabulary. The
pipeline has two modes:

- **Dev mode:** ggen reads Turtle and generates the reader, the gate, the
  byte constants, the pre-canonicalized bytes, and the pre-absorbed BLAKE3
  states.
- **Prod mode:** ggen's OUTPUT is the input — `compile_from_bytes` receives
  build-stamped bytes; admission matches against the generated table or
  hashes directly, binds params, executes.

The hot 80% is parameterized instances of fixed schemas compiled once per
pattern (the functorial stamp,
[designs/PHASE1-generated-gate.md](designs/PHASE1-generated-gate.md) at its
terminal point). The tail 20% is the dev-mode rebuild cycle OR an explicit
marked cold path paying parse cost — there is **no silent warm path**.

**Collapses by construction:**

- **§3.1.1 fallback arm deleted** — there is no hand-built-text entry in
  prod at all (completing Scope decision 1's deletion of the
  `(None, None)` legacy path, `src/duckdb_eval.rs:183-188`).
- **SQLRDF-17 raw-SQL hole** — `ast.text` is replaced by build-stamped
  constants; the census court shrinks to "every admitted byte sequence is
  in the generated table."
- **CTQ #1 refusal** — the refusal is a table lookup on a generated enum;
  the prose question does not arise at runtime.
- **Runtime lexer strip** — a consequence, not a court-enforced row: ggen
  is the only Turtle reader, and ggen runs at build time.

**Forced design decision:** admission identity becomes the build artifact —
new query patterns are build+deploy, not runtime upload.

## OPEN-DESIGN: C1 cache hit vs the P28-SIL "every execution is recorded" claim (coordinator + operator ruling needed, 2026-10-22)

**The interaction:** the C1 consult seam
(`src/duckdb_eval.rs:327-337`) returns a stored `ExecutionReceipt` on a
cache hit **before** the evidentiary-ring seam (`src/duckdb_eval.rs:509-529`)
ever runs. A cache-served execution therefore produces a full
`ExecutionReceipt` but captures **zero ring/ledger evidence**: no
`RingDescriptor`, no ledger row, no ring counters. If LICENSE /
ledger language claims "every execution is recorded", a C1 hit silently
violates it: the receipt exists, the evidence does not.

**The question for the operator:** is a cache-served execution an
"execution" under the P28-SIL claim?

- **Option (a) — cache hits ARE executions:** the ring design already
  carries a `cached` flag on the descriptor (fielded in
  `src/evidence_ring.rs`, currently unused on the push path); push a
  `RingDescriptor` with `cached: true` from the consult seam before the
  early return at `duckdb_eval.rs:335`. Cost: one 64-byte push on the hit
  path — must still fit the 0.25 µs budget (TC-HOT-01 measures it).
- **Option (b) — cache hits are NOT executions:** the ledger records only
  engine work; LICENSE-P28-SIL.md and the ledger header comments must then
  state the carve-out explicitly ("cache-served results are not executions;
  only engine-executed statements are recorded"), and TC-LEGAL-02's
  separation court must pin the exact filed sentence. Otherwise the "every
  execution is recorded" claim is overbroad and TC-LEGAL-01's witness
  requirement is unmet for hits.

Either ruling is acceptable; **silence is not** — under ruling-by-omission
the code (option a's behavior: hits bypass capture) contradicts the
LICENSE language (currently reading as option b's scope without saying so).

**Resolution path:** operator picks (a) or (b); if (a), the consult seam
pushes a cached-flagged descriptor and TC-HOT-01's benchmark asserts the
push fits budget; if (b), LICENSE-P28-SIL.md gains the explicit carve-out
sentence and TC-LEGAL-02 pins it. Either way the ring-vs-receipt asymmetry
becomes a documented, court-pinned invariant instead of an undocumented
early return. File the ruling here as a Scope decision 3 addendum.

## Scope decision 3 (operator, 2026-10-22): the ω_context term is the deployment epoch

Resolves CAP-1's operator-ruling flag. The three-term H_query identity in
LICENSE-P28-SIL.md §1.5 —
$\mathbf{H}_{\text{query}} = \operatorname{BLAKE3}(\operatorname{Canon}(G_{\text{AST}})
\mathbin{\Vert} \mathbf{P}_{\text{params}} \mathbin{\Vert}
\oint_{\partial \Omega} \omega_{\text{context}})$ — is formalized:
$\oint_{\partial \Omega} \omega_{\text{context}}$ denotes the **deployment
epoch**, a `u64` assigned exactly once at substrate initialization (ggen
build stamp / deployment id), **immutable for the deployment lifetime** —
not ambient hardware state.

- **Determinism preserved:** within a deployment the epoch is constant, so
  $\mathbf{H}_{\text{query}}$ stays deterministic and replayable (same
  epoch → same receipt, TC-LEGAL-01 unaffected).
- **Cross-deployment domain separation:** different deployments carry
  different epochs, so receipts from one deployment never verify as another's.
- **Two-term = zero-epoch special case:** the landed two-term formula (PRD
  §3.1.2, sql-rdf-rec.md §1.4) is the $\omega_{\text{context}} = 0$
  single-deployment case. The term is **reserved and zero-valued**; no code
  change is required now.
- **Reserved derivation (court-pinned):**
  $\mathbf{H}^{(3)}(\text{epoch}) = \operatorname{BLAKE3}(\mathbf{H}^{(2)}
  \mathbin{\Vert} \text{epoch.to\_le\_bytes}())$, where $\mathbf{H}^{(2)}$ is
  the landed two-term hash. Epoch 0 gives the reserved single-deployment
  value; any nonzero epoch domain-separates.
- **Harmonization notes filed:** LICENSE-P28-SIL.md §1.5,
  PRD-P28-SIL.md §3.1.2, sql-rdf-rec.md §1.4 (the epoch term is reserved,
  zero-valued in single-deployment mode). Design record:
  [designs/P28-SIL-specs.md](designs/P28-SIL-specs.md).
- **Court:** `tc_epoch_reserved_three_term_formula_compatibility` in
  `tests/p28_determinism_court.rs` asserts the two-term ≡
  three-term-with-zero-epoch algebraic compatibility and that nonzero epochs
  domain-separate.
