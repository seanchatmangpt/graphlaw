# GL28 — Vision 2028 Register (C4 vs. tree, post-v26.10.28)

This file is the **single authoritative GL28 register**: one status per
GL28-n ticket lives here and nowhere else. Design authority stays with the
per-ticket files in this dir (GL28-2/-4/-6/-7/-13 specs); `WORK-ORDERS.md`
carries WO-1..WO-7 and points here for GL28-n statuses;
`VISION-2028-EXECUTION.md` is a stub pointing here (its dependency DAG and
court specifications are folded into the two sections below; it is retained
only for the PRIOR-ART-FIRST lane contract).

Part A (landed substrate) carries the receipts in RECEIPTS.md and the live
GitHub release. Parts B-D below are the honest gaps between the Vision 2028
C4 model and the tree, each with its typed falsifier.

## Dependency DAG (folded from VISION-2028-EXECUTION.md, 2026-10-29)

- WAVE 0 (window closure): GL28-9 mutation court solo -> GL28-10 ash_graphlaw
  P6/P7/P9 closure -> GL28-12 npm/Hex publish.
- WAVE 1 (cheap enforcement teeth, in-tree): GL28-6 Art.14 Ed25519 kill-switch
  -> GL28-7 A2A Authority ceiling. LANDED: GL28-5 conformance_claim matrix.
  Parallel ungated: GL28-11 graphlaw_ex residuals, GL28-13 wasm slimming,
  GL28-4 exporter spec, GL28-12 publish decisions.
- WAVE 2 (Article 12 persistence): GL28-1 wasm4pm OCEL 2.0 stream ->
  GL28-8 disk BLAKE3 receipt chain -> GL28-4 EU ledger exporter
  (GL28-4 also parallelizable).
- WAVE 3 (heavy cross-substrate): GL28-2 Smalt admission gate;
  GL28-3 ferroplan C01 lease adapter.

## Court specifications (falsifier pairs, must refuse)

Full design authority per ticket: `GL28-2-SMALT-WIRE-SPEC.md` §5-§8,
`GL28-4-EXPORTER-SPEC.md` §6-§7, `GL28-6-KILLSWITCH-DESIGN.md` §5,
`GL28-7-AUTHORITY-PORT-DESIGN.md` §4.

| Court | Ticket | Falsifier (must refuse) | Seam |
| --- | --- | --- | --- |
| `art14_killswitch_court` | GL28-6 | valid lease admits; revoked signature or bumped epoch -> `LeaseRevoked`, zero state mutation | Host pre-call envelope before `gl_alloc` (`src/attest.rs` + graphlaw_ex EngineLoad/Host) |
| `a2a_authority_ceiling_court` | GL28-7 | unsigned agent law/hooks -> `Error.Refused(:insufficient_authority)`; `:observe` ops (sniff/parse) survive | graphlaw_ex `Capability.API.run/3` pre-gate |
| `conformance_claim_court` | GL28-5 | parses `docs/reference/conformance_claim.md`, verifies every cited test binary exists in `Cargo.toml` and every digest matches `ARTIFACTS.blake3` | docs court |
| disk chain court | GL28-8 | re-verification from disk detects one flipped byte | `src/receipt_chain.rs` serialization |
| `ocel2_conformance_court` | GL28-1 | valid span passes wasm4pm schema + Petri-net check; missing `pe:sourceDigest` attribute -> trace rejected | beam4pm bridge module |
| `smalt_commit_gate_court` / `smalt_wire_parity_court` | GL28-2 | missing envelope, flipped receipt byte, engine-digest drift, payload mismatch; inverting the gate to always-commit must FAIL the parity court | Smalt router gate insertion point (spec §8) |
| `attestation_export` court | GL28-4 | forged payload byte fails verify; missing cited receipt downgrades the row to UNVERIFIED (never fabricates); missing input -> typed `MissingInput` | `src/attestation_export.rs` + `tests/attestation_export.rs` |
| `c01_plan_lease_court` | GL28-3 | critical-path sum exceeding the lease or a SHACL temporal-constraint violation refused pre-execution | graphlaw_ex `Capability.API.law/2` / `policy/2` |

## Wave 3 wire contracts

- **GL28-2**: Smalt ingress envelope `{:admitted_mutation, payload_bytes,
  receipt_blake3, engine_digest, ocel_frame}`; commit gate refuses any
  PUT/POST/DELETE without a receipt validating against the graphlaw_ex
  `:persistent_term` digest for that payload's canonical N-Quads projection.
  Authoritative shape + gate law: `GL28-2-SMALT-WIRE-SPEC.md` §2-§3
  (envelope, gate, refusals), §8 (discovered integration seam).
