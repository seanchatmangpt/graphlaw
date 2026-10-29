// ARCHIVED 2026-10-28 — executed once; receipts of record at
// docs/jira/v26.10.26/RECEIPTS.md. NOT re-runnable: its lane leases reference
// the vendor/purrdf-n3 tree deleted by the purrdf fork swap (paths now live at
// seanchatmangpt/purrdf crates/n3). Kept as a historical definition; author a
// new workflow version for any re-run.
export const meta = {
  name: 'moonshot-array-100',
  description: 'Moonshot array v2: pipelined max-concurrency execution (implement -> skeptics per lane, no barriers) of the 20 work packages + seal + integration',
  phases: [
    { title: 'W0-Seal', detail: '3 seal lanes (cached from run 1)' },
    { title: 'Array-A', detail: '11 independent WPs, pipelined implement -> skeptics' },
    { title: 'Reasoner-Tail', detail: 'P13 then P10 (serial on vendor reasoner.rs)' },
    { title: 'Array-B', detail: 'dependent WPs (P06 proof-wire, P05 seal, P09 bench-at-caps, P11 memo)' },
    { title: 'Integrate', detail: 'mutation campaign, full ladder, wasm re-pin, receipts' },
    { title: 'Synthesize', detail: 'four-pillar verdict' },
  ],
}

const REPO = "/Users/sac/graphlaw"
const ENV = "WASI_SDK_PATH=/Users/sac/wasi-sdks/wasi-sdk-34 CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS=\"-C target-feature=+exception-handling\""

const RUN_SCHEMA = {
  type: "object", required: ["lane", "status", "evidence"],
  properties: {
    lane: { type: "string" },
    status: { enum: ["done", "partial", "blocked"] },
    evidence: { type: "array", items: { type: "object", required: ["cmd", "exit"], properties: { cmd: { type: "string" }, exit: { type: "integer" }, note: { type: "string" } } } },
    notes: { type: "string" },
  },
}

function BASE(id, lease, mission, accept) {
  return [
    "You are lane " + id + " of the moonshot execution array. Canonical repo: " + REPO + " (uncommitted tree; coordinator owns git — NEVER run git state/mutation commands).",
    "FILE LEASE: " + lease + " — touch NOTHING else (parallel lanes; a lease breach reverts your work).",
    "CHECKPOINT LAW: after each milestone append one line to /tmp/lane-" + id + ".state (what just landed) — re-dispatches resume from it.",
    "No sleeps. No mocks. Real commands with real exits in your evidence. Build isolation: CARGO_TARGET_DIR=" + REPO + "/target-arr-" + id + " unless the mission says otherwise.",
    "MISSION: " + mission,
    "ACCEPTANCE: " + accept,
    "Return the schema: lane=" + id + ", status, evidence[] (cmd+exit+note).",
  ].join("\n")
}

phase("W0-Seal")
const seal = (await parallel([
  () => agent(BASE("W0-abi-pin", "src/registry or registry files ONLY as the court directs; tests/ read-only",
    "tests/abi_manifest_court has 1 failing test — run it, read the panic, reconcile the manifest digest pin it checks (this wave moved a digest; the court message names the expected flow). Fix per its own instructions, show green.",
    "cargo test --test abi_manifest_court exit 0 (CARGO_TARGET_DIR=target-arr-w0a)."),
    { label: "W0-abi-pin", phase: "W0-Seal", schema: RUN_SCHEMA }),
  () => agent(BASE("W0-vector-pin", "src/bin/dl_dualroute.rs read-only; tests/dl_dualroute_vector_pin.rs PIN TABLE ONLY; sibling fixtures dir writable by the court design",
    "tests/dl_dualroute_vector_pin fails: the fix wave changed emitted vector bytes. Run cargo run --bin dl_dualroute (CARGO_TARGET_DIR=target-arr-w0b) to re-emit all 4 fixtures, recompute the rev-normalized BLAKE3 of each (normalize: replace any line containing graphlaw_rev and a JSON-value marker with the indent-preserved normalized line), update the 4 PINS digests with reason: engine fix wave re-emit (P01/P02 landed), rerun green.",
    "cargo test --test dl_dualroute_vector_pin exit 0."),
    { label: "W0-vector-pin", phase: "W0-Seal", schema: RUN_SCHEMA }),
  () => agent(BASE("W0-runeh", "receipts/ or docs/ only for the receipt row; no src changes",
    "tests/graphlaw_run_eh_court (7 failed) is env-gated: build cargo build --features runner-eh --bin graphlaw-run-eh, run with GRAPHLAW_RUN_EH=<abs path> CARGO_TARGET_DIR=target-arr-w0c. Green under env → append the by-design env-gating row to docs/jira/v26.10.26/RECEIPTS.md (env + command + exit). Red under env → fix forward in src/bin/graphlaw_run_eh.rs.",
    "Court exit 0 under env, receipt row appended."),
    { label: "W0-runeh", phase: "W0-Seal", schema: RUN_SCHEMA }),
])).filter(Boolean)

