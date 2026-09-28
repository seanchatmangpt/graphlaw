//! Every semantic asset shipped in the repository must be accepted by the
//! upstream authority that owns its dialect. No GraphLaw parser is involved.

use std::fs;
use std::path::{Path, PathBuf};

const BASE: &str = "https://example.org/base/";

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Files that are scraped HTML specification pages, not RDF. They are excluded
/// from the corpus explicitly (never silently) and must be replaced with the
/// real vocabulary or deleted; `quarantine_is_still_broken` fails once fixed.
const QUARANTINED: &[&str] = &["ontologies/catalogs/void.ttl"];

fn corpus() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for d in ["ontologies", "packs", "queries", "innovation"] {
        walk(&root.join(d), &mut files);
    }
    files.retain(|p| !QUARANTINED.iter().any(|q| p.ends_with(q)));
    files.sort();
    files
}

fn ext(p: &Path) -> &str {
    p.extension().and_then(|e| e.to_str()).unwrap_or("")
}

fn check_all(exts: &[&str], f: impl Fn(&Path, &str) -> Result<(), String>) -> usize {
    let mut failures = Vec::new();
    let mut n = 0;
    for p in corpus().into_iter().filter(|p| exts.contains(&ext(p))) {
        n += 1;
        let text = fs::read_to_string(&p).unwrap();
        if let Err(e) = f(&p, &text) {
            failures.push(format!("{}: {e}", p.display()));
        }
    }
    assert!(n > 0, "no files matched {exts:?}");
    assert!(
        failures.is_empty(),
        "{} of {n} rejected:\n{}",
        failures.len(),
        failures.join("\n")
    );
    n
}

fn rdf(media: &'static str) -> impl Fn(&Path, &str) -> Result<(), String> {
    move |_, text| {
        praxis_graphlaw::rdf::parse_dataset(text.as_bytes(), media, Some(BASE))
            .map(|_| ())
            .map_err(|e| format!("{e:?}"))
    }
}

#[test]
fn turtle_assets_parse_in_purrdf() {
    check_all(&["ttl"], rdf("text/turtle"));
}

#[test]
fn rdfxml_and_owl_assets_parse_in_purrdf() {
    check_all(&["rdf", "owl"], rdf("application/rdf+xml"));
}

#[test]
fn ntriples_assets_parse_in_purrdf() {
    check_all(&["nt"], rdf("application/n-triples"));
}

#[test]
fn sparql_assets_parse_in_purrdf() {
    let parser = praxis_graphlaw::sparql::SparqlParser::new();
    check_all(&["rq"], |_, text| {
        parser
            .parse_query(text)
            .map(|_| ())
            .map_err(|e| format!("{e:?}"))
    });
}

#[test]
fn n3_assets_reason_in_eyeron() {
    check_all(&["n3"], |_, text| {
        praxis_graphlaw::n3::reason(text)
            .map(|_| ())
            .map_err(|e| format!("{e:?}"))
    });
}

#[test]
fn shexj_assets_parse_and_check_in_purrdf() {
    check_all(&["json"], |p, text| {
        if !p.to_string_lossy().contains("shex") {
            return Ok(());
        }
        let schema =
            praxis_graphlaw::shex::parse_shexj(text, None).map_err(|e| format!("{e:?}"))?;
        praxis_graphlaw::shex::check_structure(&schema).map_err(|e| format!("{e:?}"))
    });
}

#[test]
fn shacl_shape_files_load_in_purrdf() {
    check_all(&["ttl"], |p, text| {
        let name = p.file_name().unwrap().to_string_lossy();
        if !(name.contains("shacl") || name.contains("shapes")) {
            return Ok(());
        }
        praxis_graphlaw::shacl::engine::parse_shapes(text, Some(BASE))
            .map(|_| ())
            .map_err(|e| format!("{e:?}"))
    });
}

#[test]
fn quarantine_is_still_broken() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for q in QUARANTINED {
        let Ok(bytes) = fs::read(root.join(q)) else {
            continue;
        };
        assert!(
            praxis_graphlaw::rdf::parse_dataset(&bytes, "text/turtle", Some(BASE)).is_err(),
            "{q} now parses; remove it from QUARANTINED"
        );
    }
}
