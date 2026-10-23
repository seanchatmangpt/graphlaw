#!/usr/bin/env bash
# partition_court.sh — Chicago court over the operator's three-partition code census
# (courts/code-partition/manifest.json). Read-only over src/, crates/graphlaw-eyeron/src/,
# wasm/src/. Writes only its self-check scratch under /tmp. Exits 1 on any violation;
# exits 2 only when court inputs are missing (BLOCKED, never a silent pass).
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
COURT_DIR="$ROOT/courts/code-partition"
MANIFEST="$COURT_DIR/manifest.json"
AXIOMS="$COURT_DIR/axioms.blake3"
LEAKS="$COURT_DIR/admitted-leaks.json"

FAIL=0
note() { printf '%s\n' "$*"; }
die()  { printf 'GATE-FAIL: %s\n' "$*"; FAIL=1; }

note "== code-partition court (subject: $ROOT) =="
for f in "$MANIFEST" "$AXIOMS"; do
  [ -f "$f" ] || { echo "BLOCKED: missing court input $f"; exit 2; }
done

python3 - "$ROOT" <<'PY' || FAIL=1
import json, os, re, subprocess, sys
root = sys.argv[1]
cd = os.path.join(root, "courts/code-partition")
manifest = json.load(open(os.path.join(cd, "manifest.json")))
trees = manifest["trees"]
fails = []
def die(m):
    fails.append(m); print("GATE-FAIL:", m)
def note(m): print(m)

# ---------- G1 COMPLETENESS ----------
note("\n-- G1 COMPLETENESS --")
def real_files():
    out = set()
    for t in trees:
        for dp, _, ns in os.walk(os.path.join(root, t)):
            for n in ns:
                if n.endswith(".rs"):
                    out.add(os.path.relpath(os.path.join(dp, n), root))
    return out
real = real_files()
listed = [e["path"] for e in manifest["files"]]
listed_set = set(listed)
dups = sorted({p for p in listed if listed.count(p) > 1})
if dups: die(f"G1: duplicate manifest entries: {dups}")
unclassified = sorted(real - listed_set)
ghost = sorted(listed_set - real)
if unclassified:
    die(f"G1: {len(unclassified)} UNCLASSIFIED files (state space leaked): {unclassified}")
else:
    note(f"G1 PASS: all {len(real)} real .rs files classified exactly once")
if ghost: die(f"G1: {len(ghost)} manifest entries with no file on disk: {ghost}")

# ---------- G2 AXIOM-IMMUTABILITY ----------
note("\n-- G2 AXIOM-IMMUTABILITY --")
def b3(path):
    r = subprocess.run(["b3sum", path], capture_output=True, text=True)
    if r.returncode != 0:
        print(f"BLOCKED: b3sum failed on {path}: {r.stderr.strip()}"); sys.exit(2)
    return r.stdout.split()[0]
pin = {}
for line in open(os.path.join(cd, "axioms.blake3")):
    line = line.strip()
    if not line: continue
    h, p = line.split(None, 1)
    pin[p.strip()] = h
axm = {e["path"]: e["blake3"] for e in manifest["files"] if e["partition"] == "AXIOMATIC-KERNEL"}
if set(pin) != set(axm):
    die(f"G2: axiom file-set drift: pinned-only={sorted(set(pin)-set(axm))} manifest-only={sorted(set(axm)-set(pin))}")
for p, h in sorted(axm.items()):
    actual = b3(os.path.join(root, p))
    if h != actual:
        die(f"G2: AXIOM DRIFT {p}: manifest={h[:16]}.. actual={actual[:16]}..")
    elif p in pin and pin[p] != actual:
        die(f"G2: PIN DRIFT {p}: pinned={pin[p][:16]}.. actual={actual[:16]}..")
if not any(f.startswith("G2") for f in fails):
    note(f"G2 PASS: {len(axm)} AXIOMATIC-KERNEL digests byte-identical to manifest and pins (b3sum)")
else:
    note("G2: drift recorded above")

# ---------- G3 PROJECTION-PROVENANCE ----------
note("\n-- G3 PROJECTION-PROVENANCE --")
leaks_admitted = {}
if os.path.exists(os.path.join(cd, "admitted-leaks.json")):
    for e in json.load(open(os.path.join(cd, "admitted-leaks.json"))).get("leaks", []):
        leaks_admitted[e["path"]] = e
gen_backed = [e["path"] for e in manifest["files"] if e.get("generator")]
leaks = [e["path"] for e in manifest["files"]
         if e["partition"] == "COMPILED-PROJECTION" and not e.get("generator")
         and e["path"] not in leaks_admitted]
note(f"projection surface: {sum(1 for e in manifest['files'] if e['partition']=='COMPILED-PROJECTION')} files | "
     f"generator-backed: {len(gen_backed)} | admitted leaks: {len(leaks_admitted)}")
if leaks:
    print(f"G3 ARCHITECTURE-LEAK: {len(leaks)} handwritten projection files with no generator "
          f"provenance and no admitted-leak entry — the codegen roadmap:")
    for p in leaks: print("  LEAK", p)
    die(f"G3: {len(leaks)} unadmitted architecture leaks (exit 1 while any unadmitted leak exists)")
else:
    note("G3 PASS: every projection file is generator-backed or admitted")

