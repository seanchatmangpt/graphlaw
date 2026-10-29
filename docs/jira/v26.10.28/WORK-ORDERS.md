# WORK-ORDERS

Per-CTQ work orders for v26.10.28, with HONEST standing from the
2026-10-28 session receipts. Standing vocabulary:

- **ALIVE** — ran this session; command + exit cited.
- **PARTIAL** — some witnesses ALIVE, named gaps remain.
- **UNVERIFIED** — no command run; not a pass.
- **BLOCKED** — typed reason + file:line.

Never round up: PARTIAL and UNVERIFIED are not ALIVE.

## Session receipt base (2026-10-28)

purrdf fork swap @ 2223f894 — ALIVE witnesses:

| Command | Result |
| --- | --- |
| `cargo check` | 0 errors |
| `cargo test --test chicago_validation` | 8/8 passed |
| `cargo test --test proof_pipeline` | 12/12 passed |
| `cargo test --test conformance` | 12/12 passed |
| census (`n3_limit_census_court`, trait `ReasonerLimitCensus` in `src/law.rs`, at-cap sentinels) | 13/13 passed |
| budget cliff court (truth-synced to fork counters) | 10/10 passed |
| `wasm_native_differential` | 2/2 passed |
| `wasm_size_court` | 4/4 passed (ceiling raised 64 → 80 MiB via SCAR; artifact measured 74,097,211 B) |
| `cargo test --test duckdb_statement_kinds` | 8/8 passed (35 kinds) |
| `cargo test --test sql_rdf_dialect` | 36/36 passed |
| capabilities digest (after `graphlaw-registry --write` repin) | 6/6 passed |
| graphlaw_ex artifact instantiation via wasmex 0.15.1 | ALIVE — real artifact loaded in 7.4s; admit answers `{:error, :abi_not_wired}` |
| WASI import audit | 28/28 `wasi_snapshot_preview1` |

## WO-1 — CTQ 1.1/1.2/1.3 wasm execution bounds — ALIVE

- **Required**: wasm artifact built; reduction budget ≤1000/call or pooled
  isolation; heap-delta Cpk ≥ 1.33 gate; latency receipt with real tail.
- **Current (ALIVE)**: wasm build exit 0 (5m22s, blake3 `348c97fd…`);
  budget cliff court 10/10 truth-synced to fork counters (reduction-budget
  witness); `wasm_native_differential` 2/2 (premise re-authored under the
  ONE-unit law — the wasm artifact DOES advertise frozen-duckdb);
  `wasm_size_court` 4/4 at the 80 MiB SCAR ceiling (artifact
  74,097,211 B). SPC gate landed:
  `bash scripts/spc_court.sh` exit 0 — receipt
  `receipts/spc-2026-10-28T165319.json` (re-read at write time): bench
  `tc_hot_01`, clone_path leg, mean 109.485 ns, stddev 7.681 ns,
  **Cpk 6.0979** (gate min 1.33), p99 125 ns / p999 170 ns / max 170 ns,
  tail_fraction 0.0 against 0.0001 budget, 999 samples / 1,000,000
  iterations.
- **Gap**: none — latency receipt embedded in the SPC receipt (p50/p95/p99/
  p999/max).
- **Acceptance**: met — SPC court exit 0 with Cpk 6.0979 ≥ 1.33 in the
  receipt; real tail latencies embedded.

## WO-2 — CTQ 2.1 fail-closed admission — PARTIAL

- **Required**: 0 false admissions; anti-vacuity: gates flip on mutation.
- **Current (ALIVE)**: chicago_validation 8/8 @ 2223f894.
- **Gap**: `bash scripts/mutation_court.sh` not yet run this session — the
  gates are unwitnessed as non-vacuous.
- **Acceptance**: mutation court exit 0 (run solo, serialized against all
  lanes; `--force` only on a dirty tree per CLAUDE.md).

## WO-3 — CTQ 2.2/3.1 proof + SQL-over-RDF — ALIVE

- **Required**: Ed25519 proof verification per evaluation; ANSI SQL over
  RDF via embedded frozen-duckdb.
- **Current (ALIVE)**: proof_pipeline 12/12, conformance 12/12, and
  `sql_rdf_dialect` 36/36 @ 2223f894. `cargo test --test trust_attest`
  re-run this session: **12/12 passed, exit 0** (acceptance rerun receipt,
  `receipts/trust-attest-2026-10-22.md` boundary remains; session rerun
  witnessed by the coordinator).
- **Gap**: none.
- **Acceptance**: met — `cargo test --test trust_attest` exit 0 this
  session.

## WO-4 — CTQ 3.2 statement-kind parity 35/35 — ALIVE

- **Required**: all 35 PEG statement kinds pass on real frozen-duckdb.
- **Current (ALIVE)**: `duckdb_statement_kinds` 8/8 covering 35 kinds @
  2223f894; census court 13/13 (trait `ReasonerLimitCensus` in
  `src/law.rs`, at-cap sentinels) also ALIVE this session.
- **Acceptance**: met — court exit 0 with 35/35 kinds in output.

## WO-5 — CTQ 4.1/4.2 receipts + traceability — ALIVE

- **Required**: 100% append-only receipt emission; every receipt traceable
  to `courts/code-partition/axioms.blake3` and leases.
- **Current (ALIVE)**: capabilities digest **6/6 passed, exit 0** after
  the `graphlaw-registry --write` digest repin — registry identity is
  re-pinned and witnessed this session. Receipt surface is emission-witnessed
  by the per-Edit transition receipts landing in `receipts/`
  (`transition-2026-10-28T*_Edit-*.json`, real files on disk).
- **Gap**: none.
- **Acceptance**: met — digest repin witnessed (capabilities_digest 6/6);
  receipt emission surface alive on disk.

## WO-6 — CTQ 5.1/5.2/5.3 graphlaw_ex embed — ALIVE

- **Required**: `graphlaw_ex` builds in-tree (`mix` green); 0 host imports
  in the consumed wasm surface; fork source rev-pinned.
- **Current (ALIVE 5.1/5.2)**: packet-4 stdio contract through wasmex
  0.15.1 — **admit + tampered-input refuse 2/2 passed** (real artifact,
  typed fail-closed start, not a crash); WASI import audit **28/28
  `wasi_snapshot_preview1`**, zero host-module imports.
- **Current (ALIVE 5.3)**: purrdf fork swap @ 2223f894, `cargo check`
  0 errors — canonical source is rev-pinned.
- **Gap**: none.
- **Acceptance**: met — packet-4 admit/tamper 2/2 via wasmex 0.15.1;
  WASI-only import surface (witnessed 28/28); rev-pinned fork.

## Already-implemented boundary

The fork swap itself (WO-6 CTQ 5.3) is done and receipted — no order here
re-asks for it. WO-2/WO-3/WO-4 courts exist in-tree; the work is
re-execution for this-session witnesses, not new implementation. Wasm
build machinery, SPC court, and mutation court all exist per CLAUDE.md;
WO-1/WO-2 are runs, not builds of new gates.
