//! Every semantic asset shipped in the repository must be routed by content
//! and accepted by the upstream authority that owns its dialect. No GraphLaw
//! parser is involved.
//!
//! Two tiers: files under `SLOW_BYTES` run in the default `cargo test`; the
//! large vendored vocabularies run under `cargo test -- --ignored`.
//!
//! `ASSETS.blake3` pins the exact bytes of every asset. Regenerate it after a
//! deliberate asset change with `UPDATE_ASSET_MANIFEST=1 cargo test --test
//! corpus_conformance asset_manifest`.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use graphlaw::dialect::{Dialect, check, sniff};

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
        // SQL-in-RDF is by definition Turtle carrying `sql:SelectStatement`,
        // so a .ttl extension is legitimate for the SqlRdf dialect too.
        "ttl" => &[Dialect::Turtle, Dialect::NTriples, Dialect::SqlRdf],
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
        String::from("# blake3  bytes  dialect  path   (regenerate: UPDATE_ASSET_MANIFEST=1)\n");
    for p in all_files() {
        let bytes = fs::read(&p).unwrap();
        let digest = blake3::hash(&bytes);
        let dialect = if is_semantic(&p) {
            sniff(&bytes, Some(ext(&p))).map_or_else(|_| "refused".into(), |d| format!("{d:?}"))
        } else {
            "-".into()
        };
        let hex: String = digest.as_bytes().iter().fold(String::new(), |mut s, b| {
            write!(s, "{b:02x}").unwrap();
            s
        });
        writeln!(out, "{hex}  {}  {dialect}  {}", bytes.len(), rel(&p)).unwrap();
    }
    out
}

