"""UBO sanctions-gate court over the real wasmtime-48 module — pm4pytest
external-oracle driver (EH-26 / lane pm2).

Every collaborator is real: the DuckDB-linked wasm module executes the
sql-in-rdf documents and the recursive datalog unrolling in-module (the same
wire shapes as tests/ubo_sanctions_court.rs); pm4py replays the emitted event
log against a hand-specified Petri net of the lawful UBO gate process and
computes token-replay fitness and ET-conformance precision. No mocks.

Petri net (the lawful process model):
    start -> Request_Received -> Ownership_Resolved -> choice
      refusal path: Sanctioned_Control_Detected -> Refused -> State_Preserved -> end
      lawful path:  Cleared -> Admitted -> Account_Materialized -> Receipt_Sealed -> end

The anti-vacuity trace (broken gate: skips Ownership_Resolved) must NOT fit
(fitness < 1.0) — a court that cannot flip on the broken gate is FAIL.
"""

from __future__ import annotations

import json

import pm4py
import pytest
from pm4py.objects.petri_net.obj import Marking, PetriNet
from pm4py.objects.petri_net.utils import petri_utils

# Wire shapes copied from tests/ubo_sanctions_court.rs.
NS = "http://example.org/ubo#"
SQL = "http://graphlaw.org/sql#"
XSD = "http://www.w3.org/2001/XMLSchema#"
THRESHOLD = 0.25


# ---------------------------------------------------------------------------
# sql-in-rdf documents


def create_doc(table: str, columns: list[tuple[str, str]]) -> str:
    cols = " ".join(
        f'[ sql:columnName "{n}" ; sql:columnType "{t}" ]' for n, t in columns
    )
    return f"""@prefix sql: <{SQL}> .
@prefix ex: <{NS}> .
ex:stmt a sql:CreateStatement ;
    sql:statementIndex 0 ;
    sql:tableDefinition [
        sql:tableName "{table}" ;
        sql:columns ( {cols} ) ] ."""


def insert_doc(table: str, pairs: list[tuple[str, str, str | None]]) -> str:
    body = f"""@prefix sql: <{SQL}> .
@prefix xsd: <{XSD}> .
@prefix ex: <{NS}> .
ex:stmt a sql:InsertStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "{table}" ;
"""
    last = len(pairs) - 1
    for i, pair in enumerate(pairs):
        col, val, dt = (pair + (None,))[:3]
        sep = " ." if i == last else " ;"
        if dt is not None:
            body += f'    sql:assignColumn "{col}" ;\n    sql:assignValue "{val}"^^xsd:{dt}{sep}\n'
        else:
            body += f'    sql:assignColumn "{col}" ;\n    sql:assignValue "{val}"{sep}\n'
    return body


def select_doc(table: str, columns: list[str]) -> str:
    cols = " ".join(f'[ sql:columnName "{c}" ]' for c in columns)
    return f"""@prefix sql: <{SQL}> .
@prefix ex: <{NS}> .
ex:stmt a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "{table}" ] ;
    sql:selectList ( {cols} ) ."""


# ---------------------------------------------------------------------------
# module transport helpers


def exec_ast(mod, session: str, turtle: str) -> dict:
    return mod.request(
        {
            "op": "duckdb",
            "mode": "exec",
            "session": session,
            "query_ast": "urn:ubo:stmt",
            "subject_prefix": "urn:row/",
            "vocab_prefix": "urn:v/",
            "ast": {"turtle": turtle, "dialect": "duckdb"},
        }
    )


def seed_session(mod, session: str) -> None:
    """Real in-module DuckDB schema + ownership/sanctions data."""
    mod.request({"op": "duckdb", "mode": "open", "session": session})
    exec_ast(mod, session, create_doc("ubo_owns", [("child", "VARCHAR"), ("parent", "VARCHAR"), ("pct", "DECIMAL")]))
    exec_ast(mod, session, create_doc("ubo_sanctioned", [("party", "VARCHAR")]))
    exec_ast(mod, session, create_doc("ubo_accounts", [("entity", "VARCHAR"), ("status", "VARCHAR")]))
    for child, parent, pct in [
        ("C", "B", "0.60"),
        ("B", "SDN", "0.50"),
        ("D", "E", "0.20"),
        ("E", "F", "0.30"),
    ]:
        exec_ast(mod, session, insert_doc("ubo_owns", [("child", child), ("parent", parent), ("pct", pct)]))
    exec_ast(mod, session, insert_doc("ubo_sanctioned", [("party", "SDN")]))


