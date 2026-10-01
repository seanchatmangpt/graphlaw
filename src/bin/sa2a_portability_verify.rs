//! Verify the SA2A run12 portable evidence corpus through GraphLaw's real JSON ABI.
//! This is evidence verification only: it never grants authority or performs DO.
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::ExitCode};

fn corpus_dir() -> PathBuf {
    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("conformance/sa2a/portable/run12")
        })
}
fn request(v: Value) -> Value {
    serde_json::from_slice(&graphlaw::abi::call(v.to_string().as_bytes()))
        .expect("GraphLaw ABI returns JSON")
}
fn main() -> ExitCode {
    let dir = corpus_dir();
    let mut files: Vec<_> = fs::read_dir(&dir)
        .expect("run12 corpus")
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
        .collect();
    files.sort();
    let mut admitted = 0usize;
    let mut refused = 0usize;
    for path in &files {
        let text = fs::read_to_string(path).expect("UTF-8 Turtle");
        let parsed = request(json!({"op":"parse","text":text,"dialect":"turtle"}));
        assert_eq!(
            parsed["ok"],
            true,
            "{} failed GraphLaw parse: {}",
            path.display(),
            parsed
        );
        let canonical = request(json!({"op":"canonical","data":{"text":text,"dialect":"turtle"}}));
        assert_eq!(
            canonical["ok"],
            true,
            "{} failed RDFC canonicalization: {}",
            path.display(),
            canonical
        );
        if text.contains("sa2a:expected \"ADMIT\"") {
            admitted += 1;
            for invariant in [
                "sa2a:authority \"NONE\"",
                "sa2a:consequence \"EVIDENCE_ONLY\"",
                "sa2a:canonicalization \"RDFC-1.0\"",
            ] {
                assert!(
                    text.contains(invariant),
                    "{} missing {}",
                    path.display(),
                    invariant
                );
            }
        } else if text.contains("sa2a:expected \"REFUSE\"") {
            refused += 1;
            assert!(
                text.contains("sa2a:violation"),
                "{} refusal lacks typed violation",
                path.display()
            );
        } else {
            panic!("{} lacks expected disposition", path.display());
        }
    }
    assert_eq!(
        files.len(),
        50,
        "run12 corpus must contain exactly 50 vectors"
    );
    println!(
        "{{\"ok\":true,\"vectors\":{},\"admit\":{},\"refuse\":{},\"authority\":\"NONE\"}}",
        files.len(),
        admitted,
        refused
    );
    ExitCode::SUCCESS
}
