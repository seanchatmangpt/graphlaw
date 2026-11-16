"""Packet-4 client for `graphlaw-run-eh`: one persistent subprocess, many ABI requests."""

from __future__ import annotations

import base64
import json
import os
import struct
import subprocess
from typing import Any, Optional, Sequence, Union

MAX_REQUEST_BYTES = 16 * 1024 * 1024

Spec = Union[str, dict]


def _spec(data: Spec) -> dict:
    """A data spec is {"text", "dialect"?, "hint"?, "base"?}; a bare string is sniffed."""
    return {"text": data} if isinstance(data, str) else data


class GraphLawError(Exception):
    """An ABI refusal (`{"ok": false, "error": {...}}`)."""

    def __init__(self, error: Any):
        self.error = error
        if isinstance(error, dict):
            super().__init__(f"{error.get('kind')}: {error.get('message')}")
        else:
            super().__init__(str(error))

    @property
    def kind(self) -> Optional[str]:
        return self.error.get("kind") if isinstance(self.error, dict) else None


class RunnerExit(Exception):
    """The runner process exited with a typed exit code (see graphlaw_run_eh.rs)."""

    def __init__(self, code: int, stderr: str):
        self.code = code
        self.stderr = stderr
        super().__init__(f"graphlaw-run-eh exited {code}: {stderr.strip()}")


def _default_runner() -> str:
    env = os.environ.get("GRAPHLAW_RUN_EH")
    if env:
        return env
    repo = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    return os.path.join(repo, "target", "debug", "graphlaw-run-eh")


def _default_wasm() -> Optional[str]:
    return os.environ.get("GRAPHLAW_WASM")


