#!/usr/bin/env python3
"""Generate graphlaw-derived oracle vectors for wasm4pm-cognition breeds.

Encodes Reiter successor-state progression (situation_calculus breed) and
STRIPS step replay (strips breed) as Datalog in N3, runs the graphlaw
eyeron reasoner to derive holds-at-situation facts, and emits fixture
vectors per wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw/SCHEMA.md (v1).

Usage (from /Users/sac/graphlaw):
  cargo build --release -p graphlaw-eyeron
  python3 scripts/gen_wasm4pm_graphlaw_vectors.py
"""

import argparse
import json
import os
import subprocess
from pathlib import Path

GRAPHLAW = Path("/Users/sac/graphlaw")
EYERON = Path(
    os.environ.get("EYERON_BIN", str(GRAPHLAW / "target/release/graphlaw-eyeron"))
)
DEFAULT_OUT = Path(
    "/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw"
)

PURRDF = "2.0.2"
EYERON_VERSION = "0.7.8"


def graphlaw_rev():
    """Live workspace rev, stamped at generation time (mirrors
    src/bin/dl_dualroute.rs:303 — never a hardcoded constant)."""
    proc = subprocess.run(
        ["git", "-C", str(GRAPHLAW), "rev-parse", "HEAD"],
        capture_output=True, text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"git rev-parse failed: {proc.stderr}")
    return proc.stdout.strip()


def graphlaw_version():
    """Workspace package version from the root Cargo.toml [package]."""
    for line in (GRAPHLAW / "Cargo.toml").read_text().splitlines():
        if line.startswith("version ="):
            return line.split('"')[1]
    raise RuntimeError("no version = line in root Cargo.toml [package]")


def generator_note(rev):
    short = rev[:7]
    return (
        f"graphlaw:scripts/gen_wasm4pm_graphlaw_vectors.py@{short} — encodes "
        "successor-state progression / STRIPS replay as N3 Datalog over explicit "
        "situation constants s0..sN (holds(F,do(A,S)) :- causes(F,A); "
        "holds(F,do(A,S)) :- holds(F,S), keeps(F,A), keeps = closed-world "
        "completion of not causes_del), derives holds fluents with "
        f"graphlaw-eyeron {EYERON_VERSION}, emits SCHEMA.md v1 vectors"
    )


PREFIX = "@prefix : <http://example.org/sc#> ."


def run_eyeron(n3, tag):
    """Run eyeron; return set of (subject, situation) from derived ':holds' lines."""
    n3_path = Path(f"/tmp/glvec_{tag}.n3")
    n3_path.write_text(n3)
    proc = subprocess.run([str(EYERON), str(n3_path)], capture_output=True, text=True)
    if proc.returncode != 0:
        raise RuntimeError(f"eyeron failed on {tag}:\n{proc.stderr}")
    holds = set()
    for line in proc.stdout.splitlines():
        parts = line.split()
        if len(parts) == 4 and parts[1] == ":holds" and parts[0].startswith(":"):
            holds.add((parts[0][1:], parts[2][1:]))
    return holds


def build_sitcalc_n3(fluents, actions, sequence):
    """holds(F,do(A,S)) as N3 Datalog over explicit situation constants s0..sN."""
    universe = set(fluents)
    for d in actions.values():
        universe |= d["pre"] | d["add"] | d["del"]

    lines = [PREFIX, ""]
    for f in sorted(fluents):
        lines.append(f":{f} :holds :s0 .")
    for n, a in enumerate(sequence):
        d = actions[a]
        for f in sorted(d["add"]):
            lines.append(f":{f} :causes :a{n} .")
        for f in sorted(universe - d["del"]):
            lines.append(f":{f} :keeps :a{n} .")
        lines.append("")
        lines.append(
            f"{{ ?f :causes :a{n} }} => {{ ?f :holds :s{n + 1} }} ."
        )
        lines.append(
            f"{{ ?f :holds :s{n} . ?f :keeps :a{n} }} => {{ ?f :holds :s{n + 1} }} ."
        )
    return "\n".join(lines) + "\n", universe


def n3id(s):
    """Sanitize a pred=val atom string into a safe N3 pname local part."""
    import re as _re
    return _re.sub(r"[^A-Za-z0-9]", "_", s)


