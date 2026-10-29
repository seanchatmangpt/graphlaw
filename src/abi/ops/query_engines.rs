use purrdf::{SparqlEngine, SparqlRequest, SparqlResult};
use serde_json::{Value, json};

use super::super::{
    Fail, MAX_DATALOG_FACTS, MAX_DATALOG_RULES, Res, bad, check_limit, limit, nquads_of, opt_str,
    state_field, str_field, term_json,
};
use crate::dialect::{Dialect, Engine, Refusal, RefusalKind};

pub(in crate::abi) fn op_sparql(v: &Value) -> Res<Value> {
    let data = state_field(v, "data")?;
    let query = str_field(v, "query")?;
    let result = crate::sparql::NativeSparqlEngine::new()
        .query(
            data.dataset(),
            SparqlRequest {
                query,
                base_iri: opt_str(v, "base"),
                substitutions: &[],
            },
        )
        .map_err(|e| Refusal::engine(Dialect::Sparql, e))?;
    Ok(match result {
        SparqlResult::Solutions {
            variables, rows, ..
        } => json!({
            "kind": "solutions",
            "variables": variables,
            "rows": rows.iter().map(|r| r.iter().map(|c| c.as_ref().map_or(Value::Null, term_json)).collect::<Vec<_>>()).collect::<Vec<_>>(),
        }),
        SparqlResult::Graph(g) => {
            json!({"kind": "graph", "nquads": nquads_of(&g)?, "quads": g.quad_count()})
        }
        SparqlResult::Boolean(b) => json!({"kind": "boolean", "value": b}),
    })
}

pub(in crate::abi) fn op_shacl(v: &Value) -> Res<Value> {
    let data = state_field(v, "data")?;
    let shapes_text = str_field(v, "shapes")?;
    // Bound shape-nesting depth / degree / quads / bytes BEFORE the evaluator
    // recurses (see law.rs Step::AdmitShacl for the same guard, rendered
    // there as a SHACL result); here it surfaces as the typed ResourceLimit
    // refusal.
    if let Err(over) = crate::shacl_guard::check_shape_limits(shapes_text) {
        return Err(limit(
            over.limit_name(),
            over.observed(),
            over.cap(),
        ));
    }
    let shapes = crate::shacl::engine::parse_shapes(shapes_text, opt_str(v, "base"))
        .map_err(|e| Refusal::engine(Dialect::Turtle, e))?;
    let report = crate::shacl::engine::validate_dataset(data.dataset(), &shapes)
        .map_err(|e| Refusal::engine(Dialect::Turtle, e))?;
    Ok(json!({
        "conforms": report.conforms,
        "results": report.results.iter().map(|r| json!({
            "focus": r.focus_node.to_string(),
            "path": r.result_path.as_ref().map(ToString::to_string),
            "value": r.value.as_ref().map(ToString::to_string),
            "severity": format!("{:?}", r.severity),
            "component": r.source_constraint_component.to_string(),
            "shape": r.source_shape.to_string(),
            "message": r.messages.first().map(|l| l.value().to_string()),
        })).collect::<Vec<_>>(),
    }))
}

pub(in crate::abi) fn op_shex(v: &Value) -> Res<Value> {
    let data = state_field(v, "data")?;
    let text = str_field(v, "schema")?;
    let base = opt_str(v, "base");
    let (dialect, schema) = match opt_str(v, "schema_dialect").unwrap_or("shexc") {
        "shexc" => (Dialect::ShExC, crate::shex::parse_shexc(text, base)),
        "shexj" => (Dialect::ShExJ, crate::shex::parse_shexj(text, base)),
        other => return Err(bad(format!("unknown schema_dialect `{other}`"))),
    };
    let schema = schema.map_err(|e| Refusal::engine(dialect, e))?;
    crate::shex::check_structure(&schema).map_err(|e| Refusal::engine(dialect, e))?;
    let map = crate::shex::validate_shape_map(
        &schema,
        data.dataset(),
        str_field(v, "map")?,
        base,
        &crate::shex::ValidationOptions::default(),
    )
    .map_err(|e| Refusal::engine(dialect, e))?;
    Ok(json!({
        "conforms": map.all_conformant(),
        "entries": map.entries.iter().map(|e| json!({
            "node": term_json(&e.node),
            "shape": format!("{:?}", e.shape),
            "status": format!("{:?}", e.status),
            "reason": e.reason,
        })).collect::<Vec<_>>(),
    }))
}

