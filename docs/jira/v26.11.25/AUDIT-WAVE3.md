# AUDIT-WAVE3

Standing: UNVERIFIED

Register of the 20-lens audit findings for the wave-3 fix wave. Source:
`docs/jira/v26.11.19/AUDIT-20-LENS-2026-11-25.md` (agent-verified-by-reading, not run). Every
row started OPEN. Statuses were updated from the 21 lane receipts in workflow journal
`wf_7644e109-9cb` (lane H3). FIXED = the receipt names the edit; every FIXED is by reading only,
none compiled or run (receipts state UNVERIFIED), so FIXED is not ALIVE. A row stays OPEN when no
receipt names its pointer. IN-FLIGHT = owning fix lane (R2-receipts, R3-law-types, R4-storage-perf)
was lease-refused and is being redone in a parallel lane.
Row id = `<audit lane>-<n>` in the order findings appear under that lane's `LANE-DONE` marker.
Severity is the audit's label. Owner = wave-3 fix area (lane name assigned by coordinator).

Audit lane map: R1 receipt_chain/law; M3 elixir test hygiene; R2 abi ops; Z1 ggen_sync/pack;
M2 telemetry; Z2 error taxonomy/ash_r2rml; V2 config/application; V3 error class/extension;
Z3 registry/introspection; V1 facade/result/context; M1 guides/fetch task; A2 kill switch;
A1 pool/host/supervision; A3 fetch/engine_load/wasm_config; R4 duckdb_op; R5 ledger/evidence
ring; R6 public API/Cargo; R3 wasm ffi; X1 CI/scripts; R7 hooks/policy/attest/shacl_guard.

