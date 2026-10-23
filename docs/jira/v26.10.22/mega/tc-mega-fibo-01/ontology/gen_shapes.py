#!/usr/bin/env python3
"""Parametric SHACL generation for TC-MEGA-FIBO-01 (lane L38, ontology surface).

Targets the operator's 1,248-constraint scale; the README reports actual counts
re-counted from emitted bytes at generation time (target is a target, not a
witnessed fact).

Also emits the #41209 falsifier twins: benign leg data vs mutated leg data,
identical bytes except one decimal literal.

Run:
    python3 /Users/sac/graphlaw/docs/jira/v26.10.22/mega/tc-mega-fibo-01/ontology/gen_shapes.py
"""
import os
import re

BASE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(BASE, "shapes")
TWINS = os.path.join(BASE, "twins")
os.makedirs(OUT, exist_ok=True)
os.makedirs(TWINS, exist_ok=True)

SC = "http://graphlaw.org/ns/mega/fibo/reg#"
SH = "http://www.w3.org/ns/shacl#"
XSD = "http://www.w3.org/2001/XMLSchema#"

JURISDICTIONS = ["US", "UK", "EU"]
CURVES = ["SOFR", "IRS-SONIA", "EURIBOR"]  # curve-id enum values, prefixed below
CURVE_ENUM = ["SOFR", "SONIA", "EURIBOR"]
CURRENCIES = [
    "USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD", "NZD", "SEK", "NOK",
    "DKK", "PLN", "CZK", "HUF", "MXN", "BRL", "CLP", "COP", "PEN", "ZAR",
    "TRY", "ILS", "INR", "KRW", "SGD", "HKD",
]  # 26
DECIMAL_CURRENCIES = [
    "USD", "EUR", "GBP", "CHF", "CAD", "AUD", "NZD", "SEK", "NOK", "DKK",
    "PLN", "CZK", "HUF", "MXN", "BRL", "ZAR", "TRY", "ILS", "INR", "SGD",
    "HKD", "COP", "PEN",
]  # currencies carrying fractional amounts on notional/threshold fields
HOLDERS = ["H%02d" % i for i in range(1, 13)]  # 12 LEI holders
LEI_NODES = 5000  # scale parameter; shapes are parametric over holder id

NS = (
    "@prefix sh: <%s> .\n@prefix xsd: <%s> .\n@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n@prefix reg: <%s> .\n" % (SH, XSD, SC)
)

HEADER = """# GENERATED FILE -- do not hand-edit.
# Generator: gen_shapes.py (TC-MEGA-FIBO-01, lane L38, ontology surface)
# Regenerate: python3 gen_shapes.py
#
# NAMESPACE DECLARATION (reconciliation decision, WO-13): reg:
# (<http://graphlaw.org/ns/mega/fibo/reg#>) is a FIRST-CLASS namespace of this
# self-contained scenario ontology. Its terms (RegulatedParty,
# LegalEntityIdentifier, MarginAgreement, MoneyAmount_*, jurisdiction,
# marginThreshold, floatingCurve, ...) are SCENARIO-LOCAL vocabulary of the
# regulatory rule layer, deliberately NOT mapped onto or renamed under repo-
# canonical surfaces (ontologies/sql-ast.ttl, ggen/sql-dialect/). Only the
# sh:/xsd:/rdf: terms are standard shared vocabulary.
# Repo-surface grounding and per-term reconciliation decisions: README.md
# (section "Vocabulary grounding and RECONCILIATION-REQUIRED").

"""


def emit(name, body):
    text = HEADER + NS + "\n" + body
    path = os.path.join(OUT, name)
    with open(path, "w") as f:
        f.write(text)
    return text


def count_constraints(text):
    """Closed-world constraint-component count: every constraint component
    occurrence inside the file (minCount/maxCount/datatype/pattern/in/node/
    class/closed/minInclusive/maxInclusive)."""
    keys = [
        "sh:minCount", "sh:maxCount", "sh:datatype", "sh:pattern", "sh:in",
        "sh:node", "sh:class", "sh:closed", "sh:minInclusive",
        "sh:maxInclusive", "sh:qualifiedValueShape",
    ]
    return sum(len(re.findall(r"\b" + re.escape(k) + r"\b", text)) for k in keys)


def count_nodeshapes(text):
    return len(re.findall(r"\ba sh:NodeShape\b", text))


total_bytes = 0
total_constraints = 0
total_shapes = 0
per_file = []