pub(in crate::abi) fn op_entail(v: &Value) -> Res<Value> {
    use crate::entailment::Materialization as M;
    let data = state_field(v, "data")?;
    let plan = match str_field(v, "regime")? {
        "simple" => M::Simple,
        "rdf" => M::Rdf,
        "rdfs" => M::Rdfs,
        "owl-rl" => M::OwlRl,
        "d" => M::D,
        other => return Err(bad(format!("unknown regime `{other}`"))),
    };
    let (closure, _report) = crate::entailment::materialize(data.dataset().as_ref(), plan)
        .map_err(|e| Refusal::engine(Dialect::Turtle, e))?;
    Ok(json!({
        "nquads": nquads_of(&closure)?,
        "added": closure.quad_count().saturating_sub(data.quad_count()),
    }))
}

fn clause_term(t: &Value) -> Res<crate::datalog::clause::ClauseTerm> {
    use crate::datalog::clause::ClauseTerm;
    let s = t.as_str().ok_or_else(|| bad("datalog terms are strings"))?;
    Ok(if let Some(var) = s.strip_prefix('?') {
        ClauseTerm::var(var)
    } else if s.starts_with('"') {
        ClauseTerm::literal(s)
    } else {
        ClauseTerm::iri(s)
    })
}

fn triple(t: &Value) -> Res<[&Value; 3]> {
    match t.as_array().map(Vec::as_slice) {
        Some([s, p, o]) => Ok([s, p, o]),
        _ => Err(bad("expected a [subject, predicate, object] triple")),
    }
}

fn atom(t: &Value) -> Res<crate::datalog::clause::ClauseAtom> {
    let [s, p, o] = triple(t)?;
    let p = p
        .as_str()
        .ok_or_else(|| bad("datalog predicate must be an IRI string"))?;
    Ok(crate::datalog::clause::ClauseAtom::positive(
        clause_term(s)?,
        p,
        clause_term(o)?,
    ))
}

fn surface(t: &Value) -> Res<String> {
    let s = t
        .as_str()
        .ok_or_else(|| bad("datalog fact terms are strings"))?;
    Ok(if s.starts_with('"') {
        s.to_string()
    } else {
        format!("<{s}>")
    })
}

fn unsurface(s: &str) -> String {
    s.strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(s)
        .to_string()
}

pub(in crate::abi) fn op_datalog(v: &Value) -> Res<Value> {
    use crate::datalog::{
        clause::DlClause,
        seminaive::{compile, evaluate},
        store::RelationStore,
    };
    let dl = |e: &dyn std::fmt::Debug| Refusal {
        kind: RefusalKind::EngineRejected,
        dialect: None,
        engine: Some(Engine::PurRdf),
        message: format!("datalog: {e:?}").into(),
    };
    let mut rules = Vec::new();
    let rules_arr = v
        .get("rules")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing `rules` array"))?;
    check_limit("datalog_rules", rules_arr.len(), MAX_DATALOG_RULES)?;
    for r in rules_arr {
        let body = r
            .get("body")
            .and_then(Value::as_array)
            .ok_or_else(|| bad("rule needs a `body` array"))?
            .iter()
            .map(atom)
            .collect::<Res<Vec<_>>>()?;
        rules.push(DlClause::datalog(
            atom(r.get("head").ok_or_else(|| bad("rule needs a `head`"))?)?,
            body,
        ));
    }
    let exe = compile(rules).map_err(|e| dl(&e))?;
    let mut facts = RelationStore::new();
    let facts_arr = v
        .get("facts")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing `facts` array"))?;
    check_limit("datalog_facts", facts_arr.len(), MAX_DATALOG_FACTS)?;
    for f in facts_arr {
        let [s, p, o] = triple(f)?;
        facts.insert(
            &surface(s)?,
            &surface(p)?,
            &surface(o)?,
            RelationStore::DEFAULT_GRAPH,
        );
    }
    let ev = evaluate(&exe, facts).map_err(|e| dl(&e))?;
    let all = ev.facts().facts_sorted();
    let nquads: String = all
        .iter()
        .map(|f| format!("{} {} {} .\n", f.subject, f.predicate, f.object))
        .collect();
    Ok(json!({
        "count": all.len(),
        "facts": all.iter().map(|f| json!([unsurface(&f.subject), unsurface(&f.predicate), unsurface(&f.object)])).collect::<Vec<_>>(),
        "nquads": nquads,
    }))
}

pub(in crate::abi) fn op_n3(v: &Value) -> Res<Value> {
    let out = crate::law::reason_n3_bounded(str_field(v, "text")?).map_err(|e| match e {
        crate::law::N3Error::Refused(r) => Fail::from(r),
        crate::law::N3Error::Fuse { report } => super::super::fail_with_details(
            Refusal {
                kind: RefusalKind::Fuse,
                dialect: Some(Dialect::N3),
                engine: Some(Dialect::N3.engine()),
                message: report.clone().into(),
            },
            json!({"code": "Fuse", "message": report}),
        ),
        crate::law::N3Error::Limit {
            limit: name,
            observed,
            max,
            ..
        } => limit(name, observed, max),
    })?;
    Ok(json!({"derived": out}))
}
