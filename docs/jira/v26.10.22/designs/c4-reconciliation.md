# C4 reconciliation: c4-architecture.md vs. repo state

Lane L180, 2026-10-22. Sole output this file.
Subject: `docs/jira/v26.10.22/designs/c4-architecture.md` (operator text,
mtime 21:53 Oct 22) mapped against the tree at `615dbfa` plus in-flight
untracked work. Every row grounded in a command run this session.

## L1 - System context

| C4 entity | State | Evidence |
|---|---|---|
| developer | REAL | Operator CLI flows; `OPERATOR-DECISIONS.md` (19:23), `claude-md-ladder-receipt.md` (16:52), both under `docs/jira/v26.10.22/designs/` |
| platform | ASSEMBLED | Claude Code runtime (transcripts `~/.claude/projects/-Users-sac-graphlaw/`); hooks `.claude/hooks/` (topology_guard, disk_guard, manifest_guard, ban_generic_agents 11:04-16:27 mtimes; graphlaw_receipt_attest.py + lithography pair 21:51-21:53); `graphlaw-run` binary witnessed `graphlaw-run-live-receipt.md` (18:20) |
| courts | REAL | `tests/` acceptance set (chicago_validation, sql_rdf_dialect, duckdb_statement_kinds, trust_attest); `scripts/mutation_court.sh`, `scripts/spc_court.sh` (receipts: mutation-court-full-receipt.md 19:19, spc-red-remediation-design.md 20:27) |
| ledger | REAL | `receipts/CHAIN.blake3`: 31 receipt lines (wc -l, sha256 head ca5ebdea3c249be7) - grew from the 27 cited at dispatch; chain-resync-receipt.md 21:43 |

## L2 - Containers

| Container | State | Evidence |
|---|---|---|
| agent_runtime | REAL | This session runs in it; `ban_generic_agents.py` enforces agent scoping (settings.json PreToolUse `Agent`) |
| lifecycle_hook | PARTIAL | 4 witnessed python guards + `graphlaw_receipt_attest.py` (21:51, unwitnessed provenance, see lithography section) wired in `.claude/settings.json`; NOT the C4 shape: no PostToolUse reification of tool calls into RDF quads, no wasm invocation from any hook. P1's `graphlaw_admission_guard` prototype is NOT present in `.claude/hooks/` (grep for admission_guard across .claude/ and designs/ = zero hits; the prototype exists only as prose references in SQLRDF-19/20 and session-lithography.md) |
| wasm_kernel | DESIGNED, NOT HOSTED | Artifact builds (`m4-wasm-boundary-receipt.md` 19:20) and carries EH Tag sections, but pinned wasmi 2.0.0 has no EXCEPTIONS feature bit and panics on tag imports (`wasmi-eh-research.md` 19:17, citing wasmi src/module/import.rs:86); `eh-runner-decision-memo.md` (18:26) holds the wasmtime-now vs wasmi-upgrade vs dual-host decision open. No available runtime executes the EH artifact |
| projection_engine | EXISTS | ggen, witnessed via the ffi.rs sync (`ggen-pack-reconciliation.md` 16:58, `ggen-pack-drift.md` 16:33) |

## L3 - Components (GraphLaw kernel)

| C4 component | State | Repo counterpart |
|---|---|---|
| quad_reifier | IMPLEMENTED (host tier) | `src/hooks.rs` - `load`/`hooks`/`materialize` (src/hooks.rs:206,291,296) materialize law hooks; 10 SPARQL refs. Reification of a live PreToolUse payload into act: quads is NOT wired (that is the lifecycle_hook gap above) |
| shape_engine | IMPLEMENTED | `src/shacl_guard.rs` + `src/sql_ast/` (ast.rs, refusals.rs, serializer.rs) against `ontologies/sql-ast.ttl`; SQL-in-RDF admission via src/law.rs |
| datalog_evaluator | PARTIAL | `src/rules/builtins.rs` + `src/rules/mod.rs` (builtins only); lease vocabulary in `src/law.rs:113,165`; `src/abi/lease.rs` exists but is deliberately OFF (session-lithography.md: "It does not turn on `src/abi/lease.rs`"). Temporal calculus / monotonic counters MISSING |
| sparql_refusal | IMPLEMENTED (host tier) | Typed SPARQL refusal hooks: `src/law.rs` `Refusal`, `src/sql_ast/refusals.rs`, `src/hooks.rs`; refusal register `docs/refusals.md` REFUSAL-01..04 |
| receipt_sealer | IMPLEMENTED | `src/attest.rs` (Ed25519 SigningKey/VerifyingKey, attest.rs:101,172), `src/receipt_chain.rs`, `receipts/CHAIN.blake3` |

