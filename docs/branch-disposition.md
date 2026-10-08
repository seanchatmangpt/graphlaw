# Branch Disposition

> **Historical record.** This is a dated campaign snapshot: dispositions were decided by
> `git merge-tree` against origin/main 3983c64 at the time of writing and are not maintained
> against later main movement. Do not treat rows as current state.

Decided by `git merge-tree`, ancestry checks against origin/main 3983c64. Do not re-derive.

| Branch | Disposition |
|---|---|
| test/v26.10.1-chicago-n3-datalog | MERGED |
| factory-b/v26.9.29-evidence-courts-run14 | MERGED (54 fixtures; runner is a separate lane) |
| feat/sa2a-diataxis-v26.9.29 | MERGED, ASSETS.sha256 regenerated |
| factory-b/v26.9.29-sa2a-executable-gates-r2, feat/v26.9.27-*, feat/v26.9.28-*, test/standalone-operational-v26.9.27 | ALREADY MERGED |
| refactor/library-backed-engine-v26.9.28 | STALE-SKIP (deletes ~96k lines) |
| extract/praxis-*, fix/extraction-*, fix/preseed-graphlaw-ci-v26.9.27, fix/retrigger-canonical-extraction-v26.9.27 | STALE-SKIP |
| dependabot/* | left to Dependabot |

## See Also
[[api-stability]]
