# Dual-Artifact Policy — Route C design (L145)

Date: 2026-10-22 · Lane: L145 (design lane; owns this file only, read-only elsewhere) · Repo: `/Users/sac/graphlaw`
Status: **DESIGN — nothing landed; every implementable claim cites a receipt.**
Supersedes the route pick in `eh-runner-decision-memo.md` (L143, amended by the L126 stand-down): Route C is the only short-term route that keeps the wasmi court tier on a real module.

## 0. Receipt base (read before implementing)

| Receipt | What it witnesses |
|---|---|
| L116 `gws4-postlink-receipt.md` | Link succeeds but the module carries a Tag section (id 13, `__cpp_exception`); wasmi 2.0.0 has no exceptions knob (`config.rs:78-95`); tag traces member-level to wasi-sdk `libunwind.a`; gws4 court 1/2 FAIL at validation (`exceptions proposal not enabled`) |
| L121 `smalt-pairing-receipt.md` | Engine artifact (45 MB, `940af4c7…`) exits 95 under wasmi; EH-disabled rebuild `6c0af07f…` fails identically; **pre-duckdb generation (6,655,651 B, `8b4f4bc8…`) serves real packet-4 frames in wasmi** (capabilities abi=1 + stub-mode envelope); smalt court 3/7 (gates 2/2b/5 PASS; 1/1b/1c/3 fail exit 95); no protocol drift (frame grammar, exit 84/86/timeout fault modes match `Smalt.Wasm.Worker`) |
| L109 `wasi-sdk-reproducibility.md` | node 24 (V8) EH-OK probe (`:58-59`); wasmi-unhostable-by-design (`:149-151`); excluding libunwind fails the link with typed undefs (`:144-148`) |
| L115/L116 link-green | ENGINE build recipe witnessed end to end: `WASI_SDK_PATH=<wasi-sdk root> cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` exit 0, 45,161,875 B (`smalt-pairing-receipt.md` build table; `link-green-receipts.md`) |
| L126 stand-down (amendment to L143 memo) | EH is MIXED legacy+new; node 26 (V8) REJECTS the linked artifact ("mix of legacy and new exception handling"); L109's node probe was hello-world only and does not extend; wasmtime 48 needs `legacy_exceptions`; **artifact as built is hostable by NO available runtime** |
| L143 `eh-runner-decision-memo.md` | Route comparison; per-tier `--expect-blake3` pin as the anti-claim-inflation mechanism (already exists — `smalt-pairing-receipt.md:18`, gate 2b exit 84) |
| W9 / L121 Attempt 3 | The 6.79 MB class was witnessed serving real frames — the ARTIFACT-PURE tier's reason to exist |

W9 is the load-bearing positive control: the pure module is not a stand-in — it served real frames. L126 is the load-bearing negative control: the engine artifact as built is unhostable by any staged runtime, so the engine tier is PIN-ONLY until A1' lands — a typed loud state, not a fake pass.

## 1. Artifact taxonomy

Two named artifact generations of `-p graphlaw-wasm`, distinguished by build recipe, not by intent:

| | ARTIFACT-PURE | ARTIFACT-ENGINE |
|---|---|---|
| Recipe | Pre-duckdb generation: pure-Rust kernel, wasmi-hostable. Rebuildable from this tree only if a feature-off knob exists (§6 D1a — open). Immediate pin source: beam4pm's copy (D1b). | `WASI_SDK_PATH=<wasi-sdk root> cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` (L121/L116-witnessed) |
| EH | None — wasmi-valid | Legacy+new mixed (L126); unhostable today |
| Size / digest class | 6.79 MB class: `8b4f4bc8…` 6,655,651 B (W9-witnessed bytes); registry row `332ea218…` 6,793,617 B is the 26.10.21 pin of the same generation, different bytes (L121 finding 2) | 45 MB class: `940af4c7…` (L121) / `500333a4…` (L116) — same recipe, different runs |
| Witnessed serving frames | YES (W9: capabilities + stub-mode wrap over real packet-4) | NO — never witnessed serving any frame on any host |
| Hosts | wasmi (witnessed, W9) | none witnessed; candidates node/wasmtime (L109, L126) |

