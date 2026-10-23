#!/usr/bin/env bash
# MACHINE-SHAPED GATE (K1 side-arm, lane M09)
#
# Defect class: tests hardcoding `/Users/<user>/...` absolute paths so that
# "suite green" is machine-shaped — green on the author's box, loud on every
# other checkout. This gate fails closed on any `/Users/` literal in tests/ or
# src/ unless the line is:
#   (a) a comment line (Rust `//`, `///`, `//!`, block `/*`, `*`; shell/TTL `#`),
#   (b) carry a `MACHINE-PINNED: <reason>` annotation on the same line, or
#   (c) carry `MACHINE-PINNED: <reason>` on the immediately preceding line.
#
# Usage:
#   bash scripts/machine_shaped_gate.sh [dirs...]   # default: tests src
#   bash scripts/machine_shaped_gate.sh --self-test # anti-vacuity court
set -u

SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"

scan() {
    local root="$1" bad=0
    [ -d "$root" ] || return 0
    # One awk pass per root: comment lines are exempt; all other /Users/
    # literals require a MACHINE-PINNED annotation (same or previous line).
    awk '
        function iscomment(x) { return x ~ /^[ \t]*(\/\/\/|\/\/!|\/\/|\/\*|\*|#)/ }
        function pinned(x)    { return x ~ /^[ \t]*(\/\/\/|\/\/!|\/\/|#)[ \t]*MACHINE-PINNED:[ \t]*[^ ]/ }
        /\/Users\// {
            line = $0
            sub(/^[ \t]+/, "", line)
            if (iscomment(line)) next                     # comment line (a)
            if (pinned(line)) next                        # same-line comment annotation (b)
            if (prevpinned) { prevpinned = 0; next }      # prev-line annotation (c), consumed once
            printf "%s:%d: unallowlisted /Users/ literal: %s\n", FILENAME, FNR, line
            bad = 1
        }
        { prevpinned = pinned($0) }
        END { exit bad ? 1 : 0 }
    ' $(find "$root" -type f \
        \( -name '*.rs' -o -name '*.sh' -o -name '*.py' -o -name '*.ttl' \) \
        ! -path '*/target*' ! -path '*/target-lane*' ! -path '*/.git/*')
}

self_test() {
    # Anti-vacuity: a gate that cannot exit 1 is the failure. Plant a literal
    # in a scratch dir, require the gate to flag it, then require the
    # allowlist annotation and the comment exemption to both suppress it.
    local tmp
    tmp="$(mktemp -d)"      # one scratch dir per check: a finding from an
                            # earlier check must not leak into a later one
    mkdir -p "$tmp/violation"
    printf 'const P: &str = "/Users/sac/.duckdb/extensions";\n' > "$tmp/violation/violation.rs"
    if bash "$SELF" "$tmp/violation" >/dev/null 2>&1; then
        echo "SELF-TEST FAIL: gate accepted a planted /Users/ literal" >&2
        rm -rf "$tmp"; return 1
    fi
    mkdir -p "$tmp/allowlisted"
    printf '// MACHINE-PINNED: self-test allowlist\nconst P: &str = "/Users/sac/.duckdb/extensions";\n' > "$tmp/allowlisted/allowlisted.rs"
    if ! bash "$SELF" "$tmp/allowlisted" >/dev/null 2>&1; then
        echo "SELF-TEST FAIL: gate flagged a MACHINE-PINNED-allowlisted line" >&2
        rm -rf "$tmp"; return 1
    fi
    mkdir -p "$tmp/comment"
    printf '// /Users/ in a doc comment is exempt\n' > "$tmp/comment/comment.rs"
    if ! bash "$SELF" "$tmp/comment" >/dev/null 2>&1; then
        echo "SELF-TEST FAIL: gate flagged a comment line" >&2
        rm -rf "$tmp"; return 1
    fi
    # (d) prev-line annotation binds to ONE following line only
    mkdir -p "$tmp/leak"
    printf '// MACHINE-PINNED: one reason\nconst A: &str = "/Users/x/y";\nconst B: &str = "/Users/x/y";\n' > "$tmp/leak/leak.rs"
    if bash "$SELF" "$tmp/leak" >/dev/null 2>&1; then
        echo "SELF-TEST FAIL: one prev-line MACHINE-PINNED covered two literals" >&2
        rm -rf "$tmp"; return 1
    fi
    # (e) annotation inside a string literal is not a comment - must be flagged
    mkdir -p "$tmp/substring"
    printf 'const P: &str = "/Users/x/y MACHINE-PINNED: fake";\n' > "$tmp/substring/substring.rs"
    if bash "$SELF" "$tmp/substring" >/dev/null 2>&1; then
        echo "SELF-TEST FAIL: substring MACHINE-PINNED inside a string literal was accepted" >&2
        rm -rf "$tmp"; return 1
    fi
    # (f) two consecutive literals, each with its own prev-line annotation: both exempt
    mkdir -p "$tmp/twopins"
    printf '// MACHINE-PINNED: r1\nconst A: &str = "/Users/x/y";\n// MACHINE-PINNED: r2\nconst B: &str = "/Users/x/y";\n' > "$tmp/twopins/twopins.rs"
    if ! bash "$SELF" "$tmp/twopins" >/dev/null 2>&1; then
        echo "SELF-TEST FAIL: paired prev-line annotations should both exempt" >&2
        rm -rf "$tmp"; return 1
    fi

    echo "SELF-TEST PASS: planted literal flagged; allowlist + comment exemptions hold"
    rm -rf "$tmp"
}

if [ "${1:-}" = "--self-test" ]; then
    self_test
    exit $?
fi

dirs=("$@")
[ ${#dirs[@]} -eq 0 ] && dirs=(tests src)

fail=0
for d in "${dirs[@]}"; do
    scan "$d" || fail=1
done

if [ "$fail" -ne 0 ]; then
    echo "MACHINE-SHAPED GATE: FAIL — machine-pinned absolute paths above." >&2
    echo "Fix: derive from \$HOME/std::env::temp_dir(), or allowlist with '// MACHINE-PINNED: <reason>'." >&2
    exit 1
fi
echo "MACHINE-SHAPED GATE: PASS"