- **GL28-3**: ferroplan candidate plans as `glx:CandidatePlan` quads
  submitted through `Capability.API.law/2` under a monotonic lease ceiling.
  The exact gaps to close are enumerated in the GL28-3 discovery section
  below.

## Status sync 2026-10-28 (state re-read at write time)

- **GL28-5 — LANDED.** `docs/reference/conformance_claim.md`
  exists on disk (verified this sync). The enforcement-teeth sequencing
  slot GL28-5 held is closed; no further order re-asks for it.
- **GL28-6 — LANDED @ 4d2ffd0** (kill-switch: KillSwitch epoch, Host pre-envelope, :lease_revoked; court `test/graphlaw_ex/kill_switch_test.exs`);
  implementation lane in flight. Standing: DESIGN, not LANDED.
- **GL28-7 — LANDED @ 4d2ffd0** (authority: claim/check_op/check_ceiling, :ceiling_unmet pre-gate in run/3; court `test/a2a_authority_ceiling_court_test.exs`);
  implementation lane in flight. Standing: DESIGN, not LANDED.
- **GL28-13 — census filed** (`GL28-13-SLIMMING-CENSUS.md`): ICU drop
  (−3 MB) + MinSizeRel wasm32 rebuild (−10–18 MB) restores the original
  64 MiB gate without the 80 MiB SCAR ceiling.
- **GL28-2 — spec filed** (`GL28-2-SMALT-WIRE-SPEC.md`); implementation
  not started. Standing: DESIGN.
- **Upstream pack** — branch `blake3-digest-keys` @ `041a039ae` landed in
  /Users/sac/ggen-marketplace (verified: branch checked out, HEAD
  matches; 48 files, unpushed). Standing: LANDED-local,
  BLOCKED-external(push).
- **npm publish path** (`receipts/2026-10-24-npm-attempt-ledger.md`,
  re-read at this sync): `@seanchatmangpt/graphlaw` **already exists on
  the npm registry at 26.10.14** (ledger: until rerun A3 succeeds, the
  published row stays `26.10.14`). No manual first publish is needed —
  the fix is the **Trusted Publisher mapping** (operator console config,
  2-day expiry) + rerun A3 (pending operator). Hex publish path
  documented (dry-run needs no auth). Standing: DOCUMENTED, publish
  itself BLOCKED-external(TP-config).

## Part B — drawn but unbuilt

