# R2 — capability route table (design record)

Withdrawn as an implementation plan. DuckDB is not a Cargo feature and is not
`CompiledOut` on wasm. `admit_sql_rdf` always calls `compile_turtle`.
`REFUSAL-04` is the non-document-backed builtin reader (`docs/refusals.md`).
The sections below record the rejected route. Do not implement them.

## Current state
- `admit_sql_rdf` compiles every UTF-8 `sql` document through `compile_turtle`. There is no wasm `capability_refusal()` arm.
- `REFUSAL_04_CAPABILITY_REFUSAL` (`src/tier.rs`) is the document-backed builtin trigger (`docs/refusals.md`).
- Registry `ENGINES` (`src/registry.rs:170`) descriptive only; `capabilities()` (`src/abi.rs:473-488`) exposes no engines today.
- `NATIVE_ONLY_OPS = ["duckdb"]` (`src/registry.rs:889-897`); wasm mirror `wasm/src/abi_meta.rs` pins OPS+ERROR_CODES only.

## Design
- No new op. Extend `capabilities` response with `engines: [{name, tiers, admission}]`,
  `Admission ∈ {Native, CompiledOut, Remote}` (`#[non_exhaustive]`), built from
  `ENGINE_ROUTES: &[EngineRow]` (sibling of `DialectRow`).
- **Target-invariance:** row SET (names+tiers) identical on both targets; wasm carries
  FrozenDuckdb `CompiledOut` visibly (deliberate inversion of NATIVE_ONLY_OPS).
- **Option B digests:** hashed registry document carries names+tiers only (`engine_routes`
  key); `admission` is a runtime fact in the capabilities response, never hashed →
  native≡wasm digest courts stay green.
- **Collapse:** `route_for_tier(Tier::Relational)` first-match; `admit_sql_rdf` becomes one
  body — Native → compile, CompiledOut → REFUSAL-04, else fail closed. Compile body stays
  link-gated (`#[cfg(feature="duckdb")]` at the call site); `debug_assert!` in the Native
  arm that the substrate is linked; cross-check court asserts row==Native ⇔ feature linked.
- **Future engines as rows:** SQLite appended after FrozenDuckdb = wasm fallback with zero
  code change (first-match order); `Remote` arm = compile + tag for host execution; new
  tiers slot into `#[non_exhaustive] Tier`.

## Finding (feed-forward → SQLRDF-17)
`compile_turtle` is publicly callable (`graphlaw::sql_ast::compile_turtle`) without passing
tier admission — the wasm fail-closed guarantee ("no SQL parse or execution of any kind",
`docs/refusals.md:45`) is bypassable today by any caller that skips `admit_*`. Required:
`pub(crate)` or route public exposure through `route_for_tier`. Strengthens the C2 case.

## Courts
Target-uniform `tests/engine_route_court.rs`: per-row expected admission on that target +
per-row behavior (admit SQL_DOC / REFUSAL-04 typed); collapses `src/tier.rs:186-219` into
one court. `capabilities_digest.rs` field list + positive control extended (admission
excluded from digest inputs). REFUSAL-04 message unchanged.

## Migration order
1. `ENGINE_ROUTES` + `route_for_tier` + `Admission` (additive). 2. capabilities `engines`
field + registry `engine_routes` key (names+tiers hashed). 3. Flip `admit_sql_rdf`;
collapse cfg-split tests. 4. SQLRDF-17 (admission bypass). 5. `docs/refusals.md:45`
remediation wording (receipt-gated, same commit).
What breaks: capabilities_digest courts, abi-manifest contract, registry regen,
refusals.md row. NOT affected: `wasm/src/abi_meta.rs` (no new op/code), wasm ABI courts.
