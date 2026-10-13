# strata_gate_differential fixtures (lane L142)

Byte-identical pinned copies of ggen-marketplace strata-pack sources, copied
2026-10-13 (lane L142):

- `<pack>/ontology.ttl` from `/Users/sac/ggen-marketplace/packs/strata-<pack>-pack/ontology.ttl`
- `<pack>/gates/*.rq` from the same pack's `gates/` directory

`manifest.sha256` is the pin of record; the court
(`tests/strata_gate_differential.rs::pinned_fixtures_match_manifest`) refuses to
evaluate against drifted pins.

## Regenerating rdflib row-JSON (subprocess-blocked sandboxes only)

The court runs rdflib through a live `python3` subprocess by default. If a
sandbox forbids subprocess spawn, pre-record row counts with:

```bash
for p in hash valve ledger tx governance; do
  python3 - <<EOF
import json
from rdflib import Graph
import glob, os
g = Graph(); g.parse("tests/fixtures/strata_gate_differential/$p/ontology.ttl", "turtle")
for rq in sorted(glob.glob("tests/fixtures/strata_gate_differential/$p/gates/*.rq")):
    print(json.dumps({"gate": rq, "rows": len(list(g.query(open(rq).read())))}))
EOF
done
```

then feed the printed JSON to a fixture-reader variant of `rdflib_rows`. The
live-subprocess path is preferred (Chicago: real collaborator).