- **GL28-1** wasm4pm OCEL 2.0 conformance engine (Art. 12) — L.
  **ALIVE** (promoted 2026-10-29 from BLOCKED-external / drawn-but-
  unbuilt; promotion receipt: RECEIPTS.md, "GL28-1.1 register promotion",
  this dir). Landed substrate, verified-on-disk 2026-10-29:
  `BeamPM.GraphlawBridge` (/Users/sac/beam4pm/lib/beam4pm_graphlaw_bridge.ex)
  — `attach/0` (idempotent `:telemetry.attach_many` on
  `[:graphlaw_ex, :admit, :stop]`), `mapper/2` (pure
  engine_op_mapper-style admit-span -> OCEL 2.0 envelope), `ingest/1`
  (typed all-or-nothing refusals into the real `BeamPM.OcelAccumulator`),
  `conform/1` (native RF3 oracle binary -> Elixir engine-handle -> typed
  `{:error, {:conformance_oracle_absent, _}}`, one
  `beam4pm-graphlaw-bridge-conformance/v1` receipt on every path);
  envelope spec `GL28-1-ENVELOPE-SPEC.md` (this dir); 8-test court
  /Users/sac/beam4pm/test/beam4pm_graphlaw_bridge_test.exs (flip-twins,
  non-vacuity, conform leg — the loud-skip is now only the binary-absent
  fallback). (b) the pe:sourceDigest Art. 12 leg — **CLOSED 2026-10-29**:
  mapper emits `attributes["pe:sourceDigest"]` from the caller-pinned
  engine-minted `blake3:` LawState.id (`blake3:dc0088be…a46e`, two real
  collaborators agreeing: law-op `states` head == canonical-op id, mint
  receipt embedded), bridge ingest gate refuses absence typed
  (`{:missing_field, :source_digest}`), flip-twin witnessed zero-writes;
  receipt `receipts/graphlaw_bridge/gl28-1-legb-conform-1793284150930.json`
  (conformant, native RF3 oracle). (c) the real `[:graphlaw_ex, :admit,
  :stop]` emission witness — **CLOSED 2026-10-29**: real supervised
  GraphlawEx over the pinned 26.10.28 artifact through the PUBLIC
  admit path (made lawful by the admit/3 opts passthrough,
  graphlaw_ex.ex + authority.ex), detach anti-vacuity flip witnessed;
  witness receipt `receipts/graphlaw_bridge/
  gl28-1-legc-witness-1793281937341.json` (engine_sha256 `34fd4ada…`
  = on-disk artifact). Courts: graphlaw_ex 133 passed; bridge file
  11 passed / 0 skips (receipts: RECEIPTS.md, "GL28-1 legs (b)+(c) —
  ALIVE flip"). Leg (a) — executed
  oracle-binary court receipt — CLOSED 2026-10-29:
  `native/rf3-conformance-oracle` built from `native/rf3-ocel-oracle`
  (cargo release; 40.6s cited from the build lane) and placed, sha256
  `00fd5642…03361` byte-identical to the release artifact; bridge court
  8/8 with the native path witnessed by two
  `receipts/graphlaw_bridge/graphlaw-bridge-conform-*.json` receipts
  (`outcome: conformant` + flip-twin `refused` on
  `oracle_stats_diverged_from_accumulated`, both `oracle:
  native_rf3_binary`); suite re-run this session: 8 tests, 0 failures
  (receipt: RECEIPTS.md, "GL28-1 leg (a) — oracle-binary court executed").
  Falsifier unchanged: an injected non-conformant event must fail the
  checker (the refused flip-twin receipt is that flip, witnessed).
- **GL28-2** Smalt local-cloud integration — XL. Deterministic wire protocol,
  wire-parity court (same receipt bytes both paths), admission-only commit
  gate keyed on R_receipt presence.
- **GL28-3** autofde-lab/ferroplan bounded-plan integration — L. Planner
  adapter emitting law-op payloads with monotonic lease ceilings; court:
  an over-ceiling plan is refused at the turnstile (catalog C01).
- **GL28-4** EU High-Risk AI attestation exporter — M, policy-gated. Signed
  conformance-claim document per release (claims_and_evidence.md pattern).

## Part C — thin enforcement paths

- **GL28-5** Reclassify seed receipts under the EU-Article vocabulary — S.
  **LANDED 2026-10-28**: docs/reference/conformance_claim.md naming
  court -> article clause, receipt-gated, falsifier column.
- **GL28-6** Article 14 kill-switch — M. Operator-held Ed25519 revocation
  lease checked per call by EngineLoad/Host; court: revocation stops
  actuation with a typed refusal, flip-twin with an honest survivor.
- **GL28-7** Untrusted-agent ingress (A2A -> turnstile) — M. Port the
  Authority.claim/check_ceiling trio into graphlaw_ex; unsigned leases
  capped at :observe.
- **GL28-8** Receipt-chain serialization — M. Persist src/receipt_chain.rs's
  BLAKE3-chained frames per admission; court: re-verification from disk
  detects a single flipped byte.

## Part D — v26.10.28 debt gating the vision story

- **GL28-9** mutation court solo run (WO-2) — the window stands
  GO-except-WO-2 until it lands. No 2026-10-28/29 mutation-court receipt
  exists in `receipts/` (re-checked at this consolidation).
- **GL28-10** ash_graphlaw P6/P7/P9 parity closure + full suite + coordinator
  commit (in flight; = WORK-ORDERS.md WO-7).
- **GL28-11** graphlaw_ex residuals: dual refusal vocabulary consolidation,
  top-level verb re-exports, Host/Pool/Telemetry/EngineLoad unit courts,
  capability smoke default-path fix.
- **GL28-12** npm publish via the Trusted Publisher mapping + rerun A3, and
  the mix hex.publish decision. **NOT executed — no receipt exists**
  (RECEIPTS.md publish section stands PENDING; no publish receipt file in
  `receipts/`). The npm package itself already exists at 26.10.14 — see the
  Status sync row above.
- **GL28-13** wasm slimming SCAR (80 MiB ceiling is carrying cost) —
  census at `GL28-13-SLIMMING-CENSUS.md`; implementation not started.

### GL28-3 discovery (2026-10-29 census)

Both repos on disk (/Users/sac/ferroplan, /Users/sac/autofde-lab). The seam
is more wired than the C4 draws: graphlaw `Policy::admit`
(src/policy.rs:466-471) already consumes ferroplan `UniversalPlan` JSON
(planning_runtime.rs:313; PlanStep carries start + duration at :288-294),
and graphlaw_ex exposes `policy` as one of the 14 typed verbs. Gaps to
close for C01: (a) `glx:CandidatePlan` RDF projection (only `ce:`
vocabulary exists, upstream in chatman-ecosystem.ttl:15-16); (b) explicit
step-dependency edges (implicit in tsched.rs:93-185 consume/produce
logic, not materialized on the plan struct); (c) the lease-ceiling court
(`c01_plan_lease_court`) against `Capability.API.law/2`/`policy/2`.

### GL28-14 — MCP manifest projection (new, 2026-10-29)

A SPARQL CONSTRUCT over ontology.ttl emitting `tools/list` JSON-Schema for
the 14 typed verbs — a fourth ggen target platform alongside the Elixir,
wasm and npm projections. Thin by construction: the request field maps are
already cross-checked 14/14 against contracts/abi-manifest.json, and the
ggen projection machinery proved itself on the ash_graphlaw regeneration.
Receiver surface: graphlaw_ex's 14 typed verbs over packet-4 (75/75) are
the tools/call implementation; the projection is the manifest.

Standing discipline carried into the Vision-2028 positioning (2026-10-29
review): (1) "0 DPMO" is not claimable — what exists is bounded-input
correctness under the census ceilings plus P01 forgery resistance; inputs
outside the admitted grammar get typed refusals, not proofs. (2) The
118 ns SPC number is the hot-path receipt-fold leg, not a full
parse→reason→plan composite — only the first leg is witnessed. (3)
frozen-duckdb gives exact joins over the dataset; closure completeness is
rule-set-dependent — claim exactness, not universality. (4) GL28-3
dependency edges + glx:CandidatePlan remain unbuilt; (5) GL28-7 ceiling
gate is spec+design, not code (implementation in flight). The drop-in
claim is gated on WO-2 (mutation court) and the npm Trusted-Publisher
bootstrap.

### GL28-1 discovery (2026-10-29 census): the consumer already exists

/Users/sac/beam4pm carries the full OCEL 2.0 side: BeamPM.Ocel
(envelope validate/decode), BeamPM.OcelIngest.Router (POST /ocel/events,
typed refusals), BeamPM.OcelAccumulator, RF2 alignment/Petri-net
conformance + RF3 OCEL oracle (native binaries), and — decisive —
BeamPM.Evidence's telemetry-bridge pattern (event_names/0, attach_all/0,
engine_op_mapper/2, OtelBridge, ReceiptBridge at
lib/beam4pm_evidence.ex:77/97/143/217/259) already consuming
[:beam4pm, :engine, :graphlaw, op] events. GL28-1's remaining work is one
new bridge module (GraphlawBridge) attaching to
[:graphlaw_ex, :admit, :stop] with an engine_op_mapper-style mapping,
paired with the OcelIngest path — not a new consumer. The
ggen-generated BeamPM.Graphlaw facade is do-not-edit; admission helpers
live in BeamPM.GraphlawAdmission.

