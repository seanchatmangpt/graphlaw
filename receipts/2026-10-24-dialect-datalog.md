# Dialect verification receipt — DATALOG surface — 2026-10-24 (lane dial-datalog)

- Repo: `/Users/sac/graphlaw` (canonical checkout, verified as-is; in-flight release-closeout working tree).
- HEAD: `6da1d33a93f61068fa9b033bec353db78d335460`
- Dirty disclosure: 468 porcelain entries (modified + untracked). Load-bearing for this receipt:
  `registry/capability-registry.json` is **M** in the working tree (10 insertions / 10 deletions vs HEAD).
  No git mutations, no file edits outside this receipt. Lane build root:
  `/Users/sac/graphlaw/target-lane-dial-datalog` (cargo lease; created by this lane).
- Pin b3sum: `courts/code-partition/axioms.blake3` =
  `2add10ce9ff782e33b75de099cba8dc8a114ab759855f93a7be85abb18581506`
- Wasm artifact under test: `/Users/sac/graphlaw/priv/graphlaw.wasm`,
  b3sum `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a`
  (git: commit `499e759` "pin(v26.10.22): artifact 58a44095 …"; clean in `git status`).
- Prereqs witnessed present: `pm4py 2.7.22.1`; shim `tests/fixtures/wasm-shim/glaw_wrap_shim.o`;
  sibling packs `/Users/sac/ggen-marketplace/packs/insa-law-pack/ontology.ttl` and
  `/Users/sac/ggen-marketplace/packs/refusal-vocabulary-pack/`. Disk: 69 GiB free at start.

## Per-court commands and verbatim results (sequential, no rerun-to-green)

Env for every cargo invocation: `CARGO_TARGET_DIR=/Users/sac/graphlaw/target-lane-dial-datalog`.
`GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm` on the engine/wasm courts.

### (a) tests/chicago_n3_datalog.rs — native ABI, N3 + datalog

Command: `cargo test --test chicago_n3_datalog`

Verbatim:

    running 6 tests
    test malformed_n3_and_malformed_datalog_fail_closed ... ok
    test w3c_n3_implication_and_chaining_fire ... ok
    test n3_semantics_are_invariant_to_statement_order ... ok
    test souffle_transitive_closure_example_has_exact_least_fixpoint ... ok
    test datalog_fixpoint_is_invariant_to_fact_and_rule_order ... ok
    test n3_and_datalog_agree_on_the_shared_recursive_closure ... ok

    test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

### (b1) tests/ubo_sanctions_court.rs — recursive datalog UBO unrolling in the wasm engine

Command: `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm cargo test --test ubo_sanctions_court`

Verbatim:

    running 3 tests
    test tc_ubo_01_sanctioned_transitive_control_refused_state_preserved has been running for over 60 seconds
    test tc_ubo_02_lawful_applicant_admitted_receipt_sealed has been running for over 60 seconds
    test tc_ubo_03_broken_gate_flags_illegal_materialization has been running for over 60 seconds
    test tc_ubo_01_sanctioned_transitive_control_refused_state_preserved ... ok
    test tc_ubo_03_broken_gate_flags_illegal_materialization ... ok
    test tc_ubo_02_lawful_applicant_admitted_receipt_sealed ... ok

    test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 638.83s

(The `has been running for over 60 seconds` lines are cargo's progress notices on the
debug-wasmtime engine, not failures; all 3 courts executed green in 638.83s wall.)

### (b2) tests_py/test_ubo_sanctions.py — pm4py token-replay conformance over the same gate

Command: `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm GRAPHLAW_TARGET_DIR=/Users/sac/graphlaw/target-lane-dial-datalog pytest tests_py/test_ubo_sanctions.py -v`

conftest resolved `GRAPHLAW_WASM` (no wasm rebuild) and built the EH host once into the lane
root (`cargo build --bin graphlaw-run-eh --features runner-eh`, `CARGO_TARGET_DIR=` lane root),
then spawned the real packet-4 host over the artifact. No typed skip taken.

Verbatim:

    tests_py/test_ubo_sanctions.py::test_ubo_pm4pytest_sanctions_unrolling PASSED [ 50%]
    tests_py/test_ubo_sanctions.py::test_ubo_pm4pytest_session_isolation PASSED [100%]

    ============================== 2 passed in 37.53s ==============================

### (c1) tests/op_examples.rs — registry op examples (incl. 4 `datalog` examples: 2 ok + 2 refused), native+wasm byte identity

Command: `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm cargo test --test op_examples`

First run:

    test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 128.58s

Second invocation of the identical failing command, to capture the verbatim panic block
(evidence capture, not a green-rerun; outcome identical — FAILED both times):

    test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 70.46s

