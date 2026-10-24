# Re-pin staging gap — wasi-sdk-34 vs the emsdk-built staging (L122 prerequisite)

Lane L164, 2026-10-22. Read-only lane + this one file. Repo: `/Users/sac/graphlaw`.
Status: **GAP DOCUMENTED** — nothing re-pinned, nothing rebuilt; every claim carries a
receipt citation or live probe output embedded below. Every probe in §1/§4 was re-run by
this lane at use time against the real archives.

## 0. Scope

L122's artifact re-pin (gonogo P2, `integration-gonogo.md:42`) is gated
(`post-commit-runbook.md` Step 6: "coordinator MUST NOT re-pin early", `docs/WASM.md:88-100`
pin-currency law). This doc states the EXACT staging gap that blocks a wasi-sdk-34-based
rebuild, the decision tree L122 inherits, and the exact commands each branch runs.
Feeds: `dual-artifact-policy-design.md` (L145), `a1prime-trytable-recipe.md` (L144),
`eh-runner-decision-memo.md` (L143 + the L126 amendment in-file).

## 1. The gap, stated exactly

### 1.1 The personality-ABI mismatch (M4's finding, re-witnessed live this lane)

M4's finding, as relayed in the L164 dispatch (M4's own lane receipt is UNFILED —
`RECEIPTS.md` Wave-M: "M4 — OUT"; falsifier: its receipt landing in `designs/`):
SDK 34's eh libc++abi exports `__gxx_personality_wasm0` while the cached libduckdb.a
references legacy `__gxx_wasm_personality_v0` + `_Unwind_GetWasmLPadContext`.

Re-executed this lane against the real archives (emsdk llvm-nm,
`/Users/sac/emsdk/upstream/bin/llvm-nm`; full commands in §4):

- Cached `libduckdb.a` (`~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/libduckdb.a`,
  87,620,792 B, the L109-staged archive @ duckdb `b6013f7aa2`):
  - `U __gxx_wasm_personality_v0` — **41 undefined**
  - `U _Unwind_GetWasmLPadContext` — **41 undefined**
  - `U __gxx_personality_wasm0` — **0**
- wasi-sdk-34.0 (`/Users/sac/.local/wasi-sdk/wasi-sdk-34.0-arm64-macos`),
  `share/wasi-sysroot/lib/wasm32-wasip1/eh/libc++abi.a`:
  - `T __gxx_personality_wasm0` (defined)
  - `U _Unwind_RaiseException` (delegates to its eh/libunwind.a)
  - `__gxx_wasm_personality_v0` and `_Unwind_GetWasmLPadContext` — **absent
    defined-or-undefined from the entire SDK-34 eh stack** (eh/libunwind.a defines
    `_Unwind_RaiseException`, `U __gxx_personality_wasm0`; LPadContext appears nowhere)
- wasi-sdk-34 non-eh `libc++abi.a`: defines NEITHER personality — the EH personality
  lives only under `eh/`.

Reading: the cached libduckdb.a was compiled by emsdk clang 24.0.0git with
`-fwasm-exceptions` in its DEFAULT (legacy-EH) mode, so it imports the **legacy unwinder
ABI** (`__gxx_wasm_personality_v0` + `_Unwind_GetWasmLPadContext`). wasi-sdk-34's EH
runtime implements the **new-EH (exnref) personality ABI** (`__gxx_personality_wasm0`),
a different symbol contract. Consequences:

1. **No mixed staging closes the link.** Cached libduckdb.a + SDK-34 eh runtime leaves
   82 typed undefined symbols (41 `__gxx_wasm_personality_v0` + 41
   `_Unwind_GetWasmLPadContext`). The duckdb archives must be rebuilt by the SAME
   toolchain generation whose EH runtime is staged beside them.
2. **The symbol mismatch and the hostability wall are one defect, not two.** The same
   legacy-EH compile mode that mints those legacy symbols also mints legacy
   `try`/`delegate`/`rethrow` opcodes. Fixing the staging to close the link (legacy
   runtime) reproduces the Tag-section artifact that no runtime hosts (L126 stand-down:
   wasmi 2.0.0 has no EH support at all; node 26/V8 rejects the mixed legacy+new module;
   wasmtime 48 removed legacy-exceptions — `STAGING-NOTES.md:14-35` of the staging root,
   echoed in `eh-runner-decision-memo.md` amendment). A staging that closes the link is
   therefore, by construction, wasmi-unhostable.

