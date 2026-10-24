# SQLRDF-28

- **Id**: SQLRDF-28
- **Title**: Mutation-court coverage gap — `src/abi.rs` response builders + `spec_of` survive all mutants (post-release)
- **Status**: Open
- **Conformance target**: Anti-vacuity — `scripts/mutation_court.sh` must not admit `vec![]`/`vec![0]`/`vec![1]` replacements or deleted/guard-deleted match arms in `src/abi.rs`; every published response byte and error path is pinned by an exact-bytes/shape court.

## Current behavior

Witnessed by the live mutation-court run
(`mutants.out/missed.txt`, run of 2026-10-23, 11 surviving mutants, all in `src/abi.rs`):

Verbatim `mutants.out/missed.txt` (11 lines, all `src/abi.rs`):

```text
src/abi.rs:152:5: replace limit_response -> Vec<u8> with vec![]
src/abi.rs:152:5: replace limit_response -> Vec<u8> with vec![0]
src/abi.rs:152:5: replace limit_response -> Vec<u8> with vec![1]
src/abi.rs:157:5: replace missing_buffer_response -> Vec<u8> with vec![]
src/abi.rs:157:5: replace missing_buffer_response -> Vec<u8> with vec![0]
src/abi.rs:157:5: replace missing_buffer_response -> Vec<u8> with vec![1]
src/abi.rs:167:5: replace unknown_buffer_response -> Vec<u8> with vec![]
src/abi.rs:167:5: replace unknown_buffer_response -> Vec<u8> with vec![0]
src/abi.rs:167:5: replace unknown_buffer_response -> Vec<u8> with vec![1]
src/abi.rs:271:9: delete match arm Some(Value::String(_)) in spec_of
src/abi.rs:270:20: replace match guard s.is_object() with true in spec_of
```

The surviving mutants:

- `src/abi.rs:152` `limit_response(name, observed, max)` — replacing its body
  with `vec![]`, `vec![0]`, or `vec![1]` **survives the full suite**: no test
  asserts the actual response bytes of the `MAX_REQUEST_BYTES` over-limit
  path.
- `src/abi.rs:157` `missing_buffer_response()` — same three replacements
  survive; the `gl_alloc`-refused wire response is unpinned.
- `src/abi.rs:167` `used by the wasm shim` — `unknown_buffer_response()`
  (`gl_call` given a pointer not owned / length not paired) — same three
  replacements survive.
- `src/abi.rs:268` `spec_of(v, k)` — deleting the
  `Some(Value::String(_))` arm (line 271) survives: no test drives a string
  into a data-spec slot and asserts the typed error
  `` `{k}` must be a data spec object {"text": ...} ``.
- `src/abi.rs:270` guard `s.is_object()` → `true` survives: a non-object,
  non-string `spec` value (e.g. a number) would then be admitted as a "data
  spec" instead of refused with ``missing data spec `{k}` ``-class error, and
  no test distinguishes the two.

### Already-implemented boundary (not re-asked)

The release-gate set is covered and its mutants are caught (see
`mutants.out/caught.txt`): `registry.rs` / `capabilities()` / `dispatch()`
mutants all flip tests. The three response builders are `pub` and reachable
from a host test directly, and `spec_of` is exercised via public dispatch ops
taking a `spec` field. Nothing here re-asks for `registry.rs` or `dispatch`
coverage.

All 11 survivors are **accepted** for the release gate (exit-2 accepted,
post-release filing) — none touch the gate set — but they are real coverage
gaps: a corrupt `limit_response` shipping in the wasm shim answers every
over-limit request with an empty/garbage wire document.

## Required behavior

New court file `tests/abi_response_builders_court.rs` (Chicago: real
`pub` functions, assert on exact returned bytes/shape, no mocks):

1. **`limit_response`** (`src/abi.rs:151`): assert the exact returned byte
   sequence is `respond(Err(limit(name, observed, max)))` — parse the JSON and
   pin at minimum:
   - top-level `error` present, `error.kind` == `"ResourceLimit"`,
   - `error.details` == `{"code":"ResourceLimit","limit":name,"observed":observed,"max":max}`,
   - `error.message` == `` resource limit `{name}` exceeded: {observed} > {max} ``.
   Any of `vec![]` (no `error` key), `vec![0]`/`vec![1]` (invalid UTF-8/JSON)
   then fails the parse or the exact-shape assert.
2. **`missing_buffer_response`** (`src/abi.rs:156`): parse JSON, assert
   `error.kind == "Unsupported"` and
   `error.message == "request buffer missing: gl_alloc refused or was never called"`.
3. **`unknown_buffer_response`** (`src/abi.rs:166`): parse JSON, assert
   `error.kind == "Unsupported"` and
   `error.message == "request buffer is not a live allocation from gl_alloc"`.
4. **`spec_of` string arm** (via a public dispatch surface that takes a `spec`
   field, e.g. the op handler calling `state_field`): request whose `spec` is
   the string `"x"` (not an object) must be refused with the typed message
   `` `spec` must be a data spec object {"text": ...} `` — catches the deleted
   `Some(Value::String(_))` arm.
5. **`spec_of` guard** (same surface): `spec` = a non-object non-string value
   (e.g. `json!(7)`) must be refused (not silently admitted), and `spec` = a
   real `{"text": "..."}` object must still be **admitted** — catches the
   `s.is_object()` → `true` guard mutation while proving no over-refusal.

## Acceptance

- `cargo test --test abi_response_builders_court` — GREEN with all five groups
  present.
- Rerun `bash scripts/mutation_court.sh`; acceptance is the flip:
  all 11 `src/abi.rs` mutants currently in `mutants.out/missed.txt` move to
  `caught.txt`. A rerun that still lists any of
  `abi.rs:152/157/167/270/271` in `missed.txt` fails the ticket.
- Non-vacuity: deleting any assert group from the court file must re-red the
  corresponding mutants on the next mutation-court run.

## References

- Misses receipt: `mutants.out/missed.txt` (2026-10-23 run)
- Court runner: `scripts/mutation_court.sh`
- Subject: `src/abi.rs` (`limit_response` :151, `missing_buffer_response`
  :156, `unknown_buffer_response` :167, `spec_of` :268-275)
- Gate boundary: release gate set (`registry.rs`, `capabilities()`,
  `dispatch()`) already caught — see `mutants.out/caught.txt`
