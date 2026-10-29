# GL28-2 — Smalt Upgrade Implementation (in flight, 10-lane wave)

Baseline: graphlaw_ex is a path dep of smalt (smalt mix.exs:102,
`path: "../graphlaw/graphlaw_ex"`); the merge to upstream 5c30be06 is
resolving in a parallel lane; the gate config posture is landed
(config.exs/dev/test — enabled per-env, fail_closed: true, max_queue 64).

| Lane | Lease | Standing |
| --- | --- | --- |
| AdmissionGate module (lib/smalt/graphlaw_ex/admission_gate.ex) | new file + unit tests | IN-FLIGHT — envelope validation, SMALT-* codes, method filter, 16 MiB cap |
| Router seam insertion (:1680 + child-PUT :2166) | arm_gateway/router.ex | IN-FLIGHT — gate step after Policy.enforce, before Store.put; 422 via the fail-closed render path |
| Gate courts (flip-twin, anti-vacuity) | test/graphlaw_ex_gate_court_test.exs | IN-FLIGHT — valid/tamper/no-receipt/replay/oversize + gate-inverted flip |
| Config posture | config/{config,dev,test}.exs | **LANDED** — enabled per-env, fail_closed: true, prod host-owned |
| Data-plane insertion census | report | **LANDED** — cosmos.ex + post-merge blob_storage.ex entry points named |
| Test-infra census | report | **LANDED** — no ConnCase; real shared ETS Store, keyspace-prefix cleanup, poll helper, async: false |
| Verb migration map | report | IN-FLIGHT (resumed after 429) |
| Boot-order census | report | **LANDED** — zero implicit load; sanctioned start {GraphlawEx, wasm_path: vendored_path()} after Smalt.Store; boot-fatal typed at start_link, per-call degrade after |
| Receipt-source delegation audit | report | IN-FLIGHT — vc_export/affidavit_seal vs new minting shim |
| Dep wiring | smalt mix.exs | **LANDED** (coordinator) |

Gates to cite on completion: smalt `mix compile --warnings-as-errors`
exit 0; gate courts exit 0 (real Store sandbox + real bridge); wire-parity
court (byte-identical receipt hashes both paths); anti-vacuity flip
(gate inverted → tampered payload lands).

Falsifier: an injected non-conformant mutation must fail the gate; a
direct Store.put bypassing the gate is the residual risk the data-plane
census names for follow-up lanes.

## Insertion matrix (census, 2026-10-29 — full detail in lane receipt)

Natural (no threading): router.ex child-PUT chokepoint validate_child_put
(:2161, covers handle_child_put :2235 + sb_child_put :2566 + parented),
eg_topic_put (:6479), both roleAssignment PUTs (:6784/:6837 — currently
ungated beyond shape). Separate insertions: handle_cosmos_child_put
(:2892, re-rolls the chain inline). Thread via headers/opts: cosmos.ex
(signatures lack conn; routes pass tenant_headers) and blob_storage.ex
(MERGED side; 17 write entries, commit points enumerated). Fabricate
envelope: messaging/* engine writers bypass the wire guard into
BlobStorage — synthetic engine-principal envelope or explicit exemption
tier (the only chokepoint covering both faces is the BlobStorage layer).

## Validator migration map (census, 2026-10-29)

Order: (1) validator.ex:415-434 engine_verdict → Capability.API.shacl/2
(validator.ex:422 apply swap; :engine_unavailable maps 1:1); (2)
router.ex:3657 do_check_shape inherits via the validator (6 upstream
check_shape sites); (3) sql_row_gate.ex:315 per-row shacl; (4)
powl2/law_compiler.ex:92-153 → law verb (the ONLY step needing a
:construct lease — :ceiling_unmet pre-gate applies); (5) tests dual-run
with validate_shape/2 as differential oracle. Non-migrating: projections
(envelope remap only). New adapter surface: JSON payload → JSON-LD
serialization for `data` (dialect enum includes jsonld).
