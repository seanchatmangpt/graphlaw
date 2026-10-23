#!/usr/bin/env python3
"""PreToolUse guard: the root manifest is coordinator-owned (witnessed 3x).

Concurrent lanes editing /Users/sac/graphlaw/Cargo.toml reverted each other's
manifest state three times in one session (optional flag, features, license
line). The coordinator owns manifest transitions, like git.

Mechanical rule: Edit/Write/MultiEdit/NotebookEdit on the ROOT Cargo.toml are
refused. The coordinator applies manifest transitions via Bash (scripted edit)
— that path is not intercepted, which is the deliberate escape hatch: lanes
use file tools, the coordinator owns transitions.

deny.toml is included (same contention class, CI licensing surface).
"""
import json
import os
import sys

GUARDED = {"Cargo.toml", "deny.toml"}


def main() -> None:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        sys.exit(0)  # unreadable input: fail open

    if payload.get("tool_name") not in (
        "Edit", "Write", "MultiEdit", "NotebookEdit"
    ):
        sys.exit(0)

    fp = payload.get("tool_input", {}).get("file_path", "") or ""
    if os.path.basename(fp) in GUARDED and "/.claude/" not in fp:
        reason = (
            f"MANIFEST-REFUSED(coordinator-owned): {fp} is a coordinator-owned "
            "file — concurrent manifest edits revert each other (witnessed 3x, "
            "v26.10.22). Lanes: put the manifest change in your report. "
            "Coordinator: apply the transition via Bash (scripted edit), which "
            "owns git and serializes manifest state."
        )
        print(json.dumps({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
            }
        }))
    sys.exit(0)


if __name__ == "__main__":
    main()