// ---- Array-A: 11 independent WPs, pipelined implement -> skeptics ----
const A = [
  { id: "P01-proof", tier: 1, lease: "vendor/purrdf-n3/src/proof_check_n3.rs + proof_check.rs + proof_writer.rs + ast.rs + parser.rs (digest site only); vendor/purrdf-n3/Cargo.toml + tests/n3_forgery.rs; src/proof_lease.rs (new); src/lib.rs (one mod line); src/attest.rs (pub one helper); tests/n3_proof_lease_court.rs (new); vendor/README.md; CHANGELOG",
    mission: "Harden the proof checker (WP P01). H1: CheckPolicy trusted_resolve bool default false + legacy(); impure-builtin (IMPURE_BUILTINS) step conclusions excluded from resolve/match_condition supports/boundary evidence when hardened — obligation stays visible, resolution power dies. H2: SourceRef gains pub digest Option<String> (BLAKE3 at parse, single ctor in parser.rs); writer emits pe:sourceDigest on fact steps; read_steps parses it; check_fact 5-row policy: expect+equal → proceed + label match; expect+missing → C1; expect+differ → C1 cross-document splice killer; no-expect+present → Trusted(unverified source digest); none → as-is. H3: rule_applications rejects body-determined-but-unbound/self-bound head variables (concludes every instance of itself) — head-only universals and source-anchored general facts survive. TDD forge courts F1 (impure laundering splice → C4), F2 (cross-doc digest splice → C1), F3 (variadic head → C1) with flip twins (legacy() restores; honest variants pass). src/proof_lease.rs: verify_leased_proof binding check_proof to a caller lease digest. Update vendor/README parity count; CHANGELOG.",
    accept: "vendor suite green (new count recorded); forge courts green; n3_proof_pipeline_court + trust_attest + api_stability green; flips witnessed." },
  { id: "P02-budget", tier: 1, lease: "vendor/purrdf-n3/src/reasoner.rs (budget sites + census); src/law.rs (consts + dispatch rewrite); src/registry/emit_json.rs; src/abi/ops/governance.rs (fallback); tests/n3_budget_cliff_court.rs + tests/n3_limit_census_court.rs (new)",
    mission: "Continuous honest budgets (WP P02). SearchBudget.total_steps_seed set from report.match_steps at both construction sites; tick checks steps>=max_steps AND total_steps_seed+steps>=max_total_steps at ONE gate before increment — zero overshoot at search granularity. nested_options: max_iterations becomes remaining (max_iterations - iterations_used via absorb_result; keep the max(1) clamp). for_proof(options, max_depth): all caps from options; drop the 4096 at proof_writer.rs:295. Census: ReasonerLimit::graphlaw_name() wildcard-free 8 arms + ReasonerLimit::ALL + graphlaw_row(result, options) per-variant (name, observed, max); ReasonerStatistics gains backward_depth / backward_solutions_per_goal / term_depth recorded at hit sites (real observed, not max+1 fabrications). law.rs: consts N3_MAX_BACKWARD_DEPTH=32, N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL=1024, N3_MAX_TERM_DEPTH=64 declared; options literal sets all caps explicitly; dispatch = census-ordered find + graphlaw_row; DELETE the catch-all and the generic fallback. emit_json LIMIT_META + limits rows for the 3 new caps + n3_max_total_steps; governance fallback stops fabricating n3_iterations. TDD: 10-generator cliff court (chain/fanout/rematch/backward_chain/cycle-silent-pin/solutions/doubler/wrapper/crossproduct/nested_conclusion) with binary-searched max/max+1 pairs + census court (8 arms set-equal with the law table + registry + Display).",
    accept: "All 10 generators green with exact observed==max; census courts green; vendor + root suites green; registry regenerated via graphlaw-registry --write." },
  { id: "P03-wasm", tier: 1, lease: "tests/common (enclosure module), tests/wasm_memory_wall.rs, tests/wasm_enclosure_court.rs (new), src/bin/graphlaw_run_eh.rs, docs/reference/runner-exit-codes.md, docs/reference/wasm-gap-register.md",
    mission: "Hard WASM enclosure (WP P03). EnclosureHost in tests/common: wasm_exceptions + consume_fuel + epoch_interruption + OptLevel::None pinned; WallLimiter 64MiB; FuelPolicy Unbounded | PerOp(table); watchdog thread increments engine epoch every 50ms until stop-flag; seed fuel AND epoch deadline BEFORE instantiate (else _initialize traps — the number-one false red); reset both per request cycle; classify Trap::OutOfFuel then Trap::Interrupt. Typed envelopes synthesized host-side (panic=abort forbids in-guest): ok:false, error.kind ResourceLimit, details limit guest_fuel | guest_epoch, observed 0, max budget — matching graphlaw abi limit() byte-shape. Calibration court: per-op positive controls from registry/op-examples.json, seed 1e12, spend = seed minus get_fuel, budget = next_pow2_ceil(4×spend) floor 1e9 — never reuse 1e11 globally; admission leg proves headroom; determinism leg 1.2x. Epoch stall leg: doubling fixture, fuel Unbounded, 2s deadline → Interrupt + typed envelope + capabilities still serves. Campaign: 9 ops × 1000 leaking calls, 10 windows, fenced page curves (worst delta ≤ fence, non-accelerating), response byte-identical per window, every frame typed, instance survives + respawn serves. Stack probes: formula nesting 63 admits / 64 refuses typed; backward depth 31/33 with fuel ≥ 4× measured so the in-module limit fires first. run_eh: watchdog + exit 99 for epoch + flip the docs line that says epoch cannot fire.",
    accept: "wasm_enclosure_court green (calibration + envelope + epoch + campaign + probes); memory_wall rebound green; 9000 frames all typed; run_eh legs plus new 99; wasm_abi + finance courts unchanged." },
  { id: "P07-shacl", tier: 1, lease: "src/shacl_guard.rs, src/law.rs (one refusal site), src/sql_ast.rs (one site), src/abi/ops/query_engines.rs (one site), tests/shacl_boundary_cliff_court.rs (new), benches/shacl_boundary_cliff.rs (new) + one Cargo.toml bench block",
    mission: "Hard SHACL caps (WP P07). MEASURE FIRST: cliff bench with 3 isolated sweeps — depth {1,2,4,8,12,16,17-control-typed-refusal}, degree {1..1024 no cap exists today}, targets {64..16MiB}; p50/p95/p99 per cell; find where p99 crosses 5ms. Then pick MAX_DEGREE_CAP + MAX_QUAD_CAP + shapes-bytes cap at a defensible margin FROM the measurements and enforce typed (ResourceLimit, fail-closed) at all 3 call sites (law.rs, sql_ast.rs, query_engines.rs). Court: at-cap admits; over-cap refuses typed in microseconds; depth-17 control stays a wall; cyclic-shapes stays cheap; guard-removal flip witnessed. Adversarial worst-case-at-caps table into benches/reports/.",
    accept: "Caps enforced at 3 sites; cliff table recorded; at-cap Art.14 measurement recorded; shacl courts green." },
  { id: "P08-sieve", tier: 1, lease: "src/duckdb_eval.rs, src/duckdb_op.rs, tests/sieve_court.rs (new), benches/sieve_cliff.rs (new) + one Cargo.toml bench block, docs/refusals.md, README.md + docs/explanation claims lines, CHANGELOG",
    mission: "The sieve becomes law (WP P08). duckdb_eval.rs: DEFAULT_ROW_ENVELOPE = 64; EvaluatorError::RowEnvelopeExceeded with limit duckdb_result_rows, observed, max; eval_member takes max_rows, guard INSIDE the row loop at row_index > max_rows (mid-materialization, strictly pre-receipt — never post-loop which would materialize 10^6); C1 consult re-checks the envelope (hit over envelope → typed refusal); eval_ast_query/document delegate with the default + new bounded variants. duckdb_op.rs: exec wire field max_rows (absent → default, 0 refused; stream-mode max_rows stays a page size — document the distinction), typed ResourceLimit mapping on both exec paths, document-mode keeps BatchMemberFailed + adds limit fields. sieve_court legs: 10^6-row in-engine COPY → Parquet → mount → GROUP BY → rows_emitted == 64 + receipt + seal; unbounded AST → observed==65 max==64 exact; refusal not cached; wire legs via graphlaw::abi::call; RAW control leg proves the differential (raw pulls ≥65 where eval refuses). sieve_cliff bench: N in {1e4, 1e5, 1e6} × reduce cold/warm-stmt/warm-cache + refuse + raw control. Claims strip: README lines 57/59/64 and edge-sovereignty 17/20 lose unbacked millions/microseconds/500k-to-1KB text → court-scoped claim only. CHANGELOG Breaking entry.",
    accept: "sieve_court green incl. the 10^6 leg; refusal flip + raw differential witnessed; bench table printed; claims stripped." },
  { id: "P19-fuzz", tier: 1, lease: "tests/n3_boundary_fuzz.rs (new only)",
    mission: "The first hostile-N3 campaign (WP P19), following the wasm_boundary_fuzz.rs skeleton: seeded xorshift64* (fixed seeds), 6 classes × ≥1000 frames total, class histogram asserted complete, survival canary after EVERY frame, every response typed or ok — never a panic. Classes: (1) deep nesting braces/parens at depths 0,1,63,64,65,1000 — at-cap parses, over refuses typed; nested log:conclusion depth attack toward backward-depth 32; (2) unicode adversarial in prefixed names/IRIs/literals — lone continuation bytes, overlong encodings, BOM/ZWSP inside prefixes, RTL overrides; N3_MAX_TERM_BYTES boundary: term of exactly 65535 vs 65536 BYTES with multibyte chars straddling (limit counts BYTES); (3) closure bombs on the limit grid — chains vs ITERATIONS, fan-out vs MATCH_STEPS, cross-products vs DERIVED_FACTS/TOTAL_BYTES — typed refusals + canary; (4) prefix chaos — redeclaration shadowing, hostile-IRI prefix, suffix collision ex/exs, empty prefix, math-namespace hijack (a math: prefix rebinding that silently disables builtins — pin fail-closed silence or typed behavior, whichever is real), circular self-reference; (5) benign twins mandatory: every hostile doc has a one-mutation twin that MUST parse/derive (anti-vacuity); (6) native-vs-wasm byte-identity on refusals via tests/common helpers (typed skip without artifact).",
    accept: "0 untyped frames, 0 canary deaths, histogram complete, twins witnessed, divergences reported as findings." },
  { id: "P04-det", tier: 2, lease: "tests/determinism_court.rs (new only)",
    mission: "Byte-determinism court (WP P04), 5 legs: (1) seal stability N=100 fresh-engine iterations — query_hash/receipt_v2_hex/columnar_digest_hex/result_triples bytes identical + BLAKE3 fold-chain anti-vacuity (final digest differs from iteration-0 chain and from zeros); (2) envelope-minus-clock: redact execution_ms/consult_ms to Null, re-serialize, byte-equal across N (cached stays — it is a bool); disclose serde Value alphabetical order; (3) version pins: h_triples_serterm/h_columnar_canon goldens on a frozen fixture (mint via nocapture, paste consts with provenance) + permutation tripwires (row order changes hash; object_ids [a,b] vs [b,a] chain hashes differ) + re-mint protocol header; (4) cross-process: self-spawn via current_exe + env GLAW_P04_PROBE_CHILD=1, N=3 children print pure-lib payload hex (admitted_statement_hash + receipt_v2 with empty arms + 3-record chain with 16 const object_ids + attest receipt_payload rider) — children identical to each other AND in-process; (5) chains: FixedClock 5-record ledger built twice byte-identical + validator ok; SystemClock version self-verifies and head differs from FixedClock(42) (guaranteed); never assert two SystemClock builds diverge.",
    accept: "Legs 1/2/4/5 green + anti-vacuity; leg 3 green as documented version pins; zero src diff." },
  { id: "P14-halt", tier: 2, lease: "tests/n3_halt_latency_court.rs (new only)",
    mission: "Halt-latency honesty court (WP P14): (a) best case — fuse premise satisfiable at iteration 1, time parse+derive to N3Error::Fuse; (b) worst legal — fuse premise satisfiable only after full closure, time to Fuse refusal; (c) structural bound: in-process derivation cannot be preempted mid-flight — the ceiling is the budget; measure a budget-exhausted run time-to-typed-refusal as the honest ceiling; (d) note P03 host-side fuel/epoch abort as the wall-clock remedy (reference only). Assert halt is ALWAYS typed within the documented bound, never silent. Latency table + the memo honesty line in the receipt.",
    accept: "Court green, worst-case number recorded with command+exit." },
  { id: "P15-isolation", tier: 2, lease: "tests/n3_hypothesis_isolation_court.rs (new only)",
    mission: "Pillar 3 quarantine court (WP P15), 8 legs: (1) baseline snapshot via canonical_form rendering; (2) hypothesis A assert-route (quoted log:conclusion premise) — finding lands, hypothesis internals appear NOWHERE in the outer derived set, byte-identical rerun; evaluate-route (object variable) — formula value contains input+finding, not the contradiction; (3) contradicting hypothesis B → Ok with EMPTY derived (never an error, never fabricated); (4) PreparedReasoner order-flip: reason(A) then reason(B) vs B then A — per-batch sets identical, intersection empty (zero cross-batch residue); (5) re-baseline tripwire — a string:matches premise populates REGEX_CACHE mid-court, re-run baseline byte-identical (lazy-global alarm); (6) law rung: DeriveN3 receipt.added == 1 for the asserting pack, == 0 for the contradicting pack (quarantine twin), data-join sub-twin (state fact + hypothesis result in one rule); (7) ABI rung: op n3 request order-flip byte-identical derived per request; (8) MANDATORY negative twin: a plain forward rule DOES write (added==1, child != parent) proving the quarantine assertions can detect a real write. No mocks; gate hygiene (no mock markers).",
    accept: "All 8 legs green; the negative twin genuinely discriminates." },
  { id: "P16-transition", tier: 2, lease: "tests/n3_transition_isolation_court.rs (new only)",
    mission: "LawState transition isolation court (WP P16), 7 legs: (1) gate refusals leave the parent byte-identical (parse-fail child TierMismatch; ruleset-count disagreement; named-graph Unsupported default-graph-only) — id + serialized N-Quads + quad count unchanged; (2) budget-limit child isolated (limit fixture at the law rung); (3) blank-collision refusal isolated — PROBE FIRST: does the reasoner emit the parent bound blank verbatim in derived output? If it re-labels, the refusal is unreachable — receipt a typed known-limitation in the court header and pin the adjacent reachable contract, never fake it; (4) unknown-capability refusal isolated + zero residue (subsequent good transition yields the same child id as a fresh parse); (5) fork exactness: child quads == parent union independently-derived (public reason_n3_bounded cross-check) + receipt fields; (6) replay determinism: statement-order noise → same ids; refused-then-fork ≡ fork alone; (7) 5-step receipt chain offline auditor audit(root_id, receipts) reconstructs id chain + final id (honest boundary pinned: content bytes not reconstructable — API extension noted, out of scope).",
    accept: "All green or the collision leg receipted as known-limitation with probe evidence." },
  { id: "P17-wire", tier: 3, lease: "tests/wire_completeness_court.rs (new only)",
    mission: "Wire completeness court (WP P17): every BACKEND_AUTHORITIES row (src/lib.rs) has at least one ABI op exercising it (map via the registry op surface + src/abi/ops dispatch); execute each mapped op via graphlaw::abi::call with a minimal valid request — ok:true, or typed refusal (error.kind + details.code, never a panic); receipt-bearing ops assert audit fields (query_hash / step / authority / receipts array); refusal vocabulary check. Print the capability → op → fields matrix as the receipt. npm display-field proposal in notes only (no js edits).",
    accept: "Court green with the full matrix witnessed." },
]

