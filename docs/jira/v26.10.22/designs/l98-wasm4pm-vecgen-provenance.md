# L98 — vecgen provenance stamping (2026-10-22)

scripts/gen_wasm4pm_graphlaw_vectors.py: removed hardcoded
GRAPHLAW_REV/GENERATOR consts; graphlaw_rev()/graphlaw_version() now
stamp at run time (mirrors src/bin/dl_dualroute.rs:303). 6
situation_calculus/strips fixtures regenerated in wasm4pm — parsed-JSON
SEMANTIC_IDENTICAL, provenance-only diff (rev 82c67c2→615dbfa, version
26.10.13→26.10.21). Gate 3/4 passed; the failing leg
(naive-physics-no-fall.json missing vector_id) is a pre-existing legacy
schema family → routed to L106 (schema-v1 emit fix).
