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

## Symbolic anchors + current-window scan (P7 W4a, 2026-10-24)

Gate diff (`scripts/doc_staleness_gate.sh`), per the X2 gate-first design:

- New `ANCHOR_SYM` regex
  `\b([A-Za-z0-9_./-]+\.(?:rs|sh|ttl|md|toml|json))#([A-Za-z0-9_]+)\b`:
  `path#SYMBOL` resolves iff the target exists AND some line of it matches the
  existing `SYMBOLIC` regex and ALSO contains the symbol token
  (`\b`-bounded, `re.escape`d). `:NN` (and `:A-B`) validation unchanged; both
  forms accepted. `NEARMISS` unchanged. Symbol-form failures are coded
  `R5-SYMBOL` (register precedent: `R5-NEARMISS`); `:NN` failures stay `R5`.
- Scan set: the recursive `docs/jira/**` glob is replaced by current-window-only —
  `docs/jira/v*` dirs sorted by (major, minor, patch), only the highest scanned
  recursively. Closed windows leave scope when the next window dir lands.
  `docs/*.md` and `docs/explanation/**` unchanged.
- R4 (`RECEIPTS.md` receipt-pointer rule) still walks all `docs/jira` — untouched.

### Current-tree counts (real runs, before vs after)

- BEFORE (pre-diff gate): exit 1, `doc_staleness_gate: 1705 finding(s)`
  (1683 R5 = 1091 in docs/jira/v26.10.22 + 434 in docs/jira/v26.10.23 + 158
  living docs; plus 16 R2 + 6 R4).
- AFTER: exit 1, `doc_staleness_gate: 714 finding(s)`
  (592 R5 = 434 v26.10.23 + 158 living — v26.10.22's 1091 left scope; 100
  R5-SYMBOL; 16 R2 + 6 R4 unchanged).
- The 100 R5-SYMBOL findings are new real surface minted by `ANCHOR_SYM`:
  99 in docs/PHD_THESIS_GRAPHLAW_WASM_SQLRDF.md (GitHub-style permalinks whose
  captured path is `Users/sac/graphlaw/src/...` — `\b` cannot start the match
  at a leading `/`, so the target never resolves on disk) and 1 in
  docs/refusals.md:233 (`abi-reference.md#13`, a section-number cite). No
  anchors migrated by this lane; E11 owns the corpus.

### Flip proof (protocol of this receipt, corpus `/tmp/dsg-anchor-flip.kQNeaX`)

mktemp corpus: `src/lib.rs` = `pub fn LEGACY_FALLBACK_ENABLED() {}` (1 line);
`docs/explanation/a.md` cites `../src/lib.rs#LEGACY_FALLBACK_ENABLED` (valid);
`b.md` cites `../src/lib.rs#LEGACY_FALLBACK_ENABLEDX` (rotted); `c.md` cites
`../src/lib.rs:1` (valid), `../src/lib.rs:999` (out of range),
`../src/ghost.rs:1` (missing file). Real gate on the corpus root:

Run A (all six cases present), exit=1:

    doc_staleness_gate: 3 finding(s)
    FINDING R5-SYMBOL docs/explanation/b.md:1: anchor 'src/lib.rs#LEGACY_FALLBACK_ENABLEDX' resolves to no line with a symbol token containing 'LEGACY_FALLBACK_ENABLEDX'
    FINDING R5 docs/explanation/c.md:2: anchor 'src/lib.rs:999' out of range (1 lines)
    FINDING R5 docs/explanation/c.md:3: anchor target 'src/ghost.rs' does not exist on disk

Run B (positive control: rotted anchor freshened to
`#LEGACY_FALLBACK_ENABLED`, range/missing cites removed), exit=0:

    doc_staleness_gate: 0 finding(s)

Both directions fire; both valid anchors (symbol and `:NN`) produce zero
findings. Corpus left at /tmp/dsg-anchor-flip.kQNeaX for replay.

### Selfcheck (P17 G4)

`scripts/doc_staleness_selfcheck.sh` (chmod +x) rebuilds this corpus shape in
mktemp and runs the real gate per leg: (1) rotted-symbol leg must exit 1 AND
name `LEGACY_FALLBACK_ENABLEDX`; (2) freshened leg must exit 0; (3) line-form
leg (valid `:1` + out-of-range `:999`) must exit 1 naming the out-of-range
anchor — proving the `:NN` path survived the ANCHOR_SYM addition. Gut-R5
mutation simulation is out of scope: the rotted leg IS the non-vacuity proof.
Inversion witness (selfcheck run against the HEAD pre-ANCHOR_SYM gate copied
from `git show HEAD:`): leg 1 cannot flip —

    FAIL rot-exit1-names-anchor: gate exited 0, expected 1
        doc_staleness_gate: 0 finding(s)
    doc_staleness_selfcheck: FAIL — a leg did not flip as required
    (exit=1)

so the selfcheck detects the missing rule rather than passing vacuously.

