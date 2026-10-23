# Examples Census — lane L70 — 2026-10-22

Subject: /Users/sac/graphlaw @ 615dbfa (rev 615dbfaf94d407cffe174af02804d0dbfb3f8f1d)

## Gate

    CARGO_TARGET_DIR=target-lane70 cargo build --examples
    # Finished `dev` profile ... in 1m 49s — EXIT=0 (all 15 examples compiled)

## Verdict counts: 15 COMPILED+RAN, 0 COMPILE-FAIL, 0 SKIP-TIMEOUT

Note: the two *_vector_gen + 3 oracle/vector generators write fixtures into
../wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw/ (cross-repo side
effect — observed, disclosed, not lane-owned).

## Per-example verdicts

| example | verdict | command | exit | last output line |
|---|---|---|---|---|
| allen_datalog | COMPILED+RAN | `cargo run --example allen_datalog` | 0 | `RESULT: exit 0` |
| asp_tableaux_vector_gen | COMPILED+RAN | `cargo run --example asp_tableaux_vector_gen` | 0 | `generated vectors; rev 615dbfaf...; now run with --verify` |
| bayes_pop_vectors | COMPILED+RAN | `cargo run --example bayes_pop_vectors` | 0 | `generated 9 vector(s); rev 615dbfaf...; now run with --verify` |
| breed_oracle_gen | COMPILED+RAN | `cargo run --example breed_oracle_gen` | 0 | `generated 4 vector(s); rev 615dbfaf...; now run with --verify` |
| cold_chain_biologics_verification | COMPILED+RAN (cited, L41 witness) | `cargo run --example cold_chain_biologics_verification` | 0 | L41 receipt; not rerun per lane task |
| event_calculus_vectors | COMPILED+RAN | `cargo run --example event_calculus_vectors` | 0 | `generated 4 vector(s); ... dual-route agreement on all scenarios` |
| gws8_shacl_native | COMPILED+RAN | `cargo run --example gws8_shacl_native -- /tmp/gws8_req.json 50` | 0 | `{"engine":"native","n":50,...,"response_blake3":"310279e8...","response_bytes":114}` |
| naive_physics_vectors | COMPILED+RAN | `cargo run --example naive_physics_vectors` | 0 | `wrote .../ENCODERS.md` |
| plan_replay_oracle_gen | COMPILED+RAN | `cargo run --example plan_replay_oracle_gen` | 0 | `generated 9 vector(s); rev 615dbfaf...; now run with --verify` |
| policy_admission | COMPILED+RAN | `cargo run --example policy_admission` | 0 | `ADMITTED reachable=["g","s0"] goals=["g"] entries=[("s0","flip")]` |
| quickstart_plan | COMPILED+RAN | `cargo run --example quickstart_plan` | 0 | `final state: blake3:3be6530bdf3597afdcc5f60a2a7a0c15a952a1e8f8dd440e0a0bf49082e05896` |
| receipt_feedback | COMPILED+RAN | `cargo run --example receipt_feedback` | 0 | `after record: admitted by 'admit:require-receipt'` |
| refused_plan | COMPILED+RAN | `cargo run --example refused_plan` | 0 | `REFUSED at index 1, action 'paint'` |
| sql_in_rdf_duckdb | COMPILED+RAN (cited, L41 witness) | `cargo run --example sql_in_rdf_duckdb` | 0 | L41 receipt; not rerun per lane task |

## Deviations + notes

- **gws8_shacl_native** is an arg-required bench harness
  (/Users/sac/graphlaw/examples/gws8_shacl_native.rs:16-17 `.expect` on args;
  first run bare `cargo run --example gws8_shacl_native` = exit 101, usage
  panic — expected, not a defect). No committed request.json fixture exists
  anywhere in the repo (repo-wide grep, only reference is the example itself);
  a minimal valid `shacl` op request was constructed at /tmp/gws8_req.json
  (`{"op":"shacl",...}`) and the harness ran clean: n=50, 21.7k ops/sec,
  response_blake3 `310279e89e3c03b2d36a5dcee1cce4856991d9cc86f13a64589ae6071a6f944f`,
  114 response bytes.
- No example exceeded 120s; zero SKIP-TIMEOUT entries.
- Cross-repo fixture writes land in `../wasm4pm/...` — disclosed, outside lane
  ownership, byte-level drift there is a wasm4pm-cognition concern.
