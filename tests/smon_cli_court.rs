//! Real subprocess courts over the two smon CLI binaries (`smon-transcript-to-turtle`,
//! `smon-broaden-topic`): help/usage surfaces, typed argv refusals and one happy-path
//! transcript run against a real file on disk. No mocks, no sleeps.
//! FEATURE GATE: the bins carry `required-features = ["pack-tools"]`, so the whole
//! court is gated on the same feature; under the default set this target compiles to
//! nothing (zero tests, exit 0) by design, not by accident. Run with `--features pack-tools`.

#![cfg(feature = "pack-tools")]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn t2t() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_smon-transcript-to-turtle"));
    c.env_remove("RUST_LOG");
    c
}

fn broaden() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_smon-broaden-topic"));
    c.env_remove("RUST_LOG");
    c
}

fn out(c: &mut Command) -> Output {
    c.output().expect("binary runs")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("graphlaw-smon-cli-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

// ---------------------------------------------------------------- help surfaces

#[test]
fn help_exits_zero_for_both_bins() {
    for (mut c, marker) in [(t2t(), "--transcript"), (broaden(), "--in-ttl")] {
        let o = out(c.arg("--help"));
        let so = stdout(&o);
        assert_eq!(o.status.code(), Some(0), "stdout={so}");
        assert!(
            so.contains(marker),
            "help must document {marker}, got:\n{so}"
        );
        // Anti-vacuity twin: the help banner is the load-bearing text; prove the
        // assertion bites by checking the exit-0 marker really is in stdout and
        // not, say, only on stderr.
        assert!(!stderr(&o).contains(marker));
    }
}

// ------------------------------------------------------------- argv refusals

#[test]
fn unknown_flag_exits_one_pointing_at_help() {
    for mut c in [t2t(), broaden()] {
        let o = out(c.arg("--wat").arg("x"));
        let code = o.status.code();
        assert_eq!(code, Some(1), "stdout={} stderr={}", stdout(&o), stderr(&o));
        let so = stdout(&o);
        assert!(
            stderr(&o).contains("see --help"),
            "expected `see --help` on stderr, got: {}",
            stderr(&o)
        );
        // Anti-vacuity: help text must NOT appear on the failure path's stdout.
        assert!(
            !so.contains("Usage:"),
            "help text leaked onto the unknown-flag stdout:\n{so}"
        );
    }
}

#[test]
fn flag_as_value_is_refused() {
    // `--transcript --out x`: the value slot holds another flag.
    let mut c = t2t();
    let o = out(c.args(["--transcript", "--out", "x"]));
    assert_eq!(
        o.status.code(),
        Some(1),
        "stdout={} stderr={}",
        stdout(&o),
        stderr(&o)
    );
    let se = stderr(&o);
    assert!(
        se.contains("needs a value") && se.contains("--out"),
        "expected flag-as-value refusal, got: {se}"
    );
}

#[test]
fn missing_transcript_file_names_the_path() {
    let dir = scratch("missing");
    let missing = dir.join("nope.jsonl");
    let mut c = t2t();
    let o = out(c.args([
        "--transcript",
        missing.to_str().unwrap(),
        "--out",
        dir.join("o.ttl").to_str().unwrap(),
    ]));
    assert_eq!(
        o.status.code(),
        Some(1),
        "stdout={} stderr={}",
        stdout(&o),
        stderr(&o)
    );
    let se = stderr(&o);
    assert!(
        se.contains("nope.jsonl"),
        "error message must name the missing path, got: {se}"
    );
    assert!(!dir.join("o.ttl").exists());
    let _ = fs::remove_dir_all(&dir);
}

// ------------------------------------------------------------------ happy path

#[test]
fn tiny_inline_transcript_runs_end_to_end() {
    let dir = scratch("happy");
    let jsonl = dir.join("t.jsonl");
    fs::write(
        &jsonl,
        concat!(
            "{\"type\":\"user\",\"message\":{\"content\":\"What capability grounds this? (what grounds the run)\"},\"origin\":{\"kind\":\"human\"}}\n",
            "{\"type\":\"assistant\",\"message\":{\"id\":\"a1\",\"content\":[{\"type\":\"text\",\"text\":\"Grounded in the real run.\"}]}}\n",
        ),
    )
    .unwrap();
    let ttl = dir.join("out.ttl");
    let mut c = t2t();
    let o = out(c.args([
        "--transcript",
        jsonl.to_str().unwrap(),
        "--out",
        ttl.to_str().unwrap(),
    ]));
    let so = stdout(&o);
    assert_eq!(
        o.status.code(),
        Some(0),
        "stdout={so} stderr={}",
        stderr(&o)
    );
    assert!(
        so.contains("wrote:") && so.contains("total extracted turns: 2"),
        "expected run summary, got:\n{so}"
    );
    // Real readback, not just the summary line.
    let written = fs::read_to_string(&ttl).unwrap();
    assert!(
        !written.trim().is_empty() && written.contains("smon"),
        "emitted Turtle must be non-empty smon: facts, got:\n{written}"
    );
    let _ = fs::remove_dir_all(&dir);
}
