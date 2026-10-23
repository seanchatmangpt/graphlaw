#!/usr/bin/env bash
# doc_staleness_gate.sh — CI-level staleness court for prose (L35).
# Design: docs/jira/v26.10.22/designs/doc-staleness-gate-design.md
# Exit 0 = pass, 1 = findings, 2 = internal error. Loud, never silent.
set -u
cd "$(dirname "$0")/.." || exit 2
exec python3 - "$@" <<'PYEOF'
import os, re, sys

findings = []
def fail(rule, loc, msg):
    findings.append(f"FINDING {rule} {loc}: {msg}")

# ---------- Rule 1: fenced shell commands in README.md + CLAUDE.md ----------
HEAD_OK = {"cargo","git","bash","sh","just","test","ls","mkdir","cp","rm","mv",
           "python3","grep","jq","npm","npx","node"}
def check_fenced(path):
    if not os.path.exists(path):
        fail("R1", path, "file missing"); return
    lines = open(path, encoding="utf-8").read().splitlines()
    inblock = False; lang = ""
    cont = {}
    for i, line in enumerate(lines, 1):
        if line.startswith("```"):
            if not inblock:
                lang = line[3:].strip()
                inblock = lang in ("sh","bash","console")
            else:
                inblock = False
            continue
        if not inblock: continue
        cmd = re.sub(r"#.*$", "", line).strip()
        if not cmd: continue
        if cmd.endswith("\\"):
            # continuation: merge with next physical line (README.md:146-147)
            cont[i] = True
            continue
        if cont.get(i-1):
            cont[i-1] = False
            continue  # continuation tail of a previous command
        toks = cmd.split()
        head = toks[0].split("/")[-1] if toks[0].startswith("./") else toks[0]
        if head not in HEAD_OK and not toks[0].startswith(("./","scripts/","bash scripts/")):
            fail("R1", f"{path}:{i}", f"command head '{toks[0]}' not resolvable (not a known program or repo-relative path)")
            continue
        # path-like token resolution
        for j, t in enumerate(toks):
            nxt = toks[j+1] if j+1 < len(toks) else ""
            if t == "--test" and nxt and not os.path.exists(f"tests/{nxt}.rs"):
                fail("R1", f"{path}:{i}", f"--test '{nxt}' has no tests/{nxt}.rs")
            if t == "--example" and nxt and not os.path.exists(f"examples/{nxt}.rs"):
                fail("R1", f"{path}:{i}", f"--example '{nxt}' has no examples/{nxt}.rs")
            if t == "--bin" and nxt and not (os.path.exists(f"src/bin/{nxt}.rs") or os.path.exists(f"crates/*/src/bin/{nxt}.rs")):
                fail("R1", f"{path}:{i}", f"--bin '{nxt}' has no src/bin/{nxt}.rs")
            if t.startswith(("./","scripts/")) and t.endswith(".sh") and not os.path.exists(t):
                fail("R1", f"{path}:{i}", f"script token '{t}' does not exist on disk")

for p in ("README.md","CLAUDE.md"):
    check_fenced(p)

# ---------- Rule 2: 64-hex digests in docs/ must be registered or historical ----------
reg = ""
if os.path.exists("registry/ARTIFACTS.blake3"):
    reg = open("registry/ARTIFACTS.blake3", encoding="utf-8").read()
# (L157 §c-2/§c-3) admitted digest sets: Cargo.lock package checksums and the
# receipts/CHAIN.blake3 chain heads — both are real on-disk authorities.
reg += open("Cargo.lock", encoding="utf-8", errors="replace").read() if os.path.exists("Cargo.lock") else ""
reg += open("receipts/CHAIN.blake3", encoding="utf-8", errors="replace").read() if os.path.exists("receipts/CHAIN.blake3") else ""
for root, dirs, files in os.walk("docs"):
    dirs[:] = [d for d in dirs if d not in ("reference-generated",)]
    for f in files:
        if not f.endswith(".md"): continue
        p = os.path.join(root, f)
        lines = open(p, encoding="utf-8").read().splitlines()
        for i, line in enumerate(lines, 1):
            for d in re.findall(r"\b[0-9a-f]{64}\b", line):
                if d in reg: continue
                # historical marker may wrap to the adjacent line inside a
                # parenthetical/list item (witnessed: wasm-integration.md:163-164)
                prev = lines[i-2] if i >= 2 else ""
                if re.search(r"historical", line + " " + prev, re.I): continue
                # On-disk-backed receipt: a digest is admitted if the line cites
                # a file that exists and hashes to it (b3sum for blake3 receipts,
                # shasum -a 256 for corpus receipts). Never admit unbacked hex.
                import subprocess as _sp
                ok = False
                m = re.search(r"([A-Za-z0-9_./-]+\.(?:ttl|trig|nq|json|wasm|md|blake3))", line)
                cands = []
                if m:
                    for base in (os.path.dirname(p), "."):
                        cands.append(os.path.join(base, m.group(1)))
                for path in cands:
                    if not os.path.exists(path): continue
                    for cmd in (["b3sum", path], ["shasum", "-a", "256", path]):
                        try:
                            out = _sp.run(cmd, capture_output=True, text=True, timeout=30).stdout
                            if out.lower().startswith(d.lower()):
                                ok = True; break
                        except Exception: pass
                    if ok: break
                if ok: continue
                fail("R2", f"{p}:{i}", f"digest {d[:12]}… not in registry/ARTIFACTS.blake3, not on-disk-backed, and not marked historical")

