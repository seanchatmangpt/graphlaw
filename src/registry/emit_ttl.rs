use std::fmt::Write as _;

use serde_json::Value;

use super::definitions::*;
use super::emit_json::{Triple, canonical, document, sorted_entries};

struct Graph {
    triples: Vec<Triple>,
}

fn node(fragment: &str) -> String {
    format!("<{REGISTRY_BASE}{fragment}>")
}

fn lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn text_of(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn int_of(v: &Value, key: &str) -> String {
    v.get(key).map_or_else(|| "0".into(), Value::to_string)
}

fn bool_of(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_bool)
        .unwrap_or(false)
        .to_string()
}

fn items<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
}

impl Graph {
    fn add(&mut self, subject: &str, predicate: &str, object: String) {
        self.triples
            .push((node(subject), predicate.to_owned(), object));
    }

    fn typed(&mut self, subject: &str, class: &str) {
        self.add(subject, "a", format!("gac:{class}"));
    }

    fn fields(&mut self, fields: &[Value], owner: &str, prefix: &str, side: &str) {
        for f in fields {
            let name = text_of(f, "name");
            let iri = format!("{prefix}/{name}");
            self.typed(&iri, "CapabilityField");
            self.add(&iri, "gac:fieldOwner", node(owner));
            self.add(&iri, "gac:fieldSide", lit(side));
            self.add(&iri, "gac:fieldOrder", int_of(f, "order"));
            self.add(&iri, "gac:fieldName", lit(&name));
            self.add(&iri, "gac:fieldType", lit(&text_of(f, "type")));
            self.add(&iri, "gac:fieldRequired", bool_of(f, "required"));
            self.add(&iri, "gac:fieldNullable", bool_of(f, "nullable"));
            self.add(&iri, "gac:fieldDoc", lit(&text_of(f, "doc")));
            if let Some(d) = f.get("default").filter(|d| !d.is_null()) {
                self.add(&iri, "gac:fieldDefault", lit(&canonical(d)));
            }
            for (i, e) in items(f, "enum").iter().enumerate() {
                let eiri = format!("{iri}/enum/{}", i + 1);
                self.typed(&eiri, "EnumValue");
                self.add(&eiri, "gac:enumOf", node(&iri));
                self.add(&eiri, "gac:enumOrder", (i + 1).to_string());
                self.add(&eiri, "gac:enumValue", lit(e.as_str().unwrap_or_default()));
            }
        }
    }
}

