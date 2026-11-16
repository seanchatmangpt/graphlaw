# How to entail, run rules, run hooks and admit policies

## RDF/RDFS/OWL-RL entailment
<!-- cap: op:entail -->
<!-- cap: regime:simple -->
<!-- cap: regime:rdf -->
<!-- cap: regime:rdfs -->
<!-- cap: regime:owl-rl -->
<!-- cap: regime:d -->

```python
for regime in ("simple", "rdf", "rdfs", "owl-rl", "d"):
    gl.entail(text, regime)
```

An unknown regime is refused.

## Notation3 rules
<!-- cap: op:n3 -->

```python
gl.n3("{ ?a ?b ?c } => { ?a ?b ?c } .")
```

## Datalog
<!-- cap: op:datalog -->

```python
gl.datalog(rules=[...], facts=[["edge", "a", "b"]])
```

## Knowledge hooks
<!-- cap: op:hooks -->

```python
m = gl.hooks(data, pack)       # pack: kh: hooks as a Turtle spec
m["firings"], m["nquads"]
```

The same pack runs as one receipted transition with `steps.hooks(pack)` inside `gl.law(...)`.

## FOND policies
<!-- cap: op:policy -->

```python
gl.policy(problem, policy)     # JSON or string; admitted as strong-cyclic or refused
```

Refusals name a `policy_kind`; see [handle refusals](handle-refusals.md).

## See Also

[Govern with law steps](govern-with-law-steps.md) | [Python API](../reference/api.md)
