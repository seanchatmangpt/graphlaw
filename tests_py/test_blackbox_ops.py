"""External black-box court for the graphlaw binaries.

Every case drives a real subprocess and asserts on real stdout / exit codes.
No Rust test imports and no mocks: the only Rust side-channel is the
`blackbox_fixture` example, itself launched as a subprocess, which produces a
real admitted plan + Ed25519-signed receipt store (mirroring
tests/verify_cli.rs, asserted from OUTSIDE the Rust suite).
"""

from __future__ import annotations

import base64
import json
import os
import struct
import subprocess
import warnings

import pytest

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

TURTLE_LAW_DOC = """
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"""

WASM_PATH = os.path.join(REPO, "priv", "graphlaw.wasm")
# The pin is the sidecar written with priv/graphlaw.wasm. A second hardcoded
# digest exits 84 before the SQL parse runs whenever the wasm row is re-pinned.
WASM_DIGEST = open(
    os.path.join(REPO, "priv", "graphlaw.wasm.blake3"), encoding="utf-8"
).read().split()[0]


# ---------------------------------------------------------------------------
# helpers


def _run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def packet4_frame(body: bytes) -> bytes:
    """packet-4 framing: 4-byte big-endian length prefix, then the body."""
    return struct.pack(">I", len(body)) + body


def read_packet4_frame(stream):
    """Read one packet-4 frame over binary stdin/stdout (latin1 framing per tools.md)."""
    len_bytes = stream.read(4)
    if len(len_bytes) < 4:
        return None
    (length,) = struct.unpack(">I", len_bytes)
    body = b""
    while len(body) < length:
        chunk = stream.read(length - len(body) or 1)
        if not chunk:
            raise AssertionError(f"EOF mid-frame: got {len(body)}/{length} body bytes")
        body += chunk
    return body


def graphlaw_run(args, input_bytes=b"", env_extra=None, timeout=120):
    env = dict(os.environ)
    env["GRAPHLAW_WASM"] = WASM_PATH
    if env_extra:
        env.update(env_extra)
    return subprocess.run(
        [os.path.join(REPO, "target", "debug", "graphlaw-run")] + args,
        input=input_bytes,
        capture_output=True,
        env=env,
        timeout=timeout,
    )


def eval_envelope(request_obj: dict) -> bytes:
    """The smalt eval envelope: `payload_b64` carries the raw ABI request."""
    payload = base64.b64encode(json.dumps(request_obj).encode("utf-8")).decode("ascii")
    return json.dumps(
        {"op": "eval", "payload_b64": payload, "predicate_frames": []}
    ).encode("utf-8")


def verify_cmd(graphlaw_binaries, fx, trusted_hex=None):
    return [
        graphlaw_binaries["verify"],
        "--start", fx["start"],
        "--plan", fx["plan"],
        "--receipts", fx["receipts"],
        "--trusted-key", trusted_hex if trusted_hex is not None else fx["trusted_hex"],
    ]


# ---------------------------------------------------------------------------
# (a) graphlaw-verify: real plan + real signed receipt store


def test_verify_happy_path_admitted(verify_fixture, graphlaw_binaries):
    r = _run(verify_cmd(graphlaw_binaries, verify_fixture))
    assert r.returncode == 0, f"stdout={r.stdout} stderr={r.stderr}"
    assert r.stdout.strip() == "ADMITTED"


def test_verify_receipt_tamper_is_typed_refusal_exit_1(verify_fixture, graphlaw_binaries):
    """Flip one digit inside a stored receipt file -> `REFUSED: ReceiptTampered`, exit 1."""
    subject_dir = verify_fixture["subject"]
    candidates = sorted(f for f in os.listdir(subject_dir) if f.endswith(".json"))
    assert candidates, "fixture store holds no receipt files"
    path = os.path.join(subject_dir, candidates[0])
    with open(path, "r", encoding="utf-8") as fh:
        doc = json.load(fh)
    assert "added" in doc, f"receipt file lacks an 'added' field: {path}"
    doc["added"] = int(doc["added"]) + 1
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(json.dumps(doc))

    r = _run(verify_cmd(graphlaw_binaries, verify_fixture))
    assert r.returncode == 1, f"stdout={r.stdout} stderr={r.stderr}"
    assert r.stdout.startswith("REFUSED: ReceiptTampered:"), r.stdout