**Boundary law for PURE:** a pin must point at bytes a documented recipe rebuilds, not at foreign bytes. The beam4pm copy served real frames, but it is beam4pm's tree at its own SHA. If the feature-off knob does not exist, PURE v1 is pinned with `pin-source=beam4pm@<sha>` and flagged `recipe-external` in the pin comment — honest, weaker, immediate. Per the pin-source discipline (`ARTIFACTS.blake3` header, W09): "digest was minted at the 26.10.21 source state" — the pin must state pin-source honestly.

## 2. Pinning

### 2.1 ARTIFACTS.blake3 rows

The single `wasm` row (`332ea218…` 6,793,617 B, the 26.10.21 release pin) is joined by two named rows, same format:

```
wasm-pure   <blake3> <bytes> graphlaw_wasm_pure.wasm
wasm-engine <blake3> <bytes> graphlaw_wasm_engine.wasm
```

Defaults (pending §6): `wasm-pure` pins the W9-witnessed bytes `8b4f4bc8…` (6,655,651 B, `pin-source=beam4pm@<sha>`, flagged recipe-external); `wasm-engine` pins `940af4c7…` (L121) with `status=PIN-ONLY (no EH host witnessed; see A1')` until A1' lands. Header comment law preserved: "wasm row: never re-pinned by graphlaw-registry; re-pinned by the wasm rebuild flow" — both new rows inherit this.

### 2.2 Per-tier contract (smalt GWS-2 coverage law: every artifact cell is pass / registered-divergence / loudly-MISSING — never a silent skip)

| Tier | Artifact | Host | Courts | State if pair unhostable |
|---|---|---|---|---|
| PURE | `wasm-pure` row bytes | wasmi in-process | §2.4 PURE list | — (witnessed working, W9) |
| ENGINE | `wasm-engine` row bytes | EH host (node/wasmtime; D2) | §2.4 ENGINE list | **PIN-ONLY**: court asserts the pin row exists, names the witnessed digest, and emits a loud typed skip on the execution leg. A loud skip is not a pass; smalt gates 1/1b/1c/3 stay honestly red as unhostable-pair refusals |

Every wasm-tier court gains a generation banner: at startup it digests the artifact it just loaded, compares to its tier's pin, and refuses typed on mismatch. Mechanism: the `--expect-blake3` exit-84 path (witnessed L121 gate 2b).

### 2.3 Host selector in tests/common/mod.rs

`tests/common/mod.rs` is currently wasmi-bound (`use wasmi::…`, in-process engine). It gains a selector; additions only — the frozen public API (v26.9.29: `native`, `native_bytes`, `wasm`, `wasm_bytes`, `ok`, `refused`, `read`) is unchanged:

```rust
pub enum WasmHost { Wasmi, Node, Wasmtime }

pub fn wasm_host() -> WasmHost {
    // env GRAPH_LAW_WASM_HOST = wasmi|node|wasmtime, default wasmi.
    // Unknown value → panic typed `GRAPHLAW-WASM-HOST-INVALID`.
    // Read once via OnceLock (the file's existing pattern).
}
```

Compatibility matrix (typed refusals, no fake passes):

- wasmi × ENGINE → typed refuse `GRAPHLAW-WASM-HOST-INCOMPATIBLE` (L116: exit 95 `exceptions proposal not enabled`).
- node × ENGINE-as-built → typed refuse, citing L126: V8 rejects the mixed-EH module. The pair is known-unhostable; refusing it typed is the anti-fake-pass.
- wasmtime × ENGINE → not implemented → loud typed SKIP (a skip is loud, never a pass).
- wasmi × PURE → the default witnessed path (W9).

wasmi stays in-process; node/wasmtime are subprocess hosts speaking packet-4 through `graphlaw-run` (`--wasm <artifact> --expect-blake3 <digest>`), so the digest-banner and exit-84 digest refusal apply on every subprocess leg. ENGINE-tier test files set the env var in their own file scope (selector is OnceLock read-once per process; per-file selection via a new `wasm_bytes_with(host, artifact)` helper, keeping the frozen API intact).

### 2.4 Court → tier binding (the 12 wasm-tier courts)

