# v26.10.10 Publish — Definition-of-Done Receipt (skeleton)

Lane 6 (receipt-claim verification), 2026-10-10. Read-only audit; no cargo run.
Source receipt: `docs/receipt-2026-10-09-receipt-chain-and-refuse-hooks.md`.

## 1. Verified claims (checked against tree, 2026-10-10)

| Claim | Evidence |
|---|---|
| Working-tree inventory of the receipt doc matches disk | `git status --porcelain` — all 11 modified + 5 untracked files present as listed (note: `CHANGELOG.md`, `registry/capability-registry.{json,ttl}` are also modified, owned by other lanes, not in the receipt's inventory) |
| `src/receipt_chain.rs` 2,083 lines | `wc -l` → 2083 |
| 4 golden tests | `tests/receipt_chain_golden_test.rs` — 4 `#[test]` |
| 4 validator tests | `tests/receipt_chain_validator_test.rs` — 4 `#[test]` |
| 4 migration-epoch tests | `tests/receipt_epoch_migration_port_test.rs` — 4 `#[test]` |
| 11 refuse-hook tests | `tests/knowledge_hooks.rs` — 11 `#[test]` |
| Version 26.10.10 | `Cargo.toml:3`; `CHANGELOG.md:10` |
| release.yml auto-publishes on push to main when version unreleased | `.github/workflows/release.yml` header; honors `RELEASE_HOLD` repo variable |
| 3-commit split plan | Consistent with tree partition: commit 1 files (`src/receipt_chain.rs` family slice, golden+validator tests, `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, doc), commit 2 (epoch slice + epoch test), commit 3 (`src/hooks.rs`, `tests/knowledge_hooks.rs`) — matches Lane 2's independent plan |

## 2. Stale / corrected claims

| Claim | Status | Evidence |
|---|---|---|
| "443 passed, 0 failed, 1 ignored" (refresh 2026-10-10) | RESOLVED by fresh run 2026-10-10 | Coordinator-owned `cargo test --workspace` on the canonical checkout: **436 passed, 0 failed, 1 ignored** (both default and `--features graphlaw/abi`). The 443 tally was a point-in-time receipt from a lane build root; the canonical number is 436/0/1. |
| 443 total plausibility | MOOT | Superseded by the fresh 436/0/1 receipt above. |
| Receipt's working-tree inventory omits `CHANGELOG.md` + `registry/*` | CORRECTED | Those files are modified on disk (other lanes' churn); the receipt's "owned by other lanes" list is incomplete. |

## 3. Remaining human-gated steps to publish

1. ~~Fix or adjudicate the 2 `capabilities_digest` failures~~ DONE 2026-10-10:
   registry artifacts regenerated (`graphlaw-registry --write`), digest independently
   re-verified; fresh suite run 436/0/1 (see §2).
2. Execute the 3-commit split (coordinator owns commits; lanes own files):
   receipt-chain family → epoch port → hooks refuse effect. Include the
   receipt doc and CHANGELOG/registry edits in the appropriate commit.
3. Merge to `main` (no rebase/force-push; merge only).
4. `release.yml` fires on push to main: builds WASI + workspace, tags
   `v26.10.10`, publishes `graphlaw-eyeron` then `graphlaw`. Confirm
   `RELEASE_HOLD` repo variable is unset (HOLD skips all steps).
5. Verify on crates.io: `graphlaw` 26.10.10 present with `.sha256`-attested
   release artifacts; record the release run URL in this receipt's final
   standing block.

Standing: UNKNOWN until step 1's fresh receipt exists.

Note (2026-10-10, post-landing): step 4's expectation that `release.yml` publishes to
crates.io is superseded — publishing is local-only as of the v26.10.11 release
(`cargo cicd publish run` from a logged-in machine; CI never holds registry credentials).
The body above is preserved as written.
