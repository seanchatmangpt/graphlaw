"""External black-box court fixtures.

Every collaborator is a real subprocess: the graphlaw binaries built by cargo
and the `blackbox_fixture` example that produces a real admitted-plan receipt
store. No Rust test imports, no mocks. Shared binaries are built once per
pytest session.
"""

from __future__ import annotations

import os
import subprocess
import sys

import pytest

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def _bin(name: str) -> str:
    return os.path.join(REPO, "target", "debug", name)


@pytest.fixture(scope="session")
def graphlaw_binaries():
    """Build the real binaries once; yield their paths."""
    r = subprocess.run(
        ["cargo", "build", "--bins", "--example", "blackbox_fixture", "--features", "runner"],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, f"cargo build failed:\n{r.stderr}"
    for name in ("graphlaw-verify", "graphlaw-run", "graphlaw-attest"):
        assert os.path.exists(_bin(name)), f"missing binary {name}"
    return {
        "verify": _bin("graphlaw-verify"),
        "run": _bin("graphlaw-run"),
        "attest": _bin("graphlaw-attest"),
        "fixture": os.path.join(REPO, "target", "debug", "examples", "blackbox_fixture"),
    }


@pytest.fixture()
def verify_fixture(graphlaw_binaries, tmp_path):
    """A real admitted plan + signed receipt store in tmp_path.

    Yields a dict with the file paths and the trusted key hex printed by the
    fixture producer (its last stdout line).
    """
    r = subprocess.run(
        [graphlaw_binaries["fixture"], str(tmp_path)],
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, f"fixture producer failed:\n{r.stderr}"
    lines = [ln.strip() for ln in r.stdout.strip().splitlines() if ln.strip()]
    assert lines, "fixture producer printed no trusted key"
    trusted_hex = lines[-1]
    assert len(trusted_hex) == 64, f"not a 32-byte key hex: {trusted_hex!r}"
    return {
        "start": str(tmp_path / "start.nt"),
        "plan": str(tmp_path / "plan.json"),
        "receipts": str(tmp_path / "receipts"),
        "subject": str(tmp_path / "receipts" / "0123456789abcdef0123456789abcdef01234567"),
        "trusted_hex": trusted_hex,
        "dir": str(tmp_path),
    }


def is_windows() -> bool:
    return sys.platform.startswith("win")
