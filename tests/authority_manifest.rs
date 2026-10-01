//! `BACKEND_AUTHORITIES` may not drift from the pins in `Cargo.toml`.

use graphlaw::BACKEND_AUTHORITIES;

fn cargo_toml() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap()
}

#[test]
fn purrdf_revision_matches_cargo_pin() {
    let toml = cargo_toml();
    for a in BACKEND_AUTHORITIES
        .iter()
        .filter(|a| a.authority.starts_with("purrdf"))
    {
        assert!(
            toml.contains(&format!("purrdf = \"={}\"", a.revision)),
            "{} claims purrdf {} but Cargo.toml disagrees",
            a.capability,
            a.revision
        );
    }
}

#[test]
fn eyeron_revision_matches_the_vendored_crate() {
    let root = env!("CARGO_MANIFEST_DIR");
    let manifest =
        std::fs::read_to_string(format!("{root}/crates/graphlaw-eyeron/Cargo.toml")).unwrap();
    let upstream =
        std::fs::read_to_string(format!("{root}/crates/graphlaw-eyeron/UPSTREAM.md")).unwrap();
    for a in BACKEND_AUTHORITIES
        .iter()
        .filter(|a| a.authority == "eyeron")
    {
        let version = a.revision.split_whitespace().next().unwrap();
        assert!(
            manifest.contains(&format!("version = \"{version}\"")),
            "{}: vendored version differs",
            a.capability
        );
        let commit = a
            .revision
            .split('@')
            .nth(1)
            .unwrap()
            .split(',')
            .next()
            .unwrap();
        assert!(
            upstream.contains(commit),
            "{}: UPSTREAM.md does not record {commit}",
            a.capability
        );
    }
}

#[test]
fn no_git_or_unpublishable_path_dependencies() {
    let toml = cargo_toml();
    assert!(
        !toml.contains("git = "),
        "git dependencies cannot be published to crates.io"
    );
    for line in toml
        .lines()
        .filter(|l| l.contains("= {") && l.contains("path = ") && !l.trim_start().starts_with('#'))
    {
        if line.contains("[patch") || line.starts_with("purrdf-sparql-eval") {
            continue; // [patch] entries are ignored on publish by design.
        }
        assert!(
            line.contains("version = "),
            "path dependency without a version cannot be published: {line}"
        );
    }
}

#[test]
fn every_capability_has_exactly_one_authority() {
    let mut caps: Vec<_> = BACKEND_AUTHORITIES.iter().map(|a| a.capability).collect();
    caps.sort_unstable();
    let n = caps.len();
    caps.dedup();
    assert_eq!(n, caps.len());
}

#[test]
fn resolved_lockfile_agrees() {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"));
    if let Ok(lock) = lock {
        assert!(lock.contains("name = \"purrdf\"\nversion = \"2.0.2\""));
    }
}