def read_table(mod, session: str, table: str, columns: list[str]) -> tuple[int, str]:
    sel = exec_ast(mod, session, select_doc(table, columns))
    n = sel["rows_emitted"]
    assert isinstance(n, int), f"rows_emitted missing from select envelope: {sel}"
    body = stream_all(mod, session, sel["cursor"])
    return n, body


def stream_all(mod, session: str, cursor: str) -> str:
    body = []
    while True:
        page = mod.request(
            {
                "op": "duckdb",
                "mode": "stream",
                "session": session,
                "cursor": cursor,
                "max_rows": 100,
            }
        )
        if isinstance(page.get("nquads"), str):
            body.append(page["nquads"])
        if page.get("more") is not True:
            return "".join(body)


def parse_edges(body: str) -> list[tuple[str, str, float]]:
    """Rows of (child, parent, pct) from a streamed ubo_owns select."""
    import re

    # <row> <urn:v/child> "C" .   (column = last path segment of predicate)
    rows: dict[str, dict] = {}
    for line in body.splitlines():
        line = line.strip()
        if not line.startswith("<"):
            continue
        parts = line.split(" ", 2)
        if len(parts) < 3:
            continue
        col = parts[1].strip("<>").rsplit("/", 1)[-1].rsplit("#", 1)[-1]
        obj = parts[2]
        m = re.match(r'"(.*)"(?:\^\^<[^>]+>)? \.$', obj)
        value = m.group(1) if m else obj[:-2].strip()
        rows.setdefault(parts[0], {})[col] = value
    out = []
    for fields in rows.values():
        if {"child", "parent", "pct"} <= fields.keys():
            out.append((fields["child"], fields["parent"], float(fields["pct"])))
    return out


def effective_pct(edges, root, target) -> float:
    """Best transitive path product (same law as the Rust court's DFS)."""
    best = 0.0

    def dfs(cur, acc, seen):
        nonlocal best
        if cur == target:
            best = max(best, acc)
            return
        if cur in seen:
            return
        seen.add(cur)
        for c, p, pct in edges:
            if c == cur:
                dfs(p, acc * pct, seen | {cur})

    dfs(root, 1.0, set())
    return best


def transitive_control(mod, edges) -> list[tuple[str, str]]:
    """Recursive datalog unrolling in-module: direct + transitive control."""
    facts = [
        [f"urn:ubo:e/{c}", "urn:ubo:owns", f"urn:ubo:e/{p}"] for c, p, _ in edges
    ]
    r = mod.request(
        {
            "op": "datalog",
            "rules": [
                {"head": ["?x", "urn:ubo:control", "?y"], "body": [["?x", "urn:ubo:owns", "?y"]]},
                {
                    "head": ["?x", "urn:ubo:control", "?z"],
                    "body": [["?x", "urn:ubo:control", "?y"], ["?y", "urn:ubo:owns", "?z"]],
                },
            ],
            "facts": facts,
        }
    )
    out = []
    for f in r["facts"]:
        if f[1] == "urn:ubo:control":
            out.append(
                (f[0].removeprefix("urn:ubo:e/"), f[2].removeprefix("urn:ubo:e/"))
            )
    return out


# ---------------------------------------------------------------------------
# gate model (emits the process-mining event log)


class Decision:
    def __init__(self, admitted: bool, events: list[str]):
        self.admitted = admitted
        self.events = events


def run_gate(mod, session: str, applicant: str, *, broken: bool = False) -> Decision:
    """Lawful gate: readback -> datalog unrolling -> threshold + sanctions
    check -> admit or refuse. `broken=True` is the anti-vacuity operator that
    skips the ownership traversal and admits unconditionally."""
    if broken:
        return Decision(True, ["Request_Received", "Cleared", "Admitted"])
    events = ["Request_Received"]
    _, owns_body = read_table(mod, session, "ubo_owns", ["child", "parent", "pct"])
    _, sdn_body = read_table(mod, session, "ubo_sanctioned", ["party"])
    edges = parse_edges(owns_body)
    events.append("Ownership_Resolved")

    control = transitive_control(mod, edges)
    blocked = False
    for who, target in control:
        if who != applicant:
            continue
        if f'"{target}"' in sdn_body and effective_pct(edges, who, target) >= THRESHOLD:
            blocked = True
    if blocked:
        events += ["Sanctioned_Control_Detected", "Refused", "State_Preserved"]
        return Decision(False, events)
    events += ["Cleared", "Admitted"]
    return Decision(True, events)


