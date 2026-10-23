//! Pins the README "First 10 minutes" claims: the three featureless examples must
//! produce exactly the outputs the README quotes. Real examples, real subprocesses,
//! asserted outputs — Chicago, no mocks. If this fails, either the examples' output
//! changed (update the README *and* this test in the same change) or the README's
//! credibility claim ("expected outputs are real") has rotted.

use std::process::Command;

/// Run a no-feature example, capture stdout.
fn run_example(name: &str) -> String {
    let out = Command::new(env!("CARGO"))
        .args(["run", "-q", "--offline", "--example", name])
        .output()
        .expect("cargo available");
    assert!(
        out.status.success(),
        "{name} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn quickstart_plan_matches_readme_hashes() {
    let out = run_example("quickstart_plan");
    assert!(out.contains("plan-action -> blake3:8a0d0202"), "{out}");
    assert!(out.contains("plan-action -> blake3:c66ea90e"), "{out}");
    assert!(out.contains("plan-action -> blake3:3be6530b"), "{out}");
    assert!(
        out.contains(
            "final state: blake3:3be6530bdf3597afdcc5f60a2a7a0c15a952a1e8f8dd440e0a0bf49082e05896"
        ),
        "{out}"
    );
}

#[test]
fn refused_plan_matches_readme_text() {
    let out = run_example("refused_plan");
    assert!(out.contains("REFUSED at index 1, action `paint`"), "{out}");
    assert!(
        out.contains("<urn:d:door> <urn:p:has> <urn:v:key> ."),
        "{out}"
    );
}

#[test]
fn receipt_feedback_matches_readme_lines() {
    let out = run_example("receipt_feedback");
    assert!(
        out.contains("before record: receipt required: no recorded receipt for step `derive:rdfs`"),
        "{out}"
    );
    assert!(
        out.contains("after record:  admitted by `admit:require-receipt`"),
        "{out}"
    );
}