def build_strips_n3(initial_atoms, actions, plan):
    """STRIPS step replay as N3 Datalog over explicit time constants t0..tN."""
    universe = set(initial_atoms) | {
        x for a in plan for x in actions[a]["add"] + actions[a]["del"]
    }
    lines = [PREFIX, ""]
    for x in sorted(initial_atoms):
        lines.append(f":{n3id(x)} :holds :t0 .")
    for k, a in enumerate(plan):
        d = actions[a]
        for x in sorted(d["add"]):
            lines.append(f":{n3id(x)} :adds :step{k} .")
        for x in sorted(universe - set(d["del"])):
            lines.append(f":{n3id(x)} :keeps :step{k} .")
        lines.append("")
        lines.append(f"{{ ?x :adds :step{k} }} => {{ ?x :holds :t{k + 1} }} .")
        lines.append(
            f"{{ ?x :holds :t{k} . ?x :keeps :step{k} }} "
            f"=> {{ ?x :holds :t{k + 1} }} ."
        )
    return "\n".join(lines) + "\n", universe


def fact(k, v="true"):
    return {"key": k, "value": v}


def sitcalc_breed_facts(dom):
    bfacts = [fact(f"fluent:{f}") for f in sorted(dom["fluents"])]
    for a, d in dom["actions"].items():
        for slot in ("pre", "add", "del"):
            for f in sorted(d[slot]):
                bfacts.append(fact(f"action:{a}:{slot}", f))
    for n, a in enumerate(dom["sequence"]):
        bfacts.append(fact(f"do:{n}", a))
    return bfacts


def emit_vector(out_dir, vector_id, breed, breed_input, derived_facts,
                verdicts, notes, rev):
    vector = {
        "vector_id": vector_id,
        "breed": breed,
        "provenance": {
            "graphlaw_rev": rev,
            "graphlaw_version": graphlaw_version(),
            "purrdf": PURRDF,
            "eyeron": EYERON_VERSION,
            "generator": generator_note(rev),
            "stratified": False,
        },
        "breed_input": breed_input,
        "expected": {
            "derived_facts_sorted": derived_facts,
            "verdicts": verdicts,
            "notes": notes,
        },
    }
    path = out_dir / f"{vector_id}.json"
    path.write_text(json.dumps(vector, indent=2) + "\n")
    print(f"wrote {path}")
    print(f"  derived_facts : {derived_facts}")
    return vector


# ---------------------------------------------------------------- domains --

def domain_sitcalc_paper():
    """Reiter 1991 blocks-world pickup/putdown (mirrors papers fixture)."""
    return {
        "intent": "progress blocks world through pickup(a); putdown(a, table)",
        "fluents": {"on_a_b", "on_b_table", "clear_a", "handempty", "color_b_red"},
        "actions": {
            "pickup_a": {
                "pre": {"clear_a", "handempty", "on_a_b"},
                "add": {"holding_a", "clear_b"},
                "del": {"on_a_b", "handempty", "clear_a"},
            },
            "putdown_a": {
                "pre": {"holding_a"},
                "add": {"on_a_table", "handempty", "clear_a"},
                "del": {"holding_a"},
            },
        },
        "sequence": ["pickup_a", "putdown_a"],
    }


def domain_sitcalc_three_action():
    """3-action progression with deletes: home->office package delivery."""
    return {
        "intent": "3-action package delivery with deletes: load;go;unload",
        "fluents": {"at_home", "has_key", "package_at_home"},
        "actions": {
            "load": {
                "pre": {"at_home", "package_at_home"},
                "add": {"in_car"},
                "del": {"package_at_home"},
            },
            "go": {
                "pre": {"at_home", "has_key"},
                "add": {"at_office"},
                "del": {"at_home"},
            },
            "unload": {
                "pre": {"in_car"},
                "add": {"package_at_office"},
                "del": {"in_car"},
            },
        },
        "sequence": ["load", "go", "unload"],
    }


