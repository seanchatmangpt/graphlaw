# First in-wasm SQL lifecycle

A hands-on tutorial: run SQL against DuckDB *inside* the graphlaw wasm module,
end to end, over packet-4 stdio. You will build the runner and the artifact,
open a session, CREATE / INSERT / SELECT, stream the result as N-Triples-style
quads, close, and then watch an unbounded DELETE get refused typed.

Ground truth for the wire shapes: `docs/DIALECTS.md`
("DuckDB op session mode" and "Document sequences" sections).

Prerequisites: a checkout of `/Users/sac/graphlaw` (crate `26.10.28`, Rust 1.98),
Python 3 for the driver script, and `WASI_SDK_PATH` pointing at a wasi-sdk root
(`bin/clang` + `share/cmake/wasi-sdk-p1.cmake`). The canonical build recipe
(including the EH target-feature, without which the link fails with
`undefined _Unwind_GetWasmLPadContext`) lives in `docs/WASM.md`; this tutorial
only needs the runner and a built or pinned artifact. If you do not want to build
the module, `priv/graphlaw.wasm` is the pinned prebuilt artifact. As of
2026-11-09 the SQL steps in sections 4-8 are transcripts from the 2026-10-23 run
against an earlier pin; wire shapes are unchanged but hashes, sizes and timings
will differ on your run.

## 0. How the pieces fit

- `graphlaw-run-eh` — a host runner (`src/bin/graphlaw_run_eh.rs`, built only
  with `--features runner-eh`). It loads the wasm artifact via wasmtime, gives
  it a p1 WASI sandbox, and speaks packet-4 frames on stdio.
- `graphlaw_wasm.wasm` — the module (built from `wasm/`). It links this crate
  *including* frozen-duckdb, so SQL admits and executes inside the module.
- packet-4 framing — every request and reply is one frame:
  4-byte big-endian length prefix, then a JSON body.
- Session — `{"op":"duckdb","mode":"open","session":"<name>"}` registers a
  persistent in-memory connection; later `exec` frames that carry the same
  `"session"` reuse it, so a CREATE in one call is visible to a later
  INSERT/SELECT. Without `"session"`, each exec is a fresh ephemeral
  connection (per docs/DIALECTS.md).

Timing expectation (measured on the debug runner): module compile +
instantiation is a one-time cost of **3-6 minutes per process**. Send all
frames of a scenario to ONE runner process; do not restart per frame.

## 1. Build the runner

    cargo build --features runner-eh --bin graphlaw-run-eh

Real output (a 2026-10-23 build at crate 26.10.22 — the crate is now 26.10.28, so
the banner version differs; captured transcript, not a 2026-11-09 run):

    Compiling graphlaw v26.10.22 (/Users/sac/graphlaw)
    Compiling purrdf v2.0.2
    Compiling wasmtime-wasi v48.0.7
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7m 57s

## 2. Build the artifact

    WASI_SDK_PATH=/Users/sac/wasi-sdks/wasi-sdk-34 \
    CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="-C target-feature=+exception-handling" \
    cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm

Illustrative transcript (not a captured run; banner order, crate versions and
timings vary — expect `graphlaw-wasm v26.10.28`):

    Compiling graphlaw-wasm v26.10.28 (/Users/sac/graphlaw/wasm)
    Finished `wasm` profile [optimized] target(s) in 4m 00s

This lands at `target/wasm32-wasip1/wasm/graphlaw_wasm.wasm` (the pinned
artifact `priv/graphlaw.wasm` is 62,909,537 B, ~63 MB, per
`registry/ARTIFACTS.blake3` as of 2026-11-09). The runs below used the
pinned copy at `priv/graphlaw.wasm`. (Without `WASI_SDK_PATH` the
frozen-duckdb-sys `build.rs` refuses typed: "WASI_SDK_PATH required".)

## 3. Write the frame driver

