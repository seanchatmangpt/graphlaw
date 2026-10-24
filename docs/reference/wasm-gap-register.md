# Wasm Gap Register

Consolidated gap register for milestone v26.10.23, derived from
`docs/jira/v26.10.23/ROADMAP.md` and the approved plan
`~/.claude/plans/here-is-the-voice-golden-bengio.md`. Status is as of
register authoring (pre-Wave A); flips are receipt-gated — a row flips
only on a landed receipt.

| ID | Description | Evidence locus | Wave | Status | Closing witness |
|---|---|---|---|---|---|
| G1 | receipt_v2 has no emission site | attest.rs:704 | D | OPEN | Twin A; v2 e2e court |
| G2 | CanonArrow DCE'd in-wasm | egress/arrow.rs | D | OPEN | Twin B; pinned-view; FIBO |
| G3 | conftest hardcode; no env honor | conftest.py:35 | B | OPEN | pytest, env set AND unset |
| G4 | read_frame unbounded alloc | run_eh.rs:336-350 | C | OPEN | bound court: exit 92 |
| G5 | WALL vs MAX_REQUEST drift | abi.rs:48; abi_meta:29 | C | OPEN | limits table; assert |
| G6 | benches unreceipted | no receipts/ files | E | PENDING | bench vs REQ budgets |
| G7 | abi builders uncovered | src/abi.rs | B | OPEN | builders court; mut rerun |
| G8 | sniff literal/IRI gaps | sniff.rs:252-279 | C | OPEN | twins; corpus_conformance |
| D1 | WASI inventory absent | SECURITY.md | A | OPEN | count receipt; gate |
| D2 | 256 MiB doc errors | WASM.md:206 | A + C | OPEN | delta; pointer rows |
| D3 | GWS stale; INDEX orphans | integration.md:49 | A | OPEN | appends; GWS-2/3 flip |
| D4 | witnesses not durable | terminal receipt 2 | A | OPEN | rerun on 472d5557 |
| BB1 | blackbox pins wasmi | tests_py blackbox | F | OPEN | host flip; tree -i |
| UU1 | unknown-unknown smoke | audit, deferred | E | PENDING | battery re-run |
| W08 | differential no receipt | common/mod.rs:20 | C | OPEN | differential receipt |

Row count: 15 (G1-G8, D1-D4, BB1, UU1, W08).

Behavioral-audit deferred witness classes: G1 = emission-site court (Twin
A), G2 = columnar-bytes twins (Twin B + pinned-view), G3 = env-matrix
pytest (set/unset), BB1 = host-flip green + absence-from-graph witness,
UU1 = full-battery re-run, W08 = run+receipt against the pinned artifact.
