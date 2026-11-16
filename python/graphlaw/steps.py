"""Builders for every `law` step (registry `law_steps`)."""

from __future__ import annotations

from typing import Any, Dict, Sequence

from .client import Spec, _spec


def shacl(shapes: str) -> Dict[str, Any]:
    return {"step": "shacl", "shapes": shapes}


def n3(rules: str) -> Dict[str, Any]:
    return {"step": "n3", "rules": rules}


def rdfs() -> Dict[str, Any]:
    return {"step": "rdfs"}


def owl_rl() -> Dict[str, Any]:
    return {"step": "owl-rl"}


def sql() -> Dict[str, Any]:
    return {"step": "sql"}


def hooks(pack: Spec) -> Dict[str, Any]:
    return {"step": "hooks", "pack": _spec(pack)}


def plan(actions: Sequence[Dict[str, Any]], goal: str, goal_not: str = "") -> Dict[str, Any]:
    """Actions: {name, pre, pre_not?, add, del}; N-Triples strings."""
    p: Dict[str, Any] = {"actions": list(actions), "goal": goal}
    if goal_not:
        p["goal_not"] = goal_not
    return {"step": "plan", "plan": p}


def record_receipts() -> Dict[str, Any]:
    return {"step": "record-receipts"}


def require_receipt(step_name: str) -> Dict[str, Any]:
    return {"step": "require-receipt", "step_name": step_name}


def require_signed_receipt(step_name: str, trusted_keys: Sequence[str]) -> Dict[str, Any]:
    return {"step": "require-signed-receipt", "step_name": step_name, "trusted_keys": list(trusted_keys)}