Verbatim failure block:

    ---- every_example_matches_native_and_wasm stdout ----

    thread 'every_example_matches_native_and_wasm' (8636290) panicked at tests/common/mod.rs:225:5:
    native and wasm responses diverge for {"op":"capabilities"}
    native: {"abi":1,...,"registry_blake3":"blake3:2a4d109e8ba2c71ff0fc49cc38dbb829dbc8970e13960d9e6202da633ea81c7e",...,"surface_blake3":"blake3:da96e1375a76610b5d91aedb9b01d5ac0807b0f850ce4440628c37a3b96b9ee9"}
    wasm:   {"abi":1,...,"registry_blake3":"blake3:80ef3ebf957e2d6eac6950d9a08362ad6d339ffde7a235b27f5d748c746f23c7",...,"surface_blake3":"blake3:da96e1375a76610b5d91aedb9b01d5ac0807b0f850ce4440628c37a3b96b9ee9"}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

(Full JSON bodies embedded in the lane's captured output; the ONLY differing field is
`registry_blake3` — the `purrdf::datalog` authority row
`{"authority":"purrdf::datalog","capability":"Datalog","revision":"2.0.2"}` is byte-identical
on both sides, as are all ops lists and `surface_blake3`.)

The court panicked at the harness positive control (`capabilities` byte identity) BEFORE the
example loop, so the 4 `datalog` op-examples were NOT executed by this court on either runtime.

### (c2) tests/op_forward_compat.rs — same registry over native+wasm with unknown-key padding

Command: `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm cargo test --test op_forward_compat`

    test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 71.28s

Verbatim failure headers (byte-array payloads omitted here; both decode to the identical
JSON pair embedded under (c1)):

    ---- a_control_every_example_reaches_its_declared_outcome_on_both_runtimes stdout ----
    thread 'a_control_every_example_reaches_its_declared_outcome_on_both_runtimes' (8876193) panicked at tests/op_forward_compat.rs:63:9:
    assertion `left == right` failed: capabilities.default: native == wasm

    ---- unknown_top_level_keys_never_change_any_response_byte stdout ----
    thread 'unknown_top_level_keys_never_change_any_response_byte' (8876197) panicked at tests/op_forward_compat.rs:109:9:
    assertion `left == right` failed: capabilities.default: baseline native == wasm

Same root cause and same consequence: the loop aborts at the `capabilities` example, so the
`datalog` examples later in file order were NOT executed by this court on either runtime.

### (c3) tests/registry_dispatch.rs — datalog op registry/dispatch arms (native ABI)

Command: `cargo test --test registry_dispatch`

    test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

### (d) tests/insa_law_order.rs

Command: `cargo test --test insa_law_order`

    running 6 tests
    test ontology_matches_pinned_blake3 ... ok
    test chain_length_declaration_matches_reality ... ok
    test reversed_trace_is_flagged ... ok
    test exactly_8_transitions_form_the_linear_chain ... ok
    test transposed_pair_is_flagged_by_both_legs ... ok
    test lawful_trace_passes_n3_and_sparql_legs ... ok

    test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

Dispatch note: the dispatched expectation was a typed BLOCKED on a missing pack fixture.
It did NOT materialize: `/Users/sac/ggen-marketplace/packs/insa-law-pack/ontology.ttl` exists
on this machine and its blake3 matches the pin, so all 6 courts executed green. The BLOCKED
path (`require_ontology` -> eprintln "BLOCKED: insa-law-pack ontology absent at …") is
present in the file but was not taken on this run.

### (e) tests/refusal_crosswalk_differential.rs

Command: `cargo test --test refusal_crosswalk_differential`

    test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.91s

Scope note (read from source before running): this file contains **no datalog legs** — zero
occurrences of `op: datalog` or `graphlaw::datalog`. Its legs are SPARQL (NativeSparqlEngine)
vs python rdflib differentials over the refusal-vocabulary pack, plus a sabotage twin. The
dispatch's "datalog legs" phrasing does not correspond to a datalog surface in this file;
nothing was skipped — the surface is absent. All 6 tests ran and are green.

## AGGREGATE (executed-and-green only; skips/BLOCKED never merged)

| Court | Executed green | Failed |
|---|---|---|
| chicago_n3_datalog | 6 | 0 |
| ubo_sanctions_court | 3 | 0 |
| test_ubo_sanctions.py (pm4py) | 2 | 0 |
| op_examples | 2 | 1 |
| op_forward_compat | 3 | 2 |
| registry_dispatch | 14 | 0 |
| insa_law_order | 6 | 0 |
| refusal_crosswalk_differential | 6 | 0 |
| **Total** | **42** | **3** |

- Executed-and-green total: **42** (of which **37** are datalog-surface-attributable:
  6 + 3 + 2 + 14 + 6 + 6; the 5 greens inside the two red courts are file-structure/schema
  tests that execute no datalog surface and carry no datalog bits).
- Real-red tests: **3**, all one root cause (see classification).
- Typed skips: **0**. Ignored: **0**. Env-BLOCKED observed: **0** (see (d) note).

## Classification of every non-green

