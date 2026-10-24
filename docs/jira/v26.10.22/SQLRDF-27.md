# SQLRDF-27 — tc08 COPY-egress EINVAL: musl↔wasi-libc fcntl ABI mismatch

- **Id**: SQLRDF-27
- **Title**: tc08 COPY egress fails with `Cannot open file ...: Invalid argument` —
  libduckdb.a compiled against musl-layout fcntl constants resolves `open`/`openat`
  to wasi-libc, which rejects the mismatched flags word with EINVAL
- **Status**: **Closed** — both close conditions met on the canonical
  relink-8 artifact `4d9083af…eb67b`: FIBO 4/0 incl. tc08 COPY egress
  (receipt body) and lifecycle 3/0
  (`court_a`/`court_b`/`court_c` ok, `test result: ok. 3 passed; 0 failed;
  ... finished in 284.03s`, receipt RESOLUTION AMENDMENT:
  [receipts/2026-10-23-sqlrdf27-verdict.md](../../../receipts/2026-10-23-sqlrdf27-verdict.md)).
  Owed, non-gating: host-contract `-p graphlaw-wasm` rung and formal
  anti-vacuity inversion court on the final topology (same receipt).
- **Conformance target**: tc08 COPY-egress court
  ([designs/fibo-pm4py-court-matrix.md](designs/fibo-pm4py-court-matrix.md),
  `tc08_copy.ttl`, `sql:copyOperation`, CSV|PARQUET|JSON formats) must ADMIT and
  execute on the wrap-enabled wasm artifact; close condition = **FIBO 4/0 +
  lifecycle 3/0** on that artifact
- **Owner lane**: EH-19 (per wave-close receipt
  [receipts/2026-10-23-wave-close.md](../../../receipts/2026-10-23-wave-close.md))

## Current behavior

On the DuckDB-linked wasm artifact (EH-closure wave, commit `0d11bc3`), any
operation that opens an output file through DuckDB's COPY path — witnessed as
tc08 COPY egress — fails with:

```
Cannot open file ...: Invalid argument
```

Root cause (witnessed, three C-level probes under `/tmp/glwasi-probe/`,
preserved — do not delete):

1. **Bit-scan probe** (`cprobe7.c` / `cprobe7.wasm`): called
   `__wasilibc_nocwd_openat_nomode(3, "glc7", flags)` with
   `0x1`, `0x1001`, and `0x1001 | {bit25, bit26, bit27, bit28}`. Every flags
   word **without bits 25-28 set** returned `-1, errno=EINVAL`; adding bit 26
   (`0x04000000`, wasi `O_RDONLY`-class) succeeded. `stat` calls are unaffected
   — the rejection is specific to the access-mode check inside wasi-libc's
   openat.
2. **Constant-identity probe** (`cprobe4.c`): the cached sysroot's headers
   expose **musl-layout** fcntl constants — `O_CREAT=0100 (64), O_EXCL=0200,
   O_TRUNC=01000, O_WRONLY=1` — while the `openat` implementation linked at
   the final link is wasi-libc, which expects the wasi layout
   (`O_RDONLY=0x04000000`, `O_WRONLY=0x10000000`, `O_CREAT=0x1000`,
   `O_EXCL=0x4000`, `O_TRUNC=0x8000`, `O_APPEND`=fdflags bit 0).
3. Conclusion: the cached sysroot pairs **musl headers with wasi-libc objects**;
   `libduckdb.a` emits musl words; wasi-libc's openat access-mode check
   (`bits 25-28`) sees none set → EINVAL. This is an ABI mismatch at the final
   link, not a DuckDB or admission-law defect.

## Required behavior

Translate the musl fcntl flags word into the wasi word at the final link, then
defer to the real wasi-libc `open`/`openat`:

1. **Translation stub** in
   `/Users/sac/frozen-duckdb/crates/frozen-duckdb-sys/wasi_posix.c` —
   `__wrap_open` / `__wrap_openat` with `wasi_fcntl_translate()`:
   musl access bits (`flags & 3`) → wasi access bits
   (`1→0x10000000`, `2→0x0C000000`, `0→0x04000000`);
   `0100→0x1000` (CREAT), `0200→0x4000` (EXCL), `01000→0x8000` (TRUNC),
   `0200000→0x2000` (DIRECTORY), `0400000→0x01000000` (NOFOLLOW),
   `02000→0x0001` (APPEND fdflag). Musl O_CLOEXEC/O_DSYNC/O_NONBLOCK/O_SYNC
   have no wasi oflag equivalent and are dropped. Variadic `mode` is forwarded
   only when O_CREAT is set. (Probe-validated at C level; stub landed in
   wasi_posix.c.)
