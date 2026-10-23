#!/usr/bin/env python3
"""graphlaw_receipt_attest.py — PostToolUse receipt-pipeline hook (Vision-2030 step 3).

Reads a Claude Code PostToolUse JSON event from stdin, computes a per-transition
receipt (tool + target path + BLAKE3 of the produced artifact + timestamp), writes
it into receipts/ in the CHAIN.blake3 ecosystem, and advances the chain head by
calling scripts/receipt_chain_sync.sh — the SAME fold logic the run-level courts
use. The hook NEVER rewrites CHAIN.blake3 itself; the sync script's tamper check
is the append-only invariant (it refuses on any drift, exit 1).

Receipt file naming (flat, dated, matching the existing receipt family):
    receipts/transition-<UTC ISO, colon-free>-<tool>-<b3_8>.json

Receipt record (JSON):
    {
      "type": "transition-receipt",
      "schema": "v2030-step3",
      "tool": "<tool_name>",
      "target": "<tool_input.file_path>",
      "artifact_blake3": "<b3sum of artifact bytes>",
      "artifact_path": "<absolute path>",
      "timestamp_utc": "<ISO8601>",
      "chain": {"script": "scripts/receipt_chain_sync.sh", "mode": "append-only-fold"},
      "signature": {"status": "SIGNED"|"UNSIGNED-NO-KEY", "attestation": path|null}
    }

Signing: if GRAPHLAW_ATTEST_KEY_SEED_HEX (64 hex chars) is exported in the hook
environment, the receipt file is signed with the repo's Ed25519 path
(`graphlaw-attest sign --key-seed-hex ... --in <receipt>`); the attestation JSON
lands beside the receipt. Without a key the receipt records
`"status": "UNSIGNED-NO-KEY"` — a documented gap, never a silent skip.

Non-vacuity contract: a mutating payload (Edit/Write/MultiEdit/NotebookEdit with
a real file_path whose bytes are readable) produces a receipt and advances the
chain head; a non-mutating payload (Read/Grep/Bash, or missing/unreadable
file_path) produces NO receipt and leaves the chain untouched.

Exit codes: 0 always for Claude Code (PostToolUse must not block), unless
GRAPHLAW_RECEIPT_STRICT=1, in which case receipt-pipeline failure exits 1.
"""

from __future__ import annotations

import datetime
import json
import os
import re
import subprocess
import sys

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RECEIPTS_DIR = os.path.join(REPO_ROOT, "receipts")
CHAIN = os.path.join(RECEIPTS_DIR, "CHAIN.blake3")
SYNC_SCRIPT = os.path.join(REPO_ROOT, "scripts", "receipt_chain_sync.sh")

MUTATING_TOOLS = {"Edit", "Write", "MultiEdit", "NotebookEdit"}


def log(msg: str) -> None:
    print(f"[receipt-attest] {msg}", file=sys.stderr)


def chain_head():
    if not os.path.exists(CHAIN):
        return None
    with open(CHAIN, "r", encoding="utf-8") as fh:
        for line in fh:
            if line.startswith("chain_head "):
                return line.split()[1]
    return None