### 1.2 L109's staging works but is not wasmi-hostable — verified, and bounded

L109's script (`/Users/sac/frozen-duckdb/scripts/build_wasi_duckdb.sh`) stages the
emsdk `-wasmexcept` runtime (libc++/libc++abi/libunwind, sha-pinned;
`wasi-sdk-reproducibility.md:25-50`, verify-only 19/19 PASS, exit 0). That staging IS
linkable — L109 clean-room attempt 7 built end-to-end exit 0
(`wasi-sdk-reproducibility.md:104-118`) — but:

- The tag enters from DuckDB's own `-fwasm-exceptions` EH code, NOT only the libunwind:
  L126's stand-down (`STAGING-NOTES.md:3-35`): even with the tag-free source-rebuilt
  libunwind, the artifact walk still shows `section id=13 (TAG) size=3` present; wasm2wat
  shows real `try`/`catch`/`rethrow` bodies, tag `(param i32)`.
- Therefore **NO staging fixes wasmi-hostability** (L126, per the dispatch): the staging
  question is only WHICH EH-capable host the pinned artifact targets. L109's node 24
  EH-OK probe was hello-world only and does not extend to the full artifact (mixed
  legacy+new EH — node 26 rejects it, `STAGING-NOTES.md:20-21`).
- Host census (all receipted): wasmi ≤ 2.0.0 — no EH support at any released version,
  issue #1137 open since 2024-07 with no PR (`wasmi-eh-research.md`); node 26/V8 —
  rejects mixed EH; wasmtime 48 — `legacy_exceptions feature required`, flag removed in
  48 (`wasi-sdk-reproducibility.md:58-59`, L126). Only the pre-duckdb PURE generation
  hosts in wasmi (W9, `smalt-pairing-receipt.md:74-87`).

**The staging gap, one line**: the only staging that closes the link for the cached
archives targets hosts that refuse the result; the only hostable target (try_table/new-EH)
requires rebuilding the archives with the SAME flag change, and no wasi-sdk-34 staging
can consume the emsdk-legacy-EH-built cached archives.

## 2. Decision tree for the re-pin (L122 inherits this)

Per the L143 memo (as amended by L126) and the L145 defaults (§6, `dual-artifact-policy-design.md:128-133`):

```
Q: which artifact do the wasm-engine pin rows point at, and under what status?

Branch 1 — pin the emsdk-built (legacy-EH) artifact as ENGINE-pin-only
           (L145 D2 default: "none — ENGINE tier stays PIN-ONLY until A1'
           witnesses a host")
  state: honest PIN-ONLY, execution legs are loud typed skips; smalt stays 3/7
  falsifier if chosen wrongly: a court silently passes an execution leg
  → FAIL (a loud skip is not a pass)
  commands: §3.1

Branch 2 — wait for A1' (L144 try_table rebuild) and pin THAT artifact
           (wasmtime-hostable per L144: wasmtime 48 ran the try_table probe
           exit 0, legacy probe exit 1; node 24.19.0 exit 0)
  state: ENGINE tier flips PIN-ONLY → real host legs; smalt 3/7 → 7/7 target
  falsifier: still-mixed EH or host rejection at the §3.2 acceptance gate
  → A1' dies loudly, fallback ladder A3 (old legacy-EH runtime pin) → B
    (-fno-exceptions duckdb rebuild), per the L143 memo
  commands: §3.2

PREF     — regardless of branch, the re-pin commit MUST NOT pair the cached
           emsdk-legacy archives with a wasi-sdk-34 runtime staging: that link
           cannot close (82 typed undefs, §1.1), and any patch-around would be
           a fake pass.
```

Interaction with L145's operator decisions: D2 default governs Branch 1;
Branch 2 is A1' landing per the L144 recipe; D6 default ("re-build once at pin time and
pin THAT digest") applies in both branches. D1/D3 (PURE tier) are unchanged and
independent of the staging gap.

