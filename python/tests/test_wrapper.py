"""Real-subprocess courts for the wrapper (no mocks). Loud skip if artifacts are absent."""

import os
import subprocess

import pytest

from graphlaw import GraphLaw, GraphLawError, RunnerExit

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
WASM = os.path.join(REPO, "priv", "graphlaw.wasm")
DIGEST = open(WASM + ".blake3").read().split()[0]
RUNNER = os.path.join(REPO, "target", "debug", "graphlaw-run-eh")

TTL = '@prefix ex: <http://example.org/> . ex:a ex:b ex:c .'

needs = pytest.mark.skipif(
    not (os.path.exists(RUNNER) and os.path.exists(WASM)),
    reason="SKIP: graphlaw-run-eh (cargo build --bins --features runner-eh) or priv/graphlaw.wasm missing",
)


@needs
def test_capabilities_and_parse_one_session():
    with GraphLaw(wasm=WASM, expect_blake3=DIGEST) as gl:
        assert gl.capabilities()
        r = gl.parse(TTL, "turtle")
        assert r["quads"] == 1


@needs
def test_refusal_is_typed_exception():
    with GraphLaw(wasm=WASM) as gl:
        with pytest.raises(GraphLawError):
            gl.op("no-such-op")


@needs
def test_digest_mismatch_is_runner_exit():
    gl = GraphLaw(wasm=WASM, expect_blake3="0" * 64)
    with pytest.raises(RunnerExit) as e:
        gl.capabilities()
    assert e.value.code == 84


SHAPES = """
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:S a sh:NodeShape ; sh:targetNode ex:a ;
  sh:property [ sh:path ex:b ; sh:minCount 2 ] .
"""


@needs
def test_ops_real_results():
    with GraphLaw(wasm=WASM) as gl:
        assert gl.sniff(TTL)["dialect"] == "Turtle"
        c = gl.canonical(TTL, )
        assert c["quads"] == 1 and "example.org" in c["nquads"]
        nt = gl.convert(TTL, "ntriples", dialect="turtle")
        assert "<http://example.org/a>" in nt["text"]
        q = gl.sparql({"text": TTL, "dialect": "turtle"}, "SELECT ?o WHERE { ?s ?p ?o }")
        assert q["kind"] == "solutions" and len(q["rows"]) == 1
        assert gl.sparql(TTL, "ASK { ?s ?p ?o }")["value"] is True
        rep = gl.shacl({"text": TTL, "dialect": "turtle"}, SHAPES)
        assert rep["conforms"] is False and rep["results"]
        assert gl.shacl(TTL, SHAPES.replace("minCount 2", "minCount 1"))["conforms"] is True


@needs
def test_refusals_typed_per_op():
    with GraphLaw(wasm=WASM) as gl:
        for call in (
            lambda: gl.parse("not rdf <<<", "turtle"),
            lambda: gl.sparql(TTL, "SELEKT nonsense"),
            lambda: gl.convert(TTL, "sparql", dialect="turtle"),
            lambda: gl.call({"op": "sparql", "query": "ASK {}"}),
        ):
            with pytest.raises(GraphLawError) as e:
                call()
            assert e.value.kind


@needs
def test_law_shacl_step():
    with GraphLaw(wasm=WASM) as gl:
        ok = gl.law(TTL, [{"step": "shacl", "shapes": SHAPES.replace("minCount 2", "minCount 1")}])
        assert ok
        with pytest.raises(GraphLawError) as e:  # violating graph is refused, not admitted
            gl.law(TTL, [{"step": "shacl", "shapes": SHAPES}])
        assert "SHACL admission refused" in str(e.value)


@needs
def test_duckdb_session_persists_in_process_not_across():
    with GraphLaw(wasm=WASM) as gl:
        gl.duckdb_open("s1")
        with pytest.raises(GraphLawError):  # same name twice in one process
            gl.duckdb_open("s1")
        gl.duckdb_close(session="s1")
    with GraphLaw(wasm=WASM) as gl2:
        gl2.duckdb_open("s1")  # fresh process: name is free -> nothing persisted
        gl2.duckdb_close(session="s1")


FIXTURE = os.path.join(REPO, "target", "debug", "examples", "blackbox_fixture")
VERIFY = os.path.join(REPO, "target", "debug", "graphlaw-verify")


@pytest.mark.skipif(
    not (os.path.exists(FIXTURE) and os.path.exists(VERIFY)),
    reason="SKIP: cargo build --bins --example blackbox_fixture first",
)
def test_verify_receipts_admit_and_refuse(tmp_path):
    from graphlaw import verify_receipts

    out = subprocess.run([FIXTURE, str(tmp_path)], capture_output=True, text=True, check=True)
    key = out.stdout.strip().splitlines()[-1].strip()
    subject = str(tmp_path / "receipts")
    args = (str(tmp_path / "start.nt"), str(tmp_path / "plan.json"), subject)
    assert verify_receipts(*args, trusted_keys=[key]).admitted is True
    bad = verify_receipts(*args, trusted_keys=["00" * 32])
    assert bad.admitted is False and bad.exit_code == 1


PACK_DIR = os.path.join(REPO, "packs", "self-monitoring-pack")


def _ttl(*parts):
    return {"text": open(os.path.join(PACK_DIR, *parts), encoding="utf-8").read(), "dialect": "turtle"}


@needs
def test_knowledge_hooks_materialize_and_law_step():
    pack, data = _ttl("hook.ttl"), _ttl("fixtures", "session-real-broad-topic.ttl")
    with GraphLaw(wasm=WASM) as gl:
        m = gl.hooks(data, pack)
        assert len(m["firings"]) == 3 and all(f["hook"].endswith("#derive_escalation_obligation") for f in m["firings"])
        assert m["quads"] > 0 and "EscalationObligation" in m["nquads"]
        r = gl.law(data, [gl.law_hooks_step(pack)])
        assert r["receipts"][0]["step"] == "derive:hooks" and r["receipts"][0]["added"] > 0
        empty = gl.hooks(data, {"text": "@prefix ex: <http://example.org/> . ex:a ex:b ex:c .", "dialect": "turtle"})
        assert empty["firings"] == []  # a pack with no hooks fires nothing
