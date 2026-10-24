"""Lane conftest for the pm4pytest BCBS 239 driver (tests/test_bcbs_aggregation.py).

Honors GRAPHLAW_TARGET_DIR: every binary the driver consumes is built into the
lane root (default `target-lane-pm1`) so concurrent lanes never collide. All
collaborators are real subprocesses: cargo, the graphlaw-run-eh host, and the
DuckDB-linked wasm module inside it. No mocks (Chicago).
"""

from __future__ import annotations

import os
import struct
import subprocess

import pytest

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

WASI_SDK = os.environ.get(
    "WASI_SDK_PATH",
    os.path.join(os.path.expanduser("~"), ".frozen-duckdb", "cache",
                 "v2.0.0-dev-wasm32-wasip1"),
)
# SQLRDF-27 wrap link args: the musl->wasi fcntl translation must reach the
# final cdylib link. The wrap shim is the committed in-tree object
# tests/fixtures/wasm-shim/glaw_wrap_shim.o (docs/WASM.md:20-36); it defines
# only __wrap_open/__wrap_openat and defers to __real_open/__real_openat.
# It is never recompiled here. The frozen-duckdb stub wasi_posix.c provides no
# __wrap_* symbols (docs/WASM.md:38-46), so a `--whole-archive
# -lduckdb_wasi_posix` form is wrong: it duplicates the rlib-bundled
# wasi_posix.o symbols, and wasm-ld does not extract archive members for
# --wrap/-u/--export references anyway. +exception-handling per docs/WASM.md.
SHIM_OBJ = os.path.join(
    REPO, "tests", "fixtures", "wasm-shim", "glaw_wrap_shim.o"
)
WRAP_RUSTFLAGS = (
    "-C target-feature=+exception-handling "
    "-C link-arg=--wrap=open -C link-arg=--wrap=openat "
    f"-C link-arg={SHIM_OBJ}"
)
# CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS is what carries the recipe (RUSTFLAGS
# would also poison any host build sharing the invocation).
WRAP_ENV_KEY = "CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS"


def lane_root() -> str:
    return os.environ.get("GRAPHLAW_TARGET_DIR", os.path.join(REPO, "target-lane-pm1"))


def host_bin() -> str:
    return os.path.join(lane_root(), "debug", "graphlaw-run-eh")


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


def _build_once(args, env, log_label):
    r = subprocess.run(args, cwd=REPO, env=env, capture_output=True, text=True)
    if r.returncode != 0:
        pytest.fail(
            f"{log_label} build failed (exit {r.returncode}):\n{r.stderr[-4000:]}",
            pytrace=False,
        )


@pytest.fixture(scope="session")
def bcbs_binaries():
    """Build the runner-eh host and the wrap-enabled wasm artifact once."""
    root = lane_root()
    os.makedirs(root, exist_ok=True)

    if not os.path.isfile(SHIM_OBJ):
        pytest.fail(
            f"wrap shim object missing: {SHIM_OBJ} "
            "(build per docs/WASM.md:20-23; it is never recompiled here)"
        )

    host = host_bin()
    if not os.path.exists(host):
        _build_once(
            ["cargo", "build", "-p", "graphlaw", "--bin", "graphlaw-run-eh",
             "--features", "runner-eh"],
            {**os.environ, "CARGO_TARGET_DIR": root},
            "graphlaw-run-eh",
        )
    assert os.path.exists(host), f"host binary missing after build: {host}"

    wasm = wasm_artifact()
    if not os.path.exists(wasm):
        _build_once(
            ["cargo", "build", "-p", "graphlaw-wasm", "--target", "wasm32-wasip1",
             "--profile", "wasm"],
            {**os.environ, "CARGO_TARGET_DIR": root, "WASI_SDK_PATH": WASI_SDK,
             WRAP_ENV_KEY: WRAP_RUSTFLAGS},
            "graphlaw-wasm",
        )
    assert os.path.exists(wasm), f"wasm artifact missing after build: {wasm}"
    return {"host": host, "wasm": wasm}


@pytest.fixture()
def eh_runner(bcbs_binaries):
    """Factory: one real graphlaw-run-eh subprocess per test (fresh module
    state per session name). Packet-4 framing over binary pipes."""
    spawned = []

    def _spawn():
        p = subprocess.Popen(
            [bcbs_binaries["host"], "--wasm", bcbs_binaries["wasm"]],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        spawned.append(p)
        return Packet4(p)

    yield _spawn
    for p in spawned:
        try:
            if p.stdin and not p.stdin.closed:
                p.stdin.close()
        except Exception:
            pass
        p.wait(timeout=30)


class Packet4:
    """packet-4 stdio framing (4-byte BE length prefix) over binary pipes."""

    def __init__(self, proc):
        self.proc = proc

    def request(self, body: bytes) -> bytes:
        self.proc.stdin.write(struct.pack(">I", len(body)) + body)
        self.proc.stdin.flush()
        len_buf = self._read_exact(4)
        (length,) = struct.unpack(">I", len_buf)
        return self._read_exact(length)

    def _read_exact(self, n: int) -> bytes:
        buf = b""
        while len(buf) < n:
            chunk = self.proc.stdout.read(n - len(buf))
            if not chunk:
                err = self.proc.stderr.read().decode(errors="replace")
                raise AssertionError(
                    f"runner EOF mid-frame (exit pending); stderr: {err[-2000:]}"
                )
            buf += chunk
        return buf
