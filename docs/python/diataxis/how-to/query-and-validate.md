# How to sniff, parse, convert, query and validate

Each snippet assumes `gl = GraphLaw()`. Data arguments accept a bare string (dialect sniffed) or
a spec `{"text", "dialect"?, "hint"?, "base"?}`.

## Route content
<!-- cap: op:sniff -->

```python
gl.sniff("@prefix e: <https://e/> . e:s e:p e:o .")["dialect"]    # 'Turtle'
```

Ambiguous input (`<g> { <s> <p> <o> }`) is refused `Ambiguous`; pass `hint="trig"`.

## Parse and convert
<!-- cap: op:parse -->
<!-- cap: op:convert -->

```python
gl.parse(text, "turtle")["id"]
gl.convert(text, to="ntriples", dialect="turtle")["text"]
```

## Canonical form
<!-- cap: op:canonical -->

```python
gl.canonical(text)["nquads"]      # stable bytes; same id for isomorphic graphs
```

## SPARQL, SHACL, ShEx
<!-- cap: op:sparql -->
<!-- cap: op:shacl -->
<!-- cap: op:shex -->

```python
gl.sparql(text, "SELECT ?o WHERE { ?s ?p ?o }")["rows"]
gl.shacl(text, shapes_ttl)["conforms"]
gl.shex(text, schema="<http://e/S> {}", map="<http://example.org/a>@<http://e/S>")
```

## Dialects

Every wire dialect is probed by the gate (`convert` then `parse` must preserve the graph id for
RDF dialects; a minimal sample must parse for the others).

| Dialect | Kind | Notes |
|---|---|---|
| `turtle` | RDF | see [DIALECTS](../../../DIALECTS.md) |
| `trig` | RDF | see [DIALECTS](../../../DIALECTS.md) |
| `ntriples` | RDF | see [DIALECTS](../../../DIALECTS.md) |
| `nquads` | RDF | see [DIALECTS](../../../DIALECTS.md) |
| `rdfxml` | RDF | see [DIALECTS](../../../DIALECTS.md) |
| `jsonld` | RDF | see [DIALECTS](../../../DIALECTS.md) |
| `yamlld` | RDF | see [DIALECTS](../../../DIALECTS.md) |
| `trix` | RDF | see [DIALECTS](../../../DIALECTS.md) |
| `hextuples` | RDF | see [DIALECTS](../../../DIALECTS.md) |
| `n3` | other | see [DIALECTS](../../../DIALECTS.md) |
| `sparql` | other | see [DIALECTS](../../../DIALECTS.md) |
| `sql` | other | see [DIALECTS](../../../DIALECTS.md) |
| `shexc` | other | see [DIALECTS](../../../DIALECTS.md) |
| `shexj` | other | see [DIALECTS](../../../DIALECTS.md) |

<!-- cap: dialect:turtle -->
<!-- cap: dialect:trig -->
<!-- cap: dialect:ntriples -->
<!-- cap: dialect:nquads -->
<!-- cap: dialect:rdfxml -->
<!-- cap: dialect:jsonld -->
<!-- cap: dialect:yamlld -->
<!-- cap: dialect:trix -->
<!-- cap: dialect:hextuples -->
<!-- cap: dialect:n3 -->
<!-- cap: dialect:sparql -->
<!-- cap: dialect:sql -->
<!-- cap: dialect:shexc -->
<!-- cap: dialect:shexj -->

## See Also

[Python API](../reference/api.md) | [Dialects](../../../DIALECTS.md)