def b3sum(path):
    try:
        out = subprocess.run(
            ["b3sum", path], capture_output=True, text=True, timeout=30
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if out.returncode != 0:
        return None
    return out.stdout.split()[0] if out.stdout.split() else None


def signing_status() -> str:
    """Decided BEFORE the receipt file is written: the attestation must cover
    the final receipt bytes, so the signature block records intent, and the
    signature itself lives in the sidecar attestation JSON over those bytes."""
    seed = os.environ.get("GRAPHLAW_ATTEST_KEY_SEED_HEX", "").strip()
    if not seed:
        return "UNSIGNED-NO-KEY"
    if not re.fullmatch(r"[0-9a-fA-F]{64}", seed):
        return "UNSIGNED-BAD-KEY-FORMAT"
    return "SIGNED"


def sign_receipt(receipt_path: str, out_path: str) -> dict:
    seed = os.environ.get("GRAPHLAW_ATTEST_KEY_SEED_HEX", "").strip()
    bin_ = os.path.join(REPO_ROOT, "target-lane-v20303", "debug", "graphlaw-attest")
    if not os.path.exists(bin_):
        bin_ = "graphlaw-attest"  # fallback to PATH
    out_path = receipt_path + ".attest.json"
    try:
        proc = subprocess.run(
            [bin_, "sign", "--key-seed-hex", seed, "--in", receipt_path, "--out", out_path],
            capture_output=True, text=True, timeout=60,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return {"status": "SIGN-ERROR", "attestation": None, "error": str(exc)}
    if proc.returncode != 0:
        return {"status": "SIGN-REFUSED", "attestation": None, "stderr": proc.stderr[-400:]}
    return {"status": "SIGNED", "attestation": out_path}


def main() -> int:
    raw = sys.stdin.read()
    try:
        event = json.loads(raw)
    except json.JSONDecodeError as exc:
        log(f"no receipt: payload is not JSON ({exc})")
        return 1 if os.environ.get("GRAPHLAW_RECEIPT_STRICT") else 0

    tool = event.get("tool_name", "")
    tool_input = event.get("tool_input") or {}
    target = tool_input.get("file_path") or ""

    # Non-vacuity gate: only mutating tools with a readable artifact produce a receipt.
    if tool not in MUTATING_TOOLS or not target:
        log(f"no receipt: tool={tool or '<none>'} is non-mutating or has no target")
        return 0

    if not os.path.isfile(target):
        log(f"no receipt: target {target} not readable as a file")
        return 0

    artifact_b3 = b3sum(target)
    if artifact_b3 is None:
        log(f"no receipt: b3sum failed for {target}")
        return 0 if not os.environ.get("GRAPHLAW_RECEIPT_STRICT") else 1

    ts = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H%M%SZ")
    before_head = chain_head()
    receipt_name = f"transition-{ts}-{tool}-{artifact_b3[:8]}.json"
    receipt_path = os.path.join(RECEIPTS_DIR, receipt_name)

    record = {
        "type": "transition-receipt",
        "schema": "v2030-step3",
        "tool": tool,
        "target": target,
        "artifact_blake3": artifact_b3,
        "timestamp_utc": ts,
        "chain": {"script": "scripts/receipt_chain_sync.sh", "mode": "append-only-fold"},
        "signature": {"status": signing_status(),
                      "attestation": receipt_path + ".attest.json"},
    }

    # Write the FINAL receipt bytes first; the Ed25519 attestation then covers
    # exactly those bytes (verify re-reads the file, so it must not change after
    # signing). Signing status is decided before the write.
    body = dict(record)
    os.makedirs(RECEIPTS_DIR, exist_ok=True)
    with open(receipt_path, "w", encoding="utf-8") as fh:
        json.dump(body, fh, indent=2, sort_keys=True)
        fh.write("\n")

    sig = {"status": signing_status(), "attestation": body["signature"]["attestation"]}
    if sig["status"] == "SIGNED":
        result = sign_receipt(receipt_path, sig["attestation"])
        sig = result
        if result["status"] != "SIGNED":
            # No attestation exists over the current bytes; rewriting the status
            # block is safe (nothing verifies a signature that was never made).
            body["signature"] = result
            with open(receipt_path, "w", encoding="utf-8") as fh:
                json.dump(body, fh, indent=2, sort_keys=True)
                fh.write("\n")

    # Advance the chain head via the SAME fold the courts use (append-only).
    sync = subprocess.run(
        ["bash", SYNC_SCRIPT, "sync"], capture_output=True, text=True, timeout=120
    )
    if sync.returncode != 0:
        log(f"CHAIN SYNC REFUSED (append-only invariant held): {sync.stdout} {sync.stderr}")
        if os.environ.get("GRAPHLAW_RECEIPT_STRICT"):
            return 1
        log("receipt file written but chain not advanced (non-strict mode)")
        return 0

    after_head = chain_head()
    advanced = before_head != after_head
    log(
        f"receipt {receipt_name} written; signature={sig['status']}; "
        f"chain advanced={advanced}; head={after_head}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
