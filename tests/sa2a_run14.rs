//! Integration test: runs the real `sa2a-run14-verify` binary over the run14 corpus.
//! Expectations are derived independently from the fixture text (not from the binary).
//! Standing: finite published corpus passes on exact subject; authority NONE; synthetic
//! subject; no general safety claim. Dispositions are declared fixture properties.
#![cfg(feature = "abi")]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_sa2a-run14-verify");

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/sa2a/run14")
}

fn run(dir: &Path) -> (i32, Value) {
    let out = Command::new(BIN).arg(dir).output().expect("spawn runner");
    let v: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "runner stdout not JSON ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        )
    });
    (out.status.code().expect("exit code"), v)
}

/// (file name, namespace label, disposition) by scanning fixture text.
fn derive() -> Vec<(String, &'static str, &'static str)> {
    let mut files: Vec<_> = fs::read_dir(corpus())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ttl"))
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let t = fs::read_to_string(p).unwrap();
            let ns = if t.contains("<https://sa2a.dev/ontology#>") {
                "v1"
            } else if t.contains("<https://chatmangpt.com/sa2a#>") {
                "v2"
            } else {
                panic!("{p:?}")
            };
            let d = if t.contains("sa2a:expected \"ADMIT\"")
                || t.contains("sa2a:expectedDecision \"ADMIT\"")
            {
                "ADMIT"
            } else if t.contains("sa2a:expected \"REFUSE\"")
                || t.contains("sa2a:expectedDecision \"REFUSE\"")
            {
                "REFUSE"
            } else {
                panic!("{p:?} lacks disposition")
            };
            (p.file_name().unwrap().to_string_lossy().into_owned(), ns, d)
        })
        .collect()
}

fn copy_corpus(tag: &str) -> PathBuf {
    let dst = std::env::temp_dir().join(format!("sa2a_run14_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dst);
    fs::create_dir_all(&dst).unwrap();
    for e in fs::read_dir(corpus()).unwrap() {
        let p = e.unwrap().path();
        fs::copy(&p, dst.join(p.file_name().unwrap())).unwrap();
    }
    dst
}

fn codes(v: &Value) -> Vec<String> {
    v["failures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["code"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn published_corpus_has_exactly_65_fixtures_with_expected_split() {
    let d = derive();
    assert_eq!(d.len(), 65);
    assert_eq!(d.iter().filter(|x| x.2 == "ADMIT").count(), 31);
    assert_eq!(d.iter().filter(|x| x.2 == "REFUSE").count(), 34);
    assert_eq!(d.iter().filter(|x| x.1 == "v1").count(), 14);
    assert_eq!(d.iter().filter(|x| x.1 == "v2").count(), 51);
}

#[test]
fn runner_matches_every_fixture_verdict() {
    let (code, v) = run(&corpus());
    assert_eq!(code, 0, "runner failed: {v}");
    assert_eq!(v["ok"], true);
    assert_eq!(v["vectors"], 65);
    assert_eq!(v["admit"], 31);
    assert_eq!(v["refuse"], 34);
    assert_eq!(v["v1"], 14);
    assert_eq!(v["v2"], 51);
    assert_eq!(v["authority"], "NONE");
    let got: Vec<(String, String, String)> = v["fixtures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["file"].as_str().unwrap().into(),
                f["namespace"].as_str().unwrap().into(),
                f["disposition"].as_str().unwrap().into(),
            )
        })
        .collect();
    let want: Vec<(String, String, String)> = derive()
        .into_iter()
        .map(|(a, b, c)| (a, b.into(), c.into()))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn disposition_contradicting_filename_is_refused() {
    let dir = copy_corpus("flip");
    let f = dir.join("001_exact_subject_bound.ttl");
    let t = fs::read_to_string(&f)
        .unwrap()
        .replace("sa2a:expected \"ADMIT\"", "sa2a:expected \"REFUSE\"");
    fs::write(&f, t).unwrap();
    let (code, v) = run(&dir);
    assert_eq!(code, 1);
    assert_eq!(codes(&v), ["DISPOSITION_MISMATCH"]);
    assert_eq!(v["failures"][0]["file"], "001_exact_subject_bound.ttl");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn authority_escalation_is_refused() {
    let dir = copy_corpus("auth");
    let f = dir.join("043_authority_none.ttl");
    let t = fs::read_to_string(&f)
        .unwrap()
        .replace("sa2a:authority \"NONE\"", "sa2a:authority \"DO\"");
    fs::write(&f, t).unwrap();
    let (code, v) = run(&dir);
    assert_eq!(code, 1);
    assert_eq!(codes(&v), ["INVARIANT_VIOLATED"]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn unknown_namespace_and_unparsable_fixture_are_typed_failures() {
    let dir = copy_corpus("ns");
    let f = dir.join("003_source_locator_bound.ttl");
    let t = fs::read_to_string(&f)
        .unwrap()
        .replace("https://sa2a.dev/ontology#", "https://example.org/other#");
    fs::write(&f, t).unwrap();
    fs::write(
        dir.join("004_source_locator_missing.ttl"),
        "@prefix broken <<< .",
    )
    .unwrap();
    let (code, v) = run(&dir);
    assert_eq!(code, 1);
    let mut c = codes(&v);
    c.sort();
    assert_eq!(c, ["PARSE_REFUSED", "UNKNOWN_NAMESPACE"]);
    assert_eq!(v["vectors"], 65);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn removed_fixture_changes_count_and_missing_dir_exits_2() {
    let dir = copy_corpus("rm");
    fs::remove_file(dir.join("060_manufacture_authoritative.ttl")).unwrap();
    let (code, v) = run(&dir);
    assert_eq!(code, 0);
    assert_eq!(v["vectors"], 64);
    assert_ne!(v["vectors"], 65);
    let _ = fs::remove_dir_all(dir);
    let (code, v) = run(Path::new("/nonexistent/run14"));
    assert_eq!(code, 2);
    assert_eq!(v["ok"], false);
}