## Honest gap summary

1. The C4 "Lifecycle Hook Adapter" does not exist as specified. What exists
   is fail-open python guards, not a PostToolUse quad reifier feeding the
   wasm kernel; no hook calls graphlaw-run or the wasm hooks op today.
2. The wasm_kernel is built but unhostable: EH Tag sections have no runtime
   (wasmi 2.0.0 lacks the feature bit entirely).
3. Datalog temporal calculus is vocabulary, not evaluator: `src/rules/` has
   builtins only; `src/abi/lease.rs` is switched off.
4. The P1 admission-guard prototype cited at dispatch does not exist on
   disk - zero files match `admission_guard` anywhere in `.claude/` or
   `docs/jira/v26.10.22/` (grep exit 1). Its state is PROSE ONLY.
5. CHAIN.blake3 is at 31 receipts, not the 27 cited at dispatch - the
   ledger is live and advancing during the wave.

## Lithography verdict (Part 2)

See companion section above; summary here.

- WHAT IT ENFORCES: `.claude/hooks/lithography_card.py` (SessionStart)
  prints a static policy card (exit 0, verified by run). `.claude/hooks/
  lithography_guard.py` (PreToolUse Edit/Write/MultiEdit/NotebookEdit,
  registered in `.claude/settings.json`) exits 2 + JSON block on five
  LITHO-REFUSED classes: unreadable stdin (fail-closed), generated
  (docs/reference-generated/**), registry-artifact (registry/*), pin-file
  (courts/code-partition pins), staging (any *_split/ path). Bash is the
  coordinator hatch. Verified: 11-case payload matrix run this session,
  all exits match `session-lithography.md`'s check table exactly.
- PROVENANCE: mtimes 21:51-21:53 Oct 22. The only transcript in
  `~/.claude/projects/-Users-sac-graphlaw/` mentioning "lithography" is
  223f3838 (spans 18:11 Oct 22 - 04:55 Oct 23, 11182 lines); all 10 of its
  lithography mentions are 04:52+ Oct 23 (the current coordination session
  dispatching this lane). NO logged Write of any lithography file exists at
  the 21:51-21:53 window despite the session being live throughout it.
  UNLOGGED CHANNEL.
- COLLISION: none current. P1's admission guard prototype is absent from
  disk and from settings.json's Edit/Write matcher (manifest_guard +
  lithography_guard only). If P1 lands its prototype on the same matcher,
  both will run (Claude Code runs multiple hooks per matcher); they are
  composable since lithography_guard fails closed on unreadable stdin while
  the documented prototype design would add a receipted-admission path.
- VERDICT: the content is coherent operator-policy work (documented in-tree
  by `session-lithography.md`, mtime-matched 21:53, consistent with
  CLAUDE.md Lithography section at CLAUDE.md:121) and the code is
  functional and fail-closed. But its entry into the tree is UNLOGGED:
  no transcript Write, files untracked (git status ?? on
  .claude/hooks/lithography_*.py and .claude/settings.json), and
  .claude/settings.json itself is new-untracked, i.e. hook registration
  itself arrived through the same channel. Escalate to the operator for
  authorship confirmation before treating the guards as admitted
  infrastructure. A deny is not a receipt; an unwitnessed hook registration
  is not an admitted hook.

## Later state (2026-10-22, loop close)

The L180 rows above stay as that lane wrote them. The tree has since moved.

- `.claude/hooks/graphlaw_admission_guard.py` is on the Edit/Write
  PreToolUse matcher, after `manifest_guard.py` and `lithography_guard.py`.
  It builds one `act:ToolCall` and asks `graphlaw-run` for the `shacl` op
  against `.claude/policy/graphlaw-seed-shapes.ttl`. A missing runner,
  module, or shapes file still fails open.
- `.claude/hooks/graphlaw_receipt_attest.py` is on PostToolUse for the same
  matcher. A mutating edit with a readable file writes
  `receipts/transition-*.json` and folds the chain through
  `scripts/receipt_chain_sync.sh`. A `Read` writes nothing.
- Gap 4 in the honest-gap list ("admission guard is prose only") is no
  longer the tree. `src/abi/lease.rs` is still off.
  `.claude/policy/fs-boundary-shapes.ttl` is still a specification.
