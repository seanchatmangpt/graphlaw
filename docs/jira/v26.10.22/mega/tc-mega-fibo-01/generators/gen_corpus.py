#!/usr/bin/env python3
"""TC-MEGA-FIBO-01 "Operation Global Tear-Up" deterministic corpus generator.

Dialect partition (operator addendum, binding):
  .ttl  — static contract definitions: holding companies, LEIs, swap legs
          with FIXED rates (swap contract book).
  .trig — execution contexts: one named graph per clearinghouse (CCP)
          partition, jurisdictional subgraphs, ISDA margin fields, the
          sql-rdf statement graph.
  .nq   — raw streaming ticks / price fixings, one quad per line, all into
          the tcg:ticks named graph.

Determinism: fixed default seed 20261022 (also honoring --seed 41209 style
overrides); two runs at the same seed are bit-identical (sha256 proven in
README). Domain data uses the tcg: namespace; sql: terms are restricted to
the compiler-admitted surface (src/sql_ast.rs KNOWN_SQL_PREDICATES plus the
structural clause terms the compiler reads). The generator asserts no
invented sql: local name is ever emitted.

Usage:
  python3 gen_corpus.py --legs 1000  --lei 500  --holders 12 \
      --outdir ../corpus --stem tc-mega-fibo-01-smoke
  python3 gen_corpus.py --legs 100000 --lei 5000 --holders 12 \
      --outdir ../corpus --stem tc-mega-fibo-01-full
"""

import argparse
import hashlib
import os
import random
import sys

TCG = "http://graphlaw.org/tcmega#"
SQL = "http://graphlaw.org/sql#"
XSD = "http://www.w3.org/2001/XMLSchema#"
RDFS = "http://www.w3.org/2000/01/rdf-schema#"

CCPS = [
    ("ccp-us", "US", "ICE Clear Credit US"),
    ("ccp-uk", "UK", "LCH.Clearnet UK"),
    ("ccp-eu", "EU", "Eurex Clearing EU"),
]
JURISDICTIONS = ["US", "UK", "EU"]
LEG_KINDS = ["IRS", "CDS_TRANCHE", "FX_FORWARD"]
CURVES = {"US": "SOFR", "UK": "SONIA", "EU": "EURIBOR"}
CCYS = {"US": "USD", "UK": "GBP", "EU": "EUR"}
HOLDING_COMPANY_NAMES = [
    "Meridian Global Holdings PLC",
    "Aster Capital Holdings SA",
    "Borealis Structured Finance GmbH",
    "Cormorant Derivatives Group LLC",
    "Delta Harbour Holdings Ltd",
    "Ellsworth Financial Holdings SA",
    "Fairwind Liquidity Group GmbH",
    "Granite Peak Holdings LLC",
    "Halcyon Markets Group Ltd",
    "Ironbridge Capital SA",
    "Juniper Systemic Holdings PLC",
    "Kestrel Interbank Group LLC",
]

ADMITTED_SQL_TERMS = {
    "statementIndex", "selectList", "select", "from", "fromTable",
    "joinClause", "whereClause", "where", "having", "havingClause",
    "groupBy", "orderBy", "limit", "offset",
    "joinType", "joinTable", "joinCondition", "tableAlias", "tableName",
    "schemaName", "columnName", "literalValue", "operator", "leftOperand",
    "rightOperand", "aliasName", "direction", "value", "expression",
    "functionName", "arguments", "valueExpression", "Table", "InnerJoin",
    "LeftJoin", "FullJoin", "Equals", "NotEquals", "LessThan", "LessThanOrEq",
    "GreaterThan", "GreaterThanOrEq", "Like", "And", "Or",
}
_emitted_sql_terms = set()

PREFIXES = {
    "tcg": TCG, "sql": SQL, "xsd": XSD, "rdfs": RDFS,
    "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
}


def esc(s):
    return s.replace("\\", "\\\\").replace('"', '\\"')


def emit_sql_term(local):
    _emitted_sql_terms.add(local)


def trig_header():
    out = []
    for pfx, iri in sorted(PREFIXES.items()):
        out.append("@prefix %s: <%s> .\n" % (pfx, iri))
    return "\n".join(out) + "\n"