# ---------------------------------------------------------------------------
# pm4py oracle


def build_petri_net():
    net = PetriNet("ubo_gate")
    start = PetriNet.Place("start")
    p_rr = PetriNet.Place("after_request")
    p_or = PetriNet.Place("after_ownership")
    p_choice = PetriNet.Place("decision")
    p_scd = PetriNet.Place("after_detected")
    p_ref = PetriNet.Place("after_refused")
    p_clr = PetriNet.Place("after_cleared")
    p_adm = PetriNet.Place("after_admitted")
    p_am = PetriNet.Place("after_materialized")
    sink = PetriNet.Place("end")
    for p in (start, p_rr, p_or, p_choice, p_scd, p_ref, p_clr, p_adm, p_am, sink):
        net.places.add(p)

    def t(name, label):
        tr = PetriNet.Transition(name, label)
        net.transitions.add(tr)
        return tr

    t_rr = t("t_rr", "Request_Received")
    t_or = t("t_or", "Ownership_Resolved")
    t_scd = t("t_scd", "Sanctioned_Control_Detected")
    t_ref = t("t_ref", "Refused")
    t_sp = t("t_sp", "State_Preserved")
    t_clr = t("t_clr", "Cleared")
    t_adm = t("t_adm", "Admitted")
    t_am = t("t_am", "Account_Materialized")
    t_rs = t("t_rs", "Receipt_Sealed")

    for a, b in [
        (start, t_rr), (t_rr, p_rr), (p_rr, t_or), (t_or, p_or),
        (p_or, t_scd), (t_scd, p_scd), (p_scd, t_ref), (t_ref, p_ref),
        (p_ref, t_sp), (t_sp, sink),
        (p_or, t_clr), (t_clr, p_clr), (p_clr, t_adm), (t_adm, p_adm),
        (p_adm, t_am), (t_am, p_am), (p_am, t_rs), (t_rs, sink),
    ]:
        petri_utils.add_arc_from_to(a, b, net)

    return net, Marking({start: 1}), Marking({sink: 1})


def trace_log(traces: list[list[str]]):
    from pm4py.objects.log.obj import Event, EventLog, Trace

    log = EventLog()
    for i, events in enumerate(traces):
        tr = Trace(attributes={"concept:name": f"case-{i}"})
        for e in events:
            tr.append(Event({"concept:name": e}))
        log.append(tr)
    return log


def replay_fitness(net, im, fm, log) -> float:
    from pm4py.algo.conformance.tokenreplay import algorithm as tokenreplay
    from pm4py.algo.evaluation.replay_fitness import algorithm as fitness_alg

    aligned_traces = tokenreplay.apply(
        log, net, im, fm,
        parameters={"stop_immediately_when_missing_tokens_are_needed": True},
    )
    stats = fitness_alg.apply(
        log, net, im, fm,
        variant=fitness_alg.Variants.TOKEN_BASED,
        parameters={"align_variant": tokenreplay.Variants.TOKEN_REPLAY},
    )
    return float(stats["log_fitness"])


def et_precision(net, im, fm, log) -> float:
    from pm4py.algo.evaluation.precision import algorithm as precision_alg

    return float(
        precision_alg.apply(
            log, net, im, fm, variant=precision_alg.Variants.ETCONFORMANCE_TOKEN
        )
    )


# ---------------------------------------------------------------------------
# courts


