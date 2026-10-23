# Receipts chain audit — lane L31, 2026-10-22

Read-mostly verification lane over `/Users/sac/graphlaw/receipts/`.
Report file: the only file this lane created.

## Method

- Per-file digests: `b3sum` (blake3 1.8.5, `/opt/homebrew/bin/b3sum`), the
  only BLAKE3 CLI present. `graphlaw-verify` was absent from
  `target/debug/` at audit time (lane-concurrent build churn); it verifies
  plan receipts, not file chains, so it is not the tool for this surface.
- Embedded-output check: read all 6 receipt files in full.
- Chain-linkage check: grepped every receipt for `prev/chain/parent` hash
  fields; grepped `registry/` and `scripts/` for any chain ledger over
  `receipts/`. None exists.

## Verification commands + exits

```
$ b3sum receipts/gws-9-wasm-boundary-fuzz.md receipts/gws-10-e2e-coldchain-wasm.md \
        receipts/gws-16-wasm-duckdb-op15.md receipts/sa2a/factory-b-v26.9.29-executable-gates.md \
        receipts/spc-2026-10-22.json receipts/spc-2026-10-22-inject.json
```
exit 0 (B3SUM_EXIT=0). Digests below, per receipt.

```
$ grep -v "^#" registry/ARTIFACTS.blake3        # exit 0
$ grep -o '"manifest_blake3"[^,]*' contracts/abi-manifest.json   # exit 0
$ grep -o '"registry_blake3"[^,]*' registry/capability-registry.json # exit 0
$ /Users/sac/graphlaw/target/debug/graphlaw-verify --help
  # zsh: no such file or directory (exit 127-class); binary absent at audit time
```

## Per-receipt verdicts

| receipt | blake3 | verdict | basis |
|---|---|---|---|
| `gws-9-wasm-boundary-fuzz.md` | `b48a148e…1d8919` | WITNESSED | real command (`cargo test --test wasm_boundary_fuzz`) + captured test output, pass counts (2/2, 312 hostile calls), two deterministic replays (receipts/gws-9-wasm-boundary-fuzz.md:9-32) |
| `gws-10-e2e-coldchain-wasm.md` | `604df95d…bb3c4` | WITNESSED | real command + captured `3 passed; 0 failed` output, corpus sha256 per-file digests, second deterministic run, typed PENDING limitations (:44-59, :103-108) |
| `gws-16-wasm-duckdb-op15.md` | `cb584cc0…3072c` | WITNESSED (pins stale, as-of-date) | courts + counts embedded (:66-97); but its cited pins `manifest_blake3=2321b6c7…` and `registry_blake3=0debaedb…` no longer match the live tree (`5e5b49c5…` / `7db561aa…`) — later lanes re-pinned; receipt is an as-of-2026-10-21 record, not a live pin. No flip law broken: it claims PARTIAL_ALIVE, not a current digest |
| `sa2a/factory-b-v26.9.29-executable-gates.md` | `d464dad4…d4aa6` | **NO-OUTPUT** | no command, no exit code, no counts, no hashes of anything executed; states `Verification: authored, not executed under Maximum Code Mode` (:9). Not a receipt under the repo law ("a receipt with no embedded output is not a receipt"); at best an evidence-record placeholder |
| `spc-2026-10-22.json` | `4c862608…c3435` | WITNESSED | written by the real writer `scripts/spc_court.sh:147`; embeds measured counts (mean/stddev/cpk/p50-p999/samples), honest FAIL verdict with reasons — a gate that can FAIL is the point |
| `spc-2026-10-22-inject.json` | `164dbcd4…fce4` | WITNESSED | same writer (`spc_court.sh:145`); outlier-injection leg, measured counts, honest FAIL |

Counts: 6 receipts — 5 WITNESSED, 1 NO-OUTPUT, 0 BROKEN.

## Chain continuity

**UNCHAINED — no on-disk linkage to verify.** This is the audit's main
finding. None of the 6 durable receipts carries a parent/prev-hash field;
a grep for `prev/chain/parent` hash fields across `receipts/` returned zero
linkage. The BLAKE3 chain rule
`chain_hash(t+1) = BLAKE3(chain_hash(t) || frame_bytes(t+1))` is implemented
only in the law kernel (`src/receipt_chain.rs:376-418`, genesis at :376,
advance at :414-418) over in-memory OCEL causal frames, and nothing
serializes it into `receipts/`. The `receipts/ — durable receipts
(BLAKE3-chained)` claim in the repo CLAUDE.md layout section is therefore
**unsupported on disk**: per-file digests exist (above, computable at any
time) but no frame names its parent, so there is no chain to break — and no
tamper-evidence between receipts. The b3sum digests above are a valid
genesis for such a chain; adopting it is a coordinator decision (this lane
did not modify any receipt, per lane ownership).

## Notes

- `graphlaw-verify` binary absent from `target/debug/` at audit time
  (concurrent lane build churn); rebuildable via
  `cargo build --bin graphlaw-verify`. Loud skip, not a pass: the absent
  binary is irrelevant to this surface anyway (plan receipts, not file
  chains).
- spc receipts' `verdict: FAIL` is faithful: the SPC gate (Cpk 0.0217 /
  -0.0014 < 1.33) refused; a gate that can exit 1 is the point.
- The sa2a file is pre-existing (Oct 20), not session-introduced; fix is
  either re-issue through a real executed run or annotate it as an
  evidence-record, not a receipt.
