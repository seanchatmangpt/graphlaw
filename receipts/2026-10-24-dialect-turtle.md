# Dialect Verification Receipt — TURTLE surface (parse / canonicalize / serialize)

Lane: DIALECT VERIFICATION (turtle). Date: 2026-10-24. Target dir:
`/Users/sac/graphlaw/target-lane-dial-turtle`. All Rust runs:
`CARGO_TARGET_DIR=/Users/sac/graphlaw/target-lane-dial-turtle`, engine courts with
`GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm`, `--locked`.

## Header

- HEAD: `6da1d33a93f61068fa9b033bec353db78d335460`
- Dirty disclosure: working tree carries in-flight release-closeout edits —
  466 `git status --porcelain` entries at dispatch (46 modified tracked files +
  untracked receipts/transition records). Verified as-is; no file outside this
  receipt was touched; no git mutations.
- Pin b3sums (measured this session, `/opt/homebrew/bin/b3sum`):
  - `courts/code-partition/axioms.blake3` =
    `2add10ce9ff782e33b75de099cba8dc8a114ab759855f93a7be85abb18581506`
  - `priv/graphlaw.wasm` =
    `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a`
    (matches pin file `priv/graphlaw.wasm.blake3` byte-for-byte; the pytest
    runner pinned `--expect-blake3` against that same file and passed the
    digest gate, exit 84 path otherwise)

## Turtle surface (discovery result)

`vendor/purrdf-sparql-eval` (vendored parser, excluded from workspace) feeds
`graphlaw::dialect` (`src/dialect.rs`, `Dialect::Turtle` → `parse_rdf` /
`check_typed`) and `graphlaw::law::LawState::from_dataset` (RDFC-1.0 +
BLAKE3 canonicalization). Serialization exists in two forms: canonical
N-Quads emission (`canonical` op) and the tool Turtle emitter
`src/smon.rs::to_turtle`/`write_turtle`. Courts found and run below.

## Per-court witnesses (exact commands + verbatim `test result:` lines)

1. **W3C RDF 1.1 conformance (Turtle leg + census + register proofs)**
   Command: `cargo test --locked --test w3c_conformance`
   (includes generated `tests/w3c_manifest.rs` via `#[path] mod manifest;`)
   ```
   running 8 tests
   test manifest::w3c_manifest_census_notice ... ok
   test w3c_known_divergences_are_exhaustive ... ok
   test w3c_ntriples_conformance ... ok
   test w3c_nquads_conformance ... ok
   test w3c_turtle_conformance ... ok
   test w3c_trig_conformance ... ok
   test w3c_rdfxml_conformance ... ok
   test w3c_conformance_receipt_summary ... ok
   test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
   ```
   Build: `Finished `test` profile ... in 3m 51s` (fresh lane target).
   Subset rerun for the verbatim bucket print (`cargo test --locked --test
   w3c_conformance w3c_conformance_receipt_summary -- --nocapture`):
   ```
   Turtle: passed=73 refused=90 known_diverged=5 total=168
   NTriples: passed=41 refused=27 known_diverged=2 total=70
   NQuads: passed=53 refused=32 known_diverged=2 total=87
   Trig: passed=98 refused=101 known_diverged=15 total=214
   RdfXml: passed=131 refused=2 known_diverged=40 total=173
   test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.11s
   ```
   Turtle buckets (asserted as court literals): 73 positives parsed, 90
   negatives typed-refused, 5 registered divergences, 168/168 entries land in
   exactly one bucket and the total equals the manifest length.

2. **Cross-dialect canonical identity (RDFC-1.0 + BLAKE3)**
   Command: `cargo test --locked --test canonical_cross_dialect`
   ```
   test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
   ```
   Includes `nonvacuity_one_quad_perturbation_moves_id` and Turtle/TriG/N-Quads
   same-content same-id proofs.

3. **Canonical op over the ABI surface**
   Command: `cargo test --locked --test canonical_abi`
   ```
   test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
   ```
   Turtle doc in, `blake3:`-prefixed id + canonical N-Quads + quad count out;
   typed refusal on invalid Turtle; reorder/prefix-rename invariance.

4. **Typed parse-refusal surface (Turtle diagnostics)**
   Command: `cargo test --locked --test parse_refusal_court`
   ```
   test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
   ```
   Includes `syntax_from_real_bad_turtle_carries_backend_location` and
   `formula_in_rdf_layer_refused_before_backend_parse` (N3 formulas in Turtle
   refused typed).

5. **FIBO Chicago court over real Turtle ontologies**
   Command: `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm cargo test --locked --test chicago_fibo`
   ```
   test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 279.60s
   ```
   59 published cases + 2 self-proof tests (`court_declares_exactly_the_published_case_count`,
   `court_covers_every_kind`) over `tests/fixtures/fibo/*.ttl`. 279.60 s wall
   under machine load average ~130 (fan-out compile storm); no skips.

