# GraphLaw Knowledge Hook projection

GraphLaw owns the semantic-state to Knowledge-Hook projection used by the RFC reference stack.

The projection is deliberately powerless. It preserves exact subject, immutable semantic source digest, deterministic replay identity and the canonical SA2A authority ceiling of `NONE`. It may construct a candidate intent; it cannot authorize or perform DO.

The synthetic FIBO payment case changes only semantic capability input. Downstream CASTLE reuses its existing PreparedEffect digest, independent verification receipt, BRCE boundary, outcome receipt and replay/process evidence machinery. A FIBO-specific authority, receipt, replay or consequence subsystem would violate this contract.

The executable gates in `queries/knowledge_hook/gates/` and vectors in `conformance/knowledge_hook/` inherit the same failure dimensions already used by the SA2A portable corpus: authority smuggling, exact-subject loss, source-digest loss and replay-identity loss.
