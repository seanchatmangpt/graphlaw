# Surface six-entry patch — staged, DO NOT APPLY (lane L172)

Application is deferred to the quiescent window: src is not quiescent
(live executor + L130 clippy in flight). Applies together with L150's
pre-staged `graphlaw-registry --write` and S1's misordering analysis
(registry JSON must be regenerated AFTER this patch compiles green, never
before — a --write over the current 2-entry surface would mint a stale
`registry/ARTIFACTS.blake3` that this patch immediately invalidates).

Verified read-only against the tree on 2026-10-22:
- `src/registry/emit_json.rs:74-77` currently carries exactly the 2-entry
  const S1 cited (`src/duckdb_eval.rs`, `src/sql_ast.rs` via
  `include_bytes!("../…")`).
- All four new target files exist: `src/sql_ast/ast.rs`,
  `src/sql_ast/refusals.rs`, `src/sql_ast/serializer.rs`,
  `ontologies/sql-ast.ttl`.

## 1. `src/registry/emit_json.rs` — replace the FILES const

Replace (lines ~74-77):

```rust
    const FILES: &[(&str, &[u8])] = &[
        ("src/duckdb_eval.rs", include_bytes!("../duckdb_eval.rs")),
        ("src/sql_ast.rs", include_bytes!("../sql_ast.rs")),
    ];
```

with:

```rust
    const FILES: &[(&str, &[u8])] = &[
        ("src/duckdb_eval.rs", include_bytes!("../duckdb_eval.rs")),
        ("src/sql_ast.rs", include_bytes!("../sql_ast.rs")),
        ("src/sql_ast/ast.rs", include_bytes!("../sql_ast/ast.rs")),
        (
            "src/sql_ast/refusals.rs",
            include_bytes!("../sql_ast/refusals.rs"),
        ),
        (
            "src/sql_ast/serializer.rs",
            include_bytes!("../sql_ast/serializer.rs"),
        ),
        // .ttl is pure UTF-8 data with no build-time transform, so byte
        // identity is the invariant the receipt-surface pin needs; include_str!
        // would additionally assert valid UTF-8 at compile time but yield an
        // identical &[u8] via .as_bytes(). include_bytes! keeps every entry in
        // FILES the same type (&[u8]) without a per-entry map, and digests the
        // exact on-disk bytes either way.
        (
            "ontologies/sql-ast.ttl",
            include_bytes!("../../ontologies/sql-ast.ttl"),
        ),
    ];
```

Path resolution note (S1's correction): the const lives in
`src/registry/emit_json.rs`, so `../` resolves to `src/` and the module
files need `../sql_ast/…`, while the ontology needs `../../ontologies/…`.
The `"path"` string in the JSON row stays manifest-rooted
(`ontologies/sql-ast.ttl`), which is what
`tests/registry_artifacts.rs` reads back via
`CARGO_MANIFEST_DIR.join(path)` — the readback path and the
include_bytes! path are intentionally different strings.

## 2. `tests/registry_artifacts.rs` (line ~907) — bump the assert

Replace:

```rust
    assert_eq!(files.len(), 2, "duckdb_eval + sql_ast pinned");
```

with:

```rust
    assert_eq!(files.len(), 6, "receipt-surface pins six sources");
```

The per-file loop body below the assert is path-generic and needs no
change: it reads each `path` from disk, recomputes blake3, checks byte
count, and runs the flip-one-byte drift negative control. The `blake3_of`
canonical-doc assertion (`pin == canonical(embedded doc)`) likewise needs
no change — it operates on the whole document.

## 3. include_str! vs include_bytes! (decision record)

`include_bytes!("../../ontologies/sql-ast.ttl")` is the chosen form:
(a) type uniformity with the other five entries — FILES stays
`&[(&str, &[u8])]` with no coercion; (b) the pin's invariant is
byte-identity of the shipped surface, which is exactly bytes. `include_str!`
would only add a compile-time UTF-8 assertion; if that assertion is later
wanted, it can be added as a separate `const _:
&str = include_str!("../../ontologies/sql-ast.ttl");` without changing the
digest path.

## 4. Application procedure (quiescent window)

Ordering matters (S1's misordering analysis: a stale --write before the
patch bakes a 2-entry pin into `registry/ARTIFACTS.blake3` that then
refuses the patch itself via drift detection).

1. Confirm quiescence: no live executor, L130 clippy landed, `cargo check` green on the pre-patch tree.
2. Apply §1 to `src/registry/emit_json.rs` and §2 to `tests/registry_artifacts.rs` (this patch + L150's pre-staged diff land together).
3. `cargo check` — compile gate.
4. `cargo test --test registry_artifacts` — readback court recomputes all six per-file digests and the pin against real disk (Chicago readback; the drift negative control must flip per file).
5. `cargo test` — full host suite.
6. Run `graphlaw-registry --write` (L150's pre-stage) — regenerates `registry/capability-registry.json` and `registry/ARTIFACTS.blake3` over the NEW 6-entry surface.
7. `cargo test --test registry_artifacts` again if --write touched any file the court reads; then `bash scripts/mutation_court.sh`.
8. Coordinator commits (lane L172 does not own git).

Rollback: revert the two edits; the pre-patch 2-entry pin in
`registry/ARTIFACTS.blake3` remains valid because --write runs only after
step 5 is green (fix-forward discipline; no reset).

## 5. Expected post-state

- `receipt_surface.files` has 6 rows in FILES order: duckdb_eval.rs,
  sql_ast.rs, sql_ast/ast.rs, sql_ast/refusals.rs,
  sql_ast/serializer.rs, ontologies/sql-ast.ttl — each row
  `{path, blake3: "blake3:<64hex>", bytes}`, all recomputable from disk.
- `receipt_surface_blake3` = BLAKE3 of the canonical (serde_json Value)
  serialization of that document — changes as a consequence; anything
  pinning the old 2-entry value must be regenerated in the same window.
- `tests/registry_artifacts.rs` — 22 passed, 0 failed
  (21 existing + the widened pin court still one test).

## 6. Execution receipt (lane P7, 2026-10-22)

Applied per §1-§2, S1's include-path corrections included. Ordered per §4:
patch -> check -> court -> --write -> --check -> court.

- applied-at: 2026-10-22, lane P7, CARGO_TARGET_DIR=target-lane-p7
- before: `receipt_surface_blake3 = blake3:e0215f61...` (V3, 2-entry surface)
- after: `receipt_surface_blake3 = blake3:4f28d0f1b02e9ef0b3b70a64a37af798782d5509f8f25fb685ff53707ed270a2` (historical)
- files order: src/duckdb_eval.rs, src/sql_ast.rs, src/sql_ast/ast.rs, src/sql_ast/refusals.rs, src/sql_ast/serializer.rs, ontologies/sql-ast.ttl
- `cargo check --lib` exit 0
- post-patch pre-write court run: 4 failures (exactly the courts comparing
  committed ARTIFACTS.blake3 to live emission — expected; the committed pin
  was still the 2-entry surface), 18 passed
- `graphlaw-registry --write` exit 0: rewrote capability-registry.json/.ttl,
  repinned ARTIFACTS.blake3
- `graphlaw-registry --check` exit 0
- post-write court: 22 passed, 0 failed (per §5)
- wasm row preserved byte-identical: `wasm 332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c 6793617 graphlaw_wasm.wasm` (never re-pinned by graphlaw-registry, per ARTIFACTS.blake3 header line 6)
- deviations from spec: none
