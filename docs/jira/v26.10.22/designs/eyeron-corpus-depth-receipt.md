# Eyeron corpus nesting-depth receipt (W12 / lane M15)

Date: 2026-10-22. Subject: `crates/graphlaw-eyeron/src/parser/cursor.rs:14` doc-comment
claim "deepest nesting in the example and conformance corpora is 14 levels".

## Scan

Python3 bracket-depth scan over every `.n3`/`.ttl` file in the repo excluding `target*`
build roots. String literals and `#` comments stripped before counting; bracket set
`{ } [ ] ( )`; RDF-star `<< >>` checked separately (one fixture file uses them, max
depth there is 4, below the corpus max).

Command (embedded as run):

```python
import glob, re
files = glob.glob('**/*.n3', recursive=True) + glob.glob('**/*.ttl', recursive=True)
files = [f for f in files if not f.startswith(('target', 'target-lane'))]
pairs = {'{':'}','[':']','(':')'}
best = {}
for f in files:
    text = open(f, encoding='utf-8', errors='replace').read()
    text = re.sub(r'"(?:[^"\\]|\\.)*"', '""', text)
    text = re.sub(r"'(?:[^'\\]|\\.)*'", "''", text)
    text = re.sub(r'#[^\n]*', '', text)
    depth = 0; mx = 0
    for ch in text:
        if ch in pairs: depth += 1; mx = max(mx, depth)
        elif ch in pairs.values(): depth -= 1
    best[f] = mx
for f, d in sorted(best.items(), key=lambda kv: -kv[1])[:10]: print(d, f)
print("MAX:", max(best.values()), "files scanned:", len(best))
```

## Output (real, exit 0)

```
17 ontologies/social/vcard.ttl
17 ontologies/industry/financial/vcard-ontology.ttl
17 ontologies/industry/energy/4-vcard-ontology.ttl
5 ontologies/shapes/shacl-shacl.ttl
5 tests/fixtures/w3c/rdf-trig/reports/earl.ttl
5 tests/fixtures/w3c/rdf-n-triples/reports/earl.ttl
5 tests/fixtures/w3c/rdf-n-quads/reports/earl.ttl
4 tests/fixtures/ast_governance/cold_chain_ast.ttl
3 ontologies/bibliographic/bibo.ttl
3 ontologies/schema-org/schemaorg-all-https.ttl
MAX: 17 files scanned: 676
```

## Finding and diff

Real corpus max is **17**, not 14. The "14 levels" claim was unsubstantiated; it is now
corrected to 17 in the doc comment with a pointer to this receipt. The `64` bound itself
is unaffected (17 stays far inside 64).

Diff to `crates/graphlaw-eyeron/src/parser/cursor.rs`:

```diff
-/// The deepest nesting in the example and conformance corpora is 14 levels,
+/// The deepest nesting in the example and conformance corpora is 17 levels
+/// (measured 2026-10-22 over 676 `.n3`/`.ttl` files; see
+/// `docs/jira/v26.10.22/designs/eyeron-corpus-depth-receipt.md`),
```

## Verification

Gate (compile + sql-rdf alias admission courts):

```
cargo test --test sql_rdf_dialect
```
