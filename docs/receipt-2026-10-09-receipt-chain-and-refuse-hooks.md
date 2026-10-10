# Session receipt — receipt-chain port + refuse hooks (2026-10-09)

Branch `docs/doc-hdit-scaffold-gl`. All changes listed are landed in the
working tree but **uncommitted and unpushed**.

## What landed

Two independent changes, one tree:

1. **Receipt-chain family** (`src/receipt_chain.rs`, new): verbatim port of
   praxis-core's receipt-chain family so praxis-core can retire. Public API:
   `Andon`, `Obligation`, `ReceiptMeta`, `build_admission_frame`,
   `chain_from_frame`, `RefusalScenario`, `RefusalCategory`, `ReceiptRecord`,
   `ChainRule`, `ChainStanding`, `ChainVerification`, `ChainRuleMonotonicity`,
   `RECEIPT_RECORD_VERSION`, the `epoch` submodule, and the `frame` submodule
   (99-byte `OcelCausalFrame::to_hash_bytes` layout, `DenialPolarity` lanes,
   `chain_hash(t+1) = BLAKE3(chain_hash(t) || frame_bytes(t+1))` — verbatim
   from `bcinr-powl-receipt 26.7.28`). A crates.io dependency on
   `bcinr-powl-receipt` was attempted first and is infeasible on stable
   1.96.0 (`bcinr-powl` → `wasm4pm-compat` uses nightly-only
   `#![feature(...)]`, E0554) — hence the port. `Cargo.toml` gains the
   BLAKE3/hash deps plus explanatory comments; `src/lib.rs` re-exports
   `ReceiptRecord` and the new `ReceiptValidator`
   (`tests/receipt_chain_validator_test.rs`).

2. **Refuse hooks** (`src/hooks.rs`): new `Effect` enum
   (`EmitDelta` | `Refuse`, parsed from `kh:effect`), `Verdict`
   (`Fired` | `Refuse(String)`), `HookVerdict { hook, round, row, verdict }`,
   and `Materialized::verdicts` (one per firing, same order as
   `firings`). `Hook` gains `effect` and `reason` (`kh:reason`, optional —
   defaults to `kh:name` in the verdict payload). `Materialized` now
   runs the CONSTRUCT only for `EmitDelta`; a refuse firing emits no
   delta and surfaces its reason as a verdict, never an error. The
   consumer decides what to refuse.

## Wire-format compatibility

- The `praxis-chain/*` frozen strings (record `version`, `schema`
  `"ggen-receipt/v2"`, frame byte layout, chain rule) are ported
  byte-exact; the FM-CHAIN-009 golden test pins this against ggen's
  committed TCPS fixture (`chain_hash` `d04c6d08…`, pre-fold, sealed
  2026-07-22) and the post-fold recompute (`bebae299…`), plus a real JSON
  round trip (FM-CHAIN-014's regression shape). Changing the layout or
  chain rule must fail that test.
- The hooks change is **backward-compatible**: existing packs declare
  `kh:effect "emit-delta"` only, and every emit-delta firing now
  additionally surfaces `Verdict::Fired` in `verdicts` (regression-tested
  against the self-monitoring pack). Old consumers that ignore `verdicts`
  see identical `state`/`firings` behavior; unknown `kh:effect` values are
  a typed `RefusalKind::Unsupported` at load, as before.

## Epoch-port completion (update 2026-10-10)

The migration-epoch port landed in `src/receipt_chain.rs`'s `epoch`
submodule (~line 1192 onward): `ReceiptRecordV1Legacy` (pre-migration
v1-only reader), `MigrationReceipt` (fixed law identity
`MIGRATION_LAW_1_TO_2` = `"M_1_to_2"`), and the differential-vs-praxis
proof in `tests/receipt_epoch_migration_port_test.rs` — **4/4 passed**.
This closes the last seam gap for praxis-core retirement.

## Verification (final tallies)

`TMPDIR=/tmp CARGO_TARGET_DIR=/tmp/lanegd4-target cargo test
--workspace --no-fail-fast`: **428 passed, 0 failed, 1 ignored**
(corpus_conformance) across graphlaw lib + integration suites, eyeron,
graphlaw-wasm, and 27 doctests. Includes the 4 golden, 4 validator, and
4 migration-epoch port tests, and the refuse-hook tests in
`tests/knowledge_hooks.rs` (11 passed). No retry was needed — single
clean run, no concurrent churn observed.

Refresh 2026-10-10 (lane graphlaw-doc-tallies-2, fresh target dir
`/tmp/lanegd3-target`): **443 passed, 0 failed, 1 ignored** across 48
test binaries (+27 doctests). The +15 over 428 is concurrent session
churn from other lanes (wasm ABI / op-differential / registry-limits
edits), all green on a single clean run — no retries needed.

`cargo clippy -p graphlaw --all-targets`: **clean, 0 warnings**
(51.9s). (The `-p` scoping E0433 issue on the workspace invocation
remains a known pre-existing condition, not hit at crate scope.)

## Consumers

- ggen seam rewire to `graphlaw::receipt_chain` is in flight (the port's
  stated purpose is praxis-core retirement; see ggen
  `docs/v26_10_10_praxis_retirement_plan.md`).
- FM-LAW-016 closure depends on this landing.

## Recommended commit split (revised 2026-10-10 — now three)

1. `feat(receipt-chain): verbatim port of praxis-core receipt-chain family
   (FM-CHAIN-009 golden, validator, store)` — `src/receipt_chain.rs`
   (2,083 lines: core family + `ReceiptValidator` at ~line 1803),
   `tests/receipt_chain_golden_test.rs` (4),
   `tests/receipt_chain_validator_test.rs` (4), `Cargo.toml`,
   `Cargo.lock`, `src/lib.rs`, plus this receipt doc.
2. `feat(receipt-chain): migration-epoch port (v1 legacy reader +
   M_1_to_2 migration receipt)` — the `epoch`-submodule slice of
   `src/receipt_chain.rs` (lines ~1192–1300: `ReceiptRecordV1Legacy`,
   `MigrationReceipt`, `MIGRATION_LAW_1_TO_2`) and
   `tests/receipt_epoch_migration_port_test.rs` (4). Sized separately
   because it is an independently-verified differential port (~110
   source lines + its own test file), not part of the original family
   port.
3. `feat(hooks): refuse effect with verdicts` — `src/hooks.rs`,
   `tests/knowledge_hooks.rs`. (If `src/lib.rs` touches both, land its
   receipt-chain lines with commit 1/2 and any hooks export with
   commit 3.)

Also in the working tree, owned by other lanes (not part of this
split): `README.md`, `tests/op_differential.rs`,
`tests/op_forward_compat.rs`, `tests/registry_limits.rs`,
`tests/wasm_abi.rs`, `wasm/src/lib.rs`. Neither commit is pushed yet;
push after all three land.

## Working-tree inventory (2026-10-10)

Modified: `Cargo.lock`, `Cargo.toml`, `README.md`, `src/hooks.rs`,
`src/lib.rs`, `tests/knowledge_hooks.rs`, `tests/op_differential.rs`,
`tests/op_forward_compat.rs`, `tests/registry_limits.rs`,
`tests/wasm_abi.rs`, `wasm/src/lib.rs`. Untracked (new):
`docs/receipt-2026-10-09-receipt-chain-and-refuse-hooks.md`,
`src/receipt_chain.rs` (2,083 lines),
`tests/receipt_chain_golden_test.rs` (4 tests),
`tests/receipt_chain_validator_test.rs` (4 tests),
`tests/receipt_epoch_migration_port_test.rs` (4 tests).