2. **Raw-lld link args on the final cdylib link**:
   `-Wl,--wrap=open,--wrap=openat` plus
   `-Wl,--whole-archive -Wl,-lduckdb_wasi_posix -Wl,--no-whole-archive`
   (whole-archive so the wrap symbols survive --gc-sections, which otherwise
   drops the archive member that defines `__wrap_*` and leaves the wrap
   unresolved).
3. **Cargo caveat (documented, load-bearing)**: a `cargo:rustc-link-arg`
   emitted from a **dependency's build script does NOT reach the final cdylib
   link** — rustc-link-arg from build scripts of dependencies is not propagated
   to the linking target (graphlaw-wasm). The args must be applied to the
   cdylib itself, via `RUSTFLAGS`/`-C link-arg` on the graphlaw-wasm build
   command. (frozen-duckdb-sys `build.rs` lines 133-136 do emit
   `rustc-link-lib=static=duckdb_wasi_posix` and
   `rustc-link-arg=-Wl,--wrap=open,--wrap=openat`; the lib line reaches the
   link, the arg line does not from a dep.)

## Relink command (the pending court's subject)

Run from `/Users/sac/graphlaw` (coordinator-owned transition; lane does not
execute):

```bash
# Relink-8 canonical recipe (docs/WASM.md "Build command"). Compile the
# in-tree shim once, then build with the cache-dir SDK root (legacy-EH
# sysroot matching libduckdb.a):
clang --target=wasm32-wasip1 -c \
  tests/fixtures/wasm-shim/glaw_wrap_shim.c \
  -o tests/fixtures/wasm-shim/glaw_wrap_shim.o
WASI_SDK_PATH="$HOME/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1" \
RUSTFLAGS="-C target-feature=+exception-handling \
-C link-arg=tests/fixtures/wasm-shim/glaw_wrap_shim.o \
-C link-arg=--wrap=open -C link-arg=--wrap=openat" \
cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm
```

(The originally-witnessed command used
`WASI_SDK_PATH=/opt/wasi-sdk` and `/tmp/glaw_wrap_shim.o`; the
relink-8 canonicalization moves both to the in-tree shim and the
frozen-duckdb cache sysroot — see `docs/WASM.md`. The plain-open shim
variant additionally needs `-C link-arg=--allow-multiple-definition`.)

## Acceptance

A real cargo test / build target on the wrap-enabled artifact:

- [ ] Relink completes (command above, exit 0).
- [ ] `cargo test -p graphlaw-wasm` passes on the wrap-enabled artifact
      (host contract around `wasm/src/lib.rs`).
- [ ] Module-level court: tc08 COPY-egress (`tc08_copy.ttl`) ADMITS and the
      in-wasm COPY writes a readable file — **FIBO 4/0**.
- [ ] Lifecycle court on the same artifact — **lifecycle 3/0**.
- [ ] Anti-vacuity: removing `--wrap=open,--wrap=openat` from RUSTFLAGS must
      reproduce the tc08 EINVAL (the court must flip); if it does not, the
      court is vacuous and FAIL.

## Boundaries / already-implemented (do not re-ask)

- `wasi_posix.c` already carries the ENOSYS stubs (mmap/mlock/msync/madvise/
  getrlimit/kill/sched_getcpu/getpid) — that surface is closed and out of
  scope.
- `stat`-class calls are unaffected by the mismatch (probe receipt); no
  translation is needed or filed for them.
- The admission law (`src/law.rs`), `sql_ast`, and DuckDB eval are untouched by
  this ticket — this is a link-time ABI repair in frozen-duckdb-sys, not a
  graphlaw source change.

## References

- Probe artifacts (preserved): `/tmp/glwasi-probe/` — `cprobe4.c` (constant
  identity), `cprobe7.c`/`cprobe7.wasm` (bit-scan: bits 25-28 gate),
  `probe.rs` (Rust std fs cross-check), `posix.wat`/`openat.wat` (disassembly).
- Stub + wrap symbols:
  `/Users/sac/frozen-duckdb/crates/frozen-duckdb-sys/wasi_posix.c`;
  link emission: `/Users/sac/frozen-duckdb/crates/frozen-duckdb-sys/build.rs`
  (lines 128-140).
- Wave-close pointer row: tc08 egress EINVAL → EH-19
  ([RECEIPTS.md](RECEIPTS.md) line 1373;
  [receipts/2026-10-23-wave-close.md](../../../receipts/2026-10-23-wave-close.md)).
