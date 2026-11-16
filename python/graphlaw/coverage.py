"""Combinatorial-maximalism capability gate (fail-closed).

The capability universe is *derived* (registry JSON + duckdb dispatch arms + cargo bins), never
listed by hand. Every cell must be one of:

  PASS     a real probe through the real runner/binaries succeeded
  GAP      registered in docs/python/diataxis/coverage.json `gaps` with a reason (loud, counted)
  MISSING  no probe and no registered gap  -> exit 1
  FAIL     probe raised, doc tag missing, or API symbol absent -> exit 1

Also fails on: a gap for a cell that no longer exists, a gap whose probe now passes (stale gap),
a doc `cap:` tag naming a non-cell, and a stale generated capability matrix.

    PYTHONPATH=python python3 -m graphlaw.coverage            # gate; exit 1 on any defect
    PYTHONPATH=python python3 -m graphlaw.coverage --write-matrix
    PYTHONPATH=python python3 -m graphlaw.coverage --self-check   # non-vacuity: mutations must flip
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
from typing import Any, Callable, Dict, List, Optional, Tuple

from . import attest, steps, tools
from .client import GraphLaw, GraphLawError, RunnerExit
from .registry import BINARIES, REPO, duckdb_modes, load_op_examples, load_registry, run_registry_cli
from .verify import verify_receipts

DOC_ROOT = os.path.join(REPO, "docs", "python", "diataxis")
MANIFEST = os.path.join(DOC_ROOT, "coverage.json")
MATRIX = os.path.join(DOC_ROOT, "reference", "capability-matrix.md")
WASM = os.environ.get("GRAPHLAW_WASM", os.path.join(REPO, "priv", "graphlaw.wasm"))

# Axes whose cells must be taught in a tutorial or how-to page (`<!-- cap: axis:cell -->`).
TAUGHT = ("op", "duckdb_mode", "step", "dialect", "regime", "lease_ceiling", "lease_reason",
          "receipt_reason", "policy_kind", "refusal_kind", "refusal_code", "bin", "limit", "tool")
ALL_AXES = TAUGHT + ("engine", "authority", "op_example")

KEY_A, KEY_B = "01" * 32, "02" * 32
NT_DATA = ("<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n"
           "<urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n")
DATA = {"text": NT_DATA, "dialect": "ntriples"}
TTL = "@prefix ex: <http://example.org/> . ex:a ex:b ex:c ."
SQL = ('@prefix sql: <http://graphlaw.org/sql#> . @prefix ex: <http://example.org/queries#> .\n'
       'ex:Q a sql:SelectStatement ; sql:statementIndex 1 ; sql:fromTable [ sql:tableName "t" ] ; '
       'sql:selectList ( [ sql:columnName "b" ] ) .')
OTHER_SAMPLES = {"n3": "{?a ?b ?c} => {?a ?b ?c} .", "sparql": "ASK {}", "sql": SQL,
                 "shexc": "<http://e/S> {}", "shexj": '{"type":"Schema"}'}


def universe(reg: Optional[dict] = None) -> Dict[str, List[str]]:
    reg = reg or load_registry()
    return {
        "op": [o["name"] for o in reg["ops"]],
        "duckdb_mode": duckdb_modes(),
        "step": [s["name"] for s in reg["law_steps"]],
        "dialect": [d["name"] for d in reg["rdf_dialects"]] + [d["name"] for d in reg["other_dialects"]],
        "regime": list(reg["regimes"]),
        "lease_ceiling": list(reg["lease_ceilings"]),
        "lease_reason": list(reg["lease_reasons"]),
        "receipt_reason": list(reg["receipt_reasons"]),
        "policy_kind": list(reg["policy_refusal_kinds"]),
        "refusal_kind": list(reg["refusal_kinds"]),
        "refusal_code": [c["code"] for c in reg["refusal_codes"]],
        "limit": sorted(reg["limits"]),
        "engine": list(reg["engines"]),
        "authority": sorted({a["authority"] + " | " + a["capability"] for a in reg["authorities"]}),
        "bin": list(BINARIES),
        "tool": [f"graphlaw_{o['name']}" for o in reg["ops"]] + ["graphlaw_duckdb"],
        "op_example": [e["id"] for e in load_op_examples()],
    }


# ---------------------------------------------------------------------------------------------
# Probe context: one live runner session; every refusal it sees is recorded.

class Ctx:
    def __init__(self, reg: dict):
        self.reg = reg
        self.examples = load_op_examples()
        self.gl = GraphLaw(wasm=WASM)
        self.codes: set = set()
        self.kinds: set = set()

    def close(self) -> None:
        self.gl.close()

    def call(self, req: dict) -> Tuple[bool, Any]:
        """(ok, result_or_error); records refusal code/kind."""
        try:
            return True, self.gl.call(req)
        except GraphLawError as e:
            err = e.error if isinstance(e.error, dict) else {"message": str(e.error)}
            self.kinds.add(err.get("kind"))
            self.codes.add((err.get("details") or {}).get("code"))
            return False, err

    def refuse(self, req: dict, kind: Optional[str] = None, **details: str) -> dict:
        ok, err = self.call(req)
        assert not ok, f"expected refusal, got ok: {str(err)[:200]}"
        if kind:
            assert err.get("kind") == kind, f"kind {err.get('kind')} != {kind}"
        for k, v in details.items():
            assert (err.get("details") or {}).get(k) == v, f"details.{k}={(err.get('details') or {}).get(k)!r} != {v!r}"
        return err

    def ok(self, req: dict) -> Any:
        ok, res = self.call(req)
        assert ok, f"expected ok, got refusal: {str(res)[:300]}"
        return res

    def replay(self, ex: dict) -> None:
        req = dict(ex["request"])
        if ex["outcome"] == "ok":
            self.ok(req)
        else:
            err = self.refuse(req)
            if ex.get("refusal_kind"):
                assert err.get("kind") == ex["refusal_kind"], f"{ex['id']}: kind {err.get('kind')}"
            if ex.get("refusal_code"):
                assert (err.get("details") or {}).get("code") == ex["refusal_code"], f"{ex['id']}: code"


Probe = Callable[[Ctx, str], None]


def p_op(c: Ctx, op: str) -> None:
    exs = [e for e in c.examples if e["op"] == op]
    assert exs, f"no registry example for op {op}"
    assert any(e["outcome"] == "ok" for e in exs), f"no ok example for op {op}"
    for e in exs:
        c.replay(e)
    assert hasattr(GraphLaw, op), f"GraphLaw has no `{op}` method"


def p_op_example(c: Ctx, ex_id: str) -> None:
    c.replay(next(e for e in c.examples if e["id"] == ex_id))


def p_tool(c: Ctx, name: str) -> None:
    spec = next((s for s in tools.tool_specs(c.reg) if s["name"] == name), None)
    assert spec, f"no tool spec {name}"
    assert spec["input_schema"]["type"] == "object" and spec["description"]
    if name == "graphlaw_duckdb":
        out = tools.dispatch(c.gl, name, {"mode": "open", "session": "tool-probe"})
        assert out["ok"], out
        assert tools.dispatch(c.gl, name, {"mode": "close", "session": "tool-probe"})["ok"]
        return
    op = name[len("graphlaw_"):]
    ex = next(e for e in c.examples if e["op"] == op and e["outcome"] == "ok")
    out = tools.dispatch(c.gl, name, {k: v for k, v in ex["request"].items() if k != "op"})
    assert out["ok"], out
    ref = next((e for e in c.examples if e["op"] == op and e["outcome"] == "refused"), None)
    if ref:
        out = tools.dispatch(c.gl, name, {k: v for k, v in ref["request"].items() if k != "op"})
        assert out["ok"] is False and out["error"].get("kind"), "refusal must surface as typed data"


def p_duckdb_mode(c: Ctx, mode: str) -> None:
    s = "probe-" + mode
    if mode == "open":
        c.ok({"op": "duckdb", "mode": "open", "session": s})
        c.refuse({"op": "duckdb", "mode": "open", "session": s}, kind="DuckDb")  # name taken
        c.ok({"op": "duckdb", "mode": "close", "session": s})
    elif mode == "close":
        c.ok({"op": "duckdb", "mode": "close", "session": "never-opened"})  # idempotent
    elif mode in ("exec", "stream"):
        c.refuse({"op": "duckdb", "mode": mode, "session": "never-opened", "query_ast": "x", "ast": {"turtle": "x"},
                  "cursor": "never"}, kind="DuckDb")
        assert hasattr(GraphLaw, f"duckdb_{mode}")
    else:
        raise AssertionError(f"unprobed duckdb mode {mode}")


def p_engine(c: Ctx, eng: str) -> None:
    if eng == "PurRdf":
        assert c.ok({"op": "parse", "text": TTL, "dialect": "turtle"})["quads"] == 1
    elif eng == "FrozenDuckdb":
        c.ok({"op": "duckdb", "mode": "open", "session": "probe-engine"})
        c.ok({"op": "duckdb", "mode": "close", "session": "probe-engine"})
    else:
        raise AssertionError(eng)


def p_authority(c: Ctx, key: str) -> None:
    auth, cap = key.split(" | ", 1)
    caps = c.ok({"op": "capabilities"})
    assert any(a.get("authority") == auth and a.get("capability") == cap for a in caps["authorities"]), \
        "capabilities() does not report this authority"


def p_dialect(c: Ctx, name: str) -> None:
    dials = {d["name"]: d for d in c.reg["rdf_dialects"] + c.reg["other_dialects"]}
    d = dials[name]
    base = c.ok({"op": "parse", "text": TTL, "dialect": "turtle"})
    if d["kind"] == "rdf":
        conv = c.ok({"op": "convert", "text": TTL, "dialect": "turtle", "to": name})["text"]
        back = c.ok({"op": "parse", "text": conv, "dialect": name})
        assert back["id"] == base["id"], f"{name}: convert->parse round-trip changed the graph id"
        for alias in d.get("aliases", []):
            assert c.ok({"op": "parse", "text": conv, "dialect": alias})["id"] == base["id"], f"alias {alias}"
    else:
        r = c.ok({"op": "parse", "text": OTHER_SAMPLES[name], "dialect": name})
        assert r["dialect"] == d["response_name"], r
        for alias in d.get("aliases", []):
            assert c.ok({"op": "parse", "text": OTHER_SAMPLES[name], "dialect": alias})["dialect"] == d["response_name"]


def p_regime(c: Ctx, regime: str) -> None:
    r = c.ok({"op": "entail", "data": DATA, "regime": regime})
    assert r is not None
    c.refuse({"op": "entail", "data": DATA, "regime": regime + "-nope"})


def p_step(c: Ctx, step: str) -> None:
    if step == "sql":
        c.ok({"op": "law", "data": {"text": SQL, "dialect": "sql"}, "steps": [steps.sql()]})
        assert callable(steps.sql)
        return
    if step == "require-signed-receipt":
        rc = c.ok({"op": "law", "data": DATA, "steps": [steps.rdfs()]})["receipts"][0]
        key = attest.pubkey(KEY_A)
        doc = attest.receipt_ntriples(rc, attest.sign_receipt(KEY_A, rc))
        c.ok({"op": "law", "data": {"text": doc, "dialect": "ntriples"},
              "steps": [steps.require_signed_receipt("derive:rdfs", [key])]})
        return
    found = [e for e in c.examples if e["op"] == "law" and e["outcome"] == "ok"
             and any(s["step"] == step for s in e["request"]["steps"])]
    assert found, f"no ok example and no custom probe for step {step}"
    c.replay(found[0])
    fn = getattr(steps, step.replace("-", "_"))
    assert callable(fn), f"steps.{step} missing"


def _lease(ceiling: str, scope: List[str], expires: int = 2 ** 63, issued: int = 0, lid: str = "L") -> dict:
    return {"id": lid, "holder": "h", "ceiling": ceiling, "scope": scope, "expires_unix": expires, "issued_unix": issued}


def _law_leased(c: Ctx, lease: dict, step_list: list, seed: str = KEY_A, trust: Optional[str] = None,
                data: Any = DATA, tamper: bool = False) -> dict:
    sl = attest.sign_lease(seed, lease)
    if tamper:
        sl["lease"]["expires_unix"] = 2 ** 63
    return {"op": "law", "data": data, "signed_lease": sl, "trusted_keys": [trust or attest.pubkey(KEY_A)],
            "steps": step_list}


def p_lease_ceiling(c: Ctx, ceiling: str) -> None:
    allsteps = ["admit:shacl", "admit:plan", "derive:rdfs", "derive:n3"]
    shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> . [] a sh:NodeShape ."
    if ceiling == "observe":
        c.ok(_law_leased(c, _lease("observe", allsteps), [steps.shacl(shapes)]))
        c.refuse(_law_leased(c, _lease("observe", allsteps), [steps.rdfs()]), reason="ceiling")
    elif ceiling == "select":
        pe = next(e for e in c.examples if e["id"] == "law.plan-receipts")["request"]
        plan = next(s for s in pe["steps"] if s["step"] == "plan")
        c.ok(_law_leased(c, _lease("select", allsteps), [plan], data=pe["data"]))
        c.refuse(_law_leased(c, _lease("select", allsteps), [steps.rdfs()]), reason="ceiling")
    else:
        c.ok(_law_leased(c, _lease("construct", allsteps), [steps.rdfs()]))


def p_lease_reason(c: Ctx, reason: str) -> None:
    sc = ["derive:rdfs"]
    rd = [steps.rdfs()]
    reqs = {
        "expired": lambda: _law_leased(c, _lease("construct", sc, expires=0), rd),
        "out_of_scope": lambda: _law_leased(c, _lease("construct", ["derive:n3"]), rd),
        "ceiling": lambda: _law_leased(c, _lease("observe", sc), rd),
        "bad_signature": lambda: _law_leased(c, _lease("construct", sc, expires=100), rd, tamper=True),
        "untrusted_key": lambda: _law_leased(c, _lease("construct", sc), rd, trust=attest.pubkey(KEY_B)),
        "clock_skew": lambda: _law_leased(c, _lease("construct", sc, expires=2 ** 64 - 1, issued=2 ** 64 - 1), rd),
    }
    c.refuse(reqs[reason](), reason=reason, code="LeaseRefused")


def p_receipt_reason(c: Ctx, reason: str) -> None:
    rc = c.ok({"op": "law", "data": DATA, "steps": [steps.rdfs()]})["receipts"][0]
    trust = [attest.pubkey(KEY_A)]
    docs = {
        "unattested": attest.receipt_ntriples(rc),
        "untrusted_key": attest.receipt_ntriples(rc, attest.sign_receipt(KEY_B, rc)),
        "bad_signature": attest.receipt_ntriples(dict(rc, added=rc["added"] + 1), attest.sign_receipt(KEY_A, rc)),
    }
    c.refuse({"op": "law", "data": {"text": docs[reason], "dialect": "ntriples"},
              "steps": [steps.require_signed_receipt("derive:rdfs", trust)]}, reason=reason, code="ReceiptRefused")


_PROB = {"goal": {"facts": ["done"]}, "initial_states": ["s0"], "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
         "transitions": [{"action": "flip", "from": "s0", "probability_ppm": 500000, "to": "g"},
                         {"action": "flip", "from": "s0", "probability_ppm": 500000, "to": "s0"}]}


def p_policy_kind(c: Ctx, kind: str) -> None:
    def pol(outs, state="s0"):
        return {"policy": [{"state": state, "action": "flip", "outcomes": outs}]}
    half = lambda s: {"state": s, "probability_ppm": 500000}
    cases = {
        "MissingEntry": (_PROB, {"policy": []}),
        "InventedOutcome": (_PROB, pol([half("g"), half("zzz")])),
        "BadMass": (_PROB, pol([{"state": "g", "probability_ppm": 400000}, half("s0")])),
        "DeadEnd": (dict(_PROB, transitions=[{"action": "flip", "from": "s0", "probability_ppm": 1000000, "to": "s0"}]),
                    pol([{"state": "s0", "probability_ppm": 1000000}])),
        "Malformed": (_PROB, "not json"),
    }
    problem, policy = cases[kind]
    c.refuse({"op": "policy", "problem": problem, "policy": policy}, kind="EngineRejected",
             code="PolicyRefused", policy_kind=kind)


def _prime(c: Ctx) -> None:
    """Triggers for refusal classes the op-examples do not reach."""
    c.refuse({"op": "sniff", "text": "<g> { <s> <p> <o> }"}, kind="Ambiguous")
    assert c.ok({"op": "sniff", "text": "<g> { <s> <p> <o> }", "hint": "trig"})["dialect"] == "TriG"
    LIMIT_PROBES["n3_max_term_depth"](c)  # details.code == "Refused" with details.kind


def p_refusal_kind(c: Ctx, kind: str) -> None:
    _prime(c)
    assert kind in c.kinds, f"refusal kind {kind} never observed in this gate run"


def p_refusal_code(c: Ctx, code: str) -> None:
    _prime(c)
    assert code in c.codes, f"refusal code {code} never observed in this gate run"


def _limit_probes() -> Dict[str, Callable[[Ctx], None]]:
    def depth(c):
        deep: Any = []
        for _ in range(70):
            deep = [deep]
        c.refuse({"op": "parse", "text": "x", "deep": deep}, kind="ResourceLimit", limit="json_depth")

    def acts(c):
        a = [{"name": f"a{i}", "pre": "", "add": "", "del": ""} for i in range(1001)]
        c.refuse({"op": "law", "data": DATA, "steps": [{"step": "plan", "plan": {"actions": a, "goal": ""}}]},
                 kind="ResourceLimit", limit="plan_actions")

    def atoms(c):
        add = "".join(f"<urn:s> <urn:p> <urn:o{i}> .\n" for i in range(10001))
        c.refuse({"op": "law", "data": DATA, "steps": [{"step": "plan", "plan": {
            "actions": [{"name": "a", "pre": "", "add": add, "del": ""}], "goal": ""}}]},
            kind="ResourceLimit", limit="atoms_per_field")

    def total(c):
        add = "".join(f"<urn:s> <urn:p> <urn:o{i}> .\n" for i in range(10000))
        a = [{"name": f"a{i}", "pre": "", "add": add, "del": ""} for i in range(11)]
        c.refuse({"op": "law", "data": DATA, "steps": [{"step": "plan", "plan": {"actions": a, "goal": ""}}]},
                 kind="ResourceLimit", limit="plan_total_atoms")

    def pol(c):
        entries = [{"state": f"s{i}", "action": "a", "outcomes": []} for i in range(100001)]
        c.refuse({"op": "policy", "problem": _PROB, "policy": {"policy": entries}},
                 kind="ResourceLimit", limit="policy_entries")

    def req_bytes(c):
        proc = subprocess.Popen([os.environ.get("GRAPHLAW_RUN_EH", os.path.join(REPO, "target", "debug", "graphlaw-run-eh")),
                                 ], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                env=dict(os.environ, GRAPHLAW_WASM=WASM))
        proc.stdin.write((16 * 1024 * 1024 + 1).to_bytes(4, "big"))
        proc.stdin.close()
        rc = proc.wait(timeout=60)
        proc.stdout.close(); proc.stderr.close()
        assert rc == 92, f"oversize frame prefix exited {rc}, expected 92"

    def n3_bytes(c):
        rule = '{?a <urn:b> ?c} => {?a <urn:d> "' + "x" * 70000 + '"} .'
        c.refuse({"op": "law", "data": {"text": "<urn:a> <urn:b> <urn:c> .", "dialect": "ntriples"},
                  "steps": [steps.n3(rule)]}, kind="ResourceLimit", limit="n3_term_bytes")

    def n3_depth(c):
        rule = "{?a <urn:b> ?c} => {?a <urn:d> " + "(" * 70 + ")" * 70 + "} ."
        err = c.refuse({"op": "law", "data": {"text": "<urn:a> <urn:b> <urn:c> .", "dialect": "ntriples"},
                        "steps": [steps.n3(rule)]}, kind="EngineRejected")
        assert "nested more than 64 levels" in err["message"], err["message"]

    return {"n3_max_term_bytes": n3_bytes, "n3_max_term_depth": n3_depth, "max_json_depth": depth, "max_plan_actions": acts, "max_atoms_per_field": atoms,
            "max_plan_total_atoms": total, "max_policy_entries": pol, "max_request_bytes": req_bytes}


LIMIT_PROBES = _limit_probes()


def p_limit(c: Ctx, name: str) -> None:
    caps = c.ok({"op": "capabilities"})
    surfaced = caps.get("limits", {})
    assert surfaced.get(name, c.reg["limits"][name]) == c.reg["limits"][name], "capabilities() limit != registry"
    probe = LIMIT_PROBES.get(name)
    assert probe, f"no probe for limit {name}"
    probe(c)


def p_bin(c: Ctx, name: str) -> None:
    if name == "graphlaw-run-eh":
        assert c.ok({"op": "capabilities"})
    elif name == "graphlaw-attest":
        with tempfile.NamedTemporaryFile("wb", suffix=".txt", delete=False) as f:
            f.write(b"payload")
        try:
            key = attest.pubkey(KEY_A)
            ap = f.name + ".att.json"
            attest.sign(KEY_A, f.name, out=ap)
            assert attest.verify(f.name, ap, [key]).ok
            r = attest.verify(f.name, ap, [attest.pubkey(KEY_B)])
            assert not r.ok and r.exit_code == 1
        finally:
            for p in (f.name, f.name + ".att.json"):
                if os.path.exists(p):
                    os.unlink(p)
    elif name == "graphlaw-registry":
        r = run_registry_cli("--check")
        assert r.returncode == 0, f"graphlaw-registry --check exit {r.returncode}: {r.stderr[-300:]}"
    elif name == "graphlaw-verify":
        fx = os.path.join(REPO, "target", "debug", "examples", "blackbox_fixture")
        assert os.path.exists(fx), "build first: cargo build --example blackbox_fixture --features runner-eh"
        with tempfile.TemporaryDirectory() as d:
            out = subprocess.run([fx, d], capture_output=True, text=True, check=True)
            key = out.stdout.strip().splitlines()[-1].strip()
            a = (os.path.join(d, "start.nt"), os.path.join(d, "plan.json"), os.path.join(d, "receipts"))
            assert verify_receipts(*a, trusted_keys=[key]).admitted
            bad = verify_receipts(*a, trusted_keys=[attest.pubkey(KEY_B)])
            assert not bad.admitted and bad.exit_code == 1
    else:
        raise AssertionError(name)


PROBES: Dict[str, Probe] = {
    "op": p_op, "op_example": p_op_example, "tool": p_tool, "duckdb_mode": p_duckdb_mode, "engine": p_engine,
    "authority": p_authority, "dialect": p_dialect, "regime": p_regime, "step": p_step,
    "lease_ceiling": p_lease_ceiling, "lease_reason": p_lease_reason, "receipt_reason": p_receipt_reason,
    "policy_kind": p_policy_kind, "limit": p_limit, "bin": p_bin,
    "refusal_kind": p_refusal_kind, "refusal_code": p_refusal_code,
}
ORDER = [a for a in ALL_AXES if a not in ("refusal_kind", "refusal_code")] + ["refusal_kind", "refusal_code"]


# ---------------------------------------------------------------------------------------------
# Docs

TAG = re.compile(r"<!--\s*cap:\s*([a-z_]+):(.+?)\s*-->")


def doc_tags(root: str = DOC_ROOT) -> Dict[str, List[str]]:
    """cell id -> pages (relative) carrying its tag, from tutorials/ and how-to/ only."""
    found: Dict[str, List[str]] = {}
    for sub in ("tutorials", "how-to"):
        d = os.path.join(root, sub)
        if not os.path.isdir(d):
            continue
        for fn in sorted(os.listdir(d)):
            if fn.endswith(".md"):
                for axis, cell in TAG.findall(open(os.path.join(d, fn), encoding="utf-8").read()):
                    found.setdefault(f"{axis}:{cell}", []).append(f"{sub}/{fn}")
    return found


def render_matrix(uni: Dict[str, List[str]], status: Dict[str, str], tags: Dict[str, List[str]]) -> str:
    out = ["# Capability matrix", "", "GENERATED by `python3 -m graphlaw.coverage --write-matrix` - never hand-edit.",
           "Derived from `registry/capability-registry.json`, the `duckdb` dispatch arms and the cargo bins.", ""]
    for axis in ALL_AXES:
        out += [f"## {axis} ({len(uni[axis])})", "", "| cell | status | taught in |", "|---|---|---|"]
        for cell in uni[axis]:
            pages = ", ".join(tags.get(f"{axis}:{cell}", [])) or ("(matrix only)" if axis not in TAUGHT else "MISSING")
            out.append(f"| `{cell}` | {status.get(f'{axis}:{cell}', 'UNPROBED')} | {pages} |")
        out.append("")
    return "\n".join(out)


# ---------------------------------------------------------------------------------------------
# Gate

def load_manifest(path: str = MANIFEST) -> dict:
    return json.load(open(path, encoding="utf-8")) if os.path.exists(path) else {"gaps": {}}


def run_gate(manifest: Optional[dict] = None, write_matrix: bool = False, only_axes: Optional[List[str]] = None,
             matrix_text: Optional[str] = None, verbose: bool = True,
             reg: Optional[dict] = None, tags: Optional[Dict[str, List[str]]] = None) -> Tuple[int, Dict[str, str], List[str]]:
    reg = reg or load_registry()
    uni = universe(reg)
    man = manifest if manifest is not None else load_manifest()
    gaps: Dict[str, str] = man.get("gaps", {})
    tags = tags if tags is not None else doc_tags()
    defects: List[str] = []
    status: Dict[str, str] = {}
    all_cells = {f"{a}:{c}" for a, cs in uni.items() for c in cs}

    for g, reason in gaps.items():
        if g not in all_cells:
            defects.append(f"STALE-GAP {g}: not a cell of the derived universe")
        if not str(reason).strip():
            defects.append(f"GAP-NO-REASON {g}")
    for t in tags:
        if t not in all_cells:
            defects.append(f"ORPHAN-TAG {t}: doc names a non-existent capability")

    try:
        ctx = Ctx(reg)
    except (OSError, RunnerExit) as e:
        print(f"DEFECT RUNNER-UNAVAILABLE: {e}; build with "
              "`cargo build --bins --example blackbox_fixture --features runner-eh`")
        return 1, {}, ["RUNNER-UNAVAILABLE"]
    try:
        for axis in ORDER:
            if only_axes and axis not in only_axes:
                continue
            for cell in uni[axis]:
                cid = f"{axis}:{cell}"
                probe = PROBES.get(axis)
                err: Optional[str] = None
                try:
                    if probe is None:
                        err = "no probe for axis"
                    else:
                        probe(ctx, cell)
                except Exception as e:  # a crashing probe is a FAIL cell, never a crashed gate
                    err = f"{type(e).__name__}: {str(e)[:200]}"
                if err is None:
                    status[cid] = "PASS"
                    if cid in gaps:
                        defects.append(f"STALE-GAP {cid}: probe now passes; remove the gap")
                elif cid in gaps and str(gaps[cid]).strip():
                    status[cid] = "GAP"
                else:
                    status[cid] = "FAIL" if probe else "MISSING"
                    defects.append(f"{status[cid]} {cid}: {err}")
                if axis in TAUGHT and cid not in tags and status[cid] == "PASS":
                    defects.append(f"UNTAUGHT {cid}: no `<!-- cap: {cid} -->` tag in tutorials/ or how-to/")
    finally:
        ctx.close()

    if not only_axes:
        text = render_matrix(uni, status, tags)
        if write_matrix:
            os.makedirs(os.path.dirname(MATRIX), exist_ok=True)
            open(MATRIX, "w", encoding="utf-8").write(text)
        else:
            cur = open(MATRIX, encoding="utf-8").read() if os.path.exists(MATRIX) else None
            if (matrix_text if matrix_text is not None else cur) != text:
                defects.append("STALE-MATRIX: reference/capability-matrix.md differs from the derived one "
                               "(run --write-matrix)")
    if verbose:
        from collections import Counter
        tot = Counter(status.values())
        print(f"cells={len(status)} " + " ".join(f"{k}={v}" for k, v in sorted(tot.items())))
        for g, r in sorted(gaps.items()):
            print(f"  GAP {g}: {r}")
        for d in defects:
            print("DEFECT", d)
    return (1 if defects else 0), status, defects


def self_check() -> int:
    """Non-vacuity: each mutation of the inputs must flip the gate to exit 1."""
    base_rc, status, _ = run_gate(verbose=False)
    if base_rc != 0:
        print("self-check: baseline gate is not green; fix that first")
        return 2
    man = load_manifest()
    fails = 0
    def expect_red(name: str, **kw) -> None:
        nonlocal fails
        rc, _, defects = run_gate(verbose=False, **kw)
        print(f"  mutation {name}: {'flipped (OK)' if rc == 1 else 'VACUOUS GATE'}"
              + (f" [{defects[0][:80]}]" if defects else ""))
        fails += rc != 1
    expect_red("add stale gap for a non-cell", manifest={"gaps": {**man["gaps"], "op:no_such_op": "x"}})
    expect_red("gap without reason", manifest={"gaps": {**man["gaps"], "op:parse": ""}})
    expect_red("stale gap on a passing cell", manifest={"gaps": {**man["gaps"], "op:parse": "pretend"}})
    expect_red("stale matrix", matrix_text="# not the matrix\n")
    import copy
    phantom = copy.deepcopy(load_registry())
    phantom["ops"].append({**phantom["ops"][0], "name": "phantom_op"})
    expect_red("new registry op with no wrapper/probe", reg=phantom)
    lease_extra = copy.deepcopy(load_registry())
    lease_extra["lease_reasons"].append("phantom_reason")
    expect_red("new lease reason with no probe", reg=lease_extra)
    expect_red("doc tag removed (op:parse untaught)",
               tags={k: v for k, v in doc_tags().items() if k != "op:parse"})
    expect_red("orphan doc tag", tags={**doc_tags(), "op:ghost": ["how-to/x.md"]})
    # drop one gap that exists, if any -> that cell becomes FAIL/MISSING
    if man["gaps"]:
        k = sorted(man["gaps"])[0]
        expect_red(f"remove registered gap {k}", manifest={"gaps": {x: v for x, v in man["gaps"].items() if x != k}})
    print("self-check:", "OK" if fails == 0 else f"{fails} vacuous mutation(s)")
    return 1 if fails else 0


def main(argv: List[str]) -> int:
    if "--self-check" in argv:
        return self_check()
    rc, _, _ = run_gate(write_matrix="--write-matrix" in argv)
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