# ---------- G4 DELTA-PURITY ----------
note("\n-- G4 DELTA-PURITY --")
IO = re.compile(r'^\s*(?:pub(?:\([^)]*\))?\s+)?use\s+(?:std::|::)?(fs|io|net|os::unix::net|process)\b')
EFFECT = re.compile(r'\b(println!|eprintln!|dbg!|std::process::exit)\b')
viol = 0
for e in manifest["files"]:
    if e["partition"] != "DELTA-SLAB": continue
    src = open(os.path.join(root, e["path"]), encoding="utf-8", errors="replace").read()
    hits = [(i, ln.strip()[:90]) for i, ln in enumerate(src.splitlines(), 1)
            if IO.match(ln) or EFFECT.search(ln)]
    if hits and not e.get("io_justification"):
        viol += 1
        print(f"G4-FAIL {e['path']}: {len(hits)} fs/io/net/println-class hits, no io_justification; first: {hits[:3]}")
    elif hits:
        print(f"G4-JUSTIFIED {e['path']}: {len(hits)} io hits covered by manifest io_justification")
if viol:
    die(f"G4: {viol} DELTA-SLAB purity violations (fix or reclassify with justification)")
else:
    note("G4 PASS: all DELTA-SLAB files pure or justification-recorded")

# ---------- G5 RATIO (measured and printed; no faked pass) ----------
note("\n-- G5 RATIO (S_handwritten vs S_derived) --")
S_axiom = sum(e["lines"] for e in manifest["files"] if e["partition"] == "AXIOMATIC-KERNEL")
S_gen = sum(e["lines"] for e in manifest["files"] if e.get("generator"))
S_leak = sum(e["lines"] for e in manifest["files"]
             if e["partition"] == "COMPILED-PROJECTION" and not e.get("generator")
             and e["path"] in leaks_admitted)
S_unadm = sum(e["lines"] for e in manifest["files"]
              if e["partition"] == "COMPILED-PROJECTION" and not e.get("generator")
              and e["path"] not in leaks_admitted)
S_derived = S_gen + S_leak + S_axiom
S_hand = S_unadm
S_delta = sum(e["lines"] for e in manifest["files"] if e["partition"] == "DELTA-SLAB")
total = S_axiom + S_gen + S_leak + S_unadm + S_delta
note(f"S_handwritten (unadmitted projection leaks) = {S_hand}")
note(f"S_derived (axioms {S_axiom} + generated {S_gen} + admitted-leak {S_leak}) = {S_derived}")
note(f"S_delta (leaf algorithms) = {S_delta}")
note(f"total classified = {total}")
note(f"ratio S_handwritten / S_derived = {S_hand}/{S_derived} = {S_hand/S_derived:.4f}")
if total != sum(e["lines"] for e in manifest["files"]):
    die("G5: line accounting mismatch")
note("G5 MEASURED (informational): the operator's law is S_handwritten << S_derived; "
     "the inversion is the program's goal, not today's state — no pass is claimed here.")

# ---------- ANTI-VACUITY SELF-CHECK ----------
note("\n-- ANTI-VACUITY SELF-CHECK (court must flip on corrupted copies) --")
import shutil, tempfile
tmp = tempfile.mkdtemp(prefix="partition-court-selfcheck-")
try:
    # leg 1: drop a file entry from a manifest copy -> G1 logic must fail
    m2 = json.load(open(os.path.join(cd, "manifest.json")))
    victim = m2["files"][len(m2["files"]) // 2]["path"]
    m2["files"] = [e for e in m2["files"] if e["path"] != victim]
    # point the copy's trees at the REAL trees (paths are relative, court copy dir differs)
    copy_dir = os.path.join(tmp, "courts/code-partition")
    os.makedirs(copy_dir)
    json.dump(m2, open(os.path.join(copy_dir, "manifest.json"), "w"))
    shutil.copy(os.path.join(cd, "axioms.blake3"), copy_dir)
    leg1 = subprocess.run([sys.executable, os.path.join(root, "scripts/partition_court_selfcheck_g1.py"),
                           root, copy_dir], capture_output=True, text=True)
    if leg1.returncode == 0 or victim not in leg1.stdout:
        die(f"ANTI-VACUITY leg1 VACUOUS: G1 logic did not flip on dropped entry {victim}: "
            f"rc={leg1.returncode} out={leg1.stdout[-200:]}")
    else:
        note(f"ANTI-VACUITY leg1 FLIPPED: dropped '{victim}' from a /tmp manifest copy -> G1 refused it")

    # leg 2: edit an axiom digest in the pins copy -> G2 logic must fail
    a2 = dict(pin)
    some = next(iter(a2))
    a2[some] = "0" * 64
    copy2 = os.path.join(tmp, "axioms-corrupted.blake3")
    with open(copy2, "w") as fh:
        for p, h in sorted(a2.items()): fh.write(f"{h}  {p}\n")
    leg2 = subprocess.run([sys.executable, os.path.join(root, "scripts/partition_court_selfcheck_g2.py"),
                           root, copy2], capture_output=True, text=True)
    if leg2.returncode == 0 or some not in leg2.stdout:
        die(f"ANTI-VACUITY leg2 VACUOUS: G2 logic did not flip on corrupted digest for {some}: "
            f"rc={leg2.returncode} out={leg2.stdout[-200:]}")
    else:
        note(f"ANTI-VACUITY leg2 FLIPPED: corrupted blake3 pin for '{some}' -> G2 refused it")
finally:
    shutil.rmtree(tmp, ignore_errors=True)

if fails:
    print(f"\nVERDICT: FAIL ({len(fails)} gate violations)")
    for f in fails: print("  -", f)
    sys.exit(1)
print("\nVERDICT: PASS")
PY

exit $FAIL