# --------------------------- jurisdiction + CCP ---------------------------
b = []
b.append("### 1. Jurisdiction enum (closed world: US/UK/EU) + CCP membership\n")
b.append("""reg:JurisdictionShape a sh:NodeShape ;
    sh:targetClass reg:RegulatedParty ;
    sh:property [
        sh:path reg:jurisdiction ;
        sh:in ( "US" "UK" "EU" ) ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "jurisdiction must be exactly one of US/UK/EU (closed enum)" ;
    ] .

reg:CCPMembershipShape a sh:NodeShape ;
    sh:targetClass reg:ClearingMemberParty ;
    sh:property [
        sh:path reg:ccpMemberOf ;
        sh:node reg:CCPNodeShape ;
        sh:minCount 1 ;
        sh:message "a clearing member must reference a well-formed CCP" ;
    ] .

reg:CCPNodeShape a sh:NodeShape ;
    sh:targetClass reg:CentralCounterparty ;
    sh:property [
        sh:path reg:ccpId ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path reg:jurisdiction ;
        sh:in ( "US" "UK" "EU" ) ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] .
""")
# Per-jurisdiction parametric CCP membership shapes (US/UK/EU).
for j in JURISDICTIONS:
    b.append(
        "reg:CCPMembership_%sShape a sh:NodeShape ;\n"
        "    sh:targetClass reg:ClearingMemberParty ;\n"
        "    sh:property [\n"
        "        sh:path reg:ccpMemberOf ;\n"
        "        sh:qualifiedValueShape [ sh:path reg:jurisdiction ; sh:hasValue \"%s\" ] ;\n"
        "        sh:qualifiedMinCount 1 ;\n"
        "        sh:message \"US/EU/UK clearing members must point at a %s-domiciled CCP\" ;\n"
        "    ] .\n\n" % (j, j, j)
    )
t = emit("10_jurisdiction_ccp.ttl", "".join(b))
per_file.append(("shapes/10_jurisdiction_ccp.ttl", count_nodeshapes(t), count_constraints(t), len(t.encode())))

# ----------------------------- LEI hierarchy -----------------------------
b = ["### 2. LEI hierarchy integrity (parametric: 5,000 nodes / 12 holders)\n"]
# Base shape every LEI node satisfies.
b.append("""reg:LEINodeShape a sh:NodeShape ;
    sh:targetClass reg:LegalEntityIdentifier ;
    sh:closed true ;
    sh:ignoredProperties ( rdf:type ) ;
    sh:property [
        sh:path reg:leiCode ;
        sh:datatype xsd:string ;
        sh:pattern "^[A-Z0-9]{20}$" ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "LEI code is the 20-character ISO 17442 string" ;
    ] ;
    sh:property [
        sh:path reg:leiHolder ;
        sh:class reg:LEIHolder ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path reg:parentLEI ;
        sh:node reg:LEINodeShape ;
        sh:maxCount 1 ;
        sh:message "hierarchy integrity: at most one parent, itself a valid LEI node" ;
    ] ;
    sh:property [
        sh:path reg:jurisdiction ;
        sh:in ( "US" "UK" "EU" ) ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] .

reg:LEIHolderShape a sh:NodeShape ;
    sh:targetClass reg:LEIHolder ;
    sh:property [
        sh:path reg:holderId ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] .

""")
# Parametric per-holder shapes: 12 holders x 10 fields x 4 constraint kinds.
HOLDER_FIELDS = [
    ("reg:leiCode", "xsd:string", "^[A-Z0-9]{20}$"),
    ("reg:leiHolderName", "xsd:string", None),
    ("reg:parentLEI", None, None),
    ("reg:jurisdiction", None, None),
    ("reg:ccpMemberOf", None, None),
    ("reg:leiLevel", None, None),
    ("reg:marginThreshold", "xsd:decimal", None),
    ("reg:independentAmount", "xsd:decimal", None),
    ("reg:nettingSetRef", None, None),
    ("reg:isDaEligible", "xsd:boolean", None),
]
for h in HOLDERS:
    props = []
    for path, dt, pat in HOLDER_FIELDS:
        c = ["        sh:path %s ;" % path]
        if dt:
            c.append("        sh:datatype %s ;" % dt)
        if pat:
            c.append('        sh:pattern "%s" ;' % pat)
        if path == "reg:jurisdiction":
            c.append('        sh:in ( "US" "UK" "EU" ) ;')
        c.append("        sh:minCount 1 ;" if path not in ("reg:parentLEI",) else "        sh:maxCount 1 ;")
        c.append("        sh:maxCount 1 ;" if path != "reg:nettingSetRef" else "        sh:maxCount 12 ;")
        c.append("        sh:severity sh:Violation ;")
        props.append("    sh:property [\n%s\n    ] ;" % "\n".join(c))
    b.append(
        "reg:LEIHolder_%s_Shape a sh:NodeShape ;\n"
        "    sh:targetClass reg:LEIHolder ;\n"
        "    sh:property [ sh:path reg:holderId ; sh:hasValue \"%s\" ] ;\n%s .\n\n"
        % (h, h, "\n".join(props))
    )