- **PURE tier (in-process wasmi)**: `wasm_abi`, `wasm_boundary_fuzz`, `wasm_memory_wall`, `wasm_native_differential`, `wasm_n3_admission`, plus the wasmi-eligible subset of `wasm_governance_preflight` (parse/admit/compile legs, no SQL-in-module leg).
- **ENGINE tier (EH host; PIN-ONLY until A1')**: `wasm_duckdb_op`, `wasm_duckdb_op_wired`, `gws4_ipc_injection_court`, `gws12_coldchain_cross_engine`, plus the SQL-in-module legs of `wasm_governance_preflight`.

Trust-domain note: `wasm_abi` / `wasm_native_differential` / `wasm_n3_admission` are trust courts — retiering them is a reviewed-diff-class change, not a silent edit.

## 3. Build / pin / re-sync recipe per artifact

### 3.1 ARTIFACT-ENGINE (witnessed, L121/L116)

```
export WASI_SDK_PATH=<wasi-sdk root: bin/clang + share/cmake/wasi-sdk-p1.cmake — NOT the engine cache, per the L116 header note>
CARGO_TARGET_DIR=<lane root> cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm
b3sum <artifact>  →  re-pin the wasm-engine row (and priv/graphlaw.wasm.blake3) in the same commit as any source change that moves the digest (W09 law)
```

`graphlaw-registry -- --write` does NOT touch the wasm rows (ARTIFACTS.blake3 header law), for both rows.

### 3.2 ARTIFACT-PURE (pending §6 D1)

- D1a: feature-off rebuild from this tree — requires the graphlaw-wasm unit to expose a duckdb-off knob (frozen-duckdb-sys `build.rs` currently fails typed `WASI_SDK_PATH required` without the env — `src/abi_manifest.rs:137`). If the knob is witnessed, PURE becomes locally rebuildable and the recipe mirrors §3.1 minus the env.
- D1b: pin beam4pm's copy `8b4f4bc8…` with pin-source=beam4pm@<sha>, flagged recipe-external.

D1b is the immediate step; D1a is the tightening step.

### 3.3 Court-facing contract

§2.2's table with expected digests per tier (PURE = the D1b pin `8b4f4bc8…`; ENGINE = `940af4c7…` or `500333a4…` — operator picks canonical, §6 D6). Court banner: digest the loaded artifact, assert tier name + digest, mismatch → typed refusal via the exit-84 path.

## 4. Migration path to A1' (single EH-uniform artifact)

A1' = emsdk rebuild of duckdb emitting NEW-style `try_table` uniformly (`-mllvm -wasm-use-legacy-eh=0`), hostable by V8 and modern wasmtime; wasmi still cannot host it. The wasmi court tier needs Route C regardless (L126 amendment).

1. **Witness A1' first** (L143 memo's first step): rebuild one frozen-duckdb blob set with the new-EH flag (`/Users/sac/frozen-duckdb/scripts/build_wasi_duckdb.sh`), link, and witness a REAL module load + one real eval frame on node or modern wasmtime. Witness template: build command + exit code, artifact digest, host load + one real frame through the packet-4 loopback driver (WASM-1 law: `:io.setopts(:standard_io, encoding: :latin1)` before the first frame on any Elixir driver), 3x reproduction (L121 discipline), plus the typed-death clause: still-mixed EH or host rejection → A1' dies loudly, fallback ladder A3 (pin an old legacy-EH runtime) → B (`-fno-exceptions` duckdb rebuild), per the L143 memo.
2. If A1' works: ENGINE v2 (try_table-uniform, V8/wasmtime-hostable) re-pins `wasm-engine`; ENGINE-tier courts flip from PIN-ONLY skip to real host legs; smalt 3/7 → 7/7 (gates 1/1b/1c/3 unblock).
3. **Collapse condition** (single artifact): wasmi gains exceptions support (upstream or fork) or Route B succeeds → the PURE tier's reason to exist disappears. wasmi courts migrate to the single artifact; the `wasm-pure` row retires receipt-gated — removed in the same commit as the court migration commit. The host selector stays: it is host infrastructure, not artifact infrastructure. Policy collapses back to one `wasm` row.

## 5. What does NOT change

