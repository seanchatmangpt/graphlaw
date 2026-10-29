# ROADMAP — v26.10.24

Active plan (2026-10-24, docs+registers lane). Single-release
consolidation: the v26.10.23 roadmap's open items are dispositioned here
(RETIRE vs CARRY), and the directive's new lanes are added. Sequencing law
from the parent roadmap is unchanged: `scripts/mutation_court.sh` runs
SOLO and gates `src/` work; waves verify by DELTA (zero NEW gate findings,
R2/R4 stay zero, chain `--check` exit 0, full battery re-run on this
wave's final artifact identity).

## RETIRE (closed, evidenced — no carry)

| Item | Retired because | Evidence |
|---|---|---|
| D1 WASI inventory | Landed: SECURITY.md §"Imported WASI capability inventory" with the real count command + output | `SECURITY.md:85-92` |
| D2 256-MiB doc errors | Landed: 16 MiB request + 16 MiB outstanding, uniform across targets; sessions doc carries the process-local sentence | `docs/WASM.md:237-240` |
| D3 as a *doc-fix* lane | The doc-side fixes landed (citation repair, dated re-check tokens); what remains is per-token receipts, carried as A-9 | `docs/wasm-duckdb-integration.md:49-56` re-checked 2026-10-24 |
| D4 as a *lane* | Census class durable (dialect ×6 receipts); residue narrowed to the fuzz/memory-wall/exit-codes classes, carried as A-10 | `receipts/2026-10-24-dialect-*.md` |
| G7 court | Builders court landed and witnessed (10/0, `b499fb7`); this window fixed the config that kept its killer from running — what remains is the verdict, carried as A-1 | `tests/abi_response_builders_court.rs`; commit `a00770b` |
| G3-conftest | Landed: `GRAPHLAW_WASM`-honoring lookup + canonical shim; pytest witnessed 19/0 over `graphlaw-run-eh` | `receipts/2026-10-24-wasmi-retirement-step2-green-witness.md` |
| G4 | Landed in both hosts; court 8/0 witnessed; register row CLOSED | 03:20 flip log, `docs/reference/wasm-gap-register.md` |

## CARRY (open, falsifier-named)

- **G8 sniff differential** — fixes landed `6da1d33` (HEAD_BUDGET 8192 at
  `src/dialect/sniff.rs:25`, `tests/sniff_court.rs`, SNIFF-01 registered at
  `docs/refusals.md:220`) and the corpus leg went green, but the battery L4
  differential still disagreed native-vs-wasm (sniff NTriples vs Turtle,
  `tests/op_differential.rs:181`, `/tmp/bat-L4.log` 11:35) — fresh witness,
  post-fix. Acceptance: rebuild on the settled tree + re-witness
  `op_differential` green + a wasm-gap-register row for the divergence
  (fix or typed-divergence append). Owner: coordinator.
- **G2 CanonArrow** — `src/egress/collector.rs` landed (12,197 B); no
  `egress_arrow` ABI op yet. Wave D court set unchanged (Twin A/B, tamper
  refusals, empty arm, v1 goldens untouched, FIBO under v2, in-wasm
  pinned view). Owners: code lane; OQ1-OQ3 block.
- **OQ1-OQ6** — operator open questions carried verbatim from the parent
  roadmap (`docs/jira/v26.10.23/ROADMAP.md` §Operator open questions).
  OQ1-OQ3 block Wave D; OQ4-OQ6 non-blocking.
- **Wave E** — re-witness + docs: the L5/L6 battery legs (compile-aborted
  RED at 13:25-13:33 on the in-flight vendor surgery) re-run green on the
  settled tree + the wasm re-pin ceremony at the cut.
- **Wave F** — ERRC innovation catalog carried verbatim from the parent
  roadmap (`docs/jira/v26.10.23/ROADMAP.md` §Wave F), same rank
  discipline.

## NEW lanes (directive 2026-10-24)

- **N3/Datalog query-compiler** — the compiler paradigm: SQL-RDF graph
  terms exist only as ephemeral synthesis artifacts; Eyeron's native
  FiredFuse stops invalid state transitions before DuckDB executes.
  Fuse surfacing is in flight (P9: `FiredFuse` exists in eyeron and is
  silently swallowed by `reason_n3_bounded` — surfacing + typed-refusal
  mapping, not building the mechanism).
- **purrdf 3.0.1 + eyeron 0.8.18** — atomic upstream sync, ONE CalVer
  release; **MSRV 1.98 NON-NEGOTIABLE** (f64::algebraic_* ungated in
  purrdf-core distance; 1.97 = E0658); the 7 transitional skew-reds are a
  BINARY FALSIFIER — rebuild + registry re-mint must force them green
  immediately or the workspace-integrity claim is void. Sequencing: MSRV
  commit after E01+E03, before the purrdf pin (LOOP-1 GO).
- **Gate-first doc migration** — docs move only behind their gates: a doc
  claim migrates when its gate (doc_staleness R2/R4, drift courts) can
  fail on it; prose is coupled to symbols so doc drift breaks the build
  pre-tag.
- **Publish interlock law** — the publish path is gated on the receipt
  set, not the calendar: tag → npm OIDC auto-publish watch → crates.io in
  dependency order (eyeron first, MSRV accuracy or docs.rs dies); RELEASE
  ASSETS = CI mint vs in-repo pin + npm tarball = local canonical mint —
  two identities, disclosed in TAG-PREP, never treated as contradiction.
  Abort table: tag immutability — a burned .24 becomes a .25 cut, never a
  re-tag.