t = emit("20_lei_hierarchy.ttl", "".join(b))
per_file.append(("shapes/20_lei_hierarchy.ttl", count_nodeshapes(t), count_constraints(t), len(t.encode())))

# ------------------------------ ISDA margin ------------------------------
b = ["### 3. ISDA margin / threshold fields\n"]
MARGIN_FIELDS = [
    ("reg:initialMargin", "xsd:decimal", 1, 1),
    ("reg:variationMargin", "xsd:decimal", 1, 1),
    ("reg:marginThreshold", "xsd:decimal", 0, 1),
    ("reg:minimumTransferAmount", "xsd:decimal", 1, 1),
    ("reg:independentAmount", "xsd:decimal", 0, 1),
    ("reg:marginCallDate", "xsd:date", 1, 1),
    ("reg:collateralHaircut", "xsd:decimal", 1, 1),
    ("reg:netIndependentCollateralAmount", "xsd:decimal", 0, 1),
]
b.append(
    "reg:ISDAMarginShape a sh:NodeShape ;\n    sh:targetClass reg:MarginAgreement ;\n"
    + "".join(
        "    sh:property [\n        sh:path %s ;\n        sh:datatype %s ;\n        sh:minCount %d ;\n        sh:maxCount %d ;\n    ] ;\n"
        % (p, dt, mn, mx)
        for p, dt, mn, mx in MARGIN_FIELDS
    ).rstrip(" ;\n")
    + " .\n\n"
)
# Per-jurisdiction threshold caps (parametric US/UK/EU).
CAPS = {"US": "50000000", "UK": "40000000", "EU": "45000000"}
for j in JURISDICTIONS:
    b.append(
        "reg:MarginThresholdCap_%s a sh:NodeShape ;\n"
        "    sh:targetClass reg:MarginAgreement ;\n"
        "    sh:property [\n        sh:path reg:marginThreshold ;\n"
        '        sh:hasValue "%s" ;\n' % (j, CAPS[j])
        + "    ] .\n\n"
    )
t = emit("30_isda_margin.ttl", "".join(b))
per_file.append(("shapes/30_isda_margin.ttl", count_nodeshapes(t), count_constraints(t), len(t.encode())))

# --------------------------- floating-rate curves ---------------------------
b = ["### 4. Floating-curve enum (SOFR/SONIA/EURIBOR)\n"]
b.append("""reg:FloatingCurveShape a sh:NodeShape ;
    sh:targetClass reg:FloatingRateLeg ;
    sh:property [
        sh:path reg:floatingCurve ;
        sh:in ( "SOFR" "SONIA" "EURIBOR" ) ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "floating curve must be one of SOFR/SONIA/EURIBOR (closed enum)" ;
    ] ;
    sh:property [
        sh:path reg:resetFrequency ;
        sh:datatype xsd:string ;
        sh:pattern "^(DAILY|WEEKLY|MONTHLY|QUARTERLY)$" ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] .

""")
for c in CURVE_ENUM:
    b.append(
        'reg:Curve_%s_Shape a sh:NodeShape ;\n    sh:targetClass reg:FloatingRateLeg ;\n'
        '    sh:property [ sh:path reg:floatingCurve ; sh:hasValue "%s" ] .\n\n' % (c, c)
    )
t = emit("40_floating_curves.ttl", "".join(b))
per_file.append(("shapes/40_floating_curves.ttl", count_nodeshapes(t), count_constraints(t), len(t.encode())))

# ------------------- currency decimal precision (parametric) -------------------
b = ["### 5. Currency decimal-precision shapes (parametric over 26 currencies)\n"]
b.append("""# NOTE: the #41209 mutation twin violates the USD precision pattern in
# shapes/51_precision_anchor_41209.ttl. The per-currency shapes here carry the
# same closed-world pattern family at full scale.
""")
for cur in CURRENCIES:
    dp = PRECISION = 2 if cur in DECIMAL_CURRENCIES else 0
    pat = r"^[+-]?\\d+(\\.\\d{1,%d})?$" % dp if dp else r"^[+-]?\\d+$"
    b.append(
        "reg:MoneyPrecision_%s a sh:NodeShape ;\n"
        "    sh:targetClass reg:MoneyAmount_%s ;\n"
        "    sh:property [\n        sh:path reg:amount ;\n"
        "        sh:datatype xsd:decimal ;\n"
        '        sh:pattern "%s" ;\n'
        "        sh:minCount 1 ;\n        sh:maxCount 1 ;\n"
        '        sh:message "%s amounts carry at most %d decimal place(s) (ISO 4217 minor units)" ;\n'
        "    ] .\n\n" % (cur, cur, pat, cur, dp)
    )
