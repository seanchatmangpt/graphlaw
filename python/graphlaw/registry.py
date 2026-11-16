"""Capability universe: the machine-readable registry plus the repo's CLI surfaces."""

from __future__ import annotations

import json
import os
import re
import subprocess
from typing import Any, Dict, List, Optional

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
REGISTRY_JSON = os.path.join(REPO, "registry", "capability-registry.json")
OP_EXAMPLES_JSON = os.path.join(REPO, "registry", "op-examples.json")
DUCKDB_OP_SRC = os.path.join(REPO, "src", "duckdb_op.rs")

# Cargo [[bin]] targets that are part of the public operator surface.
BINARIES = ("graphlaw-run-eh", "graphlaw-verify", "graphlaw-attest", "graphlaw-registry")


def load_registry(path: str = REGISTRY_JSON) -> Dict[str, Any]:
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def load_op_examples(path: str = OP_EXAMPLES_JSON) -> List[Dict[str, Any]]:
    with open(path, encoding="utf-8") as f:
        return json.load(f)["examples"]


def duckdb_modes(src: str = DUCKDB_OP_SRC) -> List[str]:
    """Modes of the `duckdb` op (not in the registry: derived from its dispatch arms)."""
    text = open(src, encoding="utf-8").read()
    body = text[text.index("pub fn dispatch"):]
    body = body[: body.index("\n}\n")]
    return re.findall(r'"([a-z_]+)"\s*=>\s*(?:open_session|exec|stream|close)\(', body)


def run_registry_cli(arg: str, binary: Optional[str] = None, timeout: float = 300) -> subprocess.CompletedProcess:
    """`graphlaw-registry <arg>` (`--check`, `--print-json`, `--print-ttl`, `--print-mcp`, `--print-digests`)."""
    binary = binary or os.environ.get(
        "GRAPHLAW_REGISTRY", os.path.join(REPO, "target", "debug", "graphlaw-registry")
    )
    return subprocess.run([binary, arg], capture_output=True, text=True, timeout=timeout)
