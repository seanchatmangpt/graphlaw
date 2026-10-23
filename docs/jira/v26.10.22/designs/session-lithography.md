# Session lithography (2026-10-22)

A restarted Claude Code session loads `CLAUDE.md` and `.claude/settings.json`.
Those two surfaces are the lithography. This note records what they refuse
and what a restart still does not do.

## Registered on restart

`SessionStart` matcher `startup|resume|clear|compact` runs
`.claude/hooks/lithography_card.py`. The same card is the Lithography
section of `CLAUDE.md`. Each project agent under `.claude/agents/` repeats
four lines: absolute repo path, edit only owned files, a typed refusal
stops the turn, `court-runner` stays read-only.

`PreToolUse`:

| Matcher | Scripts |
|---|---|
| `Agent` | `ban_generic_agents.py` — `GENERIC_AGENT_REFUSED`, exit 2 |
| `EnterWorktree` | `topology_guard.py` — `REFUSED(topology)` |
| `Bash` | `topology_guard.py`, then `disk_guard.py` |
| `Edit\|Write\|MultiEdit\|NotebookEdit` | `manifest_guard.py`, then `lithography_guard.py`, then `graphlaw_admission_guard.py` |

`topology_guard.py` was already tested against Bash cases in
`.claude/hooks/topology_guard_cases.json`. Settings previously matched only
`EnterWorktree`, so those Bash cases never ran. The Bash matcher now
includes it. Its deny protocol is unchanged.

## LITHO-REFUSED

`.claude/hooks/lithography_guard.py` exits 2 with
`{"decision":"block","reason":...}` before the file tool runs.

| Code | Path |
|---|---|
| `unreadable` | stdin is not a JSON object. This guard fails closed. The other four guards stay fail-open. |
| `generated` | `docs/reference-generated/**` |
| `registry-artifact` | `registry/ARTIFACTS.blake3`, `registry/capability-registry.json`, `registry/capability-registry.ttl` |
| `pin-file` | `courts/code-partition/axioms.blake3`, `axioms.blake3.next`, `manifest.json` |
| `staging` | any path containing `_split/` |

Other edits pass, including axiom `.rs` sources. The partition manifest
still names deleted `*_split` paths, so it is not a live allow-list.
Freezing kernel sources from it would block the work still in this tree.
Pin files and generated artifacts use the same hatch as `Cargo.toml`:
file tools refuse, Bash can still write.

`PostToolUse` on the same Edit/Write matcher runs
`.claude/hooks/graphlaw_receipt_attest.py`. A mutating edit with a readable
file writes `receipts/transition-*.json` and folds `receipts/CHAIN.blake3`
through `scripts/receipt_chain_sync.sh`. A `Read` does not.

## What a restart does not do

- `graphlaw_admission_guard.py` calls `graphlaw-run` for the `shacl` op on
  an `act:ToolCall`. It does not call the wasm `hooks` op. Infra failure
  (runner, module, or shapes missing) still fails open.
- A deny is not a receipt. The receipt hook runs after the tool, so a
  refused edit does not get a transition receipt.
- It does not treat the partition-court ratio (N21, 1.1425, G3 FAIL) as a pass.
- It does not turn on `src/abi/lease.rs`.
- `.claude/policy/fs-boundary-shapes.ttl` stays a specification. The seed
  shapes are the graph the admission hook sends.

## Check

`python3 .claude/hooks/lithography_guard.py` on stdin payloads
(2026-10-22, Python 3.9.6). No cargo suite and no SPC run.

| Payload | Exit |
|---|---|
| non-JSON stdin | 2 `LITHO-REFUSED(unreadable)` |
| empty stdin | 2 `LITHO-REFUSED(unreadable)` |
| `docs/reference-generated/abi.md` | 2 `LITHO-REFUSED(generated)` |
| `registry/ARTIFACTS.blake3` | 2 `LITHO-REFUSED(registry-artifact)` |
| `registry/capability-registry.json` | 2 `LITHO-REFUSED(registry-artifact)` |
| `registry/capability-registry.ttl` | 2 `LITHO-REFUSED(registry-artifact)` |
| `courts/code-partition/manifest.json` | 2 `LITHO-REFUSED(pin-file)` |
| `courts/code-partition/axioms.blake3` | 2 `LITHO-REFUSED(pin-file)` |
| `courts/code-partition/axioms.blake3.next` | 2 `LITHO-REFUSED(pin-file)` |
| `src/dialect_split/refusal.rs` | 2 `LITHO-REFUSED(staging)` |
| `/Users/sac/graphlaw/src/sql_ast.rs` | 0 |
| `contracts/abi-manifest.json` | 0 |
