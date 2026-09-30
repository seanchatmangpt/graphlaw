# FIBO synthetic fixtures

Instance data for the `chicago-fibo` court (case file `cases/b.ttl` in
`ggen-marketplace/packs/chicago-graphlaw-court-pack`, ordinals 500-999). The generated test
`tests/chicago_fibo.rs` reads these files through `"$FILE:tests/fixtures/fibo/<name>"` tokens.

## Standing

Finite published corpus passes on exact subject; authority NONE; synthetic subject; no general
financial safety claim.

- The subject is synthetic: no real parties, accounts, or payment rail
  (`UNSUPPORTED[real-payment-rail]`).
- The USD 10,000,000 threshold (`10_000_000_000_000` micros, inclusive) is Chicago-court policy in
  `policy_high_value.n3`. It is not FIBO's and not a regulation. FIBO defines no threshold and no
  ceiling.

## FIBO provenance

- Source: `ggen-marketplace` commit `2b6126abf11017527ce400b6001c063e252fdfb6`,
  path `ontologies/public/fibo` (EDM Council FIBO).
- Licence: MIT (`dct:license` in `FND/Accounting/CurrencyAmount.rdf` cites
  https://opensource.org/licenses/MIT).
- These fixtures reference FIBO IRIs only; no FIBO text is copied. Every FIBO IRI used was checked
  to exist as a subject in the pinned tree (20 term IRIs: `MarketTransaction`, `consideration`,
  `paymentTerms`, `TransactionPrincipal`, `TransactionCounterparty`, `MarketTransactionPaymentTerms`,
  `transactedUnder`, `Contract`, `MasterAgreement`, `hasCounterparty`, `MonetaryAmount`,
  `hasAmount`, `hasCurrency`, `ExchangeRate`, `ISO4217-CurrencyCodes/USD`, `.../EUR`,
  `SettlementTerms`, `hasSettlementAmount`, `SecuritiesTransaction`, `Security`).
- `UNSUPPORTED[convert-fibo-rdfxml]`: the `convert` op refuses FIBO RDF/XML ("XML with DTD
  detected"), so the fixtures are hand-authored Turtle, not converted.
- `UNSUPPORTED[full-FIBO-import]`: FIBO is referenced by IRI, not imported or reasoned over.

## Files

| file | role |
|---|---|
| `tx_usd_25m.ttl` | positive control: USD 25M market transaction, master agreement `ma-1` |
| `tx_usd_25m_masterM2.ttl` | same amounts, master agreement `ma-2` (distinct identity) |
| `tx_eur_20m.ttl` | EUR 20M with admitted EURUSD rate 1.10 (`rateValue 1100000`) |
| `tx_eur_20m_rate_1_05.ttl` | same, rate changed to 1.05 after admission |
| `dvp_securities.ttl` | DvP: cash leg and security leg both `ready` |
| `dvp_securities_failed.ttl` | same, with the DvP marked `failed` |
| `delegation.ttl` | delegation graph with a 3-cycle, a tail and a separate component |
| `policy_high_value.n3` | N3 policy rules R1 (micros), R2 (FIBO decimal), R3 (admitted FX) |
| `shapes.ttl` | SHACL: one currency, one decimal, one integer micros per amount |

Money is integer micros (`fin:amountMicros`) beside the FIBO decimal (`cur:hasAmount`); the two
must agree and the cases check both rules at the boundary.

## Not claimed

- `UNSUPPORTED[money-ceiling]`: the lease `Ceiling` is Observe, Select or Construct only. The
  money boundary is enforced by the N3 policy, not by a lease.
- `UNSUPPORTED[abi:no-receipt-signing]`: the ABI cannot sign receipts; no signed-receipt claim.
- Amounts beyond i64 were observed to work in N3 (`9223372036854775808` derives high value) but
  that is an observation, not a specified behaviour, and no case asserts it.
- Boundary variants and small graphs live inline in `b.ttl`; only larger graphs are files here.

## See Also

- `docs/CHICAGO_FIBO.md`
- `tests/chicago_n3_datalog.rs`