Reference-doc sweep (docs/reference/*.md, grep for Error.Class, lease policy, abi_bin, duckdb
op): zero hits at registration time, so no stale statement was edited.

Not verified by this register: every finding below is audit text only. Pointers were not
re-read against code by this lane; owner lanes verify before fixing and mark ALREADY-FIXED.

## Findings

| Id | Sev | Pointer | Finding | Owner | Status |
|---|---|---|---|---|---|
| R1-1 | MED | src/receipt_chain/record.rs:847 | fold_in_v2_epoch hashes base, schema, epoch_json with no length prefix or domain tag | rust-chain | IN-FLIGHT (lane R2-receipts lease-refused; redo in parallel lane) |
| R1-2 | MED | src/receipt_chain/record.rs:382-386 | admission mixed_hash unkeyed, untagged, serde-tuple dependent; unwrap on public fn | rust-chain | IN-FLIGHT (lane R2-receipts lease-refused; redo in parallel lane) |
| R1-3 | LOW | src/receipt_chain.rs:363-367 | obligations len as u32 and u32 sum truncate or overflow | rust-chain | IN-FLIGHT (lane R2-receipts lease-refused; redo in parallel lane) |
| R1-4 | LOW | src/receipt_chain.rs:614 | expect on promoted invariant can panic instead of CoreError | rust-chain | IN-FLIGHT (lane R2-receipts lease-refused; redo in parallel lane) |
| R1-5 | LOW | src/law.rs:544,604 | unreachable! on external non_exhaustive ReasonerLimit | rust-law | IN-FLIGHT (lane R3-law-types lease-refused; redo in parallel lane) |
| R1-6 | LOW | src/receipt_chain.rs:983 | check_chain_linkage compares hex case-sensitively, no canonical-lowercase check | rust-chain | IN-FLIGHT (lane R2-receipts lease-refused; redo in parallel lane) |
| M3-1 | LOW | graphlaw_ex/test/graphlaw_ex/kill_switch_test.exs:211-228 | telemetry handler detached inline, no on_exit | ex-tests | OPEN |
| M3-2 | LOW | graphlaw_ex/test/safety/kill_switch_safety_test.exs:78-80 | wall-clock 2s bound adds flake risk | ex-tests | OPEN |
| M3-3 | LOW | graphlaw_ex/test/safety/kill_switch_safety_test.exs:91-103 | concurrent-trip test may never exercise interleaving | ex-tests | OPEN |
| R2-1 | HIGH | src/abi/ops/query_engines.rs:187-215 | op_datalog has no iteration, derived-fact or output cap | rust-abi | OPEN |
| R2-2 | MED | src/abi/ops/governance.rs:347-350,369 | plan/atom/policy limits checked after earlier steps run, not before dispatch | rust-abi | OPEN |
| R2-3 | MED | src/abi/ops/governance.rs:15-26,33 | law_err recovers limit details by parsing English refusal text | rust-abi | OPEN |
| R2-4 | MED | src/abi.rs:179-190 | bad_refusal types every malformed request as RefusalKind::Unsupported | rust-abi | OPEN |
| R2-5 | MED | src/abi/lease.rs:186-204 | unverified_lease expiry vs caller now_unix, no skew check, response not marked unverified | rust-abi | OPEN |
| R2-6 | LOW | src/abi/ops/governance.rs:281-290 | op_policy counts entries only when policy parses; parse-twice inconsistency | rust-abi | OPEN |
| R2-7 | LOW | src/abi.rs:325-343 | duckdb op dispatched but absent from capabilities.ops and 14-op list (HIDDEN_OPS) | rust-abi | DEFERRED (see DEF-1) |
| R2-8 | LOW | src/abi/ops/governance.rs:341-353 | MAX_LAW_STEPS checked in loop so observed = 1001, not true count | rust-abi | OPEN |
| Z1-1 | MED | graphlaw_ex/scripts/ggen_sync.sh:~118 | run_sync always passes --skip-ea-admission; admission bypassed incl. --check | ex-ggen | OPEN |
| Z1-2 | MED | graphlaw_ex/scripts/ggen_sync.sh:~110 | prune_strays never scans capability/ or result/; orphans survive op removal | ex-ggen | OPEN |
| Z1-3 | MED | graphlaw_ex/scripts/ggen_sync.sh:~35-60 | projection set computed from JSON plus a regex, outside the ontology | ex-ggen | FIXED (E12: ops+models read from registry JSON, --check asserts manifest==registry; not run) |
| Z1-4 | LOW | graphlaw_ex/scripts/ggen_sync.sh:~140 | --check git diff conflates in-flight edits with drift; staged edits pass | ex-ggen | OPEN |
| Z1-5 | LOW | ggen-marketplace gate-court.toml | pass/fail witness court is stem-matched, not run | marketplace | OPEN |
| Z1-6 | LOW | gates/180_coverage_complete.rq | empty ledger passes vacuously in the consumer | marketplace | OPEN |
| Z1-7 | LOW | graphlaw_ex/ggen/HANDWRITTEN.md | 23 table-projectable hand-written files, no drift gate (ledger consistent with disk) | coordinator | OPEN |
| Z1-8 | LOW | graphlaw_ex/ggen.toml | no generation.rules; pack version pinned only in shell; relative pack path fails at sync | ex-ggen | OPEN |
| M2-1 | MED | graphlaw_ex/lib/graphlaw_ex/engine_load.ex:321-330 | engine admit event uses raw :telemetry.execute, bypassing redaction; no duration | ex-telemetry | OPEN |
| M2-2 | MED | graphlaw_ex/lib/graphlaw_ex/telemetry.ex:307-310 | classify(_) treats non-tuple results as ok | ex-telemetry | FIXED (E5: unexpected shapes now :refused; not run) |
| M2-3 | MED | graphlaw_ex/lib/graphlaw_ex/host.ex:497-505 | outcome vocabulary inconsistent across admit, host call, engine load | ex-telemetry | OPEN |
| M2-4 | LOW-MED | graphlaw_ex/lib/graphlaw_ex/telemetry.ex:181-182 | source_digest unvalidated, unbounded tag cardinality | ex-telemetry | OPEN |
| M2-5 | LOW-MED | graphlaw_ex/lib/graphlaw_ex/telemetry.ex:62-66 | op passes any binary, no allowlist | ex-telemetry | OPEN |
| M2-6 | LOW | graphlaw_ex/lib/graphlaw_ex/host.ex:409,549,860 | recycle reason exit tag lost in telemetry | ex-telemetry | OPEN |
| M2-7 | LOW | graphlaw_ex/lib/graphlaw_ex/telemetry.ex:219-223 | catalog Enum.find on hot path | ex-telemetry | FIXED (E5: compile-time catalog heads; not run) |
| M2-8 | LOW | graphlaw_ex/lib/graphlaw_ex/telemetry.ex:243-273 | emit inside try can mask a telemetry error as call failure | ex-telemetry | OPEN |
| M2-9 | LOW | graphlaw_ex/lib/graphlaw_ex/host.ex:328,823,843 | retry warnings have no metadata or telemetry event | ex-telemetry | OPEN |
| Z2-1 | MED | /Users/sac/ash_r2rml/lib/ash_r2rml/graphlaw.ex:135-152 | translate_refusal copies raw and details unredacted | ash-r2rml | OPEN |
| Z2-2 | MED | /Users/sac/ash_r2rml/lib/ash_r2rml/graphlaw.ex:163-186 | consumer re-implements code map; classes and retryability diverge | ash-r2rml | OPEN |
| Z2-3 | MED | /Users/sac/ash_r2rml/lib/ash_r2rml/graphlaw.ex:186 | catch-all map_code returns WORKER_CRASHED; ceiling_unmet loses class | ash-r2rml | OPEN |
| Z2-4 | MED | /Users/sac/ash_r2rml/lib/ash_r2rml/graphlaw.ex:134-152 | structured-token refinement bypassed (raw-sql-forbidden etc.) | ash-r2rml | OPEN |
| Z2-5 | LOW | /Users/sac/ash_r2rml/lib/ash_r2rml/graphlaw.ex:155-160 | every GraphlawEx.Error collapsed into WORKER_CRASHED | ash-r2rml | OPEN |
| Z2-6 | LOW | graphlaw_ex/lib/graphlaw_ex/error/table.ex:27-35,48-57 | saturated, batch_member_failed, duckdb_failure map to :invalid class | ex-error | FIXED (E4: saturated, duckdb_failure, engine_unclassified now :unknown; batch_member_failed not named in receipt; not run) |
| Z2-7 | LOW | graphlaw_ex/lib/graphlaw_ex/error.ex:187-190,226 | Error struct not Splode-native; Class moduledoc to_ash mismatches ash_r2rml | ex-error | OPEN |
| V2-1 | MED | graphlaw_ex/lib/graphlaw_ex/wasm_config.ex:~351 | runtime opts limits never validated, silently fall to defaults | ex-config | OPEN |
| V2-2 | MED | graphlaw_ex/lib/graphlaw_ex/application.ex:~83 | wasm binary read once and embedded in child specs; stale on restart | ex-config | OPEN |
| V2-3 | MED | graphlaw_ex/lib/graphlaw_ex/config.ex:~54 | trusted_keys unvalidated at boot | ex-config | OPEN |
| V2-4 | LOW | graphlaw_ex/lib/graphlaw_ex/wasm_config.ex:~385 | app_path rescue bakes in compile-time dir | ex-config | FIXED (E9: fallback removed, typed refusal; not run) |
| V2-5 | LOW | graphlaw_ex/lib/graphlaw_ex/wasm_config.ex:~210 | blake3 sidecar not cross-checked against artifact.blake3 | ex-config | ALREADY-FIXED (E9) |
| V2-6 | LOW | graphlaw_ex/lib/graphlaw_ex/application.ex:~72 | no fail-closed option when kill_switch_path absent | ex-config | OPEN |
| V2-7 | LOW | graphlaw_ex/config/test.exs, config.ex | kill_switch_test_mode accepted in any env, no prod refusal | ex-config | OPEN |
| V3-1 | MED | graphlaw_ex/lib/graphlaw_ex/error/class.ex:81-91 | kind table contradicts code table and moduledoc; doctest locks it in | ex-error | FIXED (E4: Class.of/1 reworked, doctests updated; not run) |
| V3-2 | LOW | graphlaw_ex/lib/graphlaw_ex/error/class.ex:93-101 | compile-time coverage check can pass vacuously | ex-error | FIXED (E4: CompileError guard in error/table.ex; not run) |
| V3-3 | LOW | graphlaw_ex/lib/graphlaw_ex/error/class.ex:131-136 | of/1 documented total but uses Map.fetch! | ex-error | FIXED (E4: Map.get; not run) |
| V3-4 | LOW | graphlaw_ex/lib/graphlaw_ex/capability/extension.ex:144-147 | default returns requires local t() type, undocumented | ex-extension | FIXED (E12: moduledoc documents returns: as free type expression; not run) |
| V3-5 | LOW | graphlaw_ex/lib/graphlaw_ex/capability/extension.ex:136-142 | delegate target arity never checked | ex-extension | OPEN |
| V3-6 | LOW | graphlaw_ex/lib/graphlaw_ex/capability/extension.ex:103-124 | arg_types and args not validated; unrelated crashes | ex-extension | FIXED (E12: typed ArgumentError for bad target/arg_type, tests added; not run) |
| Z3-1 | HIGH | graphlaw_ex/lib/graphlaw_ex/capability/registry.ex:2124 | generated refusal_kind_classes all :unknown vs registry JSON (pack template) | marketplace | OPEN |
| Z3-2 | HIGH | graphlaw_ex/lib/graphlaw_ex/error/class.ex:79-91 | hand-written kind table is a second source; guard checks keys only | ex-error | OPEN |
| Z3-3 | MED | graphlaw_ex/lib/graphlaw_ex/introspection/tables.ex:10,31-35 | op-to-limit scoping hand-written, disagrees with registry limit_meta scope | ex-introspection | OPEN (E5 PARTIAL: prefixes added; derivation from limit_meta needs introspection.ex) |
| Z3-4 | LOW | graphlaw_ex/lib/graphlaw_ex/introspection.ex:149 | cites nonexistent projection; three delegation layers for error_class | ex-introspection | OPEN |
| V1-1 | MED | graphlaw_ex/lib/graphlaw_ex/result/admitted.ex:38 | from_map assumes receipts is a list; can raise | ex-result | FIXED (E2; not run) |
| V1-2 | MED | graphlaw_ex/lib/graphlaw_ex.ex:100,143 | two refusal structs across facade and typed verbs | ex-result | FIXED (E2: Result.new converts to GraphlawEx.Refusal; both structs kept, facade still returns Capability.Refusal; not run) |
| V1-3 | LOW | graphlaw_ex/lib/graphlaw_ex.ex:~210-310 | 28 facade defdelegates have no @spec | ex-result | FIXED (E2; not run) |
| V1-4 | LOW | graphlaw_ex/lib/graphlaw_ex/capability/api.ex | args required for every op; capabilities() raises | ex-result | FIXED (E2: capabilities args default; not run) |
| V1-5 | LOW | graphlaw_ex/lib/graphlaw_ex/capability/dispatch.ex:58-62 | fetch_arg asymmetric for list args; step-floor may be bypassed | ex-dispatch | OPEN |
| V1-6 | LOW | graphlaw_ex/lib/graphlaw_ex/context.ex:103-115 | merge cannot clear a field; Context inspect leaks ids | ex-result | FIXED (E2: explicit nil clears, redacting Inspect impl; not run) |
| V1-7 | LOW | graphlaw_ex/lib/graphlaw_ex/result/*.ex | only Admitted, Receipt, N3 hide :raw in inspect (template) | marketplace | FIXED (E1: @derive Inspect except :raw in result_struct/result_term templates; needs ggen sync; not run) |
| M1-1 | MED | graphlaw_ex/guides/ash_r2rml_migration.md:104-119 | guide matches :resource_limit; code emits REFUSED_RESOURCE_BOUND | ex-docs | OPEN |
| M1-2 | MED | graphlaw_ex/guides/ash_r2rml_migration.md:105,127 | links docs/refusals.md, absent from package | ex-docs | OPEN |
| M1-3 | LOW | graphlaw_ex/guides/getting_started.md:8-10 | dependency snippet is a literal placeholder | ex-docs | OPEN |
| M1-4 | LOW | graphlaw_ex/lib/mix/tasks/graphlaw_ex.fetch.ex:80-88 | any read error triggers network download, silently | ex-fetch | OPEN |
| M1-5 | LOW | graphlaw_ex/lib/mix/tasks/graphlaw_ex.fetch.ex:91-100 | .part not cleaned on from_path failure | ex-fetch | FIXED (E10; not run) |
| M1-6 | LOW | graphlaw_ex/lib/mix/tasks/graphlaw_ex.fetch.ex:48 | run ignores args | ex-fetch | FIXED (E10: OptionParser strict --path; not run) |
| M1-7 | LOW | graphlaw_ex/CHANGELOG.md:7-23 | 26.11.25 entry contradictory; no 0.1.0 migration notes | ex-docs | OPEN |
| A2-1 | MED | graphlaw_ex/lib/graphlaw_ex/kill_switch.ex:125-135 | only highest-epoch document live; narrow trip re-admits earlier revocations | ex-kill | OPEN |
| A2-2 | MED | graphlaw_ex/lib/graphlaw_ex/kill_switch/store.ex:60-62 | missing or rolled-back store file fails open | ex-kill | OPEN |
| A2-3 | MED | graphlaw_ex/lib/graphlaw_ex/kill_switch/server.ex:83-90 | persist failure leaves live but non-durable trip; retry hits stale_epoch | ex-kill | FIXED (E6: pending retry, tests added; not run) |
| A2-4 | MED | graphlaw_ex/lib/graphlaw_ex/application.ex:70-82 | durability opt-in; default fail-open on restart | ex-kill | OPEN |
| A2-5 | MED | graphlaw_ex/lib/graphlaw_ex/authority.ex:439-446 | trusted_keys has two incompatible shapes (list vs map) | ex-kill | OPEN |
| A2-6 | LOW | graphlaw_ex/lib/graphlaw_ex/kill_switch/server.ex:67,100-113 | keys cached at init; rotation breaks replay | ex-kill | FIXED (E6: keys read per install; boot replay still reads once; not run) |
| A2-7 | LOW | graphlaw_ex/lib/graphlaw_ex/kill_switch.ex:155-167 | scope compare raw ==; case-sensitivity | ex-kill | FIXED (E6: check/1 downcases digest, test added; not run) |
| A2-8 | LOW | graphlaw_ex/lib/graphlaw_ex/kill_switch.ex:111-113 | node-local lock; direct install bypasses persistence | ex-kill | DEFERRED (see DEF-5) |
| A2-9 | LOW | graphlaw_ex/lib/graphlaw_ex/kill_switch.ex:195 | erase gated only by runtime env | ex-kill | OPEN |
| A1-1 | HIGH | graphlaw_ex/lib/graphlaw_ex/pool.ex:128-137 | route/1 returns unavailable member when capacity == size | ex-pool | OPEN |
| A1-2 | MED | graphlaw_ex/lib/graphlaw_ex/host.ex:419-433,808-851 | recycle/reap/instantiate block host handle_call | ex-pool | OPEN |
| A1-3 | MED | graphlaw_ex/lib/graphlaw_ex/host.ex:316-336 | init/1 loads synchronously; serial Workers start | ex-pool | FIXED (E8: handle_continue load; not run) |
| A1-4 | MED | graphlaw_ex/lib/graphlaw_ex/application.ex:94,154 | KillSwitch.Server restart restarts engine and Pool (rest_for_one) | ex-pool | FIXED (E8: :one_for_one; not run) |
| A1-5 | LOW | graphlaw_ex/lib/graphlaw_ex/pool.ex:116-122,138-145 | retry_once may run call twice; 2x deadline | ex-pool | FIXED (E8: retries only to a different live pid; not run) |
| A1-6 | LOW | graphlaw_ex/lib/graphlaw_ex/pool/leases.ex:36,83-90 | stale gate after :kill of Leases | ex-pool | DEFERRED (see DEF-4) |
| A3-1 | HIGH | graphlaw_ex/lib/graphlaw_ex/wasm_config.ex:263-268 | blake3 sidecar overrides manifest pin unchecked | ex-config | ALREADY-FIXED (E9 sidecar cross-check) |
| A3-2 | MED | graphlaw_ex/lib/mix/tasks/graphlaw_ex.fetch.ex:141-146 | fixed .part name, no lock; raw rename error | ex-fetch | OPEN |
| A3-3 | MED | graphlaw_ex/lib/mix/tasks/graphlaw_ex.fetch.ex:97-129 | http_get unbounded buffer; unrestricted redirects | ex-fetch | OPEN |
| A3-4 | MED | graphlaw_ex/lib/graphlaw_ex/engine_load.ex:66-70,169 | admit/2 no single-flight on cache miss | ex-engine-load | FIXED (E7: wired + wiring test; not run) |
| A3-5 | MED | graphlaw_ex/lib/graphlaw_ex/wasm_config.ex:224-232 | staged_path returns bad vendored file, never falls back to cache | ex-config | OPEN |
| A3-6 | LOW | graphlaw_ex/lib/mix/tasks/graphlaw_ex.verify.ex:95-105 | manifest has no artifact.size; size gate inactive | ex-fetch | OPEN |
| A3-7 | LOW | graphlaw_ex/lib/graphlaw_ex/engine_load.ex:89-103 | verify_artifact returns :ok for :unpinned | ex-engine-load | OPEN |
| A3-8 | LOW | graphlaw_ex/lib/mix/tasks/graphlaw_ex.doctor.ex:119 | doctor and Host use different pin resolution | ex-fetch | OPEN |
| R4-1 | MED | src/duckdb_op.rs:780 | exec holds global sessions mutex across exec_on | rust-duckdb | OPEN |
| R4-2 | MED | src/duckdb_op.rs:121,1051 | cursor cap pre-check and insert not atomic; limit refusal after commit | rust-duckdb | OPEN |
| R4-3 | LOW | src/duckdb_op.rs:1051 | fetch_chunk holds cursor mutex while building page | rust-duckdb | OPEN |
| R4-4 | LOW | src/duckdb_op.rs:~255 | enable_external_access on; safety rests on attach policy and AST admission | rust-duckdb | OPEN |
| R4-5 | LOW | src/duckdb_op.rs:~815 | params_of lossy u64 above i64::MAX to 0.0 | rust-duckdb | OPEN |
| R5-1 | MED | src/query_ledger/storage.rs:642-650 | fixed temp name shared by writers | rust-ledger | IN-FLIGHT (lane R4-storage-perf lease-refused; redo in parallel lane) |
| R5-2 | MED | src/query_ledger/storage.rs:138-146 | check-then-write TOCTOU; no lock on CHAIN read-modify-write | rust-ledger | IN-FLIGHT (lane R4-storage-perf lease-refused; redo in parallel lane) |
| R5-3 | MED | src/query_ledger/storage.rs:612-633 | quarantine move not crash-atomic | rust-ledger | IN-FLIGHT (lane R4-storage-perf lease-refused; redo in parallel lane) |
| R5-4 | MED | src/evidence_ring.rs:1081-1092,631 | recovery and append with no inter-process lock | rust-ledger | IN-FLIGHT (lane R4-storage-perf lease-refused; redo in parallel lane) |
| R5-5 | LOW | src/receipt_store.rs:351-353 | publish rename without dir-fsync ordering; mtime sweep | rust-ledger | IN-FLIGHT (lane R4-storage-perf lease-refused; redo in parallel lane) |
| R5-6 | LOW | src/receipt_store.rs:373-380 | publish_new removes temp before dir fsync | rust-ledger | IN-FLIGHT (lane R4-storage-perf lease-refused; redo in parallel lane) |
| R5-7 | LOW | src/evidence_ring.rs:501-504 | seq consumed on closed or full ring; gap accounting unverified | rust-ledger | IN-FLIGHT (lane R4-storage-perf lease-refused; redo in parallel lane) |
| R5-8 | LOW | src/receipt_chain/store.rs:90-117 | parent fsync failure after append reported as Err | rust-ledger | IN-FLIGHT (lane R2-receipts lease-refused; redo in parallel lane) |
| R6-1 | MED | src/duckdb_eval.rs:218-219 | EvaluatorError public enum without non_exhaustive | rust-api | DEFERRED (see DEF-3) |
| R6-2 | MED | src/proof_lease.rs:26-27 | ProofLeaseError public enum without non_exhaustive | rust-api | DEFERRED (see DEF-3) |
| R6-3 | MED | Cargo.toml:~203-260 | example files declared as installable bins; attest lacks required-features | coordinator | DEFERRED (see DEF-2) |
| R6-4 | MED | src/bin/graphlaw_attest.rs:1-8 | signing seed passed on command line | rust-api | OPEN |
| R6-5 | LOW | src/lib.rs:~172-215 | BACKEND_AUTHORITIES revision hand-maintained vs git rev | rust-api | OPEN |
| R6-6 | LOW | Cargo.toml:~170 | docs.rs feature list omits runner-eh | coordinator | DEFERRED (see DEF-2) |
| R6-7 | LOW | Cargo.toml [dependencies] | abi feature a no-op for dependencies; light-library claim overstated | coordinator | DEFERRED (see DEF-2) |
| R6-8 | LOW | Cargo.toml rust-version | MSRV equals pinned toolchain; no older-compiler check | coordinator | DEFERRED (see DEF-2) |
| R6-9 | LOW | deny.toml advisories | ignore expiries lapse 2026-11-28; two ignores unverified | coordinator | DEFERRED (see DEF-2) |
| R6-10 | LOW | src/lib.rs:109-115 | pub use purrdf crates re-exported from git fork | rust-api | DEFERRED (see DEF-3) |
| R3-1 | MED | wasm/src/ffi.rs:260-266 | saturated pack_response shares one buffer; host UAF | rust-wasm | FIXED (R1-wasm-ffi: immortal error buffer; rustfmt only; saturated path untested) |
| R3-2 | MED | wasm/src/ffi.rs:164-175,217-232 | budget counts requested bytes; OOM aborts instead of null | rust-wasm | OPEN |
| R3-3 | LOW | wasm/src/ffi.rs:199-212 | call_buf panic leaks budget under unwind | rust-wasm | FIXED (R1-wasm-ffi: BudgetGuard, unit test; not run) |
| R3-4 | LOW | wasm/src/ffi.rs:253-265 | pointer packing sound on wasm32 only; no const assert | rust-wasm | FIXED (R1-wasm-ffi: pack masks to 32 bits; const assert not added; not run) |
| R3-5 | LOW | wasm/src/ffi.rs:260 | declared length unwrap_or(u32::MAX) latent mismatch | rust-wasm | FIXED (R1-wasm-ffi: over-u32 response becomes limit_response; not run) |
| R3-6 | LOW | wasm/src/lib.rs:78-81 | register_static_extensions not gated on wasm32 | rust-wasm | OPEN |
| R3-7 | LOW | wasm/src/lib.rs:116-135 | atexit overrides shadow libc; destructors skipped | rust-wasm | OPEN (R1-wasm-ffi documented alternative only; shims unchanged) |
| X1-1 | HIGH | .github/workflows/ci.yml | none of scripts/* gates run in CI | coordinator-ci | OPEN |
| X1-2 | HIGH | .github/workflows/mutation.yml:21-30; scripts/mutation_court.sh:324 | cron-only; timeouts pass the court | coordinator-ci | OPEN |
| X1-3 | HIGH | scripts/mutation_court.sh:324 | audit text says fail-closed, not a defect | coordinator-ci | OPEN (audit: not a defect; confirm and close) |
| X1-4 | MED | .github/workflows/mutation.yml:21-30 | no libcurl or pkg-config install step | coordinator-ci | OPEN |
| X1-5 | MED | .github/workflows/ci.yml:~330 | external-oracle continue-on-error; unpinned pytest pm4py | coordinator-ci | OPEN |
| X1-6 | MED | .github/actions/wasi-duckdb/action.yml:54-57 | wasi-sdk tarball no checksum | coordinator-ci | OPEN |
| X1-7 | MED | .github/workflows/ci.yml:~22; elixir.yml:33,102,110 | floating action tags | coordinator-ci | OPEN |
| X1-8 | MED | .github/workflows/ci.yml castle-diff | unpinned nightly | coordinator-ci | OPEN |
| X1-9 | MED | .github/workflows/ci.yml semver job | lint allow-block may neutralize gate | coordinator-ci | OPEN |
| X1-10 | MED | .github/workflows/ci.yml audit job | cargo-audit unpinned; no deny warnings | coordinator-ci | OPEN |
| X1-11 | LOW | scripts/spc_court.sh:75 | verdict from self-reported load; not in CI | coordinator-ci | OPEN |
| X1-12 | LOW | scripts/mutation_court.sh:47 | callsite-revert leg: no --locked, no trap | coordinator-ci | OPEN |
| R7-1 | HIGH | src/hooks.rs:~226 | caller state can pre-seed kh:Firing and suppress a refuse hook | rust-hooks | OPEN |
| R7-2 | MED | src/policy.rs (admit_parsed, is_goal) | empty goal_facts makes every state a goal | rust-hooks | OPEN |
| R7-3 | MED | src/policy.rs (outcomes loop) | no caps on initial, outcomes, name lengths; duplicate outcomes accepted | rust-hooks | OPEN |
| R7-4 | MED | src/hooks.rs:~280-310 | per-firing dataset clone; no wall-clock or query limit | rust-hooks | OPEN |
| R7-5 | MED | src/attest.rs:~443-455 | sign_bytes/verify_bytes lack domain separation | rust-attest | OPEN |
| R7-6 | LOW | src/attest.rs:~420-450 | upper and lower case accepted; contradicts strict canonical form | rust-attest | OPEN |
| R7-7 | LOW | src/shacl_guard.rs | link-predicate set incomplete vs validator | rust-hooks | OPEN |
| R7-8 | LOW | src/shacl_guard.rs dfs | cyclic shapes graph accepted | rust-hooks | OPEN |
| R7-9 | LOW | src/policy.rs lit() | control characters emitted raw into N-Triples | rust-hooks | OPEN |

## Deferred (operator-stated)

Not worked in wave 3. Each carries a reason; reopening needs an operator decision.

| Id | Item | Reason | Linked rows | Status |
|---|---|---|---|---|
| DEF-1 | Hidden duckdb op advertisement in capabilities.ops | Surface is intentional (HIDDEN_OPS); advertising changes the manifest contract and the 14-op count claims | R2-7 | DEFERRED |
| DEF-2 | Cargo.toml autobins, runner-eh docs.rs feature, wildcard dependencies, MSRV, deny.toml expiries | Cargo.toml is coordinator-only (manifest edits by lanes revert each other) | R6-3, R6-6..R6-9 | DEFERRED |
| DEF-3 | Lease issued_unix as Option, DialectDecl, DeriveN3 types, non_exhaustive on public error enums, purrdf re-exports | Breaking public API; needs a version decision | R6-1, R6-2, R6-10 | DEFERRED |
| DEF-4 | Pool.Leases sharding | Design change beyond a leaf fix; only the stale-gate window is audited (transient) | A1-6 | DEFERRED |
| DEF-5 | Kill-switch cluster propagation beyond the hook | Needs a distribution design; node-local lock is the current contract | A2-8 | DEFERRED |
| DEF-6 | ABI version negotiation on the wire | Protocol change; ABI 1 is pinned in manifest and registry | none | DEFERRED |
| DEF-7 | frozen-duckdb gating | Conflicts with convention: DuckDB is a normal dependency, never a feature | R6-7 | DEFERRED |

## See Also

- `docs/jira/v26.11.19/AUDIT-20-LENS-2026-11-25.md`
- `docs/jira/v26.11.25/DEFINITION-OF-DONE.md`
- `docs/jira/v26.11.26/BACKLOG.md`