## 3. Exact commands per branch

### 3.1 Branch 1 — ENGINE-pin-only (D2 default)

```bash
# 1. Build the engine artifact on the emsdk staging (L115/L116-witnessed recipe,
#    dual-artifact-policy-design.md:92-93). NOTE: this recipe is PIN-ONLY-eligible
#    precisely because its output is EH-mixed and hostless (L126); the re-pin is
#    a typed loud state, never a hostability claim.
export WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1
CARGO_TARGET_DIR=target-lane164 cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm

# 2. Digest and re-pin (D6 default: pin the just-built bytes)
b3sum target-lane164/wasm32-wasip1/wasm/graphlaw_wasm.wasm
# → re-pin the wasm-engine row in registry/ARTIFACTS.blake3 +
#   priv/graphlaw.wasm.blake3, status=PIN-ONLY (no EH host witnessed; see A1'),
#   in the same commit as any source change that moved the digest.
#   graphlaw-registry -- --write does NOT touch wasm rows (L145 §3.1 law).

# 3. Witness the PIN-ONLY state loudly (L145 §7 step 5)
cargo test -p graphlaw --test wasm_duckdb_op   # pin-existence leg exit 0; execution leg = loud typed skip
cargo test -p graphlaw --test wasm_duckdb_op_wired
```

### 3.2 Branch 2 — A1' try_table rebuild (L144 recipe), then pin

```bash
# 1. Rebuild the duckdb archives emitting new-EH uniformly
#    (a1prime-trytable-recipe.md §2.1 diff to
#    /Users/sac/frozen-duckdb/scripts/build_wasi_duckdb.sh, build_archives()):
#      -DCMAKE_CXX_FLAGS="-O3 -fwasm-exceptions -mllvm -wasm-use-legacy-eh=0"
#      -DCMAKE_C_FLAGS="-O2 -mllvm -wasm-use-legacy-eh=0"
#    stage_runtime() UNCHANGED (the -wasmexcept staged runtime is already new-EH,
#    L144 §2.1). R2b carried: builder lane must llvm-ar x each staged .a and
#    census members per L144 §3.1.
#    The builder lane must ALSO re-verify §1.1's symbol census: after the A1'
#    rebuild, libduckdb.a must show U __gxx_personality_wasm0 (new ABI) and ZERO
#    U __gxx_wasm_personality_v0 / U _Unwind_GetWasmLPadContext — the symbol
#    census is the earliest, cheapest falsifier that the EH-mode flip actually
#    landed in the archives.
# 2. Link the engine artifact through the rebuilt staging
export WASI_SDK_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1
CARGO_TARGET_DIR=target-lane164 cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm

# 3. Opcode census gate (L144 §3.1; any legacy opcode = FAIL)
for f in <each rebuilt archive member> target-lane164/wasm32-wasip1/wasm/graphlaw_wasm.wasm; do
  llvm-objdump -d "$f" | grep -cE '\btry\b|delegate|rethrow'   # expect 0
  wasm-tools print "$f" | grep -cE 'try_table'                 # expect >0 where EH used
done

# 4. Symbol census gate (this doc §1.1, A1'-flipped expectation)
NM=/Users/sac/emsdk/upstream/bin/llvm-nm
$NM <staging>/libduckdb.a | grep -c "U __gxx_personality_wasm0"        # expect >0
$NM <staging>/libduckdb.a | grep -c "U __gxx_wasm_personality_v0"      # expect 0
$NM <staging>/libduckdb.a | grep -c "U _Unwind_GetWasmLPadContext"     # expect 0

# 5. Host witness (L144 §1: both hosts ran the try_table probe exit 0)
node <emsdk node 24.19.0> <WASI run of the artifact probe>    # exit 0
wasmtime run target-lane164/wasm32-wasip1/wasm/graphlaw_wasm.wasm <args>  # exit 0

# 6. Acceptance gate (L144 §3.3), then digest + re-pin as in §3.1 step 2
cargo test -p graphlaw --test gws4_ipc_injection_court
cargo test -p graphlaw-wasm
b3sum target-lane164/wasm32-wasip1/wasm/graphlaw_wasm.wasm    # pin these bytes (D6)
```

