"""Agent tool surface: one JSON-schema tool per capability, generated from the registry.

`tool_specs()` is what a coding agent registers (Anthropic `input_schema` shape; OpenAI-style
`parameters` is the same object). `dispatch(gl, name, args)` executes a tool call on a live
`GraphLaw` session and returns {"ok": True, "result": ...} or {"ok": False, "error": ...}, so
typed refusals reach the agent as data it can act on, never as a Python traceback.
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from .client import GraphLaw, GraphLawError
from .registry import duckdb_modes, load_registry

_SCALAR = {"string": "string", "integer": "integer", "boolean": "boolean", "object": "object"}


def _schema_for(t: str) -> Dict[str, Any]:
    if t in _SCALAR:
        return {"type": _SCALAR[t]}
    if t == "data_spec":
        return {
            "anyOf": [
                {"type": "string", "description": "RDF text; dialect is sniffed"},
                {"type": "object", "properties": {"text": {"type": "string"}, "dialect": {"type": "string"},
                                                  "hint": {"type": "string"}, "base": {"type": "string"}},
                 "required": ["text"]},
            ]
        }
    if t == "json_or_string":
        return {"anyOf": [{"type": "string"}, {"type": "object"}, {"type": "array"}]}
    if t.startswith("list<"):
        return {"type": "array"}
    return {}


def tool_specs(registry: Optional[dict] = None) -> List[Dict[str, Any]]:
    reg = registry or load_registry()
    specs: List[Dict[str, Any]] = []
    for op in reg["ops"]:
        props, req = {}, []
        for f in op["request"]["fields"]:
            props[f["name"]] = {**_schema_for(f["type"]), "description": f.get("doc", "")}
            if f["required"]:
                req.append(f["name"])
        specs.append({
            "name": f"graphlaw_{op['name']}",
            "description": f"GraphLaw ABI op `{op['name']}`. Refusals: {', '.join(op['refusal_kinds'])}.",
            "input_schema": {"type": "object", "properties": props, "required": req, "additionalProperties": False},
        })
    specs.append({
        "name": "graphlaw_duckdb",
        "description": "GraphLaw `duckdb` op. mode: " + " | ".join(duckdb_modes()) +
                       ". Named sessions live only for this runner process.",
        "input_schema": {"type": "object", "properties": {"mode": {"type": "string", "enum": duckdb_modes()}},
                         "required": ["mode"], "additionalProperties": True},
    })
    return specs


def dispatch(gl: GraphLaw, name: str, args: Dict[str, Any]) -> Dict[str, Any]:
    if not name.startswith("graphlaw_"):
        return {"ok": False, "error": {"kind": "UnknownTool", "message": name}}
    op = name[len("graphlaw_"):]
    try:
        return {"ok": True, "result": gl.call({"op": op, **args})}
    except GraphLawError as e:
        return {"ok": False, "error": e.error}