def test_verify_untrusted_signer_is_typed_refusal(verify_fixture, graphlaw_binaries):
    """Same fixture, verifier trusts a different (valid) key -> `UntrustedKey`, exit 1."""
    mallory_pubkey = _run(
        [
            graphlaw_binaries["attest"],
            "pubkey",
            "--key-seed-hex", "09" * 32,
        ]
    )
    assert mallory_pubkey.returncode == 0, mallory_pubkey.stderr
    mallory_hex = mallory_pubkey.stdout.strip()
    assert len(mallory_hex) == 64

    r = _run(verify_cmd(graphlaw_binaries, verify_fixture, trusted_hex=mallory_hex))
    assert r.returncode == 1, f"stdout={r.stdout} stderr={r.stderr}"
    assert r.stdout.startswith("REFUSED: UntrustedKey:"), r.stdout


def test_verify_missing_trusted_key_is_usage_exit_2(verify_fixture, graphlaw_binaries):
    cmd = verify_cmd(graphlaw_binaries, verify_fixture)
    cmd.remove("--trusted-key")
    cmd.remove(verify_fixture["trusted_hex"])
    r = _run(cmd)
    assert r.returncode == 2, f"stdout={r.stdout} stderr={r.stderr}"
    assert "trusted-key" in (r.stderr + r.stdout)


# ---------------------------------------------------------------------------
# (b) graphlaw-run: packet-4 stdio framing over a Turtle law doc


@pytest.fixture(scope="module")
def require_wasm_artifact():
    if not os.path.exists(WASM_PATH):
        warnings.warn(
            "LOUD SKIP: wasm artifact missing, packet-4 tests degraded "
            f"(no coverage): {WASM_PATH}",
            stacklevel=2,
        )
        pytest.skip(f"wasm artifact missing: {WASM_PATH}")
    return WASM_PATH


def packet4_session(runner_args, frames):
    """Drive one graphlaw-run process: write frames, read reply frames, wait."""
    env = dict(os.environ)
    env["GRAPHLAW_WASM"] = WASM_PATH
    proc = subprocess.Popen(
        [os.path.join(REPO, "target", "debug", "graphlaw-run")] + runner_args,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
    )
    assert proc.stdin is not None, "subprocess stdin pipe missing"
    assert proc.stdout is not None, "subprocess stdout pipe missing"
    assert proc.stderr is not None, "subprocess stderr pipe missing"
    replies = []
    try:
        for f in frames:
            proc.stdin.write(f)
            proc.stdin.flush()
            reply = read_packet4_frame(proc.stdout)
            if reply is not None:
                replies.append(reply)
    finally:
        proc.stdin.close()
        rc = proc.wait(timeout=60)
        err = proc.stderr.read().decode("utf-8", "replace")
    return rc, replies, err


def test_run_packet4_eval_turtle_law_doc(require_wasm_artifact, graphlaw_binaries):
    """Real wasm eval of a Turtle SQL-in-RDF law doc through packet-4 framing."""
    frame = packet4_frame(
        eval_envelope({"op": "parse", "text": TURTLE_LAW_DOC, "dialect": "turtle"})
    )
    rc, replies, err = packet4_session(["--expect-blake3", WASM_DIGEST], [frame])
    assert rc == 0, f"stderr={err}"
    assert len(replies) == 1, f"expected 1 reply frame, got {len(replies)}; stderr={err}"
    reply = json.loads(replies[0].decode("utf-8"))
    assert reply["ok"] is True
    assert reply["predicate_frames_count"] == 0
    result = reply["result"]
    assert result["dialect"] == "Turtle"
    assert result["quads"] > 0, f"no quads parsed: {result}"
    assert isinstance(result["id"], str) and len(result["id"]) >= 64

    sql_frame = packet4_frame(
        eval_envelope({"op": "parse", "text": TURTLE_LAW_DOC, "dialect": "sql"})
    )
    sql_rc, sql_replies, sql_err = packet4_session(["--expect-blake3", WASM_DIGEST], [sql_frame])
    assert sql_rc == 0, f"stderr={sql_err}"
    sql = json.loads(sql_replies[0].decode("utf-8"))["result"]
    assert sql["ok"] is True, sql
    assert sql["dialect"] == "SqlRdf"
    assert sql["id"] == result["id"], sql
    assert sql["quads"] == result["quads"], sql


