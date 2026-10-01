//! Every semantic asset shipped in the repository must be routed by content
//! and accepted by the upstream authority that owns its dialect. No GraphLaw
//! parser is involved.
//!
//! Two tiers: files under `SLOW_BYTES` run in the default `cargo test`; the
//! large vendored vocabularies run under `cargo test -- --ignored`.
//!
//! `ASSETS.sha256` pins the exact bytes of every asset. Regenerate it after a
//! deliberate asset change with `UPDATE_ASSET_MANIFEST=1 cargo test --test
//! corpus_conformance asset_manifest`.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use graphlaw::dialect::{Dialect, check, sniff};
use sha2::{Digest, Sha256};

const BASE: &str = "https://example.org/base/";
const SLOW_BYTES: u64 = 1_000_000;
const ROOTS: [&str; 4] = ["ontologies", "packs", "queries", "innovation"];

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

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

fn all_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for d in ROOTS {
        walk(&root().join(d), &mut files);
    }
    files.sort();
    files
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root())
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

fn ext(p: &Path) -> &str {
    p.extension().and_then(|e| e.to_str()).unwrap_or("")
}

fn is_semantic(p: &Path) -> bool {
    matches!(ext(p), "ttl" | "rdf" | "owl" | "nt" | "n3" | "rq")
        || (ext(p) == "json" && rel(p).contains("shex"))
}

fn semantic(slow: bool) -> Vec<PathBuf> {
    all_files()
        .into_iter()
        .filter(|p| is_semantic(p))
        .filter(|p| (fs::metadata(p).unwrap().len() >= SLOW_BYTES) == slow)
        .collect()
}

/// Dialects each extension may legitimately be written in.
fn allowed(p: &Path) -> &'static [Dialect] {
    match ext(p) {
        "ttl" => &[Dialect::Turtle, Dialect::NTriples],
        "n3" => &[Dialect::Turtle, Dialect::N3, Dialect::NTriples],
        "rdf" | "owl" => &[Dialect::RdfXml],
        "nt" => &[Dialect::NTriples],
        "rq" => &[Dialect::Sparql],
        "json" => &[Dialect::ShExJ],
        other => panic!("no dialect expectation for .{other}"),
    }
}

fn run(slow: bool) {
    let files = semantic(slow);
    let mut failures = Vec::new();
    for p in &files {
        let bytes = fs::read(p).unwrap();
        match sniff(&bytes, Some(ext(p))) {
            Err(e) => failures.push(format!("{}: routing refused: {e}", rel(p))),
            Ok(d) if !allowed(p).contains(&d) => failures.push(format!(
                "{}: content is {d:?}, extension says .{}",
                rel(p),
                ext(p)
            )),
            Ok(d) => {
                if let Err(e) = check(&bytes, d, Some(BASE)) {
                    failures.push(format!("{}: {e}", rel(p)));
                }
            }
        }
    }
    assert!(slow || !files.is_empty(), "fast tier matched nothing");
    assert!(
        failures.is_empty(),
        "{} of {} rejected:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
}

#[test]
fn every_asset_is_routed_by_content_and_accepted_by_its_engine() {
    run(false);
}

#[test]
#[ignore = "large vendored vocabularies; run with --ignored"]
fn large_assets_are_routed_and_accepted() {
    run(true);
}

#[test]
fn shacl_shape_files_load_in_purrdf() {
    let mut n = 0;
    for p in semantic(false).into_iter().filter(|p| ext(p) == "ttl") {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if name.contains("shacl") || name.contains("shapes") {
            n += 1;
            let text = fs::read_to_string(&p).unwrap();
            graphlaw::shacl::engine::parse_shapes(&text, Some(BASE))
                .unwrap_or_else(|e| panic!("{}: {e}", rel(&p)));
        }
    }
    assert!(n > 0);
}

fn manifest_text() -> String {
    let mut out =
        String::from("# sha256  bytes  dialect  path   (regenerate: UPDATE_ASSET_MANIFEST=1)\n");
    for p in all_files() {
        let bytes = fs::read(&p).unwrap();
        let digest = Sha256::digest(&bytes);
        let dialect = if is_semantic(&p) {
            sniff(&bytes, Some(ext(&p))).map_or_else(|_| "refused".into(), |d| format!("{d:?}"))
        } else {
            "-".into()
        };
        let hex: String = digest.iter().fold(String::new(), |mut s, b| {
            write!(s, "{b:02x}").unwrap();
            s
        });
        writeln!(out, "{hex}  {}  {dialect}  {}", bytes.len(), rel(&p)).unwrap();
    }
    out
}

#[test]
fn asset_manifest_pins_every_byte() {
    let path = root().join("ASSETS.sha256");
    let actual = manifest_text();
    if std::env::var_os("UPDATE_ASSET_MANIFEST").is_some() {
        fs::write(&path, &actual).unwrap();
        return;
    }
    let pinned = fs::read_to_string(&path).expect("ASSETS.sha256 missing; regenerate it");
    if pinned != actual {
        let a: std::collections::BTreeSet<_> = actual.lines().collect();
        let p: std::collections::BTreeSet<_> = pinned.lines().collect();
        let diff: Vec<_> = a.symmetric_difference(&p).take(10).collect();
        panic!(
            "asset manifest is stale; first differences:\n{}",
            diff.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn router_refuses_non_semantic_and_ambiguous_input() {
    use graphlaw::dialect::RefusalKind::*;
    assert_eq!(
        sniff(b"404: Not Found", None).unwrap_err().kind,
        NotSemanticContent
    );
    assert_eq!(
        sniff(b"<!DOCTYPE html><html></html>", None)
            .unwrap_err()
            .kind,
        NotSemanticContent
    );
    assert_eq!(sniff(b"   \n", None).unwrap_err().kind, NotSemanticContent);
    assert_eq!(
        sniff(b"ex:Cat { ex:says . }", None).unwrap_err().kind,
        Ambiguous
    );
    assert_eq!(
        sniff(b"ex:Cat { ex:says . }", Some("shex")).unwrap(),
        Dialect::ShExC
    );
    assert_eq!(
        sniff(b"{ ?x a :A } => { ?x a :B } .", None).unwrap(),
        Dialect::N3
    );
    assert_eq!(
        sniff(b"PREFIX e: <https://e/> SELECT * WHERE { ?s ?p ?o }", None).unwrap(),
        Dialect::Sparql
    );
    assert_eq!(
        sniff(
            b"<https://e/s> <https://e/p> <https://e/o> <https://e/g> .\n",
            None
        )
        .unwrap(),
        Dialect::NQuads
    );
    assert_eq!(
        sniff(b"<https://e/s> <https://e/p> \"x y\"@en .\n", None).unwrap(),
        Dialect::NTriples
    );
}
