#!/usr/bin/env python3
"""G1 COMPLETENESS logic, parameterized: (repo_root, court_dir).
Classifying court for the anti-vacuity legs — the SAME comparison the main court
runs, pointed at a (possibly corrupted) manifest directory. Exit 1 = gate flips."""
import json, os, sys
root, court_dir = sys.argv[1], sys.argv[2]
manifest = json.load(open(os.path.join(court_dir, "manifest.json")))
real = set()
for t in manifest["trees"]:
    for dp, _, ns in os.walk(os.path.join(root, t)):
        for n in ns:
            if n.endswith(".rs"): real.add(os.path.relpath(os.path.join(dp, n), root))
listed = set(e["path"] for e in manifest["files"])
unclassified = sorted(real - listed)
ghost = sorted(listed - real)
if unclassified or ghost:
    for p in unclassified: print("UNCLASSIFIED", p)
    for p in ghost: print("GHOST", p)
    print(f"G1 FAIL: {len(unclassified)} unclassified, {len(ghost)} ghost")
    sys.exit(1)
print(f"G1 PASS: {len(real)} files classified exactly once")