#[test]
fn asset_manifest_pins_every_byte() {
    let path = root().join("ASSETS.blake3");
    let actual = manifest_text();
    if std::env::var_os("UPDATE_ASSET_MANIFEST").is_some() {
        fs::write(&path, &actual).unwrap();
        return;
    }
    let pinned = fs::read_to_string(&path).expect("ASSETS.blake3 missing; regenerate it");
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

// ---------------------------------------------------------------------------
// Vector-gen examples as consumed corpus (af3c169 orphans)
//
// The seven vector generators are bins (required-features = ["abi"]); each
// test below RUNS one for real into a fresh temp GL_VECDIR, re-runs its
// --verify mode where it has one (re-evaluating every emitted vector through
// the engine), asserts the emitted JSON vectors carry the shared envelope
// (vector_id / provenance / expected), and routes the emitted semantic files
// (.n3) through the same dialect sniff/check the committed corpus gets.
// ---------------------------------------------------------------------------

#[cfg(feature = "abi")]
mod vecgen {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    const BASE: &str = super::BASE;

    /// Fresh output dir for generator `name`, stable across calls within one
    /// test process so generate -> --verify sees the same vectors.
    fn gen_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "graphlaw-vecgen-{}-{}",
            name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp vecdir");
        dir
    }

    /// Runs generator `name` (with `args`) with GL_VECDIR at `dir`.
    fn run_gen(name: &str, args: &[&str], dir: &PathBuf) -> String {
        let exe = std::env::var(format!("CARGO_BIN_EXE_{name}")).unwrap_or_else(|_| {
            panic!("{name} binary not built; the corpus tests require --features abi")
        });
        let out = Command::new(exe)
            .args(args)
            .env("GL_VECDIR", dir)
            .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
            .output()
            .expect("spawn generator");
        assert!(
            out.status.success(),
            "{name} {args:?} failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// Every emitted vector must be well-formed JSON carrying provenance and a
    /// case id (`vector_id`, or `vector` for the naive-physics fixtures);
    /// emitted .n3 goes through the same dialect court as committed assets.
    fn assert_emitted(dir: &PathBuf) -> usize {
        let mut n = 0;
        let mut entries: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap()).collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let path = e.path();
            match path.extension().and_then(|x| x.to_str()) {
                Some("json") => {
                    let text = fs::read_to_string(&path).unwrap();
                    let v: serde_json::Value = serde_json::from_str(&text)
                        .unwrap_or_else(|e| panic!("{}: not valid JSON: {e}", path.display()));
                    assert!(
                        v.get("vector_id").is_some() || v.get("vector").is_some(),
                        "{}: missing case id (vector_id/vector) in vector envelope",
                        path.display()
                    );
                    assert!(
                        v.get("provenance").is_some(),
                        "{}: missing `provenance` in vector envelope",
                        path.display()
                    );
                    n += 1;
                }
                Some("n3") => {
                    let bytes = fs::read(&path).unwrap();
                    let d = super::sniff(&bytes, Some("n3")).unwrap_or_else(|e| {
                        panic!("{}: routing refused: {e}", path.display())
                    });
                    assert!(
                        matches!(
                            d,
                            super::Dialect::N3 | super::Dialect::Turtle | super::Dialect::NTriples
                        ),
                        "{}: routed as {d:?}",
                        path.display()
                    );
                    super::check(&bytes, d, Some(BASE))
                        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                    n += 1
                }
                _ => {} // SCHEMA.md / ENCODERS.md: markdown companions, not semantic
            }
        }
        n
    }

    /// Generates, then re-evaluates every emitted vector via `--verify`.
    fn generate_and_verify(name: &str) -> usize {
        let dir = gen_dir(name);
        let stdout = run_gen(name, &[], &dir);
        let n = assert_emitted(&dir);
        assert!(n > 0, "{name}: emitted no vectors (stdout: {stdout})");
        run_gen(name, &["--verify"], &dir);
        let _ = fs::remove_dir_all(&dir);
        n
    }

    /// Generates only (self-checks run inside generation: differential courts,
    /// dual-route agreement, brute-force oracles).
    fn generate_only(name: &str) -> usize {
        let dir = gen_dir(name);
        let stdout = run_gen(name, &[], &dir);
        let n = assert_emitted(&dir);
        assert!(n > 0, "{name}: emitted no vectors (stdout: {stdout})");
        let _ = fs::remove_dir_all(&dir);
        n
    }

    #[test]
    fn vecgen_allen_datalog_transcription_matches_oracle_and_n3_roundtrips() {
        // Runs the full in-binary pipeline: 169-entry transcription-vs-oracle
        // court, engine case runs, and idempotent re-emission of the committed
        // examples/allen_comp_table.n3 (byte-identical by construction; any
        // drift dirties the tree and the drift test below fails).
        let stdout = run_gen("allen_datalog", &[], &gen_dir("allen_datalog"));
        assert!(
            stdout.contains("169/169 entries match"),
            "allen table-check did not report full agreement:\n{stdout}"
        );
    }

    #[test]
    fn vecgen_allen_committed_comp_table_is_byte_current() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static LOCK: AtomicU32 = AtomicU32::new(0);
        while LOCK.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            std::thread::yield_now();
        }
        let committed = fs::read_to_string("examples/allen_comp_table.n3").unwrap();
        run_gen("allen_datalog", &[], &gen_dir("allen_datalog"));
        let after = fs::read_to_string("examples/allen_comp_table.n3").unwrap();
        LOCK.store(0, Ordering::SeqCst);
        assert_eq!(
            committed, after,
            "examples/allen_comp_table.n3 is stale relative to the oracle in examples/allen_datalog.rs"
        );
    }

    #[test]
    fn vecgen_asp_tableaux_generate_then_verify() {
        let n = generate_and_verify("asp_tableaux_vector_gen");
        assert!(n >= 4, "asp/tableaux emitted only {n} files");
    }

    #[test]
    fn vecgen_bayes_pop_generate_then_verify() {
        let n = generate_and_verify("bayes_pop_vectors");
        assert!(n >= 4, "bayes/pop emitted only {n} files");
    }

    #[test]
    fn vecgen_breed_oracle_generate_then_verify() {
        let n = generate_and_verify("breed_oracle_gen");
        assert!(n >= 4, "breed emitted only {n} files");
    }

    #[test]
    fn vecgen_event_calculus_dual_route_agrees() {
        let n = generate_only("event_calculus_vectors");
        assert!(n >= 1, "event calculus emitted only {n} files");
    }

    #[test]
    fn vecgen_naive_physics_fixtures() {
        let n = generate_only("naive_physics_vectors");
        // 4 scene vectors + SCHEMA.md + ENCODERS.md; envelope counts the 4 JSONs.
        assert!(n >= 4, "naive physics emitted only {n} vectors");
    }

    #[test]
    fn vecgen_plan_replay_generate_then_verify() {
        let n = generate_and_verify("plan_replay_oracle_gen");
        assert!(n >= 4, "plan replay emitted only {n} files");
    }
}
