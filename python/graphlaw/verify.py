"""Wrapper over the `graphlaw-verify` receipt CLI."""

from __future__ import annotations

import os
import subprocess
from dataclasses import dataclass
from typing import Optional, Sequence


@dataclass(frozen=True)
class VerifyResult:
    admitted: bool
    exit_code: int
    stdout: str
    stderr: str


def verify_receipts(
    start: str,
    plan: str,
    receipts: str,
    trusted_keys: Sequence[str],
    binary: Optional[str] = None,
    timeout: float = 120,
) -> VerifyResult:
    """Replay a plan against a signed receipt store. Exit 0 = admitted; exit 1 = typed refusal."""
    if not trusted_keys:
        raise ValueError("at least one trusted key (hex Ed25519 public key) is required")
    if binary is None:
        repo = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
        binary = os.environ.get("GRAPHLAW_VERIFY", os.path.join(repo, "target", "debug", "graphlaw-verify"))
    cmd = [binary, "--start", start, "--plan", plan, "--receipts", receipts]
    for k in trusted_keys:
        cmd += ["--trusted-key", k]
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    return VerifyResult(r.returncode == 0, r.returncode, r.stdout, r.stderr)
