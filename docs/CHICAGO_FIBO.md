# Chicago FIBO Court

## Standing

Finite published corpus passes on exact subject; authority NONE; synthetic subject; no general
financial safety claim.

Court id `chicago-fibo`. Authority `NONE`, consequence `EVIDENCE_ONLY`. Generated from RDF case
data (pack `chicago-graphlaw-court-pack`, individual `cc:fibo_court`) into
`tests/chicago_fibo.rs`. That file is generated and is never hand-edited.

## The threshold is our policy, not FIBO's

FIBO defines financial vocabulary. It contains no monetary threshold and no grant ceiling. The
$10M review threshold used here is a policy chosen by this repository for the court:

- `fin:HighValueTransaction` holds iff `fin:amountMicros >= 10_000_000_000_000`.
- The comparison is inclusive (N3 `math:notLessThan`): exactly $10M qualifies.
- Money is integer micros. `fin:` is `https://chicago.graphlaw.dev/fin#`.

Datalog is positive-only and cannot express this threshold; the N3 path does. Integer micros and
exact 6-decimal comparison both work through the N3 math builtins. Values beyond i64 are a
documented observation only, not a spec claim.

## Synthetic subject

All fixtures are synthetic. There is no real payment rail, no real counterparty and no real
settlement. FIBO IRIs are used verbatim from the pinned copy in `ggen-marketplace`
(commit `2b6126abf11017527ce400b6001c063e252fdfb6`, path `ontologies/public/fibo`, MIT). Only the
terms the fixtures need are used, not the full ontology.

## Fixtures

`tests/fixtures/fibo/`: `README.md` (provenance), `tx_usd_25m.ttl`, `tx_usd_25m_masterM2.ttl`,
`tx_eur_20m.ttl`, `tx_eur_20m_rate_1_05.ttl`, `dvp_securities.ttl`,
`dvp_securities_failed.ttl`, `delegation.ttl`, `policy_high_value.n3`, `shapes.ttl`.
Boundary variants are inline in the case file, which is the single source.

## Case classes

Ordinals 500 to 999, case ids prefixed `fibo_`. All five kinds are present (gated). Each negative names a positive twin in its case title; that
pairing is a review convention, not a generator gate.

| kind | what it establishes |
|---|---|
| positive | a transaction at or above threshold is classified `fin:HighValueTransaction`; a plan that satisfies its preconditions is admitted |
| negative | plan refusals (including negative preconditions), lease refusals, and delivery-versus-payment failures are refused with typed details |
| invariance | canonical id and verdict are stable under statement/rule reordering and prefix renaming; master-agreement change moves the digest, not the verdict |
| cross-authority | micros and decimal representations of the same amount give the same verdict |
| boundary-mutation | threshold matrix around 10_000_000_000_000 micros, based on the 25M case; mutation must change the verdict |

Lease-bearing cases always pass a lease, because the `law` op performs no ceiling check without
one.

## Commands

    ggen sync run
    cargo test --all-features --test chicago_fibo

Also part of the wave:

    cargo test --all-features --test chicago_cross_authority
    cargo test --all-features --test sa2a_run14

The run14 runner (`sa2a-run14-verify`, feature `abi`) checks 65 files in
`conformance/sa2a/run14/` (31 ADMIT, 34 REFUSE; 14 under `https://sa2a.dev/ontology#`, 51 under
`https://chatmangpt.com/sa2a#`). Disposition is declared and cross-checked against filename and
courtId, not computed from payload. Fixtures 001 and 002 are payload-identical. All carry
authority NONE and EVIDENCE_ONLY. The corpus is read-only; failure cases run on a temp copy.

The run12 runner (`sa2a-portability-verify`, feature `abi`) runs the 50 vectors in
`conformance/sa2a/portable/run12/` through GraphLaw's real JSON ABI (`parse` plus RDFC-1.0
`canonical` on every vector), requires ADMIT vectors to carry the `authority NONE`,
`consequence EVIDENCE_ONLY` and `canonicalization RDFC-1.0` invariants and REFUSE vectors a typed
`sa2a:violation`, and asserts the corpus stays at exactly 50 vectors. It is evidence verification
only: it never grants authority or performs DO.

Elixir counterpart (`ash_a2a`, court CHI-FIN, falsifiers CHI-FIN-001 onward):

    MIX_ENV=test mix ash_a2a.chicago --court CHI-FIN --court CHI-ID --court CHI-ADM --court CHI-REAL --require-conformant

The literal command with `--court CHI-FIN` alone is `UNSUPPORTED` as specified (profile core,
gates 1 to 3 missing). The four-court command ends PARTIAL_ALIVE (court manifest drift) until
`mix ash_a2a.chicago.pin_court_manifest` adds CHI-FIN to the committed manifest; it has not been
observed CONFORMANT.

## UNSUPPORTED and BLOCKED

- `UNSUPPORTED[real-payment-rail]`: no real money movement of any kind.
- `UNSUPPORTED[full-FIBO-import]`: only fixture-needed terms are used.
- `UNSUPPORTED[convert-fibo-rdfxml]`: the `convert` op refuses the FIBO RDF/XML DTD.
- `UNSUPPORTED[money-ceiling]`: lease `Ceiling` covers only Observe, Select and Construct. The
  monetary grant ceiling is a `fin:grantCeilingMicros` fact enforced in ash_a2a by SHACL
  `sh:lessThanOrEquals` (the per-transfer envelope uses `sh:maxInclusive`); it is not a lease ceiling.
- `UNSUPPORTED[abi:no-receipt-signing]`: the ABI does not sign receipts, so no signed-receipt
  claim is made.
- `BLOCKED[wasm-artifact-missing]`: CHI-FIN (ash_a2a) returns BLOCKED for every falsifier if the
  wasm artifact is absent. In graphlaw, `tests/common` builds the wasm (or reads `GRAPHLAW_WASM`);
  a failed build panics the byte-identity cases. No staleness check exists. Never a pass, never a skip.

## PASS does not mean

- FIBO conformance or completeness;
- that $10M is a regulatory or industry threshold;
- financial safety, compliance, or fitness for any real transaction;
- any authority: evidence authority is NONE.

## See Also

`CHICAGO_CROSS_AUTHORITY.md` · `CHICAGO_N3_DATALOG.md` · `abi-reference.md` · `refusals.md`