def test_run_packet4_two_frames_one_process(require_wasm_artifact, graphlaw_binaries):
    """The eval loop serves multiple frames on one connection, then EOF exits 0."""
    frames = [
        packet4_frame(eval_envelope({"op": "parse", "text": TURTLE_LAW_DOC, "dialect": "turtle"})),
        packet4_frame(eval_envelope({"op": "capabilities"})),
    ]
    rc, replies, err = packet4_session([], frames)
    assert rc == 0, f"stderr={err}"
    assert len(replies) == 2
    cap = json.loads(replies[1].decode("utf-8"))
    assert cap["ok"] is True and cap["result"], f"capabilities reply empty: {cap}"


def test_run_digest_mismatch_is_loud_refusal_exit_84(require_wasm_artifact):
    bad_digest = "0" * 64
    rc, _replies, err = packet4_session(["--expect-blake3", bad_digest], [])
    # Refusal fires before any frame is served.
    assert rc == 84, f"stderr={err}"
    assert "ARTIFACT DIGEST MISMATCH" in err


def test_run_eof_midframe_is_typed_exit_90(require_wasm_artifact):
    """A truncated frame is a typed 90, never a hang or a generic crash."""
    truncated = packet4_frame(b'{"op": "eval"}')[:-5]  # drop 5 body bytes
    env = dict(os.environ)
    env["GRAPHLAW_WASM"] = WASM_PATH
    proc = subprocess.Popen(
        [os.path.join(REPO, "target", "debug", "graphlaw-run")],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
    )
    assert proc.stdin is not None, "subprocess stdin pipe missing"
    assert proc.stderr is not None, "subprocess stderr pipe missing"
    proc.stdin.write(truncated)
    proc.stdin.close()
    rc = proc.wait(timeout=60)
    err = proc.stderr.read().decode("utf-8", "replace")
    assert rc == 90, f"stderr={err}"
    assert "EOF mid-frame" in err


def test_run_malformed_envelope_is_typed_exit_93(require_wasm_artifact):
    frame = packet4_frame(
        json.dumps({"op": "eval", "payload_b64": "!!!not base64!!!"}).encode("utf-8")
    )
    rc, _replies, err = packet4_session([], [frame])
    assert rc == 93, f"stderr={err}"
    assert "malformed envelope" in err


# ---------------------------------------------------------------------------
# (c) typed-refusal courts
#
# Q2 fix: the old trio asserted on refusal_probe's own `[exit-class MATCH]`
# self-grading (prober, judge, scorekeeper in one). The probe example run is
# kept as a SMOKE assert (exit 0) only. Each of the six named classes is now
# asserted against the pinned register text (docs/refusals.md) in the probe's
# VERBATIM message body — the probe's own MATCH/MISS verdict lines are never
# read by this court.
#
# Reachability note: `graphlaw-run` hosts the wasm module only, and the wasm
# dispatch (src/abi.rs:334, `#[cfg(not(target_arch = "wasm32"))]`) excludes
# the `duckdb` op — `compile_turtle`/`eval_ast_query`/`builtin_eval_sql`
# therefore have NO subprocess surface today. The refusal_probe example is
# the one real process boundary to them (same pattern as blackbox_fixture);
# the graphlaw-run `canonical` op is the one wire arm that does cross, and it
# is exercised directly below.


