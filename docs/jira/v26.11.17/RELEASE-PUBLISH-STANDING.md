# RELEASE-PUBLISH-STANDING (2026-11-17)

Static inspection at HEAD dd6a80e. No registry or network checks. Registry state below is what the repo claims, not verified.

## Publish matrix
| Channel | Version | Pipeline | Status | Blocker |
|---|---|---|---|---|
| GitHub release (wasm, registry json/ttl, op-examples + .blake3) | 26.10.28 | release.yml (push main) | READY; no evidence it ran for 26.10.28 | RELEASE_HOLD unknown; needs a version bump merge |
| crates.io `graphlaw` | 26.10.28 | none (policy: no publish) | NOT PUBLISHING | purrdf is a git-rev fork dependency |
| crates.io `graphlaw-eyeron` | crate removed | none | CLAIM UNSUPPORTED | migration-plan.md:27 claims 0.7.8; nothing in-tree proves it |
| npm `@seanchatmangpt/graphlaw` | 26.10.28 | publish-npm.yml (tag v*, OIDC) | BLOCKED-external | Trusted Publisher mapping unconfigured; env `npm` must exist; GL28-12 not executed |
| Hex `graphlaw_ex` | 0.1.0 | none (manual) | UNDECIDED | GL28-12 decision open; package files list does not ship the wasm engine |
| PyPI `graphlaw` | 26.10.28 | none | NOT PLANNED | no decision |

## Standing items
- S1: receipts/2026-10-24-npm-attempt-ledger.md is cited (GL28-BACKLOG.md:107, release-publish.md) but absent from the tree (likely archived in 6961d73); "npm at 26.10.14" is unverified.
- S2: GL28-12 npm/Hex still NOT EXECUTED; no publish receipt.
- S3: CHANGELOG.md:581/634/668 claim crates.io publishes of graphlaw; contradicts the no-crates policy. Add correction or remove.
- S4: re-verify line citations/counts in docs/how-to/release-publish.md; drop `cargo search graphlaw` step.
- S5: no 26.11.x bump has happened; both Cargo.toml == package.json == tag are required.

## Operator checklist
1. `gh variable list`; RELEASE_HOLD unset.
2. Configure npm Trusted Publisher; create GitHub environment `npm`.
3. Bump Cargo.toml, js/npm/package.json, python/pyproject.toml together; merge; confirm work=yes.
4. Push tag; confirm publish-npm green.
5. `npm view @seanchatmangpt/graphlaw version`.
6. Record Hex go/no-go and PyPI no.