6. **Turtle serializer (smon tool emitter) — default feature run**
   Commands: `cargo test --locked --test smon_tools` / `cargo test --locked --test smon_cli_court`
   ```
   test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (smon_tools)
   test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (smon_cli_court)
   ```
   Both are feature-gated on `pack-tools` (off by default) — typed, declared in
   the file headers, and the gate itself is a court
   (`smon_coverage_is_feature_gated_and_the_gate_is_declared`, loud NOTICE).

7. **Turtle serializer (smon) — real coverage, `--features pack-tools`**
   Command: `GRAPHLAW_WASM=... cargo test --locked --features pack-tools --test smon_tools --test smon_cli_court`
   ```
   test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s   (smon_cli_court)
   test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.86s   (smon_tools)
   ```
   Real subprocess CLI court (`tiny_inline_transcript_runs_end_to_end`) and
   `to_turtle` round-trip + committed-Python-output parity
   (`pack_tools::broaden_reproduces_the_committed_python_output`).
   Disclosure: the first pack-tools invocation captured empty output (BSD-sed
   filter artifact, pipeline exit 0); the witnessed rerun above ran the same
   tree state with full output. Not a rerun-to-green: the first run was also
   exit 0.

8. **Black-box wasm Turtle legs (pytest, real packet-4 subprocess)**
   Command: `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm python3 -m pytest tests_py/test_blackbox_ops.py -k turtle -v`
   ```
   tests_py/test_blackbox_ops.py::test_run_packet4_eval_turtle_law_doc PASSED [ 50%]
   tests_py/test_blackbox_ops.py::test_wire_canonicalization_refused_on_malformed_turtle PASSED [100%]
   ================= 2 passed, 15 deselected in 148.05s (0:02:28) =================
   ```
   Real `graphlaw-run-eh` at `target/debug/` (conftest hardcodes that path; the
   binary is the shared tree's prebuilt debug build, mtime Oct 24 10:27),
   digest-gated against `priv/graphlaw.wasm.blake3`. Turtle-law-doc parse via
   wasm and typed canonicalization refusal on malformed Turtle.

9. **In-wasm FIBO Turtle fixture execution courts**
   Command: `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm cargo test --locked --test wasm_fibo_fixtures_eh`
   ```
   test tc_fibo_01_create_insert_readback has been running for over 60 seconds
   test tc_fibo_02_bounded_update_executes_and_readback has been running for over 60 seconds
   test tc_fibo_03_unbounded_delete_refused_and_table_unchanged has been running for over 60 seconds
   test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 314.60s
   ```
   (The >60 s lines are the harness slow-test notices under load ~130, not
   sleeps; the court serializes on a process-local ENGINE_LOCK.)

10. **TC-FIBO-08 COPY egress — FAILED (known red, witnessed live)**
    Command: `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm cargo test --locked --test wasm_fibo_egress_eh`
    ```
    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 128.71s
    ```
    Verbatim failure (`tests/wasm_fibo_egress_eh.rs:189:9`):
    ```
    assertion `left == right` failed: exec request failed: ... {"error":{"details":{"code":"DuckDb","message":"DuckDB execution failure: IO Error: Could not set lock on file \"/tmp/fibo_accounts_86976.csv\": Invalid argument. See also https://duckdb.org/docs/current/connect/concurrency"},"dialect":null,"engine":null,"kind":"DuckDb","message":"duckdb: DuckDB execution failure: IO Error: Could not set lock on file \"/tmp/fibo_accounts_86976.csv\": Invalid argument. ..."},"ok":false}
    ```
    No `/tmp/fibo_accounts_*` files exist now (checked). Disclosure: this
    invocation's cargo exit code was masked to 0 by the capturing pipeline
    (`| tail`); the FAILED `test result:` line above is the truth.
    Classification: **known limitation, not a new regression and not a pass** —
    EH-19 OPEN, fuel-independent typed EINVAL (musl ↔ wasi-libc fcntl ABI
    mismatch), ticket `docs/jira/v26.10.22/SQLRDF-27.md`; standing citations
    `docs/jira/v26.10.22/RECEIPTS.md` line 2045 ("tc08 egress stays OPEN on
    SQLRDF-27") and `docs/jira/v26.10.23/compliance-final-verdict.md` R-13.
    Same classification as the sibling lane receipt
    `receipts/2026-10-24-dialect-sql-rdf.md` item 2. Left red per lane rules;
    not fixed.

## Aggregate

- Test functions executed (unique, incl. the FAILED one): 108
- Passed: 107 · Failed: 1 (`wasm_fibo_egress_eh::tc_fibo_08_copy_executes_with_pid_keyed_egress`)
- Rust `--test` invocations: 10 (w3c_conformance, canonical_cross_dialect,
  canonical_abi, parse_refusal_court, chicago_fibo, smon_tools, smon_cli_court,
  smon_tools+smon_cli_court @pack-tools, wasm_fibo_fixtures_eh,
  wasm_fibo_egress_eh) + 1 subset rerun (w3c bucket print) + 1 pytest session.
- Zero failures on the parse leg, canonicalization leg, serializer leg, FIBO
  Chicago court, and both wasm Turtle fixture courts.

## Classification of every non-green

1. `wasm_fibo_egress_eh::tc_fibo_08_...` — **known limitation (disclosed red,
   witnessed live)**. Ticket SQLRDF-27 / EH-19 OPEN, citations above. Typed
   DuckDb refusal on the wire (`kind:"DuckDb"`), no crash, no silent pass.
2. `smon_tools` default-feature run (1 passed) — **typed feature-gate skip**:
   verbatim notice "tests/smon_tools.rs coverage is compiled out under default
   features; the pack-tools lane must run `cargo test --features pack-tools`";
   real coverage witnessed green under the feature (4 passed).
3. `smon_cli_court` default-feature run (0 passed) — **typed feature-gate
   skip**: whole target `#![cfg(feature = "pack-tools")]`, by design "not by
   accident" (file header); witnessed green under the feature (5 passed).
4. `manifest::w3c_manifest_census_notice` — **typed loud SKIP**: "data-only
   generated manifest (712 entries) consumed via #[path] by
   tests/w3c_conformance.rs"; the substantive courts ran in the same binary.
5. Registered Turtle conformance divergences (counted in the 5
   known_diverged bucket, court-green, each a tracked gap in
   `tests/w3c_conformance.rs`): `turtle-syntax-bad-lang-01` (malformed langtag
   accepted), `turtle-syntax-bad-base-03` (bad BASE accepted),
   `turtle-syntax-bad-n3-extras-03`, `turtle-syntax-bad-n3-extras-06` (N3
   extras accepted), `turtle-syntax-number-11` (DOUBLE `123.E+1` refused —
   positive-side divergence). Not passes; register-exhaustiveness is courted.

## Honesty line: machine-courted vs prose-declared

`docs/reference-generated/sql-rdf-conformance.md` is a projection of the
sql-rdf rec clauses (5 COURTED / 4 PROSE-ONLY), not of the Turtle dialect; no
generated doc states a Turtle percentage. Computed from this session's
witnesses instead:

- **Parse leg — 100% machine-courted over its courtable corpus**: every one of
  the 168 manifest Turtle entries lands in exactly one bucket per run and the
  totals are court literals; the corpus is read from disk at run time
  (`std::fs::read` + panic on read failure), so fixture drift flips the court.
- **Canonicalization leg — machine-courted**: 11 tests (6 cross-dialect + 5
  ABI) including a witnessed non-vacuity test.
- **Serializer leg — round-trip/parity-courted, not W3C-spec-courted**: the
  `smon` Turtle emitter is witnessed against committed Python outputs and a
  parse-back round-trip; no W3C Turtle-output conformance suite exists in this
  repo (prose-declared only).
- **Prose-declared / excluded by design**: 145 W3C Turtle *Eval* vectors are
  excluded by the generated manifest itself ("Eval tests assert graph
  isomorphism, not parse acceptance" — `tests/w3c_manifest.rs` header, lines
  4-5); media-type/IRI metadata in `src/dialect.rs` is doc-test-asserted only.
- Net: the parse/canonical/refusal surface is fully machine-courted; the eval
  semantics (145 vectors) and serializer spec-conformance are the two
  prose-declared residues, both named above with citations.

## Falsifiers

1. Any court above returning a non-`ok` `test result:` line on this HEAD with
   pin `58a44095…` falsifies the green aggregate (TC-FIBO-08 is already in its
   disclosed-red standing; a *second* new failure class there would extend it).
2. Perturbing one byte of
   `tests/fixtures/w3c/rdf-turtle/turtle-syntax-file-01.ttl` must flip
   `w3c_turtle_conformance` red; if it still passes, the corpus read is
   vacuous.
3. Closing any of the 5 registered Turtle divergences in the engine breaks the
   literal bucket counts (73/90/5) and fails `w3c_turtle_conformance` +
   `w3c_conformance_receipt_summary`; the divergence register is load-bearing
   in both directions.
4. Canonical non-vacuity is self-falsifying in-court:
   `nonvacuity_one_quad_perturbation_moves_id` (passed this session).
5. Landing an SQLRDF-27 fix must flip `wasm_fibo_egress_eh` green and force
   register/ticket removal; a fix that leaves the court red is not a fix.

Closing b3sum of this receipt is reported in the lane's final report (a
file cannot embed its own digest); verify with
`b3sum /Users/sac/graphlaw/receipts/2026-10-24-dialect-turtle.md`.
