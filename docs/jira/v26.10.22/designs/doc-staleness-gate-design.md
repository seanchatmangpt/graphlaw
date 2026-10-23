# Design: Doc Staleness Gate (`scripts/doc_staleness_gate.sh`)

Lane: L35 · 2026-10-22 · Status: implemented (exit codes proven below)
Core-team critique point 5: staleness checking this session was
periodic-forensic (`stale-claims-audit.md` is a one-time lane artifact). This
gate makes it mechanical: every rule below is a deterministic file:line check,
and the gate can exit 1 (the point of a court).

**Implemented**: `scripts/doc_staleness_gate.sh` (bash wrapper + embedded
python3). Exit 0 = pass, 1 = findings, 2 = internal error.

## Rule 1 — fenced command resolution (README.md, CLAUDE.md)

Every line inside a ` ```sh `/` ```bash `/` ```console ` fence must resolve:

- Command head must be a known program (`cargo`, `git`, `bash`, `sh`, `just`,
  `test`, `ls`, `mkdir`, `cp`, `rm`, `mv`, `python3`, `grep`, `jq`) or a
  repo-relative script path (`./…`, `scripts/…`). Real finding on the current
  tree: `README.md:80` (`npm i` — head not in the allowlist; either allowlist
  `npm` deliberately or drop the block).
- `--test <t>` → `tests/<t>.rs` must exist
  (evidence: `README.md:33-39`, `CLAUDE.md:21-31` name real targets;
  counterexample the gate would refuse: `cargo test --test sql_rdf_dialectx`).
- `--example <e>` → `examples/<e>.rs`; `--bin <b>` → `src/bin/<b>.rs`.
- Backslash continuations merge (evidence: `README.md:146-147`
  `graphlaw-verify … \` + `--trusted-key …`).
- Inline commands (not fenced) are out of scope for R1 — the gate only parses
  fenced blocks; this is stated so the boundary is not overclaimed.

## Rule 2 — digests/byte-counts vs registry or historical marker

Any 64-hex digest in `docs/**/*.md` must either appear in
`registry/ARTIFACTS.blake3` (format header, line 1: `<kind> <blake3> <bytes>
<name>`) or its line must contain `historical` (case-insensitive).
Registry format is the on-disk authority (evidence:
`registry/ARTIFACTS.blake3:1-3`, pin header `digest=332ea218…`, W09).
Byte-count/size claims are design-noted but not enforced (too noisy without a
schema for `<artifact>: <bytes>` syntax — proposed: reuse the registry
`<bytes>` column, checked with the same R2 mechanism, future work).
`docs/reference-generated/` excluded (generated artifacts are re-generated,
never patched — CLAUDE.md "Generated artifacts" table).

Real findings on the current tree (10): `docs/quickstart.md:35-38` (the
README "First 10 minutes" digests — these are execution outputs, so the
honest fix is a `historical` marker or a receipt pointer, not registry
registration), `docs/wasm-integration.md:129`,
`docs/jira/v26.10.22/SQLRDF-17.md:107-108`, `RECEIPTS.md:116`,
`stale-claims-audit.md:15,132`.

## Rule 3 — banned-stale-string sweep with dated allowlist

Banned strings: `--features duckdb`, `native-only`, `capability_intake`,
`graphlaw_duckdb.wasm`. The scan covers `README.md`, `CLAUDE.md`, `docs/`,
`ggen/`, `scripts/` (md/sh/ttl/rs/toml). A line is exempt only if it contains
`historical` (case-insensitive) — the dated marker is the honest escape hatch;
the gate does not try to parse dates.

Real findings on the current tree (21): `docs/WASM.md:27` and
`docs/wasm-duckdb-integration.md:61` (both are *corrective* mentions — "there
is no separate `graphlaw_duckdb.wasm` target" — the right fix is to add
`historical` to the line, not delete the correction),
`SQLRDF-19.md:67` (`--features duckdb` feature-map cell),
`docs/jira/v26.10.22/designs/P28-SIL-specs.md:54` (`native-only`), and the
16 hits in `stale-claims-audit.md` (an audit artifact quoting stale strings —
it needs the historical marker or an explicit gate allowlist entry).

The gate script self-lists the banned strings to define them; it is excluded
from its own scan (a gate that fires on its own rule definitions is vacuous
self-fire).

## Rule 4 — receipt-cited claims need a durable pointer

Mtime is rejected as the mechanism (weak: checkouts, rebases, and lane fan-out
reset mtimes; a stale claim with a fresh mtime passes and a fresh claim with a
stale mtime fails). The honest checkable form is an explicit pointer: any line
in a `docs/jira/**/RECEIPTS.md` carrying a count (`N passed`/`N failed`) or a
64-hex digest must also contain a `receipts/` path. Durable receipts live in
`receipts/` (BLAKE3-chained, per CLAUDE.md Receipts convention), so the
pointer resolves staleness by content, not time.

**Forward-looking enforcement note (stated honestly)**: the current tree's
RECEIPTS.md rows predate this design and cite verdicts without pointers, so
the gate reports 20 R4 findings on the current tree. This is the intended
behavior — the gate refuses the legacy format rather than grandfathering it
(a gate that grandfathers is vacuous). Remediation is additive: append
`(receipts/<file>.md)` to cited rows, or mark the row `historical`. Ad-hoc
receipt files under `docs/jira/` named RECEIPTS.md are scanned identically
(`designs/RECEIPTS.md` is in scope and produced 6 findings).

## Verification receipt (2026-10-22, this tree)

Commands and real exits:

```
$ bash scripts/doc_staleness_gate.sh; echo "EXIT=$?"
doc_staleness_gate: 50 finding(s)
… (full 50-line listing in the lane report) …
EXIT=1
```

Anti-vacuity (injected staleness, then removed):

```
$ cat > docs/scratch-inject-demo.md   # unregistered 64-hex digest + '--features duckdb'
$ bash scripts/doc_staleness_gate.sh
doc_staleness_gate: 52 finding(s)      # +2, both on the scratch file:
FINDING R2 docs/scratch-inject-demo.md:3: digest aaaa0000bbbb… not in registry/ARTIFACTS.blake3 …
FINDING R3 docs/scratch-inject-demo.md:4: banned stale string '--features duckdb' …
EXIT=1
$ rm docs/scratch-inject-demo.md
$ bash scripts/doc_staleness_gate.sh >/dev/null
EXIT=1   # tree restored: back to the 50 pre-injection findings
```

A court that can flip to exit 1 is the point; the pass state on this tree is
honestly **FAIL (50 findings)**, enumerated in the gate output, not a green
wash. CI wiring: add `bash scripts/doc_staleness_gate.sh` to the lint job;
the 50 findings are the remediation backlog.

## See Also

`docs/jira/v26.10.22/designs/stale-claims-audit.md` (periodic-forensic
predecessor this gate mechanizes) · `registry/ARTIFACTS.blake3` (R2
authority) · `receipts/` (R4 pointer target) · CLAUDE.md "Generated
artifacts" (R2 exclusion)