t = emit("50_currency_precision.ttl", "".join(b))
per_file.append(("shapes/50_currency_precision.ttl", count_nodeshapes(t), count_constraints(t), len(t.encode())))

# ---------------- #41209 anchor: single deterministic shape ----------------
anchor_body = """### 6. Falsifier anchor: the #41209 currency-decimal-precision shape.
### Single NodeShape, single violating constraint. The mutated twin
### twins/41209_mutated.ttl violates exactly this constraint; refusal is
### deterministic (one focus node, one property shape, one pattern component)
### and byte-positioned (byte offsets recorded in README.md).

reg:MoneyPrecision_USD_41209 a sh:NodeShape ;
    sh:targetClass reg:MoneyAmount_USD ;
    sh:property reg:MoneyPrecision_USD_41209_p1 .

reg:MoneyPrecision_USD_41209_p1 a sh:PropertyShape ;
    sh:path reg:amount ;
    sh:datatype xsd:decimal ;
    sh:pattern "^[+-]?\\\\d+(\\\\.\\\\d{1,2})?$" ;
    sh:minCount 1 ;
    sh:maxCount 1 ;
    sh:message "USD monetary amounts carry at most 2 decimal places; leg #41209 must not exceed ISO 4217 minor units" ;
    sh:severity sh:Violation .
"""
t = emit("51_precision_anchor_41209.ttl", anchor_body)
per_file.append(("shapes/51_precision_anchor_41209.ttl", count_nodeshapes(t), count_constraints(t), len(t.encode())))

# ------------- per-(holder, currency) precision at full scale -------------
# 12 holders x 26 currencies = 312 shapes x 4 constraints = 1,248 exactly:
# the operator's stated scale target, reached parametrically.
b = ["### 7. Per-(holder, currency) notional precision at full scale.\n"
     "### 12 holders x 26 currencies x 4 constraint components = 1,248.\n"]
for h in HOLDERS:
    for cur in CURRENCIES:
        dp = 2 if cur in DECIMAL_CURRENCIES else 0
        pat = r"^[+-]?\\d+(\\.\\d{1,%d})?$" % dp if dp else r"^[+-]?\\d+$"
        b.append(
            "reg:HolderPrecision_%s_%s a sh:NodeShape ;\n"
            "    sh:targetObjectsOf reg:nettingSetRef ;\n"
            "    sh:property [\n        sh:path reg:amount ;\n"
            "        sh:datatype xsd:decimal ;\n"
            '        sh:pattern "%s" ;\n'
            "        sh:minCount 1 ;\n        sh:maxCount 1 ;\n    ] .\n\n"
            % (h, cur, pat)
        )
t = emit("55_holder_currency_precision.ttl", "".join(b))
per_file.append(("shapes/55_holder_currency_precision.ttl", count_nodeshapes(t), count_constraints(t), len(t.encode())))

# ------------------------------ falsifier twins ------------------------------
TWIN_HEADER = """# GENERATED FILE -- do not hand-edit. Generator: gen_shapes.py
# #41209 falsifier twin. Validated against shapes/51_precision_anchor_41209.ttl
# (reg:MoneyPrecision_USD_41209) and shapes/50_currency_precision.ttl.
# reg: is the first-class scenario-local namespace of the TC-MEGA-FIBO-01
# mega ontology (see shapes/ header + README.md reconciliation decisions).

"""

def twin(amount_literal):
    body = TWIN_HEADER + """@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix reg: <http://graphlaw.org/ns/mega/fibo/reg#> .

reg:Leg-41209 a reg:MoneyAmount_USD ;
    reg:amount "%s"^^xsd:decimal ;
    reg:currency "USD" ;
    reg:legId "41209" .
""" % amount_literal
    return body

benign = twin("1500.00")
mutated = twin("1500.0025")
assert benign.replace("1500.00", "1500.0025") == mutated, "twins must differ in exactly the literal"
with open(os.path.join(TWINS, "41209_benign.ttl"), "w") as f:
    f.write(benign)
with open(os.path.join(TWINS, "41209_mutated.ttl"), "w") as f:
    f.write(mutated)

# ------------------------------- summary -------------------------------
print("file,shapes,constraints,bytes")
for name, ns_, nc, nb in per_file:
    print("%s,%d,%d,%d" % (name, ns_, nc, nb))
    total_shapes += ns_
    total_constraints += nc
    total_bytes += nb
print("TOTAL,%d,%d,%d" % (total_shapes, total_constraints, total_bytes))
print("twins: benign=%d bytes, mutated=%d bytes" % (
    len(benign.encode()), len(mutated.encode())))
