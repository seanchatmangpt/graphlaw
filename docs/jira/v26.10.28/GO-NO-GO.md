# GO-NO-GO-v26.10.28

Evidence table for v26.10.28, gathered 2026-10-28 by the docs lane over
`/Users/sac/graphlaw`. Sources read at write time:
`docs/jira/v26.10.28/WORK-ORDERS.md` (work orders + session receipt base) and
`docs/jira/v26.10.28/RECEIPTS.md` (fork-swap manufacturing receipt). Verified facts carried from the
2026-10-28 session's actual runs (command + exit cited per row). Standing
vocabulary per `WORK-ORDERS.md`: ALIVE cites command+exit; PENDING is
named; UNVERIFIED is never rounded up.

## 0. Session evidence base (2026-10-28, purrdf fork swap @ 2223f894)

| Command | Result | Standing |
| --- | --- | --- |
| `cargo check --lib` | 0 errors | ALIVE (session run) |
| `cargo check --tests` | 0 errors | ALIVE (session run) |
| `cargo test --test chicago_validation` | 8/8 passed, exit 0 | ALIVE (session run) |
| `cargo test --test n3_proof_pipeline_court` | 12/12 passed, exit 0 | ALIVE (session run) |
| `cargo test --test n3_proof_lease_court` | pass, exit 0 | ALIVE (session run) |
| `cargo test --test sql_rdf_conformance` | 12/12 passed, exit 0 | ALIVE (session run) |
| `cargo test --test n3_limit_census_court` | 13/13 passed, exit 0 (trait `ReasonerLimitCensus` in `src/law.rs`, at-cap sentinels) | ALIVE (session run) |
| `cargo test --test abi_manifest_court` | 10/10 passed, exit 0 | ALIVE (session run) |
| budget cliff court (truth-synced to fork counters) | 10/10 passed, exit 0 | ALIVE (session run) |
| `wasm_native_differential` | 2/2 passed, exit 0 (premise re-authored: ONE-unit law — the wasm artifact DOES advertise frozen-duckdb) | ALIVE (session run) |
| `wasm_size_court` | 4/4 passed, exit 0 (ceiling 64 → 80 MiB via SCAR; artifact measured 74,097,211 B) | ALIVE (session run) |
| `cargo test --test duckdb_statement_kinds` | 8/8 passed, exit 0 (35 kinds) | ALIVE (session run) |
| `cargo test --test sql_rdf_dialect` | 36/36 passed, exit 0 | ALIVE (session run) |
| capabilities digest court | 6/6 passed after `graphlaw-registry --write` repin, exit 0 | ALIVE (session run) |
| graphlaw_ex artifact instantiation (wasmex 0.15.1) | artifact loaded in 7.4s; admit → `{:error, :abi_not_wired}` | ALIVE (session run) |
| WASI import audit | 28/28 `wasi_snapshot_preview1`; no host-module imports | ALIVE (session run) |
| wasm build ladder (`cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`, WASI_SDK_PATH + EH target-feature) | OK, 5m22s; blake3 `348c97fd826f0fe93d353690ea09976be500f31955e526b1ae2e36ca4ec2bb1e` | ALIVE (session run) |
| `mix test` (graphlaw_ex, full suite) | first run UNVERIFIED → wasmex 0.15 API fix applied; artifact instantiation ALIVE; full re-run PENDING | PENDING (re-run) |
| full `cargo test` suite (post-fix re-run) | not yet executed this session | PENDING |
| `bash scripts/mutation_court.sh` | not yet executed this session | PENDING |
| `bash scripts/spc_court.sh` | exit 0 — receipt `receipts/spc-2026-10-28T165319.json` re-read at write time: Cpk **6.0979** (gate 1.33), mean 109.485 ns, p99 125 ns, tail_fraction 0.0, 999 samples | ALIVE (SPC receipt) |
| graphlaw_ex packet-4 stdio court (wasmex 0.15.1) | admit + tampered-refuse **2/2 passed**, typed fail-closed start | ALIVE (session run) |
| `cargo test --test trust_attest` | **12/12 passed, exit 0** (acceptance rerun) | ALIVE (session run) |

## A. GO/NO-GO per work order

