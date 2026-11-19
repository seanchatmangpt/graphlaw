# PCCF-SERVER-CLIENT-SPLIT

Standing: UNVERIFIED

Plan + scaffold receipt for the server/client PCCf split (cindy wave
v26.11.19 arch lock #6). No `admit_pccf` court has been run. Stub surfaces
exist; full SRL admission is not ALIVE.

## Decision

| Plane | Path | Constraint |
|---|---|---|
| Server | `src/pccf.rs` leaf (`pub mod pccf` in `lib.rs`) | `admit_pccf` uses stratified **SRL `NOT { }`** via `purrdf_datalog::schedule::stratify_rules`. **Never** extend `law::Step`. **Never** add a `duckdb` cargo feature — `frozen-duckdb` is already unconditional (repo law; reason `priv/graphlaw.wasm` ≈ 62 909 537 B / ~63 MB). |
| Client | `/Users/sac/cgap/crates/pccf-verifier-wasm` | Zero logic-deps + `wasm-bindgen` boundary; 736 B structural verify; target `<250 KB`. **Not** nested in `cgap-ocel2` (ffi/arrow/parquet defeat the size fence). |

Wire SoT: `/Users/sac/cindy/docs/sjira/v26.11.19/receipts/L-pccf-736-envelope.md`
(`64 + 672 = 736`).

## Why not a Step variant

`law::Step` is the leased transition vocabulary (`admit:shacl`, `derive:n3`, …).
PCCf admission is a **free-function leaf** over an already-parsed envelope +
carrier, scheduled by SRL strata — not a new transition name. Adding
`Step::AdmitPccf` would force capability-registry / lease-scope / receipt-step
renames across the abi and courts for no gain.

## Why not a duckdb feature

`Cargo.toml` line ~76–80: DuckDB links into the one unit on every target,
wasm32 included. Splitting it behind a feature would change the wasm EH
contract and the `graphlaw-wasm` digest pin. Client size isolation is achieved
by a **separate crate**, not by feature-gating DuckDB out of graphlaw.

## Scaffold landed this session

- `src/pccf.rs` — constants + `parse_pccf` length fence + `admit_pccf` stub that
  refuses until SRL rules are wired (`PccfRefusal::NotImplemented`).
- Client stub crate under cgap (see above).

## Next (not this lane)

1. Unify affidavit `curve25519-dalek-ng` → `curve25519-dalek` v4.
2. Author SRL rule text with `NOT { }` only; call `stratify_rules` before eval.
3. Wire Bulletproof open into `pccf-verifier-wasm` behind an optional feature
   that still excludes graphlaw/duckdb.
4. Court: length flip refuses; stratify cycle refuses; client wasm `<250000` B.

## See Also

- cindy `docs/sjira/v26.11.19/receipts/L-pccf-server-client-split.md`
- cindy `docs/sjira/v26.11.19/receipts/L-pccf-736-envelope.md`
- `src/law.rs` `enum Step` (do not modify)
- `purrdf` `crates/datalog/src/schedule.rs` `stratify_rules`