class GraphLaw:
    """Context-managed session. Each method sends one ABI op and returns its result."""

    def __init__(
        self,
        runner: Optional[str] = None,
        wasm: Optional[str] = None,
        expect_blake3: Optional[str] = None,
        epoch_ticks: Optional[int] = None,
    ):
        args = [runner or _default_runner()]
        wasm = wasm or _default_wasm()
        env = dict(os.environ)
        if wasm:
            env["GRAPHLAW_WASM"] = wasm
        if expect_blake3:
            args += ["--expect-blake3", expect_blake3]
        if epoch_ticks:
            args += ["--epoch-ticks", str(epoch_ticks)]
        self._proc = subprocess.Popen(
            args,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
        )

    def __enter__(self) -> "GraphLaw":
        return self

    def __exit__(self, *exc) -> None:
        self.close()

    def close(self) -> None:
        p = self._proc
        if p.stdin and not p.stdin.closed:
            p.stdin.close()
        try:
            p.wait(timeout=60)
        finally:
            if p.stdout:
                p.stdout.close()
            if p.stderr:
                p.stderr.close()

    def _read_exact(self, n: int) -> Optional[bytes]:
        out = b""
        while len(out) < n:
            chunk = self._proc.stdout.read(n - len(out))
            if not chunk:
                return None
            out += chunk
        return out

    def _exchange(self, body: bytes) -> bytes:
        if len(body) > MAX_REQUEST_BYTES:
            raise ValueError("request exceeds MAX_REQUEST_BYTES")
        try:
            self._proc.stdin.write(struct.pack(">I", len(body)) + body)
            self._proc.stdin.flush()
        except BrokenPipeError:
            self._raise_exit()
        head = self._read_exact(4)
        if head is None:
            self._raise_exit()
        (n,) = struct.unpack(">I", head)
        reply = self._read_exact(n)
        if reply is None:
            self._raise_exit()
        return reply

    def _raise_exit(self) -> None:
        code = self._proc.wait(timeout=60)
        err = self._proc.stderr.read().decode("utf-8", "replace") if self._proc.stderr else ""
        raise RunnerExit(code, err)

    def call(self, request: dict) -> Any:
        """Send a raw ABI request (smalt eval envelope); return the `result` payload."""
        payload = base64.b64encode(json.dumps(request).encode("utf-8")).decode("ascii")
        envelope = {"op": "eval", "payload_b64": payload, "predicate_frames": []}
        reply = json.loads(self._exchange(json.dumps(envelope).encode("utf-8")))
        if not reply.get("ok", False):
            raise GraphLawError(reply.get("error", reply))
        result = reply.get("result")
        if isinstance(result, dict) and result.get("ok") is False:
            raise GraphLawError(result.get("error", result))
        return result

    # --- ABI ops (src/abi.rs) -------------------------------------------------

    def capabilities(self) -> Any:
        return self.call({"op": "capabilities"})

    def sniff(self, text: str, hint: Optional[str] = None) -> Any:
        return self.call({"op": "sniff", "text": text, **({"hint": hint} if hint else {})})

    def parse(self, text: str, dialect: Optional[str] = None, **kw) -> Any:
        return self.call({"op": "parse", "text": text, **({"dialect": dialect} if dialect else {}), **kw})

    def convert(self, text: str, to: str, dialect: Optional[str] = None, **kw) -> Any:
        return self.call({"op": "convert", "text": text, "to": to, **({"dialect": dialect} if dialect else {}), **kw})

    def canonical(self, data: Spec, **kw) -> Any:
        return self.call({"op": "canonical", "data": _spec(data), **kw})

    def sparql(self, data: Spec, query: str, **kw) -> Any:
        return self.call({"op": "sparql", "data": _spec(data), "query": query, **kw})

    def shacl(self, data: Spec, shapes: str, **kw) -> Any:
        """`shapes` is Turtle SHACL text."""
        return self.call({"op": "shacl", "data": _spec(data), "shapes": shapes, **kw})

    def law(self, data: Spec, steps: Sequence[dict], **kw) -> Any:
        return self.call({"op": "law", "data": _spec(data), "steps": list(steps), **kw})

    def shex(self, data: Spec, schema: str, map: str, schema_dialect: Optional[str] = None, **kw) -> Any:
        return self.call({"op": "shex", "data": _spec(data), "schema": schema, "map": map,
                          **({"schema_dialect": schema_dialect} if schema_dialect else {}), **kw})

    def n3(self, text: str) -> Any:
        return self.call({"op": "n3", "text": text})

    def entail(self, data: Spec, regime: str) -> Any:
        """regime: simple | rdf | rdfs | owl-rl | d."""
        return self.call({"op": "entail", "data": _spec(data), "regime": regime})

    def datalog(self, rules: Sequence[dict], facts: Sequence[Sequence[str]]) -> Any:
        return self.call({"op": "datalog", "rules": list(rules), "facts": [list(f) for f in facts]})

    def policy(self, problem: Any, policy: Any) -> Any:
        """Admit a FOND policy as strong-cyclic against a planning problem (JSON or string)."""
        return self.call({"op": "policy", "problem": problem, "policy": policy})

    def hooks(self, data: Spec, pack: Spec, **kw) -> Any:
        """Materialize a knowledge-hook (`kh:`) pack over `data` to fixpoint.

        Returns {id, rounds, quads, nquads, firings[{hook, round, added, row}]}.
        """
        return self.call({"op": "hooks", "data": _spec(data), "pack": _spec(pack), **kw})

    def law_hooks_step(self, pack: Spec) -> dict:
        """A `hooks` step for `law(...)`: runs the pack as one receipted transition."""
        return {"step": "hooks", "pack": _spec(pack)}

    # --- duckdb op: named sessions live in the runner process, gone when it exits ---

    def duckdb_open(self, session: str) -> Any:
        return self.call({"op": "duckdb", "mode": "open", "session": session})

    def duckdb_exec(self, **fields) -> Any:
        return self.call({"op": "duckdb", "mode": "exec", **fields})

    def duckdb_stream(self, **fields) -> Any:
        return self.call({"op": "duckdb", "mode": "stream", **fields})

    def duckdb_close(self, session: Optional[str] = None, cursor: Optional[str] = None) -> Any:
        req = {"op": "duckdb", "mode": "close"}
        if session:
            req["session"] = session
        if cursor:
            req["cursor"] = cursor
        return self.call(req)

    def op(self, name: str, **fields) -> Any:
        """Any other registry op (`shex`, `n3`, `entail`, `datalog`, `hooks`, `policy`)."""
        return self.call({"op": name, **fields})
