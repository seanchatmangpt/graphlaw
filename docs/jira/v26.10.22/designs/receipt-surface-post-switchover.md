# Receipt-surface pin coverage after the sql_ast / registry switchover (N27, 2026-10-22)

READ-ONLY analysis. src was not quiescent during this analysis (a live
switchover executor may have been active); the code change below is
PROPOSED-NOT-APPLIED.

## 1. What the surface pins NOW

The receipt surface lives at `src/registry/emit_json.rs:73-92`
(`receipt_surface_document()`), reached through the `src/registry.rs` facade
(`pub use emit_json::*`). Its own doc-comment states the purpose:

> BLAKE3 digests of the receipt-critical compiler/eval sources, pinned so a
> receipt's standing can be tied to the exact bytes of the compiler that minted it.

(`src/registry/emit_json.rs:70-72`.) The pinned FILES list is hard-coded:

```rust
const FILES: &[(&str, &[u8])] = &[
    ("src/duckdb_eval.rs", include_bytes!("../duckdb_eval.rs")),
    ("src/sql_ast.rs", include_bytes!("../sql_ast.rs")),
];
```

The registry JSON at `registry/capability-registry.json:2100-2115` reflects
exactly this two-file list (`src/duckdb_eval.rs`, `src/sql_ast.rs`) with pin
`blake3:e0215f61...`.

**Live-state correction to the dispatch premise**: as of this read,
`src/sql_ast.rs` is NOT yet a 130-line facade. It is 2,338 lines /
100,074 bytes and still carries the semantic compile/admission core
(`validate_ast_shacl` at line 1031, `compile_turtle` at line 1137,
`DatasetReader`, the compile memo, and the test modules). The switchover is
mid-flight: the facade already declares `mod ast; mod refusals; mod serializer;`
with `pub use ast::*` etc. (lines 18-27), and the split files
(`src/sql_ast/ast.rs` 19,091 B, `refusals.rs` 28,243 B, `serializer.rs` 8,348 B
— 55,682 B total) are live and re-exported, but the facade retains ~44 KB of
residual monolith. (The 130-line facade that does exist today is
`src/registry.rs` — 130 lines / 4,809 B of pure `mod` decls + tests.)

## 2. The pin gap, quantified

The pin covers only what `include_bytes!` names. Post-switchover (facade =
mod decls only), the pinned bytes would be:

| file | pinned | semantic |
|---|---|---|
| `src/duckdb_eval.rs` | 74,573 B | yes (eval + receipts) |
| `src/sql_ast.rs` facade | ~4.8 KB (registry-facade analog) | no — mod decls, `pub use`, allow attributes |
| `src/sql_ast/ast.rs` | 0 — **not pinned** | 19,091 B (AST types, `Operator::from_iri`, `validate_identifier`) |
| `src/sql_ast/refusals.rs` | 0 — **not pinned** | 28,243 B (typed refusals, `refusal_arm`) |
| `src/sql_ast/serializer.rs` | 0 — **not pinned** | 8,348 B (parameterized SQL emission) |

- Interim gap TODAY (2,338-line facade): 55,682 B of live, re-exported
  compiler semantics are outside the pin (19,091 + 28,243 + 8,348).
  The facade pin still covers `compile_turtle`/`validate_ast_shacl` today, so
  the surface is currently under-inclusive, not vacuous.
- Post-switchover gap: the pinned sql_ast entry collapses to a
  mod-decl facade (~4.8 KB by the registry-facade analog), i.e. ~95% of the
  pinned sql_ast bytes would be non-semantic. The pin's own doc-comment law
  ("receipt's standing tied to the exact bytes of the compiler that minted it")
  is then unmet for AST/refusal/serializer logic: a semantic edit to
  `refusals.rs` would not move `receipt_surface_blake3`, the drift court
  (`tests/registry_artifacts.rs:890 receipt_surface_pin_matches_live_files_and_refuses_drift`)
  would keep passing, and the surface pin is vacuous for those files.

## 3. Should it pin: computed concatenation vs per-file digests

Keep the per-file digest shape — the surface document is
`{"schema": "graphlaw.receipt-surface/1", "files": [{path, blake3, bytes}]}` and
the drift court reads `path` off disk (`tests/registry_artifacts.rs:890-930`, per-file disk read at 904-911). The change is a list
extension, not a redesign: pin the facade (for structural drift, e.g. a
new `mod` or a removed `pub use`) **and** the split files (for semantic
drift). Dropping the facade pin would leave mod-decl drift unpinned; dropping
the split pins leaves the existing vacuity. Pin all of:

- `src/duckdb_eval.rs` (unchanged)
- `src/sql_ast.rs` (facade, structural)
- `src/sql_ast/ast.rs`
- `src/sql_ast/refusals.rs`
- `src/sql_ast pin list covers the admission surface named in the doc-comment, and the mod-decl facade is genuinely
  receipt-critical structure (a removed `pub use` changes the public surface
  the receipt law binds to).
- `src/sql_ast/serializer.rs`

Do NOT pin via a computed concatenation digest (single blake3 over a concat):
the court asserts per-file digests against on-disk bytes, and the receipt law
binds receipts to files, not to a faceless blob. Per-file {path, blake3, bytes}
rows survive file renames legibly and keep the drift negative-control per file.

## 4. Exact code change — PROPOSED-NOT-APPLIED

File: `/Users/sac/graphlaw/src/registry/emit_json.rs`, the FILES const at
lines 74-77. Replace:

```rust
const FILES: &[(&str, &[u8])] = &[
    ("src/duckdb_eval.rs", include_bytes!("../duckdb_eval.rs")),
    ("src/sql_ast.rs", include_bytes!("../sql_ast.rs")),
    ("src/sql_ast/ast.rs", include_bytes!("../sql_ast/ast.rs")),
    ("src/sql_ast/refusals.rs", include_bytes!("../sql_ast/refusals.rs")),
    ("src/sql_ast/serializer.rs", include_bytes!("../sql_ast/serializer.rs")),
];
```

(Replaces the two-entry list at `src/registry/emit_json.rs:74-77`. Relative
`include_bytes!` paths are correct for that file's location: `../sql_ast/ast.rs`
resolves to `/Users/sac/graphlaw/src/sql_ast/ast.rs`.)

Consequential court change (tests lane, not this lane's src ownership):
`tests/registry_artifacts.rs:907` asserts `files.len() == 2` — must become 5 in
the same commit, else the court that exists to refuse drift becomes the thing
that freezes the surface at 2 entries.

## 5. Timing constraint

This must land in the same quiescent window as the final registry regen.
Order within the window:

1. Apply the emit_json.rs FILES extension (5 entries).
2. Bump `tests/registry_artifacts.rs:907` len assert 2 -> 5.
3. Run `graphlaw-registry` regen so `registry/capability-registry.json` /
   `.ttl` carry the new `receipt_surface_blake3` and five-file
   `receipt_surface` document.
4. `cargo test --test registry_artifacts` green.

If the regen lands before the FILES change, the committed registry pins the
facade-only digest (pin gap of section 2, permanently — regen is the only
thing that rewrites `registry_blake3`/`receipt_surface_blake3` in the pinned
artifacts). If it lands after, `tests/registry_artifacts.rs:892-897` fails
(live recompute ≠ committed pin) and the checkout is red until the next regen.
Either ordering error is the "surface digest is wrong again" failure the
coordinator flagged. Because `include_bytes!` digests are computed at crate
compile time, the window must also be compile-quiescent: no concurrent edits
to any of the five pinned files between step 1 and step 3, or the committed
pin will be a digest of a compile-time snapshot of a moving file.

## 6. Interim state, honestly

A regen run NOW (mid-switchover) pins:
- `src/duckdb_eval.rs` 74,573 B — fully covered, semantic.
- `src/sql_ast.rs` 100,074 B (2,338 lines) — covered, and today this still
  contains `compile_turtle` / `validate_ast_shacl` / DatasetReader, so the pin
  is meaningful but under-inclusive: the 55,682 B of split-file semantics
  (ast/refusals/serializer) are live in every build via `pub use` and are
  not pinned.

So the interim surface is under-inclusive (~35.8% of the compiler surface,
55,682 of 155,755 compiler bytes, unpinned), not vacuous. The vacuity arrives
only when the facade collapses to mod decls. That is the deadline: the FILES
extension must be in the final regen window, not after it.

## Receipt

- Files read (absolute paths): `/Users/sac/graphlaw/src/registry/emit_json.rs`,
  `/Users/sac/graphlaw/src/registry.rs`, `/Users/sac/graphlaw/src/sql_ast.rs`,
  `/Users/sac/graphlaw/src/sql_ast/{ast,refusals,serializer}.rs` (sizes via
  wc), `/Users/sac/graphlaw/tests/registry_artifacts.rs` (surface court,
  lines 890-930), `/Users/sac/graphlaw/registry/capability-registry.json`
  (lines 2100-2115), `/Users/sac/graphlaw/docs/refusals.md` (line 49).
- No src edits, no test edits. One report file written (this one).
- Commands: `wc -c` / `wc -l` over the five compiler files and registry
  modules; `grep -rn "include_bytes\|receipt_surface" src`; `head`/`sed` reads
  cited above. No cargo run needed — this is a static-analysis lane; the
  surface code and courts were read directly on disk.
