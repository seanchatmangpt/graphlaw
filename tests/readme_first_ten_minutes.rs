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
    assert!(out.contains("plan-action -> sha256:e51e3e15"), "{out}");
    assert!(out.contains("plan-action -> sha256:01c373ea"), "{out}");
    assert!(out.contains("plan-action -> sha256:77b8d7c9"), "{out}");
    assert!(
        out.contains("final state: sha256:77b8d7c989b2717c1300a597db07c2a60722156183ce7e14017aec23e8d033d7"),
        "{out}"
    );
}

#[test]
fn refused_plan_matches_readme_text() {
    let out = run_example("refused_plan");
    assert!(
        out.contains("REFUSED at index 1, action `paint`"),
        "{out}"
    );
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