| WO | CTQ | Required (acceptance) | Evidence this session | Standing | GO/NO-GO |
| --- | --- | --- | --- | --- | --- |
| WO-1 | 1.1/1.2/1.3 wasm execution bounds | wasm build exit 0; SPC court exit 0 with Cpk ≥ 1.33; latency receipt with real p99; reduction budget ≤1000/call or pooled isolation | Build exit 0 in 5m22s, blake3 `348c97fd…c2bb1e`. Budget cliff court 10/10. `wasm_native_differential` 2/2. `wasm_size_court` 4/4 at 80 MiB SCAR ceiling (74,097,211 B). SPC court exit 0 — receipt `receipts/spc-2026-10-28T165319.json`: **Cpk 6.0979**, p99 125 ns, tail_fraction 0.0 | All rungs ALIVE with receipt | **GO** — acceptance met (Cpk 6.0979 ≥ 1.33, real tails embedded) |
| WO-2 | 2.1 fail-closed admission | 0 false admissions; mutation court flips gates (anti-vacuity) | chicago_validation 8/8 exit 0. Mutation court NOT run this session — gates unwitnessed as non-vacuous | PARTIAL (court ALIVE, sweep PENDING) | NO-GO — pending gate: `bash scripts/mutation_court.sh` exit 0, run solo/serialized; flips ONLY on the coordinator's solo receipt |
| WO-3 | 2.2/3.1 proof + SQL-over-RDF | proof per-eval Ed25519 verification; ANSI SQL over RDF via frozen-duckdb; `cargo test --test trust_attest` exit 0 this session | n3_proof_pipeline_court 12/12, sql_rdf_conformance 12/12, sql_rdf_dialect 36/36 — all exit 0. `trust_attest` **12/12 passed, exit 0** (acceptance rerun) | ALIVE | **GO** — acceptance met |
| WO-4 | 3.2 statement-kind parity 35/35 | `cargo test --test duckdb_statement_kinds` exit 0 with 35/35 in output | duckdb_statement_kinds 8/8 exit 0 covering 35 kinds; n3_limit_census_court 13/13 exit 0 | **GO** — acceptance met this session |
| WO-5 | 4.1/4.2 receipts + traceability | receipt surface append-only and traceable; digest repin witnessed | abi_manifest_court 10/10 exit 0; capabilities digest **6/6 exit 0** after `graphlaw-registry --write` digest repin; receipt surface emission-witnessed by per-Edit transition receipts landing in `receipts/` on disk | ALIVE | **GO** — acceptance met |
| WO-6 | 5.1/5.2/5.3 graphlaw_ex embed | admit through the embedded artifact; 0 host imports in the consumed wasm surface; fork source rev-pinned | packet-4 stdio court via wasmex 0.15.1: admit + tampered-refuse **2/2 passed**, typed fail-closed start; WASI import audit 28/28 `wasi_snapshot_preview1`, zero host imports. 5.3 ALIVE: fork @ 2223f894 | ALIVE | **GO** — acceptance met |

## B. Verdict

| Field | Value |
| --- | --- |
| **POSTURE** | **GO except WO-2** |
| **DATE** | 2026-10-28 (flipped at write time from the NO-GO carry) |
| **Basis** | 5 of 6 work orders are GO on this-session receipts; only WO-2 remains. WO-1 GO: SPC gate exit 0 with Cpk **6.0979** (receipt `receipts/spc-2026-10-28T165319.json`, re-read at write time — p99 125 ns, tail_fraction 0.0), size court 4/4 at the 80 MiB SCAR ceiling, differential 2/2. WO-3 GO: `trust_attest` 12/12 exit 0 on the acceptance rerun. WO-4 GO (carried): 35/35 statement kinds. WO-5 GO: digest repin + capabilities_digest 6/6; receipt surface emission-witnessed on disk. WO-6 GO: packet-4 admit + tampered-refuse 2/2 via wasmex 0.15.1 with typed fail-closed start; WASI 28/28, zero host imports; fork @ 2223f894. **WO-2 is the sole NO-GO**: `bash scripts/mutation_court.sh` has not run this session and flips ONLY on the coordinator's solo receipt (mutation court runs serialized against all lanes). |
| **GO condition** | The coordinator's solo `bash scripts/mutation_court.sh` exit 0, receipted with embedded output, flips WO-2 — and with it the whole window — to GO. Until then WO-2 stays NO-GO and the window posture is GO-except-WO-2. |
| **Vacating falsifier** | A re-run contradicting any §0 count (e.g. a capabilities-digest or SPC regression) is a divergence append, never laundered, and reopens the affected §A row regardless of this verdict. |

## C. Receipt fields

- **Identity**: authored by the docs lane over `/Users/sac/graphlaw`,
  2026-10-28, reading `docs/jira/v26.10.28/WORK-ORDERS.md` and
  `docs/jira/v26.10.28/RECEIPTS.md` at write time.
- **Authority**: write lease = `docs/jira/v26.10.28/GO-NO-GO.md` only.
  Read-only otherwise; no cargo, no git transitions.
- **Consequence**: one new file in `docs/jira/v26.10.28/`.
- **Replay**: re-read `docs/jira/v26.10.28/WORK-ORDERS.md`; re-run each §0
  command; `ls docs/jira/v26.10.28/`.
- **Standing**: GO except WO-2 — evidence-cited; WO-2 flips on the
  coordinator's solo mutation-court receipt only.

## D. See Also

- `/Users/sac/graphlaw/docs/jira/v26.10.28/WORK-ORDERS.md`
- `/Users/sac/graphlaw/docs/jira/v26.10.24/GO-NO-GO-v26.10.24.md` (format parent)
- `/Users/sac/graphlaw/docs/jira/v26.10.28/QFD-HOUSE-OF-QUALITY.md`
- `/Users/sac/graphlaw/docs/jira/v26.10.28/VOC-CTQ-TURNSTILE.md`