phase("Array-A")
const A_impl_all = []
const A_skepticSets = []
const A_thunks = A.map(function (wp) {
  return function () {
    const implP = agent(BASE(wp.id, wp.lease, wp.mission, wp.accept),
      { label: wp.id, phase: "Array-A", schema: RUN_SCHEMA })
    const skvP = implP.then(function (rep) {
      if (!rep) return null
      const n = wp.tier === 1 ? 2 : 1
      return parallel(Array.from({ length: n }, function (_, i) {
        return function () {
          return agent("Adversarial skeptic (default-refute) for moonshot lane " + wp.id + " (skeptic " + (i + 1) + ") in " + REPO + ". Lane receipt: " + JSON.stringify(rep).slice(0, 4000) + " — re-derive every claim: run the commands (CARGO_TARGET_DIR=" + REPO + "/target-skv-" + wp.id + "-" + i + "), read the mechanisms, one revert-test per major mechanism (restore byte-exact, sha256 verify). Refute what does not reproduce. No fixes beyond the temporary revert.",
            { label: "skv:" + wp.id + ":" + i, phase: "Array-A", schema: RUN_SCHEMA })
        }
      })).then(function (vs) { return { lane: wp.id, verdicts: vs.filter(Boolean).length } })
    })
    A_skepticSets.push(skvP)
    return implP
  }
})
const A_impl_res = (await parallel(A_thunks)).filter(Boolean)
for (const l of A_impl_res) A_impl_all.push({ lane: l.lane, status: l.status })
const A_impl = A_impl_res
const A_skv = (await parallel(A_skepticSets.map(function (p) { return function () { return p } }))).filter(Boolean)
log("Array-A: " + A_impl.length + "/" + A.length + " implemented; " + A_skv.length + " skeptic sets")

