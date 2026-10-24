#!/usr/bin/env python3
"""Extract and canonicalize GraphLaw's observable contract for ggen-legacy reconstitution."""
from __future__ import annotations

import json
import os
import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

def load_abi_manifest() -> dict:
    manifest_path = REPO_ROOT / "contracts" / "abi-manifest.json"
    with open(manifest_path, "r", encoding="utf-8") as f:
        return json.load(f)

def extract_refusal_inventory() -> dict:
    refusals_md = REPO_ROOT / "docs" / "refusals.md"
    refusals = {}
    if refusals_md.exists():
        with open(refusals_md, "r", encoding="utf-8") as f:
            for line in f:
                # Matches rows like: | `REFUSAL-01` | `UnboundedMutationRefusal` | ...
                m = re.match(r"\|\s*`?(REFUSAL-\d+)`?\s*\|\s*`?([^`|]+)`?\s*\|\s*([^|]+)\|", line)
                if m:
                    code, name, desc = m.groups()
                    refusals[code.strip()] = {
                        "name": name.strip(),
                        "description": desc.strip(),
                    }

    # Include explicit negative control E1 / REFUSAL-06 identified in audit
    refusals["REFUSAL-06"] = {
        "name": "SQLCommentBypassAttempt",
        "description": "SQL statement attempting to bypass ATTACH/COPY allowlist via SQL comments (/* */ or --)",
    }
    return refusals

def extract_contract() -> dict:
    abi = load_abi_manifest()
    refusals = extract_refusal_inventory()

    predecessor_b3 = "472d5557ba8311c0fab68bed044be9c44254e65fd523a0ff00684c0869bf80ea"
    predecessor_size = 58121500

    contract = {
        "schema": "urn:chatman:graphlaw:observable-contract:v26.9.28",
        "predecessor": {
            "artifact_blake3": predecessor_b3,
            "artifact_size_bytes": predecessor_size,
            "version": abi.get("graphlaw_version", "26.10.22"),
            "wasm_abi": abi.get("abi_version", 1),
        },
        "abi_manifest": abi,
        "refusals": refusals,
        "invariants": [
            "H_query = BLAKE3(Canon(G_AST) || P_params)",
            "receipt_v2 domain separation prefix: b'GLAW_RECEIPT_V2'",
            "non-tabular query columnar null digest: CANON_ARROW_EMPTY_V1 (BLAKE3(b'CANON_ARROW_EMPTY_V1'))",
            "frame size ceiling MAX_REQUEST_BYTES: 16777216 bytes",
            "frame length exceeding ceiling terminates with exit code 92 without body allocation",
            "zero host credential exposure: WASI context runs with empty environment (no inherit_env)",
            "unbounded UPDATE/DELETE without WHERE rejected with REFUSAL-01",
            "SQL comments preceding ATTACH rejected with REFUSAL-06",
            "deterministic 64-byte alignment with zero-byte padding for all Arrow IPC buffers",
            "CAS epoch monotonicity on all DDL/DML state mutations",
        ],
        "domain_constants": {
            "CANON_ARROW_EMPTY_V1": "db2da6fc6b00ab4ba83884e9eb0ebaa5e20dae37f295fa90cb9e24bcf44f7b0c",
            "MAX_REQUEST_BYTES": 16777216,
            "MAX_OUTSTANDING_ALLOC_BYTES": 16777216,
            "WALL_BYTES": 67108864,
            "FRAME_OVERSIZE_EXIT_CODE": 92,
        }
    }
    return contract

def main():
    contract = extract_contract()
    
    # Target 1: Local repo authority directory
    local_dir = REPO_ROOT / "authority" / "v26.9.28"
    local_dir.mkdir(parents=True, exist_ok=True)
    local_path = local_dir / "graphlaw-observable-contract.json"
    
    # Target 2: ggen-legacy authority directory if present
    ggen_legacy_dir = Path("/Users/sac/ggen-legacy/authority/v26.9.28")
    ggen_legacy_dir.mkdir(parents=True, exist_ok=True)
    ggen_legacy_path = ggen_legacy_dir / "graphlaw-observable-contract.json"

    # Canonical sorting with 2-space indentation and trailing newline
    formatted = json.dumps(contract, sort_keys=True, indent=2) + "\n"
    
    with open(local_path, "w", encoding="utf-8") as f:
        f.write(formatted)
    print(f"Extracted contract saved to: {local_path}")

    if ggen_legacy_dir.exists():
        with open(ggen_legacy_path, "w", encoding="utf-8") as f:
            f.write(formatted)
        print(f"Extracted contract synchronized to: {ggen_legacy_path}")

if __name__ == "__main__":
    main()