def test_ubo_pm4pytest_sanctions_unrolling(ubo_module):
    mod = ubo_module
    session = "ubo-pm2"
    seed_session(mod, session)

    # Real readback: the four ownership edges are in the module's DuckDB.
    _, owns_body = read_table(mod, session, "ubo_owns", ["child", "parent", "pct"])
    edges = parse_edges(owns_body)
    assert len(edges) == 4, f"four ownership edges read back: {edges}"

    # In-module datalog fixpoint resolves the transitive control edges.
    control = transitive_control(mod, edges)
    assert ("C", "SDN") in control, f"datalog must resolve C->SDN: {control}"
    assert ("D", "F") in control, f"datalog must resolve D->F: {control}"
    assert ("D", "SDN") not in control, f"lawful chain must not reach SDN: {control}"

    pct_c = effective_pct(edges, "C", "SDN")
    assert abs(pct_c - 0.30) < 1e-9 and pct_c >= THRESHOLD, f"C control {pct_c}"
    pct_d = effective_pct(edges, "D", "F")
    assert abs(pct_d - 0.06) < 1e-9 and pct_d < THRESHOLD, f"D control {pct_d}"

    # Court 1: the sanctioned applicant is refused, state preserved.
    d_c = run_gate(mod, session, "C")
    assert d_c.admitted is False, "C must be refused"
    assert d_c.events == [
        "Request_Received", "Ownership_Resolved",
        "Sanctioned_Control_Detected", "Refused", "State_Preserved",
    ], d_c.events
    n_accounts, _ = read_table(mod, session, "ubo_accounts", ["entity", "status"])
    assert n_accounts == 0, "accounts table must be empty after the refusal"

    # Court 2: the lawful applicant is admitted; the mutation seals a receipt.
    d_d = run_gate(mod, session, "D")
    assert d_d.admitted is True, "D must be admitted"
    ins = exec_ast(
        mod, session,
        insert_doc("ubo_accounts", [("entity", "D"), ("status", "OPEN")]),
    )
    assert isinstance(ins.get("receipt"), str) and isinstance(ins.get("query_hash"), str), (
        f"receipt must seal on the admitted mutation: {ins}"
    )
    n_accounts, accounts_body = read_table(mod, session, "ubo_accounts", ["entity", "status"])
    assert n_accounts == 1 and '"D"' in accounts_body and '"OPEN"' in accounts_body, accounts_body
    lawful_events = d_d.events + ["Account_Materialized", "Receipt_Sealed"]
    assert lawful_events == [
        "Request_Received", "Ownership_Resolved", "Cleared", "Admitted",
        "Account_Materialized", "Receipt_Sealed",
    ], lawful_events

    # Court 3: the broken gate skips the traversal and illegally admits C.
    d_broken = run_gate(mod, session, "C", broken=True)
    assert d_broken.admitted is True
    broken_events = d_broken.events

    mod.request({"op": "duckdb", "mode": "close", "session": session})

    # ---- pm4pytest oracle ----
    net, im, fm = build_petri_net()
    good_log = trace_log([d_c.events, lawful_events])
    fitness = replay_fitness(net, im, fm, good_log)
    precision = et_precision(net, im, fm, good_log)
    # Green bar (court-matrix design): fitness = precision = 1.0.
    assert fitness == 1.0, f"lawful gate fitness must be 1.0, got {fitness}"
    assert precision == 1.0, f"lawful gate precision must be 1.0, got {precision}"

    # Anti-vacuity: the broken-gate trace must NOT fit the lawful model.
    broken_fitness = replay_fitness(net, im, fm, trace_log([broken_events]))
    assert broken_fitness < 1.0, (
        f"broken gate (skips Ownership_Resolved) must be detected: fitness {broken_fitness}"
    )

    print(
        f"\nUBO sanctions pm4pytest: fitness={fitness:.3f} precision={precision:.3f} "
        f"broken-gate fitness={broken_fitness:.3f} (lawful traces: 2, broken: 1)"
    )


def test_ubo_pm4pytest_session_isolation(ubo_module):
    """A second session over the same module sees an independent schema —
    the unrolling is per-session state, not module-global."""
    mod = ubo_module
    session = "ubo-pm2-iso"
    seed_session(mod, session)
    n, _ = read_table(mod, session, "ubo_accounts", ["entity", "status"])
    assert n == 0, f"fresh session must start with an empty accounts table: {n}"
    control = transitive_control(
        mod, parse_edges(read_table(mod, session, "ubo_owns", ["child", "parent", "pct"])[1])
    )
    assert ("C", "SDN") in control, f"datalog unrolling works in a fresh session: {control}"
    mod.request({"op": "duckdb", "mode": "close", "session": session})