### Correction (2026-10-29)

Falsifier (corrected 2026-10-29): the byte-exact replay is the GitHub codeload URL (curl https://codeload.github.com/seanchatmangpt/purrdf/tar.gz/44f4979f… — GitHub's own gzip + prefix entry), sha256 must equal dc6f5f183e440ad91cb590107ceef7a3894b8549fda6de7ec9df3866a5004a75. `git archive` is content-equivalent only (byte-different: no codeload prefix/root-entry/pax header) — never apply the dc6f5f18 gate to it. Mismatch on the codeload path = the blob was load-bearing, restore it.

### ERRC tick-4 kept candidates (2026-10-29, adversarially scored)

- **GL28-13.1 — build native/rf3-conformance-oracle** (beam4pm): cargo
  build --release of native/rf3-ocel-oracle, placed at the path
  GraphlawBridge.conformance_oracle_bin/0 checks — converts conform/1's
  oracle leg from loud-skip to executed verdict + receipt. Falsifier:
  injected num_events divergence must flip to :refused; binary removal
  must restore the loud-skip.
- **ash_a2a.1 — denials.n3 comment shrink** (S): the comment attributes
  denial semantics to praxis `check_denials`; on the deployed engine the
  bare fuse is inert (sentinel re-route is the path). One-paragraph fix.
- **GL28-1.1 — register promotion to PARTIAL_ALIVE** (S): GL28-1 row
  cites the landed bridge + conform/1 + receipt schema; tops out at
  PARTIAL until (a) the oracle-binary court executes, (b) the
  pe:sourceDigest Art. 12 leg lands, (c) a real [:graphlaw_ex, :admit,
  :stop] emission witness replaces the synthetic-span court. (a) CLOSED
  2026-10-29 — binary built + placed, court 8/8, two native-path run
  receipts on disk (see the GL28-1 row and RECEIPTS.md); (b) CLOSED
  2026-10-29 — admit/3 passthrough + ingest gate, mint
  blake3:dc0088be…, receipt gl28-1-legb-conform-1793284150930.json;
  (c) CLOSED 2026-10-29 — witness through the public admit path,
  receipt gl28-1-legc-witness-1793281937341.json. Row: ALIVE.
- **purrdf.1 — bare-fuse finding upstream** (S): filed two-fuse
  order-flip regression court in purrdf-n3 (first-match-only fired_fuse
  is rule-order-dependent; suppresses second simultaneous denial). The
  ash_a2a sentinel re-route is the permanent consumer answer either way.