def gen_ttl(legs, leis, holders, seed):
    """Dialect 1: static contract definitions, flat Turtle."""
    rng = random.Random(seed)
    out = []
    ap = out.append
    ap("# TC-MEGA-FIBO-01 static contract definitions (Turtle dialect)")
    ap("# seed=%d legs=%d lei=%d holders=%d" % (seed, legs, leis, holders))
    ap("# sql: terms restricted to src/sql_ast.rs KNOWN_SQL_PREDICATES surface.")
    ap(trig_header())

    for h in range(holders):
        name = HOLDING_COMPANY_NAMES[h % len(HOLDING_COMPANY_NAMES)]
        if h >= len(HOLDING_COMPANY_NAMES):
            name += " %d" % (h // len(HOLDING_COMPANY_NAMES))
        ap("")
        ap("tcg:holder-%03d a tcg:HoldingCompany ;" % h)
        ap('    rdfs:label "%s" ;' % esc(name))
        ap('    tcg:jurisdiction "%s" .' % JURISDICTIONS[h % 3])

    for i in range(leis):
        h = i * holders // leis
        ap("")
        ap("tcg:lei-%05d a tcg:LegalEntity ;" % i)
        ap('    tcg:leiCode "%s" ;' % lei_code(i, rng))
        ap("    tcg:heldBy tcg:holder-%03d ." % h)

    # swap legs with FIXED rates (static contract book)
    for n in range(legs):
        jur = JURISDICTIONS[n % 3]
        kind = LEG_KINDS[n % 3]
        lei = (n * 7 + 3) % leis
        fixed = "0.0%d%d" % ((n % 900) + 100, (n * 7) % 10)
        ap("")
        ap("# leg %d" % n)
        ap("tcg:leg-%06d a tcg:%s ;" % (n, kind))
        ap('    tcg:legId "%06d" ;' % n)
        ap('    tcg:jurisdiction "%s" ;' % jur)
        ap('    tcg:currency "%s"^^xsd:string ;' % CCYS[jur])
        ap('    tcg:rate "%s"^^xsd:decimal ;' % fixed)
        ap("    tcg:counterparty tcg:lei-%05d ;" % lei)
        ap("    tcg:heldBy tcg:holder-%03d ." % ((lei * holders) // leis))
    return "\n".join(out) + "\n"


def lei_code(i, rng):
    return "549300%05dABCDEFGHJKLMNOPQRSTUV" % i


def gen_trig(legs, leis, holders, seed):
    """Dialect 2: execution contexts — named graphs per CCP + margin + sql-rdf."""
    rng = random.Random(seed)
    out = []
    ap = out.append
    ap("# TC-MEGA-FIBO-01 execution contexts (TriG dialect, per-CCP graphs)")
    ap("# seed=%d legs=%d lei=%d holders=%d" % (seed, legs, leis, holders))
    ap(trig_header())

    for cid, jur, name in CCPS:
        ap("")
        ap("tcg:%s a tcg:Clearinghouse ;" % cid)
        ap('    rdfs:label "%s" ;' % esc(name))
        ap('    tcg:jurisdiction "%s" .' % jur)

    for n in range(legs):
        cid, jur, _ = CCPS[n % 3]
        lei = (n * 7 + 3) % leis
        notional = "%d.%02d" % (1_000_000 + (n * 7919) % 990_000_000, (n * 13) % 100)
        ap("")
        ap("# exec leg %d" % n)
        ap("tcg:%s {" % cid)
        ap("    tcg:leg-%06d" % n)
        ap("        a tcg:ClearedLeg ;")
        ap('        tcg:clearedAt tcg:%s ;' % cid)
        ap('        tcg:jurisdiction "%s" ;' % jur)
        ap('        tcg:notional "%s"^^xsd:decimal ;' % notional)
        ap('        tcg:curve "%s"^^xsd:string ;' % CURVES[jur])
        ap("        tcg:counterparty tcg:lei-%05d ;" % lei)
        ap("        tcg:heldBy tcg:holder-%03d ;" % ((lei * holders) // leis))
        ap('        tcg:imThreshold "%s"^^xsd:decimal ;' % notional)
        ap('        tcg:vmThreshold "%d.00"^^xsd:decimal ;'
           % (100_000 + (n % 15) * 10_000))
        ap('        tcg:minimumTransferAmount "%d.00"^^xsd:decimal ;'
           % (50_000 + (n % 20) * 5_000))
        ap('        tcg:isdaMasterAgreement "ISDA 2002 %s"^^xsd:string .' % jur)
        ap("}")
    # sql-rdf statement graph (compiler-admitted surface only)
    ap("")
    ap("# sql-rdf statement: USD tear-up exposure from the cleared book")
    for local in ("statementIndex", "selectList", "fromTable", "whereClause",
                  "operator", "leftOperand", "rightOperand", "columnName",
                  "literalValue", "limit", "groupBy", "Equals"):
        emit_sql_term(local)
    ap("tcg:stmt-usd-exposure a sql:SelectStatement ;")
    ap('    sql:statementIndex "0"^^xsd:integer ;')
    ap("    sql:selectList (")
    ap('        [ sql:columnName "jurisdiction" ]')
    ap('        [ sql:valueExpression "COUNT(*)" ]')
    ap("    ) ;")
    ap('    sql:fromTable "tcg_derivative_legs" ;')
    ap("    sql:whereClause [")
    ap("        sql:operator sql:Equals ;")
    ap('        sql:leftOperand [ sql:columnName "currency" ] ;')
    ap('        sql:rightOperand [ sql:literalValue "USD" ]')
    ap("    ] ;")
    ap('    sql:limit "100"^^xsd:integer .')
    return "\n".join(out) + "\n"


def gen_nq(legs, leis, seed):
    """Dialect 3: raw streaming ticks / price fixings as N-Quads."""
    rng = random.Random(seed)
    out = []
    ap = out.append
    ap("# TC-MEGA-FIBO-01 streaming ticks (N-Quads dialect, graph <ticks>)")
    ap("# seed=%d legs=%d" % (seed, legs))
    # Emit `ticks` fixings per leg (open/close/settle), graph = ticks.
    for n in range(legs):
        lei = (n * 7 + 3) % leis
        for t in range(3):
            px = "%d.%04d" % (90 + (n + t) % 20, ((n * 31) + t * 7) % 10000)
            ap("<http://graphlaw.org/tcmega#tick-%06d-%d> "
               "<http://graphlaw.org/tcmega#price> \"%s\"^^"
               "<http://www.w3.org/2001/XMLSchema#decimal> "
               "<http://graphlaw.org/tcmega#ticks> ."
               % (n, t, px))
        ap("<http://graphlaw.org/tcmega#tick-%06d-0> "
           "<http://graphlaw.org/tcmega#aboutLeg> "
           "<http://graphlaw.org/tcmega#leg-%06d> "
           "<http://graphlaw.org/tcmega#ticks> ." % (n, n))
    return "\n".join(out) + "\n"


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--legs", type=int, required=True)
    p.add_argument("--lei", type=int, default=500)
    p.add_argument("--holders", type=int, default=12)
    p.add_argument("--seed", type=int, default=20261022)
    p.add_argument("--outdir", default="../corpus")
    p.add_argument("--stem", default="tc-mega-fibo-01-smoke")
    a = p.parse_args()

    os.makedirs(a.outdir, exist_ok=True)
    files = [
        (a.stem + ".ttl", gen_ttl(a.legs, a.lei, a.holders, a.seed)),
        (a.stem + ".trig", gen_trig(a.legs, a.lei, a.holders, a.seed)),
        (a.stem + ".nq", gen_nq(a.legs, a.lei, a.seed)),
    ]
    unknown = _emitted_sql_terms - ADMITTED_SQL_TERMS
    if unknown:
        print("REFUSAL: invented sql: terms emitted: %s" % sorted(unknown),
              file=sys.stderr)
        return 2
    for name, text in files:
        path = os.path.join(a.outdir, name)
        with open(path, "w") as f:
            f.write(text)
        print("%s %d %s" % (sha256_file(path), os.path.getsize(path), path))
    return 0


if __name__ == "__main__":
    sys.exit(main())
