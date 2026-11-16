"""Wrapper over the `graphlaw-attest` CLI (Ed25519 sign / verify / pubkey)."""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass
from typing import Optional, Sequence

from .registry import REPO


def _bin(binary: Optional[str]) -> str:
    return binary or os.environ.get("GRAPHLAW_ATTEST", os.path.join(REPO, "target", "debug", "graphlaw-attest"))


@dataclass(frozen=True)
class AttestResult:
    ok: bool
    exit_code: int
    stdout: str
    stderr: str


def pubkey(seed_hex: str, binary: Optional[str] = None) -> str:
    r = subprocess.run([_bin(binary), "pubkey", "--key-seed-hex", seed_hex], capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"graphlaw-attest pubkey exit {r.returncode}: {r.stderr.strip()}")
    return r.stdout.strip()


def sign(seed_hex: str, path: str, out: Optional[str] = None, binary: Optional[str] = None) -> dict:
    """Sign a file; returns the attestation {key_id, payload_blake3, signature}."""
    cmd = [_bin(binary), "sign", "--key-seed-hex", seed_hex, "--in", path]
    if out:
        cmd += ["--out", out]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"graphlaw-attest sign exit {r.returncode}: {r.stderr.strip()}")
    return json.loads(r.stdout)


def verify(path: str, attestation_path: str, trusted_keys: Sequence[str], binary: Optional[str] = None) -> AttestResult:
    """Exit 0 = VERIFIED, 1 = REFUSED (typed), 2 = usage/IO."""
    if not trusted_keys:
        raise ValueError("at least one trusted key is required")
    cmd = [_bin(binary), "verify", "--in", path, "--attestation", attestation_path]
    for k in trusted_keys:
        cmd += ["--trusted-key", k]
    r = subprocess.run(cmd, capture_output=True, text=True)
    return AttestResult(r.returncode == 0, r.returncode, r.stdout, r.stderr)


def _sign_payload(seed_hex: str, payload: str, binary: Optional[str] = None) -> dict:
    import tempfile

    with tempfile.NamedTemporaryFile("wb", suffix=".payload", delete=False) as f:
        f.write(payload.encode("utf-8"))
        path = f.name
    try:
        return sign(seed_hex, path, binary=binary)
    finally:
        os.unlink(path)


def lease_payload(lease: dict) -> str:
    """Canonical lease payload (sorted keys, no whitespace; scope order preserved)."""
    body = {"ceiling": lease["ceiling"], "expires_unix": lease["expires_unix"], "holder": lease["holder"],
            "id": lease["id"], "issued_unix": lease.get("issued_unix", 0), "scope": list(lease["scope"])}
    return json.dumps(body, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def sign_lease(seed_hex: str, lease: dict, binary: Optional[str] = None) -> dict:
    """Return a `signed_lease` for a `law` request: {"lease": ..., "attestation": ...}."""
    lease = {**lease, "issued_unix": lease.get("issued_unix", 0)}
    return {"lease": lease, "attestation": _sign_payload(seed_hex, lease_payload(lease), binary)}


_NS = "urn:graphlaw:"


def receipt_payload(r: dict) -> str:
    """Canonical receipt payload; `r` is a receipt object from a `law` response."""
    body = {"added": r["added"], "authority": r["authority"], "child": r["child"],
            "lease_id": r.get("lease_id"), "parent": r["parent"], "plan_blake3": r.get("plan_blake3"),
            "revision": str(r["revision"]), "step": r["step"], "subject_blake3": r.get("subject_blake3")}
    return json.dumps(body, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def receipt_ntriples(r: dict, attestation: Optional[dict] = None) -> str:
    """Record a receipt as N-Triples (the form `require-receipt` / `require-signed-receipt` read)."""
    subj = f"<{_NS}receipt:{r['child']}>"
    facts = [("parent", r["parent"]), ("child", r["child"]), ("step", r["step"]),
             ("authority", r["authority"]), ("revision", str(r["revision"])), ("added", str(r["added"]))]
    for k in ("lease_id", "plan_blake3", "subject_blake3"):
        if r.get(k):
            facts.append((k, r[k]))
    if attestation:
        facts += [("attest_payload_blake3", attestation["payload_blake3"]),
                  ("attest_key_id", attestation["key_id"]), ("attest_signature", attestation["signature"])]
    esc = lambda s: s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")
    return "".join(f'{subj} <{_NS}p:{k}> "{esc(v)}" .\n' for k, v in facts)


def sign_receipt(seed_hex: str, r: dict, binary: Optional[str] = None) -> dict:
    return _sign_payload(seed_hex, receipt_payload(r), binary)
