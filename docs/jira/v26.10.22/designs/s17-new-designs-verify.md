# S17 — provenance verification of two unassigned design records (2026-10-22)

Lane S17 (skeptic, annealing), read-only. Repo: /Users/sac/graphlaw.

## Subjects

- designs/athena-agent-airlock.md (7,048 B region; mtime 2026-10-22 21:48:30)
- designs/deterministic-agent-admission.md (8,106 B; mtime 2026-10-22 21:42:35)
- designs/INDEX.md (mtime 2026-10-22 21:48:35 — 5 s after athena file)

## 1. Content

**athena-agent-airlock.md** — an analysis/mapping note: Athena's
GraphWorkspace/Airlock (~/cai/athena wasm-bridge + airlock-gateway) mapped
onto this repo's Claude Code surface (`.claude/settings.json` PreToolUse
hooks, agent files, `src/hooks.rs` kh:effect refuse semantics). Explicit
non-goals: nothing wired, `src/abi/lease.rs` stays a doc-fence draft,
`LeaseContext` not compiled. Coherent with — and explicitly cross-linked
to — deterministic-agent-admission.md and the P1 admission-guard /
core-team-decomposition scope.

**deterministic-agent-admission.md** — a filed version of the operator
brief (admitted code deltas, semantic work graphs, three partitions,
five rule families), written in the skeptic register: every claim is
paired with "what exists today" and explicit not-implemented markers.
Fact-checks against disk (all confirmed):

- `src/abi/lease.rs` is a doc-fence draft, `LeaseContext` not compiled — confirmed (file head).
- `unify.rs` does not exist (`ls` exit 1); it is remainder item 1 in core-team-decomposition.md:106 — confirmed.
- N21 census: 44 unadmitted leaks, ratio 1.1425 (19999/17505) — confirmed in code-partition-census.md:20,52,111.
- `registry_hash_invariance` absent from tests/ (grep count 0) — confirmed.
- `src/query_ledger/{batch,storage}.rs` exist — confirmed.
- wasm pin 332ea218 appears in INDEX.md (grep count 2) — confirmed.

No factual error found in either file. Register is correct (design,
not enacted capability).

## 2. Provenance (transcript greps)

Session 223f3838-56c4-4326-922d-bc5cf98aedc8:

- Main transcript: `"file_path":"…/athena-agent-airlock.md"` → 0 hits;
  `…/deterministic-agent-admission.md` → 0 hits; `…/designs/INDEX.md` → 5 hits.
  Filename string appears 5x each, all in INDEX listing / coordinator
  confusion contexts ("new files I haven't seen (from... P1's blueprint? or
  another lane? P1 was dis…"). No Write/Edit tool_use, no content-bearing
  heredoc.
- Subagent transcripts: no Write/Edit file_path to either design file in
  any agent-*.jsonl. The only file_path hits are my own lane's Read calls
  (agent-a90d0ee9efe747c18 = S17). Distinctive content string
  ("GraphWorkspace is constructed with a scope string") appears ONLY in my
  lane transcript — no logged actor ever produced the athena file's bytes.
- Other sessions in -Users-sac-graphlaw: 0 mentions in all 5.

designs/INDEX.md itself IS logged: written by milestone-doc lanes
(a0567f871eebfc883, a12d44ed36a062de6, a21fe64fbdff0e7ff, a4830fe565376a11f,
a9b908594b7be6dc4) and the main session (5 file_path hits).

## 3. INDEX coverage

Both files ARE indexed:
- designs/INDEX.md:24-25 (both, with summaries).
- docs/jira/v26.10.22/INDEX.md:344 (deterministic) and :378 (athena).
Not duplicates of existing designs.

## 4. Verdicts

| File | Verdict |
|---|---|
| designs/deterministic-agent-admission.md | UNLOGGED CHANNEL (escalate): content accurate and in-scope, but no logged Write/Edit in any transcript of this session or others. |
| designs/athena-agent-airlock.md | UNLOGGED CHANNEL (escalate): same. mtime lands 5 s before INDEX.md, i.e. inside the logged INDEX-writing window, yet no logged actor wrote it. |
| designs/INDEX.md | legitimate-lane-output: logged milestone-doc writes. |

This reopens S2's unlogged-writer question (cf. s8-unlogged-writer-watch.md):
an actor outside the logged transcripts wrote content into the canonical
checkout during the active window. Content is safe (design prose, no code,
no hook changes) but provenance is unattributed. Coordinator should
attribute or revert before integration commit.

## Receipt

- mtime stat: 21:42:35 / 21:48:30 / 21:48:35 (2026-10-22).
- Transcript greps: file_path Write hits 0/0/5 as above; content-string
  grep over all subagents + main: only S17's own lane.
- Disk fact-checks: ls/grep exits 0/1 as recorded inline.
