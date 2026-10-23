#!/usr/bin/env python3
"""PreToolUse guard: disk headroom before suite/bench-class runs.

ENOSPC killed all-lane waves five times this session; one purge of stale
lane build roots freed ~180 GiB. Law (CLAUDE.md Disk discipline): check
free space before full suites/benches.

Mechanical rule: a Bash command that runs suite/bench-class cargo work
(`cargo test` without --test, `cargo bench`, `cargo run --release`,
`cargo build --release`, `--profile wasm`, mutation/spc courts) is refused
while / has < 20 GiB free, with a typed message. Single-target court runs
(`cargo test --test <t>`, `cargo check`) pass through — they are the cheap
ladder rungs.
"""
import json
import re
import shutil
import sys

HEAVY = re.compile(
    r"cargo\s+(?:test(?!\s+--test\b)|bench\b|run\b[^|;&]*--release"
    r"|build\b[^|;&]*--release)|--profile\s+wasm"
    r"|mutation_court\.sh|spc_court\.sh",
    re.M,
)
MIN_FREE_GIB = 20


def main() -> None:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        sys.exit(0)

    if payload.get("tool_name") != "Bash":
        sys.exit(0)
    cmd = payload.get("tool_input", {}).get("command", "") or ""
    if not HEAVY.search(cmd):
        sys.exit(0)

    free = shutil.disk_usage("/").free
    free_gib = free / (1024 ** 3)
    if free_gib < MIN_FREE_GIB:
        reason = (
            f"DISK-REFUSED({free_gib:.1f} GiB free < {MIN_FREE_GIB} GiB): "
            "suite/bench-class run refused — ENOSPC has killed all-lane waves. "
            "Purge stale lane build roots (target-lane*) or report DISK-BLOCKED "
            "to the coordinator. A df -h / receipt is required in the report."
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
