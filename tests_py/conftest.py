"""External black-box court fixtures.

Every collaborator is a real subprocess: the graphlaw binaries built by cargo
and the `blackbox_fixture` example that produces a real admitted-plan receipt
store. No Rust test imports, no mocks. Shared binaries are built once per
pytest session.
"""

from __future__ import annotations

import json
import os
import struct
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
        ["cargo", "build", "--bins", "--example", "blackbox_fixture", "--features", "runner-eh"],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, f"cargo build failed:\n{r.stderr}"
    for name in ("graphlaw-verify", "graphlaw-run-eh", "graphlaw-attest"):
        assert os.path.exists(_bin(name)), f"missing binary {name}"
    return {
        "verify": _bin("graphlaw-verify"),
        "run": _bin("graphlaw-run-eh"),
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


# ---------------------------------------------------------------------------
# EH module fixtures (lane pm2 — UBO sanctions pm4pytest driver)
#
# The collaborator is the real `graphlaw-run-eh` wasmtime-48 host speaking
# packet-4 over binary stdio against the real DuckDB-linked wasm artifact
# built in this lane's target root. No mocks; a missing artifact is a loud,
# typed SKIP, never a pass.

def lane_root() -> str:
    return os.environ.get("GRAPHLAW_TARGET_DIR", os.path.join(REPO, "target-lane-pm2"))


LANE_TARGET = lane_root()
EH_HOST = os.path.join(LANE_TARGET, "debug", "graphlaw-run-eh")


def wasm_artifact() -> str:
    """Resolve the wasm artifact: GRAPHLAW_WASM override first (typed failure
    if set-but-missing — never a silent pass), else the lane-root artifact."""
    p = os.environ.get("GRAPHLAW_WASM")
    if p:
        if not os.path.isfile(p):
            pytest.fail(
                f"GRAPHLAW_WASM is set but the artifact is missing: {p}"
            )
        return p
    return os.path.join(
        lane_root(), "wasm32-wasip1", "wasm", "graphlaw_wasm.wasm"
    )


def _wasi_sdk_path() -> str:
    # WASI_SDK_PATH override first; default is the frozen-duckdb cache SDK —
    # the only sysroot with the legacy-EH C++ runtime matching libduckdb.a
    # (docs/WASM.md:38-46). The wasi-sdk-34 fallback is the known-failing
    # new-EH-only sysroot, so a missing default is a typed failure.
    sdk = os.environ.get(
        "WASI_SDK_PATH",
        os.path.join(os.path.expanduser("~"), ".frozen-duckdb", "cache",
                     "v2.0.0-dev-wasm32-wasip1"),
    )
    if not os.path.isdir(sdk):
        pytest.fail(
            f"no wasi-sdk at {sdk} (set WASI_SDK_PATH; see docs/WASM.md)"
        )
    return sdk


# SQLRDF-27: the wrap args must ride the target-scoped RUSTFLAGS on the
# cdylib link — a dependency build script's rustc-link-arg does not reach the
# final link, and plain RUSTFLAGS is silently ignored for targets when the
# target-scoped variable exists. The shim is the committed in-tree object
# tests/fixtures/wasm-shim/glaw_wrap_shim.o (docs/WASM.md:20-36), never
# recompiled; it defines only __wrap_open/__wrap_openat and defers to
# __real_open/__real_openat. The frozen-duckdb stub wasi_posix.c provides no
# __wrap_* symbols (docs/WASM.md:38-46), so `--whole-archive
# -lduckdb_wasi_posix` is wrong on both counts.
SHIM_OBJ = os.path.join(
    REPO, "tests", "fixtures", "wasm-shim", "glaw_wrap_shim.o"
)
WRAP_ENV_KEY = "CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS"
_WASM_RUSTFLAGS = (
    "-C target-feature=+exception-handling "
    "-C link-arg=--wrap=open -C link-arg=--wrap=openat "
    f"-C link-arg={SHIM_OBJ}"
)


def _cargo(cmd, env_extra):
    env = dict(os.environ)
    env.update(env_extra)
    return subprocess.run(
        ["cargo"] + cmd, cwd=REPO, env=env, capture_output=True, text=True
    )


class UboModule:
    """One persistent eh-host subprocess; packet-4 request/response."""

    def __init__(self, host_path: str, wasm_path: str):
        env = dict(os.environ)
        env["WASI_SDK_PATH"] = _wasi_sdk_path()
        self.proc = subprocess.Popen(
            [host_path, "--wasm", wasm_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
        )

    def request(self, obj: dict) -> dict:
        body = json.dumps(obj).encode("utf-8")
        assert self.proc.poll() is None, "eh host exited early"
        self.proc.stdin.write(struct.pack(">I", len(body)) + body)
        self.proc.stdin.flush()
        len_bytes = self.proc.stdout.read(4)
        assert len(len_bytes) == 4, "eh host closed stdout at a frame boundary"
        (length,) = struct.unpack(">I", len_bytes)
        chunks = []
        got = 0
        while got < length:
            chunk = self.proc.stdout.read(length - got)
            assert chunk, f"EOF mid-frame: {got}/{length}"
            chunks.append(chunk)
            got += len(chunk)
        reply = json.loads(b"".join(chunks).decode("utf-8"))
        assert reply.get("ok") is True, f"eh host framed a failure: {reply}"
        return reply["result"]

    def close(self):
        try:
            if self.proc.stdin and not self.proc.stdin.closed:
                self.proc.stdin.close()
            self.proc.wait(timeout=60)
        except Exception:
            self.proc.kill()


@pytest.fixture(scope="session")
def ubo_module(request):
    """Build (once, if missing) and spawn the real module; typed SKIP if the
    build toolchain is unavailable."""
    wasm = wasm_artifact()
    missing = [p for p in (wasm, EH_HOST) if not os.path.isfile(p)]
    if missing:
        if not os.path.isfile(SHIM_OBJ):
            pytest.fail(
                f"wrap shim object missing: {SHIM_OBJ} "
                "(build per docs/WASM.md:20-23; it is never recompiled here)"
            )
        wasi = _wasi_sdk_path()
        env = {
            "WASI_SDK_PATH": wasi,
            WRAP_ENV_KEY: _WASM_RUSTFLAGS,
            "CARGO_TARGET_DIR": LANE_TARGET,
        }
        if not os.path.isfile(wasm):
            r = _cargo(
                ["build", "-p", "graphlaw-wasm", "--target", "wasm32-wasip1",
                 "--profile", "wasm"],
                env,
            )
            assert r.returncode == 0, (
                "wasm artifact build failed:\n" + r.stderr[-4000:]
            )
        r = _cargo(
            ["build", "--bin", "graphlaw-run-eh", "--features", "runner-eh"],
            {"CARGO_TARGET_DIR": LANE_TARGET},
        )
        assert r.returncode == 0, "eh host build failed:\n" + r.stderr[-4000:]

    mod = UboModule(EH_HOST, wasm)
    request.addfinalizer(mod.close)
    return mod
