# GL29 — Next-window backlog (v26.11.17)

Status: DRAFT / UNVERIFIED. Drafted 2026-11-17 from GL28-BACKLOG.md, wasm-gap-register.md, docs/jira/v26.10.22/, docs/reference/design-*.md. GL28 ids are referenced, never re-issued; the GL28 register stays authoritative for GL28-n status (as of 2026-11-09, not re-checked). A GL29 row closes only on a landed receipt. Cross-check with GL28-STATUS-REREAD.md: GL28-8 and GL28-11 have drifted (partly landed), so re-scope GL29-4 and GL29-7 first.

## Tickets
| Id | Title | Scope | Falsifier court | Area | Depends |
|---|---|---|---|---|---|
| GL29-1 | Kernel receipt_blake3 mint | `receipt_blake3 = blake3_hex(receipt_payload(..))` at the two `receipts.push` sites in src/abi/ops/governance.rs + thin Elixir envelope assembler (kernel half of GL28-2) | smalt_wire_parity_court: flipping one receipt byte must refuse | kernel (src/abi, attest.rs) | none |
| GL29-2 | Smalt wire-parity + gate-inverted flip | witness parity court and anti-vacuity flip; migrate validator steps 1-5 | smalt_commit_gate_court + smalt_wire_parity_court | smalt, graphlaw_ex | GL29-1 |
| GL29-3 | Smalt gate bypass census follow-up | close direct Store.put bypass (cosmos.ex, blob_storage.ex); explicit exemption tier | direct write with no envelope must typed-refuse | smalt | GL29-2 |
| GL29-4 | Disk receipt chain (GL28-8) — RE-SCOPE FIRST | store.rs already persists; wire per admission and add flipped-byte-from-disk court | disk chain court | kernel | GL29-1 |
| GL29-5 | EU attestation exporter (GL28-4) | build src/attestation_export.rs + tests from spec; check xaas layers first; operator sign-off before transport | forged byte fails verify; missing receipt downgrades to UNVERIFIED | kernel; xaas | GL29-4 (ideal) |
| GL29-6 | ferroplan C01 lease adapter (GL28-3) | (a) glx:CandidatePlan projection, (b) step-dependency edges, (c) lease-ceiling court | c01_plan_lease_court | graphlaw_ex; ferroplan | (c) uses landed GL28-7 |
| GL29-7 | graphlaw_ex residuals (GL28-11) — RE-SCOPE FIRST | dual refusal vocabulary consolidation (rest appears landed); receipt the b/c/d items | flip-twin per unit court; parity test for vocab | graphlaw_ex | none |
| GL29-8 | ash_graphlaw P6/P7/P9 (GL28-10) | re-vendor to current pin and 28-import MANIFEST; full-suite green | parity court + full suite exit 0 | ash_graphlaw | none |
| GL29-9 | Publish path (GL28-12) | operator configures Trusted Publisher (2-day expiry), rerun A3; decide Hex | publish receipt moving registry row off 26.10.14 | operator | BLOCKED-external |
| GL29-10 | Wasm gap closure sweep | G2 egress_arrow op, W08 differential receipt, G8 re-witness, D4 re-run, UU1 full battery, D3 retire/refresh | wasm_native_differential must flip on injected divergence | kernel; wasm/ | none (G2 largest) |
| GL29-11 | Standing-caveat receipts | (a) solo statutory_boundary_50ms p99 (G6); (b) n3_corpus_court exit 0; (c) full host suite 0-fail receipt (1035 passed / 22 failed at 2026-10-26); (d) moonshot-array-v2 triage | (a) p99 sum < 40,000 us; (c) any failure fails the receipt | test infra | none |
| GL29-12 | SQLRDF-25 deferred rows (1-2 read) | star modifiers and COLUMNS('regex') with append-only REFUSAL-01 arms; rows 3-5 unread (note: star_modifier_court.rs, columns_projection_court.rs, pivot_court.rs now exist — re-check ledger) | star_modifier_court with anti-vacuity twin | kernel (sql_ast.rs) | none |
| GL29-13 | Repo hygiene and migration close-out | per MIGRATION-VERIFICATION.md: main pushed; resolve wave1-3-negation naming; Mac-side branch/fuzz-dir cleanup | git status clean; CHAIN.blake3 verifies | repo | do first |

## Added from this window's re-reads
| Id | Title | Source |
|---|---|---|
| GL29-14 | Evidence-ring sleep-park to Condvar; annotate untagged test sleeps; typed deny for receipt_chain.rs:614 and law.rs:544/604 | RISK-SWEEP R1-R4 |
| GL29-15 | Advisory-ignore renewal (3 expire 2026-11-23, 3 on 2026-11-28); purrdf pin vs graphlaw-carried head | DEPENDENCY-STANDING |
| GL29-16 | Doc staleness cleanup incl. CONTRIBUTING docs_release_claims vapor court, retired graphlaw_run refs, doc/witness/count gates into CI | DOC-STALENESS, CI-SCRIPT-MAP |
| GL29-17 | Refusal register: arm-namespace table, REFUSAL-05 in playbook, REFUSAL-04 behavioural test | REFUSAL-AUDIT |
| GL29-18 | Ontology dedupe (28M duplicate DOID, schema.org copies) after consumer grep | ONTOLOGY-GGEN-STANDING |
| GL29-19 | N3 audit C1-C3, C5, C6 closure; remove SKV-REVERT-TEST-B marker at law.rs:458 | N3-RULES-STANDING |

## Suggested order
1. GL29-13 and GL29-11 first so receipts land on a known-clean tree; GL29-15 is time-boxed (expiries).
2. GL29-1, then GL29-2 and GL29-4 in parallel; GL29-3 after GL29-2; GL29-5 after GL29-4.
3. Parallel-ungated: GL29-6, 7, 8, 10, 12, 14, 16-19. GL29-9 waits on the operator.

## Carried, not ticketed
GL28-1, 5, 6, 7, 9, 13, 14 closed/landed (but see GL28-STATUS-REREAD for drift on 5 and unverified receipts on 1, 9). ERRC ledger small items ash_a2a.1, purrdf.1. IoT slimming `-Oz MinSizeRel` lever deferred. Claim discipline: "0 DPMO" not claimable; 118 ns SPC covers the receipt-fold leg only; frozen-duckdb exactness is rule-set dependent; kill-switch, authority ceiling and gate courts are presentation checks, the engine verifies signatures. New work takes the next free GL29-n id.
