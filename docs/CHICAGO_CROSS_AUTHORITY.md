# Chicago Cross-Authority Court

## Standing

Finite published corpus passes on exact subject; authority NONE; synthetic subject; no general
financial safety claim.

Court id `chicago-cross-authority`. Authority `NONE`, consequence `EVIDENCE_ONLY`. The court is
generated from RDF case data (pack `chicago-graphlaw-court-pack`, individual
`cc:cross_authority_court`, vocabulary `https://chicago.graphlaw.dev/court#`) into
`tests/chicago_cross_authority.rs`. That file is generated and is never hand-edited.

## Purpose

GraphLaw does not implement a rule language itself; each op delegates to an admitted upstream
authority. This court asks one question: when several authorities are given the same problem,
do they agree on the answer, and does each fail closed when the input is wrong?

Authorities exercised through `graphlaw::abi::call`: `n3`, `datalog`, `entail` (owl-rl),
`shacl`, `sparql`, `canonical`, and the `law` op where a case needs a plan or lease.
Collaborators are real. There are no mocks; assertions are on final response state.

## Case classes

Every court contains at least one case of each of five kinds (gated). Negatives outnumber
positives (40 vs 12); a positive twin per negative is not gated or claimed. Ordinals: this court owns 1 to 499 (financial court owns 500 to 999). Case ids use the
`xa_` prefix.

| kind | what it establishes |
|---|---|
| positive | SHACL and ASK over the closure hold on the expected result |
| negative | the fail-closed table: malformed or out-of-contract input returns `ok=false` with a typed error |
| invariance | reversing fixture statement order does not change the RDFC-canonical id |
| cross-authority | N3, owl-rl and SPARQL CONSTRUCT arms converge on one canonical id; Datalog agrees on the exact sorted fact set |
| boundary-mutation | dropping edge c to d from the base graph must change the result; a mutation whose expectation equals its base is refused by the pack gate |

Reachability terms use `https://chicago.graphlaw.dev/` with `edge` and `reachable`, matching
`tests/chicago_n3_datalog.rs`.

Chained requests (an array of ABI requests with `$PREV:/ptr` substitution) carry the projection
path: n3 `/derived`, then SPARQL CONSTRUCT over that text, then `canonical` `/id`.

Failure cases assert `ok`, `error.kind`, `error.engine` and `error.details.code`; success cases
assert result fields such as `/id` and `/facts`. Error message strings are not asserted.

## Commands

Generate (coordinator step, from `/Users/sac/graphlaw`):

    ggen sync run

Run this court:

    cargo test --all-features --test chicago_cross_authority

Related courts:

    cargo test --all-features --test chicago_n3_datalog
    cargo test --all-features --test sa2a_run14

Byte-identity cases also run `tests/common` native and wasm harnesses. `tests/common` builds the wasm (or reads
`GRAPHLAW_WASM`); a failed build panics the case. Not a skip; no staleness check exists.

## Proof tests

The generated file carries two tests beyond the cases:

- `court_declares_exactly_the_published_case_count`: the number of test functions equals
  `EXPECTED_CASE_COUNT + 2`.
- `court_covers_every_kind`: each of the five kinds has at least one case.

## Fixed limits (typed)

- `UNSUPPORTED[datalog-canonical-projection]`: Datalog returns facts, not a graph, so it cannot
  be projected to a canonical id. The Datalog arm is compared by exact sorted `/facts`.
- `UNSUPPORTED[generated-oversize-payload]`: `request_bytes` and `json_depth` caps are not
  expressible in the case grammar; `tests/resource_limits.rs` covers them natively.
- `UNSUPPORTED[abi:no-receipt-signing]`: the ABI does not sign receipts, so no signed-receipt
  claim is made.
- Wasm unavailable: byte-identity cases fail hard (panic in `tests/common`), never skip.

## PASS does not mean

- complete W3C N3, OWL 2 RL, SHACL, SPARQL or Datalog conformance;
- agreement on inputs outside the published cases;
- a general financial safety claim (the subject is synthetic; see `CHICAGO_FIBO.md`);
- any authority: evidence authority is NONE.

## See Also

`CHICAGO_N3_DATALOG.md` · `CHICAGO_FIBO.md` · `abi-reference.md` · `refusals.md`
