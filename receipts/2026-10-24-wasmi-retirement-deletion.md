# Receipt — wasmi retirement: deletion evidence (absent-from-graph witness)

- **Lane**: finB implementation lane
- **Date**: 2026-10-24 (evidence captured 2026-10-24T17:21Z window)
- **HEAD**: `6da1d33a93f61068fa9b033bec353db78d335460`
- **Subject**: `wasmi` / `wasmi_wasi` absent from the dependency graph after
  the runner-source deletion in 6da1d33; pre-delete blob digest recorded for
  replay.
- **Standing**: PARTIAL_ALIVE — deletion witnessed on the dirty tree at
  6da1d33; manifest commit is coordinator-owned.

## cargo tree inversion probes (verbatim)

Command:
```bash
CARGO_TARGET_DIR=target-lane-finB cargo tree -i wasmi
CARGO_TARGET_DIR=target-lane-finB cargo tree -i wasmi_wasi
```

Output + exits, verbatim:

```
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /Users/sac/graphlaw/wasm/Cargo.toml
workspace: /Users/sac/graphlaw/Cargo.toml
error: package ID specification `wasmi` did not match any packages

help: a package with a similar name exists: `wasi`
TREE_WASMI_EXIT=101
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /Users/sac/graphlaw/wasm/Cargo.toml
workspace: /Users/sac/graphlaw/Cargo.toml
error: package ID specification `wasmi_wasi` did not match any packages
TREE_WASMI_WASI_EXIT=101
```

DISCREPANCY DISCLOSURE: the dispatch predicted exit 1; the witnessed exit is
**101** (cargo's own error exit for an unmatched package-ID specification).
The error text — `error: package ID specification \`wasmi\` did not match any
packages` (backticks, not quotes as predicted) — IS the absent-from-graph
witness either way. Witnessed values are recorded, not the predicted ones.

## Pre-delete blob digest (replayable)

```bash
$ git show 6da1d33~1:src/bin/graphlaw_run.rs | /opt/homebrew/bin/b3sum
0b37e66a45007e68f3f694a2cada9fa3c7a4d9955ee2b6ff79deaeb4399aeb24  -
$ git show 6da1d33~1:src/bin/graphlaw_run.rs | wc -c
   25746
```

Deletion commit evidence: `git show 6da1d33 --stat` names
`src/bin/graphlaw_run.rs | 651 -------------------------------------`
(651-line whole-file deletion) and `src/bin/graphlaw_run_eh.rs | 11 +-`.
Commit message line: `feat(v26.10.23): set operations (UNION/INTERSECT/EXCEPT) + working-tree integration`.

## Related receipts

- Step-2 green witness (pytest 19/19 over `graphlaw-run-eh`):
  `receipts/2026-10-24-wasmi-retirement-step2-green-witness.md`
- R-1 gate rungs (host-contract + pin-gate inversion):
  `receipts/2026-10-24-r1-rungs.md`