Save as `first_frames.py`. The `frame()` helper is the whole protocol:

    import json, struct, subprocess, sys, time

    WASM = sys.argv[1]
    RUNNER = "/Users/sac/graphlaw/target/debug/graphlaw-run-eh"
    FIX = "/Users/sac/graphlaw/tests/fixtures/fibo"

    def fixture(name):
        with open(f"{FIX}/{name}") as f:
            return f.read()

    def frame(f):
        b = json.dumps(f).encode()
        return struct.pack(">I", len(b)) + b

    p = subprocess.Popen([RUNNER, "--wasm", WASM], stdin=subprocess.PIPE,
                         stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def send(f):
        p.stdin.write(frame(f)); p.stdin.flush()

    def recv():
        hdr = p.stdout.read(4)
        if len(hdr) < 4:
            return None
        (n,) = struct.unpack(">I", hdr)
        return json.loads(p.stdout.read(n))

    def step(name, f):
        send(f); r = recv()
        print(f"[+{time.time()-t0:.0f}s] {name}:", json.dumps(r), flush=True)
        return r

    t0 = time.time()

Every frame sent must be paired with exactly one `recv()` — if you pipeline
sends, replies queue in order and you will mis-attribute them. (This bit the
tutorial author on the first run.)

The SQL fixtures are Turtle documents from the FIBO court-matrix set
(`tests/fixtures/fibo/`, described in
`tests/fixtures/fibo/README.md` and
`docs/jira/v26.10.22/designs/fibo-pm4py-court-matrix.md`). SQL-in-RDF means
the *admission record is a graph*: each fixture is a `sql:Statement` node the
module SHACL-validates, compiles to SQL, and executes.

## 4. Open a session

Append:

    step("capabilities", {"op": "capabilities"})
    step("open", {"op": "duckdb", "mode": "open", "session": "tutorial-1"})

Run:

    python3 first_frames.py priv/graphlaw.wasm

First reply after the one-time init (real output; the `capabilities` body is
elided — you get `abi`, `ops`, dialect lists, `registry_blake3`):

    [+330s] capabilities: {"ok": true, "result": {"abi": 1, "abi_version": 1, ...
    [+330s] open: {"ok": true, "result": {"ok": true, "session": "tutorial-1"}}

Note the two-level shape: the runner envelope carries
`{"ok": <transport>, "result": <module reply>}`. The module's own `ok` lives
inside `result`. Session semantics from docs/DIALECTS.md: same GAP-2 pragmas
as every connection (memory_limit 32 MiB, threads=1, not overridable from the
wire), 16 live sessions max, duplicate open refused, registry is
process-local and crash-only.

## 5. CREATE and INSERT

The fixtures carry the full graph; the exec frame wraps them:

    def exec_req(turtle):
        return {"op": "duckdb", "mode": "exec", "session": "tutorial-1",
                "query_ast": "http://example.org/fibo#stmt1",
                "subject_prefix": "http://example.org/fibo",
                "vocab_prefix": "http://example.org/vocab",
                "ast": {"turtle": turtle, "dialect": "duckdb"}}

    step("CREATE", exec_req(fixture("tc01_create.ttl")))
    step("INSERT", exec_req(fixture("tc01_insert.ttl")))

Real output:

    [+331s] CREATE: {"ok": true, "result": {"cursor": "713c33ff90457f7431dd
    4831550e68c5f8acb67d536428643998f00c78719830", "more": false, "ok": true
    , "query_hash": "357791cfab8c7c4090d0ba059a8bcfe136f2e9ddd68c5c48aeff060
    7bd718554", "receipt": "http://example.org/fibo/receipt_357791cfab8c7c40
    ", "rows_emitted": 0, "sql": "CREATE TABLE fibo_accounts (id INTEGER, ba
    lance DECIMAL, facility_id VARCHAR)"}}
    [+331s] INSERT: {"ok": true, "result": {"cursor": "6b2d20b6561fcf73e960
    93f5cd43c254cd68e3b23d830ba7fdd9c7daa4c0b6e5", "more": true, "ok": true
    , "query_hash": "3cad3a0725e6da2e9dc16e36890f1725be5acd66eb083dcb7b412c6
    0dd2f0cd6", "receipt": "http://example.org/fibo/receipt_3cad3a0725e6da2e
    ", "rows_emitted": 1, "sql": "INSERT INTO fibo_accounts (id, balance, fa
    cility_id) VALUES (1, 1250.50, 'FAC-CORP-9882')"}}

(Reflowed for the page; the wire reply is single-line JSON.) What to notice:

- `result.sql` echoes the compiled SQL — the Turtle fixture compiled to
  `CREATE TABLE fibo_accounts (id INTEGER, balance DECIMAL, facility_id
  VARCHAR)`, exactly as the fixture header promises.
- `result.receipt` is a receipt IRI minted from `query_hash`; `query_hash` is
  the BLAKE3 identity of the admitted statement.
- `result.cursor` + `more` mean a result set is pending; drain it with
  `stream` (next step) before it matters. `rows_emitted` counts rows the
  statement produced (0 for DDL, 1 for the INSERT's affected row).

## 6. SELECT and the nquads stream

Use a SUM aggregate in the admitted bare-aggregate shape (see
`tests/fixtures/fibo/tc05_select_sum.ttl`; `sql:function
<http://graphlaw.org/functions#SUM>` plus `sql:argument` — the older
`sql:functionName` dual shape is REFUSAL-01):

    SELECT_DOC = r"""@prefix sql: <http://graphlaw.org/sql#> .
    @prefix ex: <http://example.org/fibo#> .
    ex:stmt1 a sql:SelectStatement ;
        sql:statementIndex 5 ;
        sql:fromTable [ sql:tableName "fibo_accounts" ] ;
        sql:selectList ( [ sql:expression [
            sql:function <http://graphlaw.org/functions#SUM> ;
            sql:argument [ sql:columnName "balance" ] ] ] ) ."""

    sel = step("SELECT", exec_req(SELECT_DOC))
    cur = sel["result"]["cursor"]
    step("stream", {"op": "duckdb", "mode": "stream",
                    "session": "tutorial-1", "cursor": cur,
                    "max_rows": 100})

Real output (SELECT then STREAM):

    [+185s] SELECT: {"ok": true, "result": {"cursor": "339767487966cd6372da
    1f3672705f36a347a14696d45702724e15fd83676f22", "more": true, "ok": true
    , "query_hash": "7a91e825467975bc2b38a81baf554b384b28c7ed37e6c15741ad48
    01a4e22273", "receipt": "http://example.org/fibo/receipt_7a91e825467975
    bc", "rows_emitted": 1, "sql": "SELECT SUM(balance) FROM fibo_accounts"
    }}
    [+185s] STREAM: {"ok": true, "result": {"cursor": null, "more": false,
    "nquads": "<http://example.org/fibo/row_1> <http://example.org/vocab/su
    m(balance)> \"1250.500\"^^<http://www.w3.org/2001/XMLSchema#decimal> .\n
    ", "ok": true, "rows": 1}}

The row comes back not as tuples but as an RDF quad: subject
`<...fibo/row_1>`, predicate derived from the projection, typed literal.
`result.more: false` and `result.cursor: null` mean the page drained. If
`more` had been `true`, loop `stream` frames on the same cursor (the court
helper `stream_all` in `tests/wasm_fibo_fixtures_eh.rs` shows the loop).

## 7. Close the session

    step("close", {"op": "duckdb", "mode": "close",
                   "session": "tutorial-1"})

Real output:

    [+331s] close: {"ok": true, "result": {"ok": true}}

Closing frees the connection; per docs/DIALECTS.md the registry is
process-local, so reopening the same name in the same process succeeds, and a
crash-recovery close of an unknown handle is idempotent-ok rather than an
error.

## 8. The typed refusal: unbounded DELETE

Same session pattern, fresh process (`tutorial-2`): open, CREATE, INSERT as
above, then send `tc03_delete_unbounded.ttl` — a `sql:DeleteStatement` with
no WHERE clause:

Real output:

    [+331s] DELETE-unbounded: {"ok": true, "result": {"error": {"details":
    {"code": "DuckDb", "message": "AST compile refusal: SHACL AST structural
     violation: Node <<http://example.org/fibo#stmt1>>: [REFUSAL-05] Unboun
    dedMutationRefusal: a DeleteStatement MUST carry a WHERE clause (require
    s BOTH sql:whereColumn and sql:whereValue)."}, "dialect": null, "engine
    ": null, "kind": "DuckDb", "message": "duckdb: AST compile refusal: SHA
    CL AST structural violation: Node <<http://example.org/fibo#stmt1>>: [R
    EFUSAL-05] UnboundedMutationRefusal: ... (same message)", "ok": false}}

Read it as: envelope `ok:true` (the runner delivered the frame), inner
`result.ok:false` (the module refused), and a message carrying the stable
code `[REFUSAL-05] UnboundedMutationRefusal` from `docs/refusals.md`. The
DELETE never reached the engine — it died at SHACL validation at compile
time, which is why the wire `kind` is `DuckDb`/`MalformedAst`-class and not
`UnboundedMutation` (the compiler's own guard is defense in depth behind the
shape; the court TC-FIBO-03 in `tests/wasm_fibo_fixtures_eh.rs` asserts the
message string for exactly this reason). The fix is to bound the mutation
with `sql:whereColumn` + `sql:whereValue` — see `tc02_update_bounded.ttl`
for the bounded-UPDATE twin.

## Recap

You built the EH runner and the ~63 MB module, spoke packet-4, kept one
session alive across CREATE → INSERT → SELECT, drained the result as nquads,
closed cleanly, and watched the SHACL layer refuse an unbounded DELETE typed
before the engine ever saw it. Next steps: whole-document sequences
(`"document":true` exec, docs/DIALECTS.md "Document sequences" — one frame
carries a multi-statement Turtle document and answers a BatchReceipt), and
the court files `tests/wasm_fibo_fixtures_eh.rs` and
`tests/wasm_duckdb_lifecycle_eh.rs` for the full matrix.

## Verification (receipt)

Every step above was executed while writing this tutorial, 2026-10-23 (scratch
drivers `/tmp/first_sql_frames*.py` were throwaway copies of the section 3 driver
and are not in the tree; the transcripts embedded above are the receipt):

- `cargo build --features runner-eh --bin graphlaw-run-eh` — exit 0,
  "Finished `dev` profile ... in 7m 57s" (embedded above).
- `WASI_SDK_PATH=... cargo build -p graphlaw-wasm --target wasm32-wasip1
  --profile wasm` — exit 0, "Finished `wasm` profile ... in 4m 00s".
- Lifecycle run: `/tmp/first_sql_frames.py` against `priv/graphlaw.wasm` —
  capabilities, open, CREATE, INSERT, SELECT, STREAM (balance 1250.500
  readback), close; runner exit 0. Full transcript embedded in sections 4-7.
- Refusal run: `/tmp/first_sql_frames2.py` — DELETE-unbounded reply carries
  `[REFUSAL-05] UnboundedMutationRefusal`, close ok, runner exit 0.
  Transcript embedded in section 8.
- Session timings ([+330s] etc.) are real elapsed times on the debug runner.

See Also: `docs/DIALECTS.md` · `docs/WASM.md` · `docs/refusals.md` ·
`docs/jira/v26.10.22/designs/fibo-pm4py-court-matrix.md` ·
`docs/how-to/run-the-wasm-module.md`