def domain_sitcalc_frame_heavy():
    """Frame inertia stress: 12 initial fluents, 3 actions touch only 4."""
    return {
        "intent": (
            "frame-persistence stress: 12 initial fluents, 3 actions touch "
            "only f01,f13,f14,f15 — the other 11 persist by inertia"
        ),
        "fluents": {f"f{i:02d}" for i in range(1, 13)},
        "actions": {
            "a1": {"pre": {"f01"}, "add": {"f13"}, "del": {"f01"}},
            "a2": {"pre": {"f13"}, "add": {"f14"}, "del": {"f13"}},
            "a3": {"pre": {"f14"}, "add": {"f15"}, "del": {"f14"}},
        },
        "sequence": ["a1", "a2", "a3"],
    }


def domain_strips_paper():
    actions = {
        "turn-on-light": {"add": ["light=on"], "del": ["light=off"]},
        "close-door1": {"add": ["door1=closed"], "del": ["door1=open"]},
    }
    return {
        "intent": "turn on the light and close door1",
        "rules": [
            {"id": "turn-on-light", "premise": ["light=off"],
             "conclusion": "light=on;!light=off", "certainty": 1.0},
            {"id": "close-door1", "premise": ["door1=open"],
             "conclusion": "door1=closed;!door1=open", "certainty": 1.0},
        ],
        "goals": [
            {"id": "g1", "predicate": "light", "value": "on"},
            {"id": "g2", "predicate": "door1", "value": "closed"},
        ],
        "state": [
            {"predicate": "light", "value": "off"},
            {"predicate": "door1", "value": "open"},
        ],
        "facts": [],
        "actions": actions,
        "plan": ["turn-on-light", "close-door1"],
    }


def domain_strips_planval():
    """Plan-validation vector: goal ORDER drives plan order.

    The strips engine's IDDFS checks each goal-adding operator for
    applicability in the current state (no precondition regression), so the
    expressible class is: each goal's achiever applicable when reached.
    Expected plan: close-door1,turn-on-light (reversed vs paper fixture).
    """
    actions = {
        "close-door1": {"add": ["door1=closed"], "del": ["door1=open"]},
        "smash-door": {"add": ["door1=closed"], "del": ["door1=open"]},
        "turn-on-light": {"add": ["light=on"], "del": ["light=off"]},
        "open-window": {"add": ["window=open"], "del": ["window=shut"]},
    }
    return {
        "intent": (
            "plan validation: given plan close-door1,turn-on-light validated "
            "step-by-step (pre/add/del + goal reach) against the eyeron oracle"
        ),
        "rules": [
            {"id": "close-door1", "premise": ["door1=open"],
             "conclusion": "door1=closed;!door1=open", "certainty": 1.0},
            {"id": "smash-door", "premise": ["door1=open"],
             "conclusion": "door1=closed,!door1=closed_alt;!door1=open",
             "certainty": 1.0},
            {"id": "turn-on-light", "premise": ["light=off"],
             "conclusion": "light=on;!light=off", "certainty": 1.0},
            {"id": "open-window", "premise": ["window=shut"],
             "conclusion": "window=open;!window=shut", "certainty": 1.0},
        ],
        "goals": [
            {"id": "g1", "predicate": "door1", "value": "closed"},
            {"id": "g2", "predicate": "light", "value": "on"},
        ],
        "state": [
            {"predicate": "door1", "value": "open"},
            {"predicate": "light", "value": "off"},
            {"predicate": "window", "value": "shut"},
        ],
        "facts": [fact("frame", "window=shut,close-door1,turn-on-light")],
        "actions": actions,
        "plan": ["close-door1", "turn-on-light"],
    }


def domain_strips_unreachable():
    dom = domain_strips_planval()
    dom["intent"] = (
        "unreachable goal: no operator adds window=closed (strips refusal case)"
    )
    dom["goals"] = [{"id": "g1", "predicate": "window", "value": "closed"}]
    dom["plan"] = None
    return dom