fn graph(doc: &Value) -> Graph {
    let mut g = Graph {
        triples: Vec::new(),
    };
    let r = "registry";
    g.typed(r, "Registry");
    g.add(r, "gac:schemaId", lit(&text_of(doc, "schema")));
    g.add(
        r,
        "gac:graphlawVersion",
        lit(&text_of(doc, "graphlaw_version")),
    );
    g.add(r, "gac:abiVersion", int_of(doc, "abi_version"));
    g.add(r, "gac:surfaceBlake3", lit(&text_of(doc, "surface_blake3")));
    g.add(
        r,
        "gac:receiptSurfaceBlake3",
        lit(&text_of(doc, "receipt_surface_blake3")),
    );
    g.add(
        r,
        "gac:registryBlake3",
        lit(&text_of(doc, "registry_blake3")),
    );
    g.add(r, "gac:opCount", items(doc, "ops").len().to_string());
    g.add(
        r,
        "gac:dialectCount",
        (items(doc, "rdf_dialects").len() + items(doc, "other_dialects").len()).to_string(),
    );

    for op in items(doc, "ops") {
        let name = text_of(op, "name");
        let oi = format!("op/{name}");
        g.typed(&oi, "Capability");
        g.add(&oi, "gac:capabilityOf", node(r));
        g.add(&oi, "gac:opName", lit(&name));
        g.add(&oi, "gac:opOrder", int_of(op, "order"));
        g.add(&oi, "gac:opSummary", lit(&text_of(op, "summary")));
        for k in items(op, "refusal_kinds") {
            g.add(
                &oi,
                "gac:opRefusalKind",
                lit(k.as_str().unwrap_or_default()),
            );
        }
        for c in items(op, "refusal_codes") {
            g.add(
                &oi,
                "gac:opRefusalCode",
                lit(c.as_str().unwrap_or_default()),
            );
        }
        let request = op.get("request").unwrap_or(&Value::Null);
        g.fields(
            items(request, "fields"),
            &oi,
            &format!("{oi}/request"),
            "request",
        );
        for (vi, variant) in items(op, "responses").iter().enumerate() {
            let tag = variant.get("tag").and_then(Value::as_str);
            let tag_field = variant.get("tag_field").and_then(Value::as_str);
            let vfrag = format!("{oi}/response/{}", tag.unwrap_or("default"));
            g.typed(&vfrag, "ResponseVariant");
            g.add(&vfrag, "gac:variantOf", node(&oi));
            g.add(&vfrag, "gac:variantTag", lit(tag.unwrap_or("")));
            g.add(&vfrag, "gac:variantTagField", lit(tag_field.unwrap_or("")));
            g.add(&vfrag, "gac:variantOrder", (vi + 1).to_string());
            g.fields(items(variant, "fields"), &vfrag, &vfrag, "response");
        }
    }

    for d in items(doc, "rdf_dialects")
        .iter()
        .chain(items(doc, "other_dialects"))
    {
        let name = text_of(d, "name");
        let di = format!("dialect/{name}");
        g.typed(&di, "Dialect");
        g.add(&di, "gac:dialectOf", node(r));
        g.add(&di, "gac:dialectName", lit(&name));
        g.add(&di, "gac:dialectOrder", int_of(d, "order"));
        g.add(&di, "gac:dialectKind", lit(&text_of(d, "kind")));
        g.add(
            &di,
            "gac:dialectResponseName",
            lit(&text_of(d, "response_name")),
        );
        if let Some(m) = d.get("media_type").and_then(Value::as_str) {
            g.add(&di, "gac:dialectMediaType", lit(m));
        }
        for a in items(d, "aliases") {
            g.add(&di, "gac:dialectAlias", lit(a.as_str().unwrap_or_default()));
        }
    }

    for c in items(doc, "refusal_codes") {
        let code = text_of(c, "code");
        let ci = format!("refusal-code/{code}");
        g.typed(&ci, "RefusalCode");
        g.add(&ci, "gac:refusalCodeName", lit(&code));
        g.add(&ci, "gac:refusalCodeOrder", int_of(c, "order"));
        if let Some(k) = c.get("kind").and_then(Value::as_str) {
            g.add(&ci, "gac:refusalCodeKind", lit(k));
        }
        g.fields(items(c, "fields"), &ci, &ci, "detail");
    }

    for s in items(doc, "law_steps") {
        let wire = text_of(s, "name");
        let si = format!("law-step/{wire}");
        g.typed(&si, "LawStep");
        g.add(&si, "gac:lawStepName", lit(&wire));
        g.add(&si, "gac:lawStepOrder", int_of(s, "order"));
        if let Some(c) = s.get("ceiling").and_then(Value::as_str) {
            g.add(&si, "gac:lawStepCeiling", lit(c));
        }
        g.fields(items(s, "fields"), &si, &si, "law_step");
    }

    for vocab in [
        "engines",
        "refusal_kinds",
        "regimes",
        "lease_ceilings",
        "lease_reasons",
        "receipt_reasons",
        "policy_refusal_kinds",
    ] {
        for (i, value) in items(doc, vocab).iter().enumerate() {
            let vi = format!("vocab/{vocab}/{}", i + 1);
            g.typed(&vi, "VocabularyEntry");
            g.add(&vi, "gac:vocabOf", lit(vocab));
            g.add(&vi, "gac:vocabOrder", (i + 1).to_string());
            g.add(
                &vi,
                "gac:vocabValue",
                lit(value.as_str().unwrap_or_default()),
            );
        }
    }

    if let Some(limits) = doc.get("limits").and_then(Value::as_object) {
        for (name, value) in sorted_entries(limits) {
            let li = format!("limit/{name}");
            g.typed(&li, "Limit");
            g.add(&li, "gac:limitName", lit(name));
            g.add(&li, "gac:limitValue", value.to_string());
        }
    }

    if let Some(meta) = doc.get("limit_meta").and_then(Value::as_object) {
        for (name, m) in sorted_entries(meta) {
            let li = format!("limit/{name}");
            g.add(&li, "gac:limitScope", lit(&text_of(m, "scope")));
            g.add(&li, "gac:limitUnit", lit(&text_of(m, "unit")));
            g.add(&li, "gac:limitSource", lit(&text_of(m, "source")));
        }
    }

    for m in items(doc, "models") {
        let name = text_of(m, "name");
        let mi = format!("model/{name}");
        g.typed(&mi, "Model");
        g.add(&mi, "gac:modelName", lit(&name));
        g.add(&mi, "gac:modelOrder", int_of(m, "order"));
        g.add(
            &mi,
            "gac:modelElixirModule",
            lit(&text_of(m, "elixir_module")),
        );
        g.add(&mi, "gac:modelRustPath", lit(&text_of(m, "rust_path")));
        g.add(&mi, "gac:modelDoc", lit(&text_of(m, "doc")));
        for f in items(m, "fields") {
            let fname = text_of(f, "name");
            let fi = format!("{mi}/{fname}");
            g.typed(&fi, "ModelField");
            g.add(&fi, "gac:modelFieldOf", node(&mi));
            g.add(&fi, "gac:modelFieldName", lit(&fname));
            g.add(&fi, "gac:modelFieldOrder", int_of(f, "order"));
            g.add(&fi, "gac:modelFieldType", lit(&text_of(f, "type")));
            g.add(&fi, "gac:modelFieldRequired", bool_of(f, "required"));
            g.add(&fi, "gac:modelFieldNullable", bool_of(f, "nullable"));
            g.add(&fi, "gac:modelFieldDoc", lit(&text_of(f, "doc")));
        }
    }

    for e in items(doc, "model_enums") {
        let name = text_of(e, "name");
        let ei = format!("model-enum/{name}");
        g.typed(&ei, "ModelEnum");
        g.add(&ei, "gac:modelName", lit(&name));
        g.add(&ei, "gac:modelOrder", int_of(e, "order"));
        for (i, v) in items(e, "values").iter().enumerate() {
            let vi = format!("{ei}/{}", i + 1);
            g.typed(&vi, "ModelEnumValue");
            g.add(&vi, "gac:enumOf", node(&ei));
            g.add(&vi, "gac:enumOrder", (i + 1).to_string());
            g.add(&vi, "gac:enumValue", lit(v.as_str().unwrap_or_default()));
        }
    }
    g
}

/// Deterministic Turtle projection of the registry: fixed prefix block, one
/// block per subject, subjects/predicates/objects sorted bytewise.
pub fn registry_turtle() -> String {
    let mut g = graph(document());
    g.triples.sort();
    g.triples.dedup();
    let mut out = String::new();
    out.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
    out.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
    out.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n");
    let _ = writeln!(out, "@prefix gac: <{GAC_NAMESPACE}> .");
    let mut i = 0;
    while i < g.triples.len() {
        let subject = &g.triples[i].0;
        // `j` starts past `i` so every pass consumes at least one triple: the
        // outer loop terminates (and `out` stays bounded) whatever the subject
        // comparison below evaluates to.
        let mut j = i + 1;
        while j < g.triples.len() && &g.triples[j].0 == subject {
            j += 1;
        }
        out.push('\n');
        out.push_str(subject);
        out.push('\n');
        for (k, (_, p, o)) in g.triples[i..j].iter().enumerate() {
            let end = if k + 1 == j - i { " ." } else { " ;" };
            let _ = writeln!(out, "    {p} {o}{end}");
        }
        i = j;
    }
    out
}