// ---- Reasoner tail: P13 (after P02) then P10 (after P13) ----
phase("Reasoner-Tail")
const p02done = A_impl.find(function (l) { return l.lane === "P02-budget" })
const p13res = []
const p10res = []
if (p02done) {
  const p13 = await agent(BASE("P13-exhaust", "vendor/purrdf-n3/src/reasoner.rs (extend the P02 census — do not duplicate), src/law.rs (dispatch finish + Hooks refuse + N3Error::UnsupportedBuiltin), src/dialect/refusal.rs (2 new variants), src/abi/ops/governance.rs (hooks verdicts), src/abi/ops/query_engines.rs (op_n3 arm), src/bin/n3_derive.rs (fuse handling), tests/exhaustiveness_court.rs (new), docs/refusals.md",
    "Fail-closed exhaustiveness (WP P13), building on the P02 census: (1) if graphlaw_row(result, options) is absent, add it (per-variant name/observed/max, wildcard-free); delete ANY remaining catch-all in the law.rs limit dispatch and the generic fallback at the errors site (incomplete means limits OR errors non-empty — vendor invariant — so no fallback survives). (2) N3Error::UnsupportedBuiltin{builtin, detail, summary} + ReasonClass::UnsupportedBuiltin + op_n3 details.code UnsupportedBuiltin (compile-forced arms). (3) Step::Hooks: refuse verdicts fired → LawError::Refused with rendered verdicts, NO new state (worst silent-proceed path in the repo); the ABI hooks op surfaces verdicts + ok:false. (4) n3_derive: result.fuse checked BEFORE the status gate → stderr fuse report + exit 1, stdout empty (the vector oracle must never certify a forbidden derivation). (5) docs/refusals.md: 8-name census table + HookRefuse + UnsupportedBuiltin rows, arms-count line updated, append-only. Court tests/exhaustiveness_court.rs: census completeness (ALL length 8, ALL[0]==Iterations, names distinct); expected-RED sequencing (land the fuse-bin leg first and watch today exit-0 behavior fail post-fix): fuse bin leg via CARGO_BIN_EXE_n3_derive + flips, hooks-refuse leg + flips, UnsupportedBuiltin leg, nested max(1) clamp pin, refusals.md drift anchors.",
    "exhaustiveness_court green with flips; no wildcard arm remains in N3 limit/error dispatch; n3_derive honest exits; existing pins stay green."),
    { label: "P13-exhaust", phase: "Reasoner-Tail", schema: RUN_SCHEMA })
  if (p13) p13res.push({ lane: "P13-exhaust", status: p13.status })
  const p10 = await agent(BASE("P10-truth", "vendor/purrdf-n3/src/reasoner.rs (C1 8 arithmetic-blank sites + C2 quotient), vendor/purrdf-n3/tests/n3_reason.rs (+2 trials), tests/chicago_n3_datalog.rs (+2 ABI mirrors), vendor/README.md, benches/reports/sub50ms_statutory_courts.md (witness lines), docs/jira/v26.10.26/DEFINITION-OF-DONE.md + RECEIPTS.md, docs/ARD-26.10.25-DIALECTS.md, docs/jira/v26.10.22/PRD-P28-SIL.md, CHANGELOG",
    "Truth-in-claims (WP P10). C1 (8 sites in the math evaluators — difference, numeric list, unary both-arms, inverse mints, exponentiation/sum blanks; locate by eval function, line numbers approximated post-P02/P13): a blank operand/result records ReasonerError::InvalidOperand{builtin, premise, detail} + returns Vec::new() — thread the budget into the 7 math evaluators (their call sites already pass it). Positive control: an ALREADY-BOUND blank (blank_binding_name path) keeps working. C2: MATH_QUOTIENT exact via all_exact/BigInt when the division is exact, else f64/decimal as today (9007199254740993/1 must stay 9007199254740993; 7/2 stays 3.5). RED courts first (blank trials + the quotient mangle witnessed pre-fix), then fix, then GREEN; ABI mirrors in chicago_n3_datalog (blank → ok:false naming math:sum; quotient exact in derived text). Ledger 14 rows: bench report wasm bytes 60,880,445 → 60,895,924 (pin 0858a760…); DoD line 58 refresh; ARD roadmap markers (Cross-Domain Alignment, tenure suppression gmeow, 78-rules single-pass, §2.4 per-item truths: SSSOM envelope-only / EDOAL none / FnO list-primitives-only, the millions line dead until P08, loss_matrix specified-not-built); PRD line 43 bracketed correction (the runtime lexer is real for N3); RECEIPTS residuals 1-2 closed-by-X2 annotations; CHANGELOG. Do NOT rebuild the wasm module (the shipped pin stays valid). Do NOT touch append-only historical receipts.",
    "RED witnessed then GREEN: vendor 74/0 (72 + 2 trials), chicago mirrors green, all n3 courts green, ledger applied, truth greps clean."),
    { label: "P10-truth", phase: "Reasoner-Tail", schema: RUN_SCHEMA })
  if (p10) p10res.push({ lane: "P10-truth", status: p10.status })
} else {
  log("P02 did not complete — Reasoner-Tail lanes BLOCKED (typed): re-dispatch after P02")
}
log("Reasoner-Tail done: " + p13res.length + " P13, " + p10res.length + " P10")

