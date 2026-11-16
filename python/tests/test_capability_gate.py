"""The capability gate as a pytest court: green gate + non-vacuity self-check, real runner."""

import os
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def _run(*args):
    env = dict(os.environ, PYTHONPATH=os.path.join(REPO, "python"))
    return subprocess.run([sys.executable, "-m", "graphlaw.coverage", *args], cwd=REPO,
                          capture_output=True, text=True, env=env, timeout=1500)


def test_gate_is_green_and_every_gap_is_registered():
    r = _run()
    assert r.returncode == 0, r.stdout[-3000:] + r.stderr[-1000:]
    assert "DEFECT" not in r.stdout
    assert "cells=" in r.stdout


def test_gate_is_not_vacuous():
    r = _run("--self-check")
    assert r.returncode == 0, r.stdout[-3000:] + r.stderr[-1000:]
    assert "VACUOUS GATE" not in r.stdout and "self-check: OK" in r.stdout
