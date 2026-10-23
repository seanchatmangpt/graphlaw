#!/usr/bin/env python3
"""G2 AXIOM-IMMUTABILITY logic, parameterized: (repo_root, pins_file).
Compares each pinned axiom digest to the real file (b3sum). Exit 1 = gate flips."""
import os, subprocess, sys
root, pins_file = sys.argv[1], sys.argv[2]
bad = 0
for line in open(pins_file):
    line = line.strip()
    if not line: continue
    h, p = line.split(None, 1); p = p.strip()
    f = os.path.join(root, p)
    if not os.path.exists(f):
        print("G2 FAIL: pinned axiom missing on disk:", p); bad = 1; continue
    r = subprocess.run(["b3sum", f], capture_output=True, text=True)
    if r.returncode != 0:
        print("BLOCKED: b3sum failed:", r.stderr); sys.exit(2)
    actual = r.stdout.split()[0]
    if actual != h:
        print(f"G2 FAIL: AXIOM DRIFT {p}: pinned={h[:16]}.. actual={actual[:16]}.."); bad = 1
if not bad: print("G2 PASS: all pins match")
sys.exit(bad)