// ---- Array-B: dependent WPs ----
phase("Array-B")
const B = [
  { id: "P06-statutory", tier: 2, lease: "packs/eu-ai-act (new), tests/eu_ai_act_packs_court.rs (new), tests/n3_statute_depth_court.rs (new), tests/abi_n3_proof_court.rs (new), src/law.rs (reason_n3_bounded_with_proof + N3_MAX_PROOF_BYTES), src/abi/ops/query_engines.rs (op_n3 proof field), src/abi_manifest.rs + contracts/abi-manifest.json + registry emission (limit row), js typegen",
    mission: "Statutory packs + depth court + ABI proof wire (WP P06). P06.4 FIRST: n3_statute_depth_court — staircase generator (n distinct chained rules), N in {10,20,50}, proof:true, Complete + proofs.len()==N + proof_to_n3 parses + check_proof valid verified==N+1 + per-hop claim lines present + the hop median tamper names the broken hop + budget witness iterations well under N3_MAX_ITERATIONS recorded. P06.1 Art.14 pack: layer-1 Horn join + fuse rule; courts at pre-engine (N3Error::Fuse report), law (RefusalKind::Fuse, no child state), ABI (error.kind Fuse) rungs + flip (minus the fuse admits, join in derived) + trigger guard. P06.2 Art.12: stratified two-file rules, the FIRST log:notIncludes court (exemption-absent derives the obligation; exemption-present flips), static stratification check (no layer-2 head predicate inside any notIncludes object), sibling-binding hazard leg (the EMIR R2.1 shape). P06.3 Art.15: time extraction from fixed dateTimes + window comparisons, time:localTime shape-only leg (wall clock — never a value assert), accuracy chain via math product/integerQuotient/notLessThan, skolem evidence determinism/anti-collapse/tamper. No quantifiers in any pack (DeriveN3 empty-data-universe). P06.5 wire: reason_n3_bounded_with_proof → N3Derivation{derived, proof, statistics}; N3_MAX_PROOF_BYTES 64KiB typed n3_proof_bytes refusal; op n3 optional proof:true (absent byte-identical; Fuse outranks; wasm32 typed unsupported); manifest/registry/js re-pin wave batched.",
    accept: "Depth-20 witnessed with N verified hops; at least 15 pack courts green with flips; wire court green incl. the first proof over the wire + over-cap refusal; manifest courts re-paired." },
  { id: "P05-seal", tier: 2, lease: "scripts/receipt_chain_sync.sh (--head/--verify-row + stored-head check), scripts/receipt_seal.sh (new), scripts/receipt_seal_court.sh (new), tests/receipt_seal_court.rs (new), registry/receipt-seal-pubkey.hex (new), registry/ARTIFACTS.blake3 (tool-written row), RECEIPTS-INDEX.md, CLAUDE.md lines 71-73 decision, docs/refusals.md one row",
    mission: "Receipt seal (WP P05). T1: receipt_chain_sync gains --head (recompute head from file text, compare stored) and --verify-row relpath; --check and sync ALSO compare the stored chain_head vs recomputed (G8 — editing only the head line must now fail); chain_flag_court extended to pin the new modes. T2: receipt_seal.sh S0-S5 (sync-check → sync → pin the receipt-chain-head row with the historical demotion grammar → doc_staleness_gate → graphlaw-attest sign of the exact 64-hex head, no newline → self-sealing receipt); registry/receipt-seal-pubkey.hex pinned (key_id = BLAKE3(pubkey) in comments; the seed is never committed). T3: tests/receipt_seal_court.rs (head recompute, attestation via TrustedKeys, sidecar verification, key_id match) + scripts/receipt_seal_court.sh with mktemp tamper twins (receipt byte flip → TAMPER-DETECTED; head-line flip → the G8 check catches; pin flip flagged). Sidecar convention: receipts/name.json.attest.json, mint one real sidecar over an existing spc receipt. D-P05-1 decision text replaces the CLAUDE.md pending decision (the sidecar is of-record; the OCEL frame chain stays in-memory; alternatives + consequences recorded). RECEIPTS-INDEX notation pass: machine-receipt digests prefixed blake3:/sha256:.",
    accept: "All courts green incl. tamper twins flipping; the seal receipt filed; the head attestation verifies offline." },
  { id: "P09-bench", tier: 3, lease: "benches/statutory_boundary_50ms.rs, benches/reports/sub50ms_statutory_courts.md",
    mission: "Bench truth (WP P09): (1) FIX THE OVERCLAIM — the Art.15 options become the law.rs cap set (max_iterations N3_MAX_ITERATIONS=4000, term_bytes 64KiB, total_bytes 256MiB, closure_facts 500k, match/total steps) with proof:true kept; the comment rewritten per-field, disclosing that shipped DeriveN3 runs proof:false and proof is a bench-side addition that force-enables include_explicit. (2) Adversarial-load probe: env SB_INJECT_OUTLIER_NS busy-spins INSIDE the Art.14 timed window (the tc_hot idiom — a spin outside the window dodges the statistic); 3-phase exits: solo baseline must PASS (else NON-PROBATIVE exit 1), injected must honest-FAIL (else VACUOUS GATE exit 1), recovery re-run must PASS (else exit 1); all three witnessed → exit 0 with a FLIP+RECOVERY WITNESSED line (the injected run still prints honest Art.14 FAIL lines). (3) VARIANCE_SELF_CHECK: 3 batches, spread under 5% on solo p50s, folded into the gate composite. (4) Disclosure block: machine + isolation + load quiet-or-injected + fixture caps. (5) The report regenerated from the acceptance run — real numbers only.",
    accept: "Solo run: ALL GATES PASS + variance PASS at law.rs-capped options; injected run: exit 0 with flip+recovery witnessed; grep exactly-as over benches/ → zero hits." },
  { id: "P11-memo", tier: 3, lease: "docs/displacement-memo.md (new), tests/docs_drift_courts.rs (2 new court fns), docs/index.md (one row)",
    mission: "Displacement memo (WP P11): docs/displacement-memo.md — Pillar 2 sovereign sandbox FIRST (60,895,924 B pin 0858a760, Import[28] all wasi_snapshot_preview1 with no net imports, zero preopen, fail-closed allowlist — every row claim → number → witness file:line + command); Pillar 1 receipts (the fold of record, e2e vs the 40ms budget); Pillar 3 scoped evidence (the court-pinned subset honesty + 72/72 parity + the RED→GREEN engine fixes + quantifier court with mutation witness + proof court); Pillar 4 determinism labeled WITNESSED-PARTIAL pending P04. The structural claims section wrapped in literal structural-begin/end comment markers, prose-only zero numerals, with the Stardog-explain-emits-proof-trees caveat carried. A generation table (bench-report generation vs the solo run of record). Carry-over table (audit C1/C2 status per P10, bypass lanes per P01, corpus row status). RULES section + 2 courts in docs_drift_courts.rs: G1 no unlabeled incumbent comparison (lines matching incumbent tokens outside the markers must carry the vendor-documented label; negative control: an embedded fake 100x-slower line must flag); G3 every number line carries a backticked witness citation (negative control embedded); pin-currency leg (the memo wasm bytes equal the ARTIFACTS row — the next re-pin fails it until refresh, intended). docs/index.md registration row.",
    accept: "Memo written with markers; G1/G3 courts green with negative controls; index row present." },
]
const B_impl = []
const B_skepticSets = []
const B_thunks = B.map(function (wp) {
  return function () {
    const implP = agent(BASE(wp.id, wp.lease, wp.mission, wp.accept),
      { label: wp.id, phase: "Array-B", schema: RUN_SCHEMA })
    const skvP = implP.then(function (rep) {
      if (!rep) return null
      return agent("Adversarial skeptic (default-refute) for moonshot lane " + wp.id + " in " + REPO + ". Lane receipt: " + JSON.stringify(rep).slice(0, 4000) + " — re-derive every claim (CARGO_TARGET_DIR=" + REPO + "/target-skv-" + wp.id + "); one revert-test per major mechanism (byte-exact restore); refute what fails. No fixes.",
        { label: "skv:" + wp.id, phase: "Array-B", schema: RUN_SCHEMA })
    })
    B_skepticSets.push(skvP)
    return implP
  }
})
const B_impl_all = (await parallel(B_thunks)).filter(Boolean)
const B_skv = (await parallel(B_skepticSets.map(function (p) { return function () { return p } }))).filter(Boolean)
for (const l of B_impl_all) B_impl.push({ lane: l.lane, status: l.status })
log("Array-B: " + B_impl.length + "/" + B.length + " implemented, " + B_skv.length + " skeptic sets")

