# Doc anchor guard — receipt (Lane M01, 2026-10-22)

Defect class (witnessed 3x this session): prose cites `path:NNN` anchors that
no longer resolve to a plausible symbol line after edits (ARCHITECTURE.md
`:2051` -> `:1031` drift class).

## Change

`scripts/doc_staleness_gate.sh` extended with **Rule 5** and a scan-root arg:

- `ANCHOR` regex `\b([A-Za-z0-9_./-]+\.(rs|sh|ttl|md|toml|json)):(\d+)\b`
- Scope: `docs/*.md` + `docs/jira/*/INDEX.md` (per defect-class dispatch).
- Per anchor: target must exist (relative to repo root, then doc dir),
  line in range, line must match `[A-Za-z_]` (symbol-ish token).
  Failure -> `FINDING R5 <doc>:<line> ...`, gate exits 1.
- Root arg: `doc_staleness_gate.sh [ROOT]` — when ROOT is given the gate
  runs entirely inside it (enables /tmp anti-vacuity corpora without
  touching the repo tree).

## Non-vacuity (flip proofs, /tmp/dsg-flip)

Corpus: `docs/scratch.md` citing `src/law.rs:1` + `src/law.rs:99999`;
`src/law.rs` = 2-line stub. Real gate, real exit codes:

Run A (valid anchor + out-of-range anchor):

    doc_staleness_gate: 3 finding(s)
    FINDING R1 README.md: file missing          # expected: scratch corpus has no README/CLAUDE.md
    FINDING R1 CLAUDE.md: file missing
    FINDING R5 docs/scratch.md:3: anchor 'src/law.rs:99999' out of range (2 lines)
    exit=1

Run B (blank target line, stub reduced to 1 line):

    FINDING R5 docs/scratch.md:2: anchor 'src/law.rs:1' resolves to a line with no symbol token
    FINDING R5 docs/scratch.md:3: anchor 'src/law.rs:2' out of range (1 lines)
    exit=1

Both flip directions fire; the valid anchor in Run A produces no R5 finding
(the rule is not vacuously all-fail). Scratch corpus left at /tmp/dsg-flip.

## Current-tree findings (real gate output, exit 1, 111 total = 80 R2 + 31 R5;
HEAD baseline run just before the diff: 80 findings, all R2, exit 1 — R2 is
pre-existing and unchanged by this lane's diff)

R5 findings on docs/ARCHITECTURE.md (the witnessed drift class, live):

- :17/:91/:227 `src/sql_ast.rs:2051` — resolves to a line with no symbol token
- :23/:227 `src/sql_ast.rs:2155` — no symbol token
- :27 anchor target `sql_ast.rs` does not exist on disk
- :33/:151 `src/duckdb_eval.rs:366` — no symbol token
- :37 `src/duckdb_eval.rs:563` — no symbol token
- :48/:111/:112 `src/query_ledger.rs:{454,475,441}` — out of range (29 lines)
- :76 `src/tier.rs:232` — no symbol token
- :86/:132/:200 `src/dialect.rs:682` — out of range (587 lines)
- :119 `src/duckdb_eval.rs:754` — no symbol token
- :124/:125 `src/abi.rs:{364,992,1152}` — out of range (341 lines)
- :167/:169/:172 `src/sql_ast.rs:{2156,2053}` — no symbol token

Other files: docs/wasm-integration.md:101-103 (`abi_meta.rs` missing),
:127 (`wasm/Cargo.toml:20` no symbol token); docs/index.md:170
(`README.md:44`), :185 (`ffi.rs` missing);
docs/jira/v26.10.22/INDEX.md:42 (`src/dialect.rs:740` out of range), :238
(`eh-runner-decision-memo.md` missing — file was renamed/moved out of designs/).

The 80 R2 findings are pre-existing unregistered-digest outputs, byte-identical
to the HEAD-version gate run executed immediately before this diff (exit 1,
"doc_staleness_gate: 80 finding(s)").