- **NATIVE_ONLY_OPS semantics** (`src/registry.rs:896`, `["duckdb"]`): unchanged. The pure module has no duckdb op — witnessed real surface (`{"op":"sql"}` → Unsupported "missing string field `op`", L121 Attempt 3). PURE-tier courts assert that real surface, not a defanged copy of the engine surface. Absence of duckdb from PURE's capabilities is real surface, not a SQL refusal and not REFUSAL-04 (`src/abi_manifest.rs:137`).
- **REFUSAL-04** (`src/tier.rs:25`, `docs/refusals.md:58`): unchanged. It is the document-backed-builtin-eval refusal, host-agnostic; the artifact split is a host/artifact compatibility boundary, not a capability boundary.
- **DuckDB-is-not-a-feature law** (`wasm/src/lib.rs:79`: "not a feature the module turns off"): unchanged. The dual-artifact policy does NOT create a duckdb feature toggle in the module. ARTIFACT-PURE is a build-generation identity, not a runtime toggle — no `cfg` gate, no runtime flag enters `wasm/src/lib.rs`. PURE v1 exists only as pinned bytes (D1b) until/unless a feature-off rebuild knob is witnessed (D1a).
- Host selector is test infrastructure only: no `src/` or `wasm/src` code changes from this design's test-side work.
- graphlaw-run's typed fault modes unchanged (exit 84 digest refusal, crash 86, hang timeout — L121).

## 6. Open operator decisions

- **D1 — PURE v1 pin source**: D1b (pin beam4pm bytes `8b4f4bc8…`, recipe-external, immediate) vs D1a (feature-off rebuild knob in graphlaw-wasm; build-system change, then re-pin). Default if no answer: D1b.
- **D2 — engine-tier EH host pre-A1'**: wasmtime `legacy_exceptions` (failed at default config, L109 `:58-59`) or A3 (pin an old wasmtime/wasmer with legacy-EH; EOL-freeze cost). Default: none — ENGINE tier stays PIN-ONLY until A1' witnesses a host; no un-witnessed host is asserted.
- **D3 — PURE tier digest**: `8b4f4bc8…` (the W9 frame-serving witness) vs `332ea218…` (the 26.10.21 registry pin — same generation, different bytes, never frame-witnessed; L121 finding 2). Default: pin `8b4f4bc8…`; `332ea218…` stays as the 26.10.21 release pin / historical note. Do NOT bind wasmi courts to `332ea218…` without a fresh frame witness against those exact bytes.
- **D6 — ENGINE canonical digest**: `940af4c7…` (L121) vs `500333a4…` (L116) — same recipe, different runs. Default: re-build once at pin time and pin THAT digest; historical digests stay in the receipts.
- **D4 — graphlaw-run `--host` flag** (A1 route work): out of scope for this design; needed for the smalt unblock only after A1' witnesses a host. Default: out of scope.
- **D5 — tier banner mechanism**: per-court banner vs `--expect-blake3` exit-84 reuse. Default: reuse exit-84 (witnessed, L121 gate 2b).

## 7. First implementation steps (ordered, each fail-closed)

1. Split the registry row: add `wasm-pure` (D1b: `8b4f4bc8…`, 6,655,651 B, pin-source=beam4pm@<sha>, recipe-external) and `wasm-engine` rows to `registry/ARTIFACTS.blake3`, engine row carrying `status=PIN-ONLY (no EH host witnessed; see A1')`.
2. Add the host selector to `tests/common/mod.rs` (additions only; OnceLock read-once; typed refusals `GRAPHLAW-WASM-HOST-INVALID` / `GRAPHLAW-WASM-HOST-INCOMPATIBLE`).
3. Retier the 12 courts per §2.4 (7 PURE + 5 ENGINE); trust courts retiering is a reviewed-diff-class change.
4. Witness PURE end to end: wasmi loads the D1b bytes, serves capabilities + one eval frame, banner asserts `8b4f4bc8…`, exit 0. Do not bind wasmi courts to `332ea218…` without a fresh frame witness (D3).
5. Witness ENGINE PIN-ONLY: `cargo test --test wasm_duckdb_op` shows the loud typed skip on the execution leg and exit 0 only via the pin-existence leg; a skip is loud, not a pass.
6. smalt pairing stays honestly 3/7 (four red gates typed as unhostable-pair refusals) until A1' lands.

## See Also

`eh-runner-decision-memo.md` (L143, incl. L126 amendment) · `gws4-postlink-receipt.md` (L116) · `smalt-pairing-receipt.md` (L121) · `wasi-sdk-reproducibility.md` (L109) · `link-green-receipts.md` (L115/L116) · `registry/ARTIFACTS.blake3` header law · `~/.claude/rules/testing-chicago-style.md` (GWS-2 coverage law)