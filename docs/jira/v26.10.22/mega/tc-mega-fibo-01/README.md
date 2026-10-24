# TC-MEGA-FIBO-01 — Operation Global Tear-Up (scenario corpus)

Deterministic derivative-tear-up corpus: 100,000 legs, 3 jurisdictions
(US/UK/EU), 3 clearinghouses (ICE Clear Credit US / LCH.Clearnet UK /
Eurex Clearing EU), 5,000 LEIs under 12 holding companies, leg kinds
IRS / CDS_TRANCHE / FX_FORWARD, floating curves SOFR/SONIA/EURIBOR mapped by
leg jurisdiction, ISDA 2002 margin fields (imThreshold, vmThreshold,
minimumTransferAmount, isdaMasterAgreement).

## Layout

- `generators/gen_corpus.py` — the generator (python3 stdlib only).
- `corpus/` — generated artifacts + one hand-authored reserved seed.
- This file.

## Dialect partitioning (operator addendum)

Each dialect is emitted from its own generator path and has a distinct role —
no gratuitous duplication:

| Dialect | Role | Content |
|---|---|---|
| `.ttl` | static contract definitions | holding companies, LEIs, swap legs with FIXED rates |
| `.trig` | execution contexts | one named graph per CCP (`tcg:ccp-us`, `tcg:ccp-uk`, `tcg:ccp-eu`) carrying cleared-leg state, jurisdiction, curve, ISDA margin fields; plus the sql-rdf statement graph |
| `.nq` | raw streaming ticks / price fixings | 3 price fixings + 1 `aboutLeg` link per leg, all quads into the `tcg:ticks` graph |
| `antivacuity-leg-41209-malformed.ttl` | reserved anti-vacuity twin seed | hand-authored (not generated), see below |

## Predicate vocabulary (compatibility with src/sql_ast.rs)

The compiler's sql: surface (`KNOWN_SQL_PREDICATES` in
`src/sql_ast.rs`, plus the structural clause terms the extractor reads:
`joinType`, `joinTable`, `joinCondition`, `tableAlias`, `tableName`,
`columnName`, `literalValue`, `operator`, `leftOperand`, `rightOperand`,
`valueExpression`, `limit`, `groupBy`, and operator IRIs `sql:Equals` etc.)
is exercised by the sql-rdf statement graph in the `.trig` dialect
(`tcg:stmt-usd-exposure`, a `sql:SelectStatement` over `tcg_derivative_legs`
filtered on `currency = "USD"`).

Domain data uses the separate `tcg:` namespace
(`http://graphlaw.org/tcmega#`). Mismatch disclosure (no silent renaming):
domain predicates (`tcg:notional`, `tcg:imThreshold`, ...) are NOT sql:
predicates and are not claimed to be compiler-admitted; the generator
refuses with exit 2 if any `sql:` local name outside the admitted set is
ever emitted. Alias spellings `sql:selectList` / `sql:fromTable` /
`sql:whereClause` are used (not the normative `select`/`from`/`where`),
matching the alias-acceptance lane. Legs are projected to the flat table
name `tcg_derivative_legs` referenced by `sql:fromTable`; the physical
projection mapping is owned by the compiler lane.

## Generation commands and exits

```bash
cd generators
python3 gen_corpus.py --legs 1000    --lei 500  --holders 12 \
    --seed 20261022 --outdir ../corpus --stem tc-mega-fibo-01-smoke   # exit 0
python3 gen_corpus.py --legs 100000  --lei 5000 --holders 12 \
    --seed 20261022 --outdir ../corpus --stem tc-mega-fibo-01-full    # exit 0
```

100k full corpus generates in ~1.3 s wall (measured 0.96 s user).

## Artifacts on disk (ls -l, 2026-10-22)

```
antivacuity-leg-41209-malformed.ttl     1230
tc-mega-fibo-01-full.nq             66650095
tc-mega-fibo-01-full.trig           54046134
tc-mega-fibo-01-full.ttl            24790702
tc-mega-fibo-01-smoke.nq              666593
tc-mega-fibo-01-smoke.trig            535909
tc-mega-fibo-01-smoke.ttl             301699
```

## Determinism invariant (proven)

Two full re-generation runs at `--seed 20261022`, sha256 over every output,
`diff` of the digest lists returned empty (DETERMINISM-OK). Recorded digests:

```
d73d77f3a813dc954f37ac33fa7c98f371d44c87cb0dbb2c3181c20c0a2e8be2  antivacuity-leg-41209-malformed.ttl # historical
b09ff4de5b635172a429ea38dad3d2aee6e90a56e256c663fd1b5a5c0e19400d  tc-mega-fibo-01-full.nq # historical
a2932316d00fcd7a1b0d78b8de1fb81e0e0aca288e21993ccab6647b75682ccf  tc-mega-fibo-01-full.trig # historical
f3e22ad6d641f4347378b0c7980fb7aa7d0e0bd1a623aa25916edf111d601e57  tc-mega-fibo-01-full.ttl # historical
763b1a4900311ad2a2931eb2079fb54b2773be0bde3a6ac06011e47ecf12a15b  tc-mega-fibo-01-smoke.nq # historical
370c47f6e8693687314b0044d3add3b706f2d21b51397f278e42bfb5e43f307d  tc-mega-fibo-01-smoke.trig # historical
19d2f840b44a23ea17728bf246ace9eb44d6db2271f8ccd62d4cad889c7c3013  tc-mega-fibo-01-smoke.ttl # historical
```

Any digest flip on regeneration is FAIL.

## Parse verification (real parsers, not inspection)

- rdflib 7.6.0: smoke `.ttl` = 8,536 triples; smoke `.trig` = 11,026 quads
  across `tcg:ccp-us`/`ccp-uk`/`ccp-eu` + default; smoke `.nq` = 4,000
  quads; full `.nq` = 400,000 quads; malformed twin = 9 triples. All exit 0.
- Not yet parser-cycled at full scale: `full.ttl` / `full.trig` under rdflib
  (slow pure-Python parser; `full.nq` at 400k quads already passes). If a
  stricter full-scale court is needed it belongs to the compiler lane's
  admission run.

## Anti-vacuity twin seed (leg #41209 style)

`corpus/antivacuity-leg-41209-malformed.ttl` carries a single leg whose
currency decimal precision is 4 fractional digits (`1234567.8901`) where the
benign corpus invariant is exactly 2 (ISO 4217 minor units). It lives in a
separate file so the benign corpus stays clean. Intended court property:
loading the twin alongside benign data must be flagged/refused; a gate that
admits it unchanged is vacuous. Hand-authored and immutable; regenerate
never touches it.
