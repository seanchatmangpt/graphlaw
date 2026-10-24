# Registry Drift Scope — L162 receipt (2026-10-22)

Lane: L162, canonical checkout /Users/sac/graphlaw, CARGO_TARGET_DIR=/tmp/gl-target.
Read-only on code/registry. Court receipts embedded (real exits, real digests).

## 1. Court run

`CARGO_TARGET_DIR=/tmp/gl-target cargo test --test registry_artifacts`

    test result: FAILED. 17 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out

FIVE tests failed (L147 said 4; the fifth is `artifacts_write_repin_matches_live_digests`,
which panicked at tests/registry_artifacts.rs:675:9 "--write must be idempotent when rows
are fresh", then 716:5). Failing tests and assert sites:

| test | site | assertion |
|---|---|---|
| a_committed_json_and_ttl_equal_emission | tests/registry_artifacts.rs:150 | `json drift` (committed JSON != `--print-json`) |
| b_emitter_check_flag_accepts_committed_files | tests/registry_artifacts.rs:170 | `--check failed: DRIFT capability-registry.json: first difference at line 2103` |
| artifacts_pin_file_carries_the_live_registry_and_surface_digests | tests/registry_artifacts.rs:624 | pinned `registry` row `blake3:4cd56fd5…` != live `blake3:60e4463a…` |
| artifacts_write_repin_matches_live_digests | tests/registry_artifacts.rs:675 | not idempotent (ARTIFACTS rows stale when it ran) |
| artifacts_check_flags_stale_owned_row | tests/registry_artifacts.rs:730 | `mutation applied` failed — the `replacen` of the live registry digest found nothing to mutate because the pinned row is stale |

## 2. Actual drift (live `graphlaw-registry --check`, exit=1)

    ok      capability-registry.json
    ok      capability-registry.ttl
    DRIFT   ARTIFACTS.blake3 registry: expected `registry blake3:60e4463a6339e923f3f54988c52db5da558ce17801f60fe7df0adb6d8c46f0df - capability-registry.json`, found `registry blake4cd...4cd56fd50675c0aa9cc95c0444ed8e378e40a270fc52fe859ffe4aaa83b7091f` (historical)
    ok      ARTIFACTS.blake3 surface
    DRIFT   ARTIFACTS state: expected receipt-surface `blake3:4c25000c0224a61d888b9394902b1f52b3e99a0a9565954937110b636560eb1f`, found `blake3:0cc02ca4100542f68de431eacf4e5d0dac5d51fdd96887b2bf2b2cbebcd3669d` (historical)
    registry drift: run `cargo run --features abi --bin graphlaw-registry -- --write`

(`--check` exit code verified directly: exit=1 on drift, 0 not claimed.)

Correction to L147: the drift is NOT "the registry row is current at 4cd56fd5 and only the
receipt-surface row is stale." Both owned ARTIFACTS rows are stale relative to live
computation — pinned 4cd56fd5/0cc02ca4 vs live 60e4463a/4c25000c — while
`capability-registry.json`/`.ttl` content ALREADY match emission (`ok`). The committed
JSON/TTL were re-pinned after the src edits but the ARTIFACTS rows were not, so the tree is
internally inconsistent: content current, pins lagging. All 5 failures trace to those two
stale ARTIFACTS rows; nothing else in the registry surface drifted (surface row ok, json ok,
ttl ok). One root cause: src/duckdb_eval.rs and src/sql_ast.rs byte changes (git diff
HEAD: sql_ast.rs +399 lines, 25 files changed) flow into `receipt_surface_document()`
(src/registry.rs:972-999) → `receipt_surface_blake3()` (:1519) → `registry_blake3()`
(registry doc embeds `receipt_surface_blake3`, src/registry.rs:1025-1026) → the two owned
rows. One cause, two stale rows, five test failures.

## 3. `--write` scope (src/bin/graphlaw_registry.rs)

Verified against source:

- Writes ONLY: `registry/capability-registry.json`, `registry/capability-registry.ttl`,
  and re-pins the three owned ARTIFACTS rows (`registry`, `surface`, `receipt-surface`)
  (module doc lines 4-9, `owned_artifacts_rows()` lines 55-66).
- The `wasm` row is preserved verbatim: `parse_artifacts` (lines 84-107) captures the wasm
  row, drops the owned rows, keeps unknown rows and comments; `artifacts_repin` re-emits the
  wasm row unchanged. The bin never touches `wasm/src/lib.rs` or `tests/wasm_duckdb_op_wired.rs`.
- `--check` exits 1 on drift; exit codes 0/1/2 documented at src/bin/graphlaw_registry.rs:10.

## 4. L125 files vs the receipt surface

`receipt_surface_document()` (src/registry.rs:977-980):

    const FILES: &[(&str, &[u8])] = &[
        ("src/duckdb_eval.rs", include_bytes!("duckdb_eval.rs")),
        ("src/sql_ast.rs", include_bytes!("sql_ast.rs")),
    ];

`wasm/src/lib.rs` and `tests/wasm_duckdb_op_wired.rs` are NOT in the receipt surface. A
L125 edit to either changes nothing the registry court sees; it cannot make the regen
unsafe. The only thing that would change the answer is further edits to
`src/duckdb_eval.rs` / `src/sql_ast.rs` between now and regen — those move
`receipt_surface_blake3()` (live 4c25000c… at receipt time), so regen must run the moment
src is quiescent (and re-run if those two files move again).

## 5. Expected post-state after `--write` on a quiescent src

- `registry` row → `blake3:60e4463a6339e923f3f54988c52db5da558ce17801f60fe7df0adb6d8c46f0df` (historical)
- `receipt-surface` row → `blake3:4c25000c0224a61d888b9394902b1f52b3e99a0a9565954937110b636560eb1f` (historical)
- `surface` row unchanged `blake3:da96e137…`; `wasm` row byte-identical
  `332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c 6793617 graphlaw_wasm.wasm`
- `--check` exit 0; all 5 registry_artifacts tests flip green.