// ---- Integrate ----
phase("Integrate")
const integ = (await parallel([
  () => agent(BASE("INT-vendor", "vendored crates/n3 read; /Users/sac/purrdf/crates/n3 write; vendor/README.md",
    "Sync in-tree vendor edits upstream: diff vendor/purrdf-n3/src vs /Users/sac/purrdf/crates/n3/src; apply the same changes upstream; in /Users/sac/purrdf run cargo test -p purrdf-n3 (full suite incl. the new courts, green); fmt check per that repo; report the commit-ready state (do NOT commit). Refresh vendor/README.md carried-changes if files drifted during the waves.",
    "Diff zero (vendor identical to upstream); upstream suite green with counts."),
    { label: "INT-vendor", phase: "Integrate", schema: RUN_SCHEMA }),
  () => agent(BASE("INT-suite", "read-only + target dir",
    "Definitive suite: " + ENV + " GRAPHLAW_WASM=" + REPO + "/target/wasm32-wasip1/wasm/graphlaw_wasm.wasm CARGO_TARGET_DIR=target cargo test --no-fail-fast. Report binaries/passed/failed (target 0 — failures classified with their panic lines, not fixed). Grep gates: live eyeron count outside vendor/history/negative-assertions.",
    "Exits + counts + failure classification if any."),
    { label: "INT-suite", phase: "Integrate", schema: RUN_SCHEMA }),
  () => agent(BASE("INT-wasm", "target/, registry/ARTIFACTS.blake3 (tool row), priv/",
    "Wasm re-mint: " + ENV + " CARGO_TARGET_DIR=target cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm; record bytes + b3sum; boundary courts (wasm_abi, chicago_cross_authority, wasm_n3_admission, wasm_size_court) with the same env; re-pin the ARTIFACTS wasm row using the historical demotion grammar; non-vacuity: GRAPHLAW_REQUIRE_WASM_PIN=1 GRAPHLAW_WASM=<fresh path> cargo test --test registry_artifacts; refresh priv/graphlaw.wasm + the .blake3 sidecar to the fresh bytes.",
    "Module bytes + digest; courts green at fresh bytes; pin verified under the REQUIRE flag."),
    { label: "INT-wasm", phase: "Integrate", schema: RUN_SCHEMA }),
  () => agent(BASE("INT-mutation", ".cargo/mutants configs + scripts/mutation_court.sh GUARDED/GATED + var/mutation-court; RUNS SOLO against the tree",
    "Mutation campaign (WP P12): widen .cargo/mutants.toml examine_globs with src/law.rs, src/dialect.rs, src/tier.rs; sync GUARDED in scripts/mutation_court.sh (the SIGKILL-leak set) + GATED + the mutation_families.sh byte-copy; resume the 91 unrun via --case over emit_json/emit_ttl with --log tee; run the widened set with the N3 killer courts; a vendor-rooted campaign in vendor/purrdf-n3 (own mutants.toml, its 3 test files as killers). Ledger per wave; survivors individually adjudicated (real gap vs noise). Time-box + receipt the ledger.",
    "Ledger with exact counts; 0 missed in GATED scope; survivors adjudicated."),
    { label: "INT-mutation", phase: "Integrate", schema: RUN_SCHEMA }),
  () => agent(BASE("INT-bench", "benches/reports/",
    "Final benches: CARGO_TARGET_DIR=target cargo bench -p graphlaw --bench statutory_boundary_50ms (ALL GATES verdict + table) and cargo bench --bench sieve_cliff (the new table). Update benches/reports/sub50ms_statutory_courts.md measured tables from THIS run (solo serialized; if a gate fails under load, rerun solo before the verdict).",
    "Bench verdicts + refreshed report."),
    { label: "INT-bench", phase: "Integrate", schema: RUN_SCHEMA }),
  () => agent(BASE("INT-receipt", "docs/jira/v26.10.26/*, RECEIPTS-INDEX.md, CHANGELOG",
    "Close-out: fold every wave receipt (the data below) into docs/jira/v26.10.26/RECEIPTS.md (each DoD row CLOSED with command+exit or OPEN(typed reason)); update DEFINITION-OF-DONE.md checkboxes; b3sum the receipt; RECEIPTS-INDEX.md top line; the CHANGELOG final 26.10.26 section.",
    "Every DoD row statused + evidence pointer; index digest matches."),
    { label: "INT-receipt", phase: "Integrate", schema: RUN_SCHEMA }),
])).filter(Boolean)
log("Integrate: " + integ.length + "/6 lanes returned")

