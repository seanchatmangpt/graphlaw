# wasmi exception-handling research (engine-tier host pre-A1')

Lane P1, 2026-10-22. Question: does any wasmi release support the wasm
exceptions proposal, and is wasmtime 48's new-EH default or opt-in?
Feeds the ENGINE-tier host decision (wasmtime-now vs wasmi-upgrade-wait vs
dual-host) in `dual-artifact-policy-design.md`.

## Local pin receipts (wasmi 2.0.0, vendored source)

Source: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wasmi-2.0.0/`

- `src/engine/config.rs` (default_features, ~L77-100): `WasmFeatures::empty()`
  then only `MUTABLE_GLOBAL, MULTI_VALUE, MULTI_MEMORY, SATURATING_FLOAT_TO_INT,
  SIGN_EXTENSION, BULK_MEMORY, REFERENCE_TYPES, GC_TYPES, TAIL_CALL,
  EXTENDED_CONST, FLOATS` (+ conditional SIMD / MEMORY64 / WIDE_ARITHMETIC).
  **No EXCEPTIONS bit is set and no setter exists.** Confirms L116/W2 witness
  (file:line reported there as `config.rs:78-95`).
- `wasmi_core-2.0.0/src/features/` : grep for `EXCEPTIONS|exceptions` →
  zero matches. The feature flag does not exist in the pin at all.
- `wasmi-2.0.0/src/module/export.rs:98` — panic string: "wasmi does not
  support the `exception-handling` Wasm proposal".
- `wasmi-2.0.0/src/module/import.rs:86` — same text on tag imports.
- `Cargo.toml:16` — `version = "2.0.0"`, `rust-version = "1.86"`.

## Upstream wasmi status (network receipts, 2026-10-22)

- crates.io `wasmi`: max_stable_version **2.0.0** (2026-09-01); prior stable
  line 1.1.0 (2026-06-12). No release newer than the local pin.
- GitHub issue **wasmi-labs/wasmi#1137** "Implement the Wasm
  `exception-handling` proposal": **still OPEN**, created 2024-07-24, last
  updated 2026-01-08, 2 comments, no linked PR. Upstream says Phase 4 moved
  the proposal to stabilization but no implementation has landed.
- GitHub issue search `exception in:title` on wasmi-labs/wasmi: total_count 2
  — #1137 (open) and #2042 (unrelated ESP32 issue). **No EH PR exists.**

### Verdict per option

| Option | EH support? | Source |
|---|---|---|
| wasmi ≤ 1.1.0 | NO | #1137 open since 2024-07; no EH commit in any release line |
| wasmi 2.0.0 (local pin, newest stable) | NO — hard gap, not config: no feature bit, panic strings in module/{export,import}.rs | vendored source, receipts above |
| wasmi future/unreleased | UNCERTAIN — #1137 open, zero PR activity observed as of 2026-01-08 last update | GitHub issue |
| wasmtime 48 | YES, new EH (try_table/throw) **default-ON**, behind crate feature `gc` | docs.rs/wasmtime/48.0.0 `Config::wasm_exceptions` — "This is true by default. Available on crate feature gc only." |
| wasmtime 48 legacy EH | opt-in via `legacy_exceptions` config; witnessed REFUSAL at default config | `wasi-sdk-reproducibility.md:58-59` (W6) |

## Ecosystem reference (wasmtime timeline)

Wasmtime's `wasm_exceptions` default-true + the observed local W6 witness
(default config refused legacy-EH but accepted try_table at default, L144)
agree: new EH is the stabilized default in wasmtime 48; legacy EH is the
opt-in compatibility knob. No wasmtime release-pin evidence contradicting
this was found; the wasmtime CHANGELOG.md fetch for v48.0.0 returned empty
(tag may not carry that file), so the exact wasmtime version where new EH
flipped default-ON is **UNVERIFIABLE without a build/fetch beyond this
report's receipts** — not load-bearing for the decision.

## Recommendation input for the engine-tier host

- **wasmtime-now**: supported by receipts. New-EH artifacts host on wasmtime
  48 at DEFAULT config (only the `gc` crate feature must be on; default
  features include it). No config knob needs flipping for new EH.
- **wasmi-upgrade-wait**: indefinite — the gap is version-shaped as "no
  version has it and the tracking issue has been open 2+ years with no PR".
  Waiting has no known landing point.
- **dual-host**: the collapse condition in
  `dual-artifact-policy-design.md:116` (wasmi gains EH upstream or via fork)
  has no upstream path in evidence; a fork of wasmi porting EH is the only
  wasmi-side route, cost unquantified.
- Net: new-EH host decision resolves to **wasmtime-now** unless the fork
  cost of wasmi is admitted as a work order. wasmtime 48 default config is a
  sufficient host for new-EH (try_table) artifacts; wasmi is not hostable at
  any released version.

## UNVERIFIABLE without a build

1. Exact wasmtime version where `wasm_exceptions` flipped default-true
   (changelog fetch empty for v48 tag) — informational only.
2. Whether a wasmi fork porting EH (issue #1137 scope) is weeks vs quarters —
   no PR exists to size.
3. Runtime behavior of wasmtime 48 `legacy_exceptions=true` on the actual
   frozen-duckdb blob (W6 witnessed the refusal only) — belongs to the A1'
   witness run, not this report.

## Verify commands (replay)

    REG=~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f
    grep -n -i exception $REG/wasmi-2.0.0/src/module/{export,import}.rs
    grep -rn 'EXCEPTIONS' $REG/wasmi_core-2.0.0/src/features/   # zero matches
    curl -s 'https://api.github.com/repos/wasmi-labs/wasmi/issues/1137' | jq .state   # open
    curl -s 'https://docs.rs/wasmtime/48.0.0/wasmtime/struct.Config.html' | grep -o 'wasm_exceptions[^<]*' | head
