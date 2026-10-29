#!/usr/bin/env python3
"""PreToolUse guard: refuse hand-edits of generated, pinned, and staging paths.

File tools (Edit/Write/MultiEdit/NotebookEdit) exit 2 with a typed
LITHO-REFUSED before the tool runs. Bash is the coordinator hatch, same as
the manifest guard. Axiom .rs sources are not frozen: the partition manifest
still names deleted *_split paths and is not a live allow-list.

Unreadable stdin fails closed. The other guards stay fail-open.
"""
import json
import sys

EDIT_TOOLS = {"Edit", "Write", "MultiEdit", "NotebookEdit"}

REGISTRY = {
    "registry/ARTIFACTS.blake3",
    "registry/capability-registry.json",
    "registry/capability-registry.ttl",
    "registry/mcp-manifest.json",
}
PINS = {
    "courts/code-partition/axioms.blake3",
    "courts/code-partition/axioms.blake3.next",
    "courts/code-partition/manifest.json",
}


def block(reason: str) -> None:
    print(json.dumps({"decision": "block", "reason": reason}))
    sys.exit(2)


def repo_rel(path: str) -> str:
    path = path.replace("\\", "/").strip()
    marker = "/graphlaw/"
    if marker in path:
        path = path.split(marker, 1)[1]
    return path.lstrip("./")


def classify(path):
    rel = repo_rel(path)
    if not rel:
        return None
    if rel.startswith("docs/reference-generated/") or rel == "docs/reference-generated":
        return "LITHO-REFUSED(generated)"
    if rel in REGISTRY:
        return "LITHO-REFUSED(registry-artifact)"
    if rel in PINS:
        return "LITHO-REFUSED(pin-file)"
    if "_split/" in f"/{rel}/":
        return "LITHO-REFUSED(staging)"
    return None


def main() -> None:
    raw = sys.stdin.read()
    try:
        payload = json.loads(raw) if raw.strip() else None
    except json.JSONDecodeError:
        payload = None
    if not isinstance(payload, dict):
        block(
            "LITHO-REFUSED(unreadable): PreToolUse stdin was not a JSON object. "
            "The edit did not run."
        )

    if payload.get("tool_name") not in EDIT_TOOLS:
        sys.exit(0)

    tool_input = payload.get("tool_input") or {}
    path = tool_input.get("file_path") or tool_input.get("notebook_path") or ""
    code = classify(path)
    if code is None:
        sys.exit(0)
    block(
        f"{code}: {path} is not a hand-edit surface. "
        "Regenerate the artifact, or have the coordinator write a pin file via Bash. "
        "Do not rephrase this tool call."
    )


if __name__ == "__main__":
    main()