phase("Synthesize")
const verdict = await agent([
  "Final synthesis for the moonshot array in " + REPO + ". Write docs/jira/v26.10.26/MOONSHOT-VERDICT.md (your only permitted write): the four PRD pillars, each claim → evidence (court/bench file:line + command) → status (WITNESSED / WITNESSED-PARTIAL / ROADMAP); the wave ledger (lanes dispatched/completed/skeptic-refuted/integration verdicts); the typed residual list; the DoD verdict sentence.",
  "DATA: " + JSON.stringify({
    seal: seal.map(function (x) { return x.lane + ":" + x.status }),
    A: A_impl.map(function (x) { return { lane: x.lane, status: x.status } }),
    A_skeptic_sets: A_skv.length,
    p13: p13res, p10: p10res,
    B: B_impl.map(function (x) { return { lane: x.lane, status: x.status } }),
    B_skeptic_sets: B_skv.length,
    integ: integ.map(function (x) { return { lane: x.lane, status: x.status, notes: (x.notes || "").slice(0, 300) } }),
  }).slice(0, 60000),
  "Return the verdict sentence + per-pillar one-liners.",
].join("\n"), { label: "synthesis", phase: "Synthesize" })

return {
  seal: seal.map(x => ({ lane: x.lane, status: x.status })),
  A_impl: A_impl.map(x => ({ lane: x.lane, status: x.status })),
  A_skeptic_sets: A_skv.length,
  reasoner_tail: { p13: p13res, p10: p10res },
  B_impl: B_impl,
  B_skeptic_sets: B_skv.length,
  integration: integ.map(x => ({ lane: x.lane, status: x.status, notes: (x.notes || "").slice(0, 300) })),
  verdict: String(verdict).slice(0, 20000),
}
