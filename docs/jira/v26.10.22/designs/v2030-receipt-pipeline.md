# Vision-2030 Step 3 — Receipt Pipeline Deployment (V2030-3)

Status: ALIVE (pipeline witnessed end-to-end, both directions, plus Ed25519 signing)

## Pipeline

```
Claude Code PostToolUse event (JSON on stdin)
        │
        ▼
.claude/hooks/graphlaw_receipt_attest.py
   ├─ non-vacuity gate: only mutating tools (Edit/Write/MultiEdit/NotebookEdit)
   │    with a readable file_path produce a receipt; anything else → exit 0,
   │    no receipt, chain untouched (witnessed: Read/Bash payloads change nothing)
   ├─ artifact digest: b3sum(tool_input.file_path)
   ├─ receipt file: receipts/transition-<ISO-Z>-<tool>-<b3_8>.json
   │    {type, schema=v2030-step3, tool, target, artifact_blake3,
   │     timestamp_utc, chain{script,mode}, signature{status,attestation}}
   ├─ signing (status decided BEFORE the write; sidecar attestation covers the
   │    final receipt bytes): graphlaw-attest sign --key-seed-hex $GRAPHLAW_ATTEST_KEY_SEED_HEX
   │    --in <receipt> --out <receipt>.attest.json   (src/attest.rs Ed25519 path)
   └─ chain fold: bash scripts/receipt_chain_sync.sh sync
        (the SAME fold the run-level courts use; its tamper check is the
         append-only invariant — it REFUSES on any drift, exit 1; the hook
         never writes CHAIN.blake3 itself)
        │
        ▼
receipts/CHAIN.blake3 chain_head advances
```

## Witnessed runs (2026-10-23 UTC, this lane)

- Mutating payload (Write, `/tmp/v2030_scratch.txt`) → receipt
  `receipts/transition-2026-10-23T045157Z-Write-b04f0b83.json` written; chain head
  `07376aba…ce064f` → `a91ae83a…a4dbd0`. CHAIN_ADVANCED=yes.
- No-op payloads (Read tool; malformed JSON; Bash tool) → "no receipt" logged,
  chain head byte-identical. NOOP-NO-RECEIPT=yes.
- Signed run: hook with `GRAPHLAW_ATTEST_KEY_SEED_HEX=<throwaway seed>` → receipt
  `transition-2026-10-23T045619Z-Write-95073190.json`, sidecar `.attest.json`;
  `graphlaw-attest verify --in <receipt> --attestation <sidecar> --trusted-key <pub>`
  → `VERIFIED`. Chain fold `--check` → `VERIFIED: 35 receipts match`.
- Tamper invariant indirectly witnessed: `receipt_chain_sync.sh sync` itself is the
  gate that refuses on drift (pre-existing court behavior); the hook delegates to it
  rather than reimplementing the fold.

## Registration (applied)

The block below is in `.claude/settings.json` under `PostToolUse`. The hook
imports on system Python 3.9 (`str | None` annotations removed). A `b3sum`
failure honors `GRAPHLAW_RECEIPT_STRICT`.

```json
"PostToolUse": [
  {
    "matcher": "Edit|Write|MultiEdit|NotebookEdit",
    "hooks": [
      {
        "type": "command",
        "command": "python3 \"$CLAUDE_PROJECT_DIR/.claude/hooks/graphlaw_receipt_attest.py\""
      }
    ]
  }
]
```

A standalone run is still `…payload… | python3 .claude/hooks/graphlaw_receipt_attest.py`.

## Signing status: SIGNED (no gap)

Signing is wired and witnessed, not staged: the repo's Ed25519 path
(`graphlaw-attest sign`, `src/bin/graphlaw_attest.rs` over `src/attest.rs`) signs the
final receipt bytes whenever `GRAPHLAW_ATTEST_KEY_SEED_HEX` (64 hex chars) is present
in the hook environment. Without the key the receipt records
`signature.status = UNSIGNED-NO-KEY` plus the exact signing command — a loud,
per-receipt-visible state, never a silent skip. Operator decision remaining: which
seed (and key ceremony) backs production receipts; the throwaway seed used here is
for non-vacuity proof only.

## P1 pairing (lane V2030-3 read-only on P1's files)

P1 (.claude/policy/ + graphlaw_admission_guard.py) may not exist yet. Pairing point
is mechanical: P1's PreToolUse admission guard and this PostToolUse attestation hook
share the payload vocabulary (tool_name, tool_input.file_path); a refused transition
(P1) should never yield a transition receipt (this hook), because PostToolUse only
fires on completed tool calls. No code coupling is required; registration order is
independent.

## Honest gaps / open operator items

1. **Volume + compaction (the flagged implication).** The chain currently holds
   run-level court receipts (28 at lane start). Per-transition receipts grow it by
   one file per mutating tool call — an active session adds hundreds per day. The
   chain is append-only, so CHAIN.blake3 grows linearly and sync cost grows with it
   (full re-enumeration + full re-fold per receipt). **Compaction is an open
   operator item**: e.g. periodic per-directory rollup receipts (fold N transition
   receipts into one dated rollup, archive the leaves outside receipts/) — but the
   current script has no archival lane; the leaves are immutable once chained, so
   compaction needs a designed, receipt-gated transition, not a hook-side patch.
2. **Registration is applied** in `.claude/settings.json`. Compaction is still
   open; every mutating edit adds one receipt and re-folds the chain.
3. **Signing key ceremony**: hook-level env seed works and is witnessed; production
   key custody (who holds the seed, rotation, trusted-key pinning for `verify`) is
   an operator decision.
4. **First registered-session receipt**: the standalone runs here are test payloads
   against /tmp scratch files; they are now immutable chain members (removal would
   TAMPER). The first hook-fired receipt from a real session is still future
   evidence.
