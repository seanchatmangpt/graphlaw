//! `graphlaw-registry`: emit or check the capability registry files.
//!
//! ```text
//! graphlaw-registry --write | --check | --print-json | --print-ttl | --print-digests
//! ```
//!
//! `--write` writes `registry/capability-registry.json` and `.ttl` under the crate
//! root; `--check` exits 1 with a difference summary when the committed files differ
//! from what the Rust table emits. Exit codes: 0 ok, 1 drift, 2 usage or I/O error.

use std::path::PathBuf;
use std::process::ExitCode;

use graphlaw::registry::{registry_json_pretty, registry_sha256, registry_turtle, surface_sha256};

fn registry_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("registry")
}

fn usage(msg: &str) -> ExitCode {
    eprintln!("graphlaw-registry: {msg}");
    eprintln!(
        "usage: graphlaw-registry --write | --check | --print-json | --print-ttl | --print-digests"
    );
    ExitCode::from(2)
}

fn first_difference(expected: &str, found: &str) -> String {
    for (i, (e, f)) in expected.lines().zip(found.lines()).enumerate() {
        if e != f {
            return format!(
                "first difference at line {}: expected `{e}`, found `{f}`",
                i + 1
            );
        }
    }
    format!(
        "line counts differ: expected {}, found {}",
        expected.lines().count(),
        found.lines().count()
    )
}

fn check() -> ExitCode {
    let dir = registry_dir();
    let mut drift = false;
    for (file, expected) in [
        ("capability-registry.json", registry_json_pretty()),
        ("capability-registry.ttl", registry_turtle()),
    ] {
        match std::fs::read_to_string(dir.join(file)) {
            Ok(found) if found == expected => println!("ok      {file}"),
            Ok(found) => {
                drift = true;
                println!("DRIFT   {file}: {}", first_difference(&expected, &found));
            }
            Err(e) => {
                drift = true;
                println!("MISSING {file}: {e}");
            }
        }
    }
    if drift {
        eprintln!(
            "registry drift: run `cargo run --features abi --bin graphlaw-registry -- --write`"
        );
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn write() -> ExitCode {
    let dir = registry_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return usage(&format!("cannot create {}: {e}", dir.display()));
    }
    for (file, body) in [
        ("capability-registry.json", registry_json_pretty()),
        ("capability-registry.ttl", registry_turtle()),
    ] {
        if let Err(e) = std::fs::write(dir.join(file), body) {
            return usage(&format!("cannot write {file}: {e}"));
        }
        println!("wrote {}", dir.join(file).display());
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [a] if a == "--write" => write(),
        [a] if a == "--check" => check(),
        [a] if a == "--print-json" => {
            print!("{}", registry_json_pretty());
            ExitCode::SUCCESS
        }
        [a] if a == "--print-ttl" => {
            print!("{}", registry_turtle());
            ExitCode::SUCCESS
        }
        [a] if a == "--print-digests" => {
            println!("surface_sha256  {}", surface_sha256());
            println!("registry_sha256 {}", registry_sha256());
            ExitCode::SUCCESS
        }
        _ => usage(
            "expected exactly one of --write, --check, --print-json, --print-ttl, --print-digests",
        ),
    }
}