## 4. Probe receipts (this lane, live runs 2026-10-22)

Tool: `/Users/sac/emsdk/upstream/bin/llvm-nm` (existence `test -x` OK).

```
# cached libduckdb.a (staging root, 87,620,792 B)
$NM ~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/libduckdb.a | grep -c 'U __gxx_wasm_personality_v0$'   → 41
$NM ~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/libduckdb.a | grep -c 'U _Unwind_GetWasmLPadContext$' → 41
$NM ~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/libduckdb.a | grep -c 'U __gxx_personality_wasm0$'    → 0
(total nm symbol lines: 155,128 — probe not vacuous)

# wasi-sdk-34.0-arm64-macos eh/libc++abi.a
T __gxx_personality_wasm0   (defined)
U _Unwind_RaiseException
# wasi-sdk-34 eh/libunwind.a
T _Unwind_RaiseException (defined), U __gxx_personality_wasm0
# _Unwind_GetWasmLPadContext: absent defined-or-undefined in both SDK-34 archives
# wasi-sdk-34 non-eh libc++abi.a: no personality symbol at all

# /opt/wasi-sdk: ABSENT on this host (ls: No such file or directory) — the CI pin
# (`INDEX.md:158`: CI installs wasi-sdk 34 at /opt/wasi-sdk) has no local instance
# at that path; the local SDK-34 lives at /Users/sac/.local/wasi-sdk/wasi-sdk-34.0-arm64-macos.
```

Probes this lane did NOT run (typed): any duckdb rebuild, any cargo rung, any link.
Lane charter is read-only + this file; the §3.1/§3.2 command sets are for the branch
owner lane.

## 5. Receipt base (claims cited)

- M4 finding: relayed via L164 dispatch; M4 lane receipt UNFILED (`RECEIPTS.md` Wave-M,
  "M4 — OUT"); symbol halves re-witnessed live this lane (§4).
- L109: `docs/jira/v26.10.22/designs/wasi-sdk-reproducibility.md` (verify-only 19/19
  `:25-50`; clean-room attempt 7 `:104-118`; node-24 EH-OK bounded hello-world `:58-59`;
  wasmtime legacy refusal `:58-59`; wasmexcept-libunwind REQUIRED `:144-148`).
- L126: staging `STAGING-NOTES.md` (`~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/STAGING-NOTES.md`,
  `:3-35` stand-down: TAG persists without libunwind's tag, node 26 mixed-EH rejection,
  wasmtime 48 legacy removal, "hostable by NO available runtime"); echoed as the
  amendment in `eh-runner-decision-memo.md:76-100`.
- L143: `docs/jira/v26.10.22/designs/eh-runner-decision-memo.md` (routes, falsifiers).
- L144: `docs/jira/v26.10.22/designs/a1prime-trytable-recipe.md` (flag set §2, gates §3,
  risk register §4; wasmtime-48 hostable try_table witness §1).
- L145: `docs/jira/v26.10.22/designs/dual-artifact-policy-design.md` (D1–D6 defaults
  `:128-133`, ENGINE PIN-ONLY contract §2.2, re-pin recipe §3.1, collapse condition §4).
- Gating law: `docs/jira/v26.10.22/designs/post-commit-runbook.md:137-166` (Step 6 GATED);
  `integration-gonogo.md:42` (P2); `docs/WASM.md:88-100` (pin currency).
- Host census: `docs/jira/v26.10.22/designs/wasmi-eh-research.md` (wasmi has no EH at any
  released version; wasmtime 48 new-EH default-ON; local W6/L144 witnesses).

## See Also

`dual-artifact-policy-design.md` · `a1prime-trytable-recipe.md` ·
`eh-runner-decision-memo.md` · `wasi-sdk-reproducibility.md` · `wasmi-eh-research.md` ·
`post-commit-runbook.md` Step 6 · `integration-gonogo.md` P2 · `docs/WASM.md` pin-currency
law · `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/STAGING-NOTES.md` (L126)
