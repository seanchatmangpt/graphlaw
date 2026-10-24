# Compliance & Transparency Map — v26.10.23

Status: mapping lane, read-only over code (no cargo/git run).
Scope: public-facing docs, artifact transparency, claims audit trail,
user-facing error transparency. Verified against the tree on disk
(2026-10-23/24).

## 1. Public-facing docs completeness

| Duty | Artifact | Evidence (disk) |
|---|---|---|
| README | `README.md` (195 lines) | present, repo root |
| Whitepaper with receipted actuals | `docs/whitepaper-v26.10.22.md` | self-declares "grounded in witnessed receipts"; cites `CHAIN.blake3` integrity VERIFIED at exit-0 `receipt_chain_sync.sh --check` (2026-10-23), re-baseline note `receipts/2026-10-23-chain-rebaseline-note.md`; witness receipts `receipts/2026-10-23-banking-4of4-witness.md` |
| DIALECTS with refusal arms | `docs/DIALECTS.md` (403 lines) | refusal arms cited with stable codes and `src/sql_ast/refusals.rs:119-144` pointers (arms 48-52: 48 VALIDATION, 51 COLUMNS_LAMBDA, 52 COLUMNS_PATTERN); REFUSAL-01 family cross-ref `docs/refusals.md` |
| How-to suite | `docs/how-to/` — 11 guides incl. `release-publish.md`, `verify-without-trusting-the-operator.md`, `run-the-wasm-module.md`, `use-from-javascript.md` | present |
| Release-publish runbook | `docs/how-to/release-publish.md` + `js/npm/PUBLISHING.md` (CI publish with `--provenance`, package sync steps) | present |
| CHANGELOG | `CHANGELOG.md` (698 lines) | present, repo root |
| Design/ADR corpus | `docs/jira/v26.10.23/` — 4 ADRs (crate-splitting, egress-op, release-lease, wasmi-retirement), 6 design notes, ROADMAP | present |

## 2. Artifact transparency

| Duty | Artifact | Evidence (disk) |
|---|---|---|
| BLAKE3 pins everywhere | `SECURITY.md` pins `priv/graphlaw.wasm` by b3sum + size (58,121,500 B) with a `wasm-objdump` import inventory: 28 imports, all `wasi_snapshot_preview1`, fail-closed import allowlist, no signing op in the wasm ABI | SECURITY.md:51-73 |
| npm package contents reviewed clean | `js/npm/package.json` uses an explicit `files` allowlist; `PUBLISHING.md` mandates validated CI publish, not ad hoc local publish | present |
| Registry digest stability | `registry/ARTIFACTS.blake3`, `capability-registry.json/.ttl/.schema.json` | present, generated (never hand-patched) |
| Receipt chain | `receipts/CHAIN.blake3` fold verified in-wave (whitepaper cites exit-0 `--check`) | present |

## 3. Claims audit trail

- Whitepaper claims are receipt-bound ("grounded in witnessed receipts",
  per-claim witness receipts named in-text). Adversarial audit/correction
  trail lives in the milestone window: `docs/jira/v26.10.22/INDEX.md`
  (s10 skeptic adversarial verification of INDEX rows; k5/k6 hostile-input
  adversarial courts), `docs/jira/v26.10.23/ROADMAP.md` carries the
  doc_staleness truth-sync.
- `register_completeness` gate is real and enforced as a Rust court:
  `tests/sql_rdf_conformance.rs:854`
  `register_completeness_every_refusal_arm_is_enumerated` — every refusal
  arm must be enumerated in the register; the catalog cannot silently lag
  the code.

## 4. User-facing error transparency

- Typed refusal register: `docs/refusals.md` (append-only, stable codes).
- Playbook: `docs/refusal-playbook.md` — 13/13 probes MATCH, exit 0
  (2026-10-22 probe run, verbatim embedded output).
- Catalog: `docs/reference/refusal-catalog.md` (20 REFUSAL-code mentions),
  generated-projected from the register; completeness enforced by the
  `register_completeness` court above.
- DIALECTS.md maps each failure arm to a typed refusal with a source
  pointer; the wasm surface never traps (DuckDb op refusal arm documented
  as never-trap, SQLRDF-17 raw-SQL refusal preserved in wasm).

## 5. Honest gaps (3)

1. **docs_staleness R5 anchor drift, 1,143 pre-existing** — cited from
   `docs/jira/v26.10.23/README... ROADMAP.md:22`
   ("doc_staleness carries 1,143 pre-existing R5 anchors"); R2/R3/R4 stay
   clean, R5 drift is pre-existing and disclosed, not a regression
   introduced this window.
2. **`docs/index.md` casing/portability** — hub file is lowercase
   `docs/index.md` while the milestone register convention is uppercase
   `INDEX.md`; on case-sensitive filesystems external links written as
   `Index.md`/`INDEX.md` break. No redirects exist.
3. **Whitepaper adversarial-audit provenance is in the milestone window,
   not the whitepaper itself** — the whitepaper cites witness receipts but
   does not name its own adversarial audit pass/corrections; the audit
   trail (skeptic corrections, hostile courts) is discoverable only via
   `docs/jira/v26.10.22/INDEX.md`. A reader of the whitepaper alone cannot
   see the audit lineage.

## Bottom line

4 duty areas mapped, all primary artifacts present with receipted actuals;
3 honest gaps registered (R5 anchor drift 1,143 pre-existing, index.md
casing/portability, whitepaper self-audit provenance). No fabricated
receipts: every count/pointer above was re-read from disk this session.
