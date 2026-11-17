# v26.11.17 — window index

Milestone: post-v26.10.28 standing re-read of the graphlaw tree at HEAD `dd6a80e` (2026-11-16), plus the GL29 proposed backlog and a publish dry run. Opened 2026-11-17. The coordinator owns git.

**Method and limits.** Twenty read-only Explore lanes each drafted one file; the coordinator wrote them here. This container is a fresh, shallow clone (50 commits, no tags, no `b3sum`, no sibling checkouts). Every file is a **static read**: no cargo, mix, npm or court run backs any verdict unless a file embeds real output. A loud SKIP is not a pass. Where a lane could not verify, the file says UNVERIFIED. This window carries no standing of its own; the GL28 register (`../v26.10.28/GL28-BACKLOG.md`) stays authoritative for GL28-n.

## Files
| file | lane | headline |
|---|---|---|
| [CHANGELOG-SINCE-v26.10.28.md](CHANGELOG-SINCE-v26.10.28.md) | 01 | 29 commits after 320e2e5; tag not reachable, baseline approximate |
| [GL28-STATUS-REREAD.md](GL28-STATUS-REREAD.md) | 02 | CONFIRMED 3/4/6/7/13/14; DRIFTED 5, 8, 11; UNVERIFIED 1, 9, 10 |
| [REFUSAL-AUDIT.md](REFUSAL-AUDIT.md) | 03 | no missing codes; arms 1-35/scope/100-104 undocumented; REFUSAL-04 weakly tested; stale anchors |
| [COURT-INVENTORY.md](COURT-INVENTORY.md) | 04 | 131 files, 982 #[test]; skip-capable wasm/lease/receipt courts lack counted-SKIP guards |
| [CI-SCRIPT-MAP.md](CI-SCRIPT-MAP.md) | 05 | only mutation.yml calls a script; doc/witness/count gates advisory; stale ci.yml:96 comment |
| [KERNEL-MAP.md](KERNEL-MAP.md) | 06 | 32,549 LOC; law<->attest/receipt/plan/hooks coupling hotspot |
| [SQL-RDF-SURFACE.md](SQL-RDF-SURFACE.md) | 07 | 35/35 kinds generated; execution depth shallow for several kinds; REFUSAL-05 not in Rec |
| [RECEIPT-CHAIN-STANDING.md](RECEIPT-CHAIN-STANDING.md) | 08 | CHAIN.blake3 covers 3698/3698 paths; digests not verified (no b3sum); dangling .attest.json |
| [WASM-BOUNDARY-STANDING.md](WASM-BOUNDARY-STANDING.md) | 09 | pin c92caeb6 consistent across surfaces; 4 OPEN gap rows; no green witness after 2026-10-29 |
| [ABI-REGISTRY-STANDING.md](ABI-REGISTRY-STANDING.md) | 10 | 14 advertised / 15 dispatched ops; no drift found; ARTIFACTS pin rows to confirm |
| [GRAPHLAW-EX-STANDING.md](GRAPHLAW-EX-STANDING.md) | 11 | GL28-11 mostly landed; dual refusal vocab open; ash_graphlaw re-vendor owed |
| [BINDINGS-STANDING.md](BINDINGS-STANDING.md) | 12 | all at 26.10.28; python/ now tracked; npm 2 releases behind source |
| [ONTOLOGY-GGEN-STANDING.md](ONTOLOGY-GGEN-STANDING.md) | 13 | 88M; ~28M duplicate DOID; ggen projections listed |
| [N3-RULES-STANDING.md](N3-RULES-STANDING.md) | 14 | 8-limit census; 14 open audit findings; stray SKV-REVERT-TEST-B marker |
| [DOC-STALENESS.md](DOC-STALENESS.md) | 15 | 21 findings; active-window pointers; vapor docs_release_claims court |
| [MIGRATION-VERIFICATION.md](MIGRATION-VERIFICATION.md) | 16 | steps 1-2 done (origin dd6a80e); local-machine claims unverifiable here |
| [DEPENDENCY-STANDING.md](DEPENDENCY-STANDING.md) | 17 | 6 advisory ignores expire 2026-11-23 / 11-28; purrdf pin != carried head |
| [RELEASE-PUBLISH-STANDING.md](RELEASE-PUBLISH-STANDING.md) | 18 | npm BLOCKED-external; Hex undecided; crates.io none by policy |
| [RISK-SWEEP.md](RISK-SWEEP.md) | 19 | evidence_ring sleep-park, receipt_chain.rs:614 expect, law.rs unreachable!s |
| [GL29-BACKLOG.md](GL29-BACKLOG.md) | 20 | GL29-1..13 carried from GL28 + 14-19 from this window |
| [PUBLISH-DRYRUN.md](PUBLISH-DRYRUN.md) | goal | publish dry run, written after the run (see file) |

## Cross-lane findings worth acting on first
1. **Time-boxed:** 3 cargo-deny advisory ignores expire 2026-11-23, 3 on 2026-11-28 (DEPENDENCY-STANDING).
2. **Receipt gaps:** receipts cited by the GL28 register (npm ledger, mutation court, graphlaw_bridge) are absent from the tree after the 2026-11-09 triage (GL28-STATUS-REREAD, RELEASE-PUBLISH-STANDING).
3. **Release blocked externally:** npm Trusted Publisher config (GL28-12) is an operator action; no 26.11.x bump exists.
4. **Pointer drift:** CLAUDE.md and docs/index.md still name v26.10.28 as the active window (DOC-STALENESS rows 1-2). Not edited here: CLAUDE.md is a pinned instruction file.
5. **Unverified-by-run:** no court has been run for this window; the cheapest next rung is `cargo check`, then the four ABI/registry courts named in ABI-REGISTRY-STANDING A2.