# ---------- Rule 3: banned stale strings with dated allowlist ----------
BANNED = ["--features duckdb", "native-only", "capability_intake", "graphlaw_duckdb.wasm"]
# (L157 §c-1) per-file DATED allowlist for quoting-corpus docs: these files quote
# banned strings as evidence (audits/triage/plan bodies), they do not use them.
# Value = ISO expiry date; after that date the entry silently stops applying and
# the findings reappear — the allowlist rots loudly, not silently.
R3_ALLOW = {
    "docs/jira/v26.10.22/designs/stale-claims-audit.md": "2026-12-31",
    "docs/jira/v26.10.22/designs/staleness-76-triage.md": "2026-12-31",
    "docs/jira/v26.10.22/designs/registry-regen-receipt.md": "2026-12-31",
    "docs/jira/v26.10.22/designs/gate-rule-changes-receipt.md": "2026-12-31",
}
import datetime as _dt
_TODAY = _dt.date.today().isoformat()
SCAN = ["README.md","CLAUDE.md","docs/","ggen/","scripts/"]
SELF = os.path.abspath(__file__ if False else "scripts/doc_staleness_gate.sh")
for base in SCAN:
    if os.path.isfile(base):
        it = [(base, None, [base])]
    else:
        it = [(os.path.join(r, f), None, None)
              for r, ds, fs in os.walk(base)
              for f in fs if f.endswith((".md",".sh",".ttl",".rs",".toml"))]
    for p, _, _ in it:
        if os.path.abspath(p) == SELF: continue  # gate self-lists BANNED; scanning itself is vacuous self-fire
        if p == "docs/jira/v26.10.22/designs/doc-staleness-gate-design.md": continue  # the spec must state BANNED to spec them
        try:
            fh = open(p, encoding="utf-8", errors="replace")
        except IsADirectoryError:
            continue
        _allow_until = R3_ALLOW.get(p)
        _allow_active = bool(_allow_until) and _TODAY <= _allow_until
        for i, line in enumerate(fh, 1):
            for s in BANNED:
                if s in line and not re.search(r"historical", line, re.I):
                    if _allow_active: continue  # dated quoting-corpus allowlist (L157 §c-1)
                    fail("R3", f"{p}:{i}", f"banned stale string '{s}' without historical marker")

# ---------- Rule 4: receipt-cited claims need an existing receipts/ pointer ----------
RECEIPT_CLAIM = re.compile(r"\d+ passed|\d+ failed|\b[0-9a-f]{64}\b")
for root, dirs, files in os.walk("docs/jira"):
    for f in files:
        if f != "RECEIPTS.md": continue
        p = os.path.join(root, f)
        # (L157 §c-4) a receipts/ pointer anywhere in the enclosing receipt
        # block (contiguous non-blank lines) admits every claim line in it.
        _lines = open(p, encoding="utf-8").read().splitlines()
        _blocks = []  # (start_idx0, end_idx0_inclusive, has_pointer)
        _start = 0
        for _k, _ln in enumerate(_lines + [""]):
            if _ln.strip() == "":
                _has = any(re.search(r"receipts/", _lines[_j]) for _j in range(_start, _k))
                _blocks.append((_start, _k - 1, _has))
                _start = _k + 1
        for i, line in enumerate(_lines, 1):
            if RECEIPT_CLAIM.search(line) and not re.search(r"receipts/", line):
                _in_block = next((_has for (_a, _b, _has) in _blocks if _a <= i - 1 <= _b), False)
                if _in_block: continue
                fail("R4", f"{p}:{i}", "count/digest claim without a receipts/ pointer (design: doc-staleness-gate-design.md §4)")

print(f"doc_staleness_gate: {len(findings)} finding(s)")
for f in findings: print(f)
sys.exit(1 if findings else 0)
PYEOF
