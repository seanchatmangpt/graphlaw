# VISION-2028 Execution Plan — Stub

Baseline: graphlaw v26.10.28 (wasm blake3 c38b1fdb…, fork 2223f894…).

**This file is a stub.** Its dependency DAG and court specifications are
folded into [GL28-BACKLOG.md](GL28-BACKLOG.md) — the single authoritative
GL28 register (DAG, court-specification table, wave-3 wire contracts, and
per-ticket statuses). This file remains on disk only because it hosts the
lane PRIOR-ART-FIRST contract below, which lanes cite by path.

The superseded GL28-13 slimming sketch that lived here (feature-gate
census, lto/codegen-units/wasm-opt steps) is replaced by the authoritative
census: [GL28-13-SLIMMING-CENSUS.md](GL28-13-SLIMMING-CENSUS.md) — the
74,097,211 B artifact's growth is DuckDB archive payload, and levers 1+2
(ICU drop −3 MB + MinSizeRel wasm32 rebuild −10–18 MB) restore the 64 MiB
gate.

## Lane-contract addition (2026-10-29): PRIOR-ART FIRST

Before any lane designs, drafts or implements, it searches the local
canonical checkouts for prior art — ggen-marketplace packs (the
blake3-digest-keys branch @ 041a039e is the working patched reference),
ggen itself (sync stages, FM-WRITE-005/FM-PACK-008/013 semantics), and the
sibling checkouts (graphlaw_ex facade, ash_graphlaw authority/telemetry/
claims templates, ash_a2a ingress, ferroplan UniversalPlan, smalt PUT
chain, wasm4pm OCEL consumer). Every deliverable opens with "Prior art
consumed" naming exact files+lines and stating why the artifact differs
from or delegates to what exists. A design that re-derives a sibling
checkout's content is a REFUSED deliverable. graphlaw/src stays unread
while the mutation court holds it solo.

Prior-art map additions (2026-10-29): /Users/sac/beam4pm (process-mining
substrate — GL28-1's OCEL 2.0 consumer side may already exist there; search
before building the wasm4pm stream) and /Users/sac/xaas (the portfolio
control plane — GL28-2/GL28-4 egress consumers; check its receipt/attestation
layers before drafting the EU exporter transport).