def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(DEFAULT_OUT))
    args = ap.parse_args()
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    rev = graphlaw_rev()

    # ---- situation_calculus vectors -------------------------------------
    for slug, dom_fn in [
        ("paper-blocks-world", domain_sitcalc_paper),
        ("three-action-deletes", domain_sitcalc_three_action),
        ("frame-heavy", domain_sitcalc_frame_heavy),
    ]:
        dom = dom_fn()
        n3, _ = build_sitcalc_n3(dom["fluents"], dom["actions"], dom["sequence"])
        holds = run_eyeron(n3, slug)
        sit_final = f"s{len(dom['sequence'])}"
        holds_final = sorted(f for (f, s_) in holds if s_ == sit_final)
        touched = set()
        for a in dom["sequence"]:
            touched |= dom["actions"][a]["add"] | dom["actions"][a]["del"]
        frame_persist = sorted(dom["fluents"] - touched)

        if slug == "paper-blocks-world":
            papers = json.loads(Path(
                "/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/papers/"
                "situation_calculus.json").read_text())
            assert sorted(holds_final) == sorted(papers["expected"]["holds_final"]), (
                "paper cross-check failed: eyeron != papers fixture")
            print("  paper cross-check vs papers/situation_calculus.json: OK")

        verdicts = {
            "final_situation": f"s{len(dom['sequence'])}",
            "frame_persist": f"{len(frame_persist)} fluents persisted by frame inertia",
        }
        notes = (
            f"Ground truth derived by eyeron N3 progression over explicit "
            f"situation constants s0..{sit_final}. Frame-persist set "
            f"{frame_persist} = initial fluents never mentioned in any "
            f"executed action's add/del lists (hand-checked)."
        )
        breed_input = {
            "intent": dom["intent"],
            "facts": sitcalc_breed_facts(dom),
            "rules": [], "goals": [], "candidates": [], "cases": [], "state": [],
        }
        emit_vector(out_dir, f"situation_calculus-{slug}", "situation_calculus",
                    breed_input,
                    sorted(f"holds:{f}" for f in holds_final),
                    verdicts, notes, rev)

    # ---- strips vectors ---------------------------------------------------
    for slug, dom_fn in [
        ("paper-fikes-nilsson", domain_strips_paper),
        ("plan-validation", domain_strips_planval),
        ("unreachable-goal-refusal", domain_strips_unreachable),
    ]:
        dom = dom_fn()
        initial_atoms = sorted(f"{a['predicate']}={a['value']}" for a in dom["state"])
        plan = dom.get("plan")
        if plan:
            n3, _ = build_strips_n3(initial_atoms, dom["actions"], plan)
            holds = run_eyeron(n3, slug)
            t_final = f"t{len(plan)}"
            all_atoms = set(initial_atoms)
            for a in plan:
                all_atoms |= set(dom["actions"][a]["add"]) | set(dom["actions"][a]["del"])
            back = {n3id(x): x for x in all_atoms}
            final_atoms = sorted(back[x] for (x, t) in holds if t == t_final)
            for g in dom["goals"]:
                goal_atom = f"{g['predicate']}={g['value']}"
                assert goal_atom in final_atoms, (
                    f"plan does not reach goal {goal_atom}; final={final_atoms}")
            print(f"  eyeron plan replay reaches goal: {final_atoms}")
        else:
            final_atoms = []
            print("  refusal vector: no plan; breed must refuse")
        derived = sorted(f"{f['key']}={f['value']}" for f in dom["facts"]) \
            if (plan and dom["facts"]) else []
        verdicts = {}
        if plan:
            verdicts["plan"] = ",".join(plan)
        else:
            verdicts["outcome"] = "refused"
            verdicts["refusal_message"] = "unreachable goal within depth 16"
        notes = (
            "Ground truth derived by eyeron N3 replay of the given plan over "
            "explicit time constants t0..tN; each step's pre/add/del encoded "
            "as adds/keeps Datalog, goal reach asserted at tN."
            if plan
            else "Refusal vector: no operator adds the goal atom, so the "
            "strips breed must fail with 'unreachable goal within depth 16'. "
            "The harness must assert this refusal message on the error path."
        )
        breed_input = {
            "intent": dom["intent"],
            "facts": dom["facts"],
            "rules": dom["rules"],
            "goals": dom["goals"],
            "candidates": [], "cases": [], "state": dom["state"],
        }
        emit_vector(out_dir, f"strips-{slug}", "strips", breed_input,
                    derived, verdicts, notes, rev)


if __name__ == "__main__":
    main()