| Test | Court | Classification | Evidence |
|---|---|---|---|
| `every_example_matches_native_and_wasm` | op_examples | **REAL RED** (as-is-tree artifact skew, not a datalog defect) | panic at `tests/common/mod.rs:225`; native `registry_blake3=blake3:2a4d109e…` (working-tree-edited `registry/capability-registry.json`, self-declares `blake3:2a4d109e…`) vs wasm `registry_blake3=blake3:80ef3ebf…` = HEAD's committed registry (`git show HEAD:registry/capability-registry.json` self-blake3 `blake3:80ef3ebf…`); pinned artifact `priv/graphlaw.wasm` (b3 `58a44095…`, commit `499e759`) embeds the pre-edit registry. `surface_blake3` identical both sides. |
| `a_control_every_example_reaches_its_declared_outcome_on_both_runtimes` | op_forward_compat | **REAL RED** (same root cause) | panic at `tests/op_forward_compat.rs:63:9`, `capabilities.default: native == wasm` |
| `unknown_top_level_keys_never_change_any_response_byte` | op_forward_compat | **REAL RED** (same root cause) | panic at `tests/op_forward_compat.rs:109:9`, same divergence |
| `datalog` op-examples legs (2 ok + 2 refused) | op_examples + op_forward_compat | **UNEXECUTED behind the red control** — listed here, NOT merged into any pass count | both courts abort at the `capabilities` example before reaching the `datalog` examples in file order |

Known deferrals: none recorded. Silent passes: none — every non-green or unexecuted leg is
named above.

## Honesty line — datalog coverage bounds (what this run actually courts)

Courted and green on this tree:
- Positive-semiring Datalog least-fixpoint recursion (base rule + recursive rule) on a cyclic
  graph, exact closure, and fixpoint invariance to fact order and rule order
  (native ABI, `chicago_n3_datalog`).
- Cross-authority agreement: N3 implication/chaining closure == Datalog closure on the shared
  recursive program; malformed N3 and malformed datalog requests fail closed
  (`chicago_n3_datalog`).
- Recursive datalog transitive-ownership unrolling inside the wasm engine (same fixpoint
  wire shape) feeding a real host gate over in-module frozen-duckdb, including the
  anti-vacuity broken-gate detection (`ubo_sanctions_court`, `GRAPHLAW_WASM`).
- The same gate's event vectors replayed by external oracle pm4py 2.7.22.1 token-replay
  fitness/precision, with the broken-gate trace rejected
  (`test_ubo_sanctions.py`).
- The `datalog` op's registry presence, dispatch arms, and request/response declarations on
  the native ABI (`registry_dispatch`).

NOT courted by this run (bounds, not claims):
- Cross-runtime byte identity of the `datalog` op-examples requests — blocked behind the
  `capabilities` control red in op_examples/op_forward_compat; the recursive datalog engine
  inside `priv/graphlaw.wasm` IS witnessed (ubo courts, green), but the registry example
  suite never reached its datalog rows on either runtime.
- Stratified negation / negation-as-failure under datalog — no court in this lane's scope
  exercises it (no negative body literals appear in any courted program; all courts are
  positive Datalog).
- Rule/fact resource caps (`MAX_DATALOG_RULES`/`MAX_DATALOG_FACTS`, `tests/abi_bounds_court.rs`)
  and the direct wasm `datalog_computes_a_least_fixpoint` wire court in `tests/wasm_abi.rs`
  exist but are outside this lane's enumerated scope; not run, not counted.
- insa_law_order is an N3/SPARQL order court (DeriveN3 legs), not a datalog recursion court;
  counted here because the dispatch listed it.

## Falsifiers

1. Artifact-skew red: rebuild `priv/graphlaw.wasm` from this tree (per docs/WASM.md with
   `WASI_SDK_PATH` + the wrap-shim RUSTFLAGS) — if `capabilities` responses then agree on
   `registry_blake3`, op_examples and op_forward_compat should proceed past the control and
   execute their `datalog` legs; if the divergence persists after a same-tree rebuild, this
   receipt's root-cause classification is wrong.
2. Coverage bound: run `cargo test --test op_examples every_example` with a same-tree-rebuilt
   artifact — if any of the 4 `datalog` op-examples fails on native or wasm, the "datalog
   engine green" claim above is falsified.
3. Anti-vacuity already embedded in the green courts: the UBO broken gate (skips
   Ownership_Resolved) is flagged by the conformance check and by pm4py fitness < 1.0; the
   crosswalk sabotage twin flips its verdict. Removing those twins should turn those courts
   vacuous-red; they are load-bearing.
4. Replay: re-run the eight commands above on this tree with the same env; the 42-green /
   3-red split must reproduce modulo wall-clock lines.

## Closing b3sum

blake3 over the receipt bytes with the final digest line removed; verify with
`sed -e '$d' receipts/2026-10-24-dialect-datalog.md | b3sum`:

`d1bbb0c3a65a41e43b7270e81350eeacb8b80628fc81e1c58697ba86f8780e2d`