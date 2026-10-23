# V-L18 — independent registry regen post-switchover (2026-10-22)

Coordinator-filed from lane V-L18's inline report (lane reported counts,
not a file).

Before: --check DRIFT on 2 rows (registry expected 153f04f6… live vs
pinned 4cd56fd5…; receipt-surface expected e0215f61… vs pinned
0cc02ca4…; json/ttl ok). --write → exit 0 (wrote json/ttl, repinned
ARTIFACTS.blake3). After: --check exit 0, all five rows ok.
registry_artifacts → 22 passed / 0 failed (the L147 4-drift set flipped
green). Wasm row byte-identical:
`wasm 332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c 6793617 graphlaw_wasm.wasm`.
The L172 6-entry FILES patch was NOT applied (operator-gated, staged).
Files touched: registry/ARTIFACTS.blake3, registry/capability-registry.json,
registry/capability-registry.ttl — regen through the bin, no hand edits.
