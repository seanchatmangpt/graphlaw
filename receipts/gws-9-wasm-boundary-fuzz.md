# GWS-9 — wasm boundary fuzz court

Standing: **PASS** (2/2 tests, two consecutive green runs, deterministic replay)

- Subject: `/Users/sac/graphlaw` @ main `0af58bf` (uncommitted tree: src/lib.rs, src/rules/mod.rs dirty — court ran against the working tree, artifact freshness-verified)
- Court file: `tests/wasm_boundary_fuzz.rs` (new, self-contained wasmi host)
- GWS-2 build state: GREEN — `wasm32-wasip1` target installed; cached artifact `target/wasm-abi/wasm32-wasip1/wasm/graphlaw_wasm.wasm` fresh vs every `src/` + `wasm/src` input (mtime-bounded poll); no skip.

## Command

```
cd /Users/sac/graphlaw && cargo test --test wasm_boundary_fuzz -- --nocapture
```

## Output (run 1, --nocapture)

```
GWS-9 subject: /Users/sac/graphlaw/target/wasm-abi/wasm32-wasip1/wasm/graphlaw_wasm.wasm
GWS-9 native SQL gate: 5 tokens refused, positive-control clean AST conforms = false
test gws9_sql_injection_ast_gate ... ok
GWS-9 PASS: 312 hostile calls, 0 panics/traps, every refusal typed (kind in vocabulary), instance survived all calls (canary after each).
test gws9_wasm_boundary_fuzz ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 51.34s
```

Run 2 (determinism replay): `2 passed; 0 failed … finished in 50.34s`.

## Fuzz counts

- 312 hostile calls total, seeded (xorshift64*, SEED `0x9E37_79B9_7F4A_7C15` + truncation RNG `0x5EED_F00D_CAFE_0001`), deterministic (same counts both runs).
- Per class: (a) depth 0/1/63/64/65/1000/10000 + random 0–2000; (b) 16 MiB exactly-at / exactly-over; (c) 4 focused malformed-UTF-8 frames + mutated-string iterations; (d) 5 SQL injection tokens × {sh:pattern shapes via wasm shacl op, token-as-data, native validate_identifier + validate_ast_shacl}; (e) NaN/Infinity/-Infinity/-NaN/1e999 in numeric fields; (f) fixed + 40 random truncation cuts + loop class.
- Survival canary (`{"op":"capabilities"}` → `ok:true`) after EVERY hostile call: 312/312.

## Findings

1. **PASS, no panics/traps.** Every hostile frame produced a typed envelope; every refusal `kind` ∈ documented `RefusalKind` vocabulary (docs/refusals.md); instance survived all 312 calls.
2. **Documented fail-closed path at the 16 MiB boundary is two-staged**: frames over `MAX_REQUEST_BYTES` never reach `abi::call`'s `ResourceLimit(request_bytes)` — `gl_alloc` returns null (ffi.rs contract, "null when `len` exceeds the request limit or the reservation would exceed the outstanding cap") and `gl_call(0, len)` answers the typed `Unsupported` / "request buffer missing" refusal. Exactly-at 16 MiB is accepted (limit is `>`, not `>=`) and the 16 MiB frame allocates, parses, and answers without trap. No divergence, but tests asserting `ResourceLimit` at the FFI layer will mis-expect; the wasm side gates one step earlier than native `call()`.
3. **Positive-control gap (GWS-5 pairing, minor)**: a clean minimal AST dataset (single `sql:fromTable`/`sql:selectList`) does NOT conform to `ontologies/sql-ast.ttl` (`validate_ast_shacl` → Err) — the canonical shapes require more required properties than the minimal case exercises. All 5 injection tokens were refused typed (`InvalidIdentifier` / `ShaclViolation`), never panicked, so the gate is fail-closed; but the clean-AST non-conformance means the shapes gate is stricter than its own doc example surface. Worth a GWS-5 follow-up to pin a minimal *conforming* AST example.
4. Depth guard confirmed: depth ≤ 64 parses; 65/1000/10000 refuse `ResourceLimit` with `details.limit = json_depth`.

## Replay

```
cd /Users/sac/graphlaw && cargo test --test wasm_boundary_fuzz
```