REFUSAL_PROBE_PINS = {
    # probe label prefix -> (pinned message substring, forbidden substring)
    "REFUSAL-04 non-document-backed reader": ("REFUSAL-04", None),
    "NO-CANONICAL-GRAPH hand-built CompiledQuery": ("no canonical graph bytes", None),
    "ATTEST-REFUSED-class admission-seal ReceiptError": ("admission seal mismatch", None),
    "ReceiptError canonicalization refused": ("Receipt generation error", None),
    "UNCLASSIFIED unknown function IRI": ("MEDIAN", "REFUSAL-0"),
    "direct validate_ast_shacl": ("SHACL AST structural violation", None),
}


@pytest.fixture(scope="module")
def refusal_probe_output(graphlaw_binaries):
    r = subprocess.run(
        ["cargo", "run", "--quiet", "--example", "refusal_probe"],
        cwd=REPO,
        capture_output=True,
        text=True,
        timeout=900,
    )
    return r


def test_refusal_probe_smoke_exit_zero(refusal_probe_output):
    """Smoke only: the probe process itself completed. Never the oracle."""
    r = refusal_probe_output
    assert r.returncode == 0, f"stdout={r.stdout} stderr={r.stderr}"


def _probe_block_message(out: str, label_prefix: str) -> str:
    """Extract one probe's verbatim message body (between the label and the
    probe's own `[exit-class ...]` annotation, which is deliberately ignored)."""
    for blk in out.split("=== "):
        if blk.startswith(label_prefix):
            lines = [
                ln
                for ln in blk.splitlines()[1:]
                if ln.strip() and not ln.startswith("[exit-class")
            ]
            assert lines, f"probe block `{label_prefix}` carries no message"
            return "\n".join(lines)
    raise AssertionError(f"probe block `{label_prefix}` not found in probe stdout")


@pytest.mark.parametrize("label_prefix", sorted(REFUSAL_PROBE_PINS))
def test_refusal_class_fires_pinned_register_message(label_prefix, refusal_probe_output):
    """Each of the six classes: the pinned refusal text (docs/refusals.md
    wording) must appear verbatim in the probe's real message body, and the
    failure arm must be the typed refusal, never a silent success."""
    out = refusal_probe_output.stdout
    assert "SUCCEEDED — expected refusal" not in out
    msg = _probe_block_message(out, label_prefix)
    pinned, forbidden = REFUSAL_PROBE_PINS[label_prefix]
    assert pinned in msg, f"{label_prefix}: pinned text missing.\nblock:\n{msg}"
    if forbidden is not None:
        assert forbidden not in msg, f"{label_prefix}:\n{msg}"


def abi_reply(request_obj: dict, require_wasm_artifact) -> dict:
    """One raw ABI request through the real graphlaw-run packet-4 passthrough.
    Returns the module's raw ABI response under the runner envelope."""
    frame = packet4_frame(json.dumps(request_obj).encode("utf-8"))
    rc, replies, err = packet4_session([], [frame])
    assert rc == 0, f"runner exited {rc}: {err}"
    assert len(replies) == 1, f"expected 1 reply, got {len(replies)}; stderr={err}"
    reply = json.loads(replies[0].decode("utf-8"))
    assert reply["ok"] is True, f"runner envelope not ok: {reply}; stderr={err}"
    return reply["result"]


def test_wire_canonicalization_refused_on_malformed_turtle(require_wasm_artifact):
    """canonicalization-refused, driven directly over the wire: malformed
    Turtle through the real canonicalization op is a typed ok:false refusal,
    never a crash. Positive control first: the good doc canonicalizes."""
    good = abi_reply(
        {
            "op": "canonical",
            "data": {"text": TURTLE_LAW_DOC, "dialect": "turtle"},
        },
        require_wasm_artifact,
    )
    assert good["ok"] is True, good
    assert good["quads"] > 0

    bad = abi_reply(
        {
            "op": "canonical",
            "data": {"text": "this is not turtle @@@", "dialect": "turtle"},
        },
        require_wasm_artifact,
    )
    assert bad["ok"] is False, bad
    assert bad["error"]["message"], bad
