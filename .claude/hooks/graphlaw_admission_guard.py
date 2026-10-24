#!/usr/bin/env python3
"""PreToolUse admission guard — graphlaw is the law, Python is only transport.

Lane P1 mapping (operator FOND decomposition, v26.10.22):

  tool call --(hook)--> RDF transition proposal (act:ToolCall)
            --(packet-4 stdio)--> graphlaw-run-eh hosting the compiled wasm module
            --(`shacl` ABI op)--> SHACL report vs .claude/policy/graphlaw-seed-shapes.ttl
            --(conforms? allow : refuse BEFORE disk alteration)

Refusal channel: the PreToolUse deny JSON (permissionDecision: deny), the
channel Claude Code actually blocks on, plus the graphlaw refusal reason on
stderr and exit 1 for standalone courts. (Exit 1 alone is a NON-blocking
signal in Claude Code; the deny JSON is the real gate — a bare exit-1 hook
would be a vacuous gate once registered, so the JSON is the load-bearing
refusal and the exit code is the court-visible one.)

Infra failures (runner/wasm/shapes missing) fail OPEN with a loud stderr
GRAPHLAW-ADMISSION: INFRA-FAIL line, matching manifest_guard.py precedent;
the non-vacuity court covers the real refuse/allow paths.

Prototype honesty: this routes through the graphlaw-run-eh HOST runner
(wasmtime-48 hosting the same compiled artifact as the WASM bridge increment;
wasmi retired 2026-10-24 per ADR-wasmi-retirement.md), not through wasmex
inside a BEAM. The bridge is sequenced, not skipped — see the blueprint.
"""
import json
import os
import struct
import subprocess
import sys
from typing import NoReturn

ROOT = os.environ.get("CLAUDE_PROJECT_DIR", "/Users/sac/graphlaw")
RUNNER = os.path.join(ROOT, "target/debug/graphlaw-run-eh")
WASM = os.environ.get("GRAPHLAW_WASM") or os.path.join(ROOT, "priv/graphlaw.wasm")
SHAPES = os.path.join(ROOT, ".claude/policy/graphlaw-seed-shapes.ttl")

FILE_TOOLS = ("Edit", "Write", "MultiEdit", "NotebookEdit")


def fail_open(reason: str) -> NoReturn:
    print(f"GRAPHLAW-ADMISSION: INFRA-FAIL (fail-open): {reason}", file=sys.stderr)
    sys.exit(0)


def run_graphlaw_shacl(proposal_turtle: str, shapes_ttl: str):
    """One packet-4 frame in (a raw ABI request), one reply frame out."""
    request = json.dumps({
        "op": "shacl",
        "data": {"text": proposal_turtle, "dialect": "turtle"},
        "shapes": shapes_ttl,
    }).encode()
    p = subprocess.run(
        [RUNNER],
        input=struct.pack(">I", len(request)) + request,
        capture_output=True,
        env={**os.environ, "GRAPHLAW_WASM": WASM},
    )
    if p.returncode != 0:
        raise RuntimeError(f"graphlaw-run-eh exit {p.returncode}: {p.stderr.decode()[:400]}")
    out = p.stdout
    if len(out) < 4:
        raise RuntimeError("graphlaw-run-eh: short reply (no length prefix)")
    (n,) = struct.unpack(">I", out[:4])
    return json.loads(out[4 : 4 + n])


def refuse(reason: str) -> NoReturn:
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))
    print(reason, file=sys.stderr)
    sys.exit(1)


def main() -> None:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        fail_open("unreadable hook stdin (fail-open per manifest_guard precedent)")

    tool = payload.get("tool_name", "")
    if tool not in FILE_TOOLS:
        sys.exit(0)  # Bash escape hatch documented in the blueprint

    file_path = payload.get("tool_input", {}).get("file_path", "") or ""
    if not file_path:
        sys.exit(0)

    if not (os.path.exists(RUNNER) and os.path.exists(WASM) and os.path.exists(SHAPES)):
        fail_open(f"runner/wasm/shapes missing (runner={RUNNER} wasm={WASM} shapes={SHAPES})")

    proposal = (
        "@prefix act: <http://graphlaw.org/act#> .\n"
        "[] a act:ToolCall ;\n"
        f"    act:tool \"{tool}\" ;\n"
        f"    act:targetsFile \"{file_path}\" .\n"
    )

    try:
        with open(SHAPES) as f:
            shapes = f.read()
        reply = run_graphlaw_shacl(proposal, shapes)
    except Exception as e:  # noqa: BLE001
        fail_open(str(e))

    result = reply.get("result", {})
    if not result.get("conforms", True):
        msgs = []
        for r in result.get("results", []):
            m = r.get("message", "")
            if isinstance(m, list):
                m = "; ".join(str(x) for x in m)
            msgs.append(m or f"{r.get('component', '?')} on {r.get('shape', '?')}")
        refuse("GRAPHLAW-ADMISSION REFUSED — " + " | ".join(msgs))
    sys.exit(0)


if __name__ == "__main__":
    main()
