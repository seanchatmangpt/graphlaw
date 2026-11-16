# Tutorial: your first GraphLaw session

You will start the packet-4 runner from Python, ask it what it can do, and parse a graph.
<!-- cap: bin:graphlaw-run-eh -->

## 1. Open a session

```python
from graphlaw import GraphLaw
gl = GraphLaw()          # spawns graphlaw-run-eh; reads GRAPHLAW_WASM
```

One process serves many requests. Close it with `gl.close()` or use `with GraphLaw() as gl:`.

## 2. Ask for the capability surface
<!-- cap: op:capabilities -->

```python
caps = gl.capabilities()
print(caps["ops"])           # the 14 registry ops
print(caps["rdf_dialects"])
```

## 3. Parse a graph
<!-- cap: op:parse -->

```python
r = gl.parse("@prefix ex: <http://example.org/> . ex:a ex:b ex:c .", "turtle")
assert r["quads"] == 1       # r["id"] is the canonical blake3 graph id
```

## 4. See a refusal

```python
from graphlaw import GraphLawError
try:
    gl.parse("not rdf <<<", "turtle")
except GraphLawError as e:
    print(e.kind, e.error["message"])   # EngineRejected
```

Refusals are typed data, never silent. Next: [admit a plan with receipts](02-admit-a-plan-with-receipts.md).

## See Also

[Handle refusals](../how-to/handle-refusals.md) | [Python API](../reference/api.md)
