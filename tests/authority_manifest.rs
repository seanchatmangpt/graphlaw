//! `BACKEND_AUTHORITIES` may not drift from the pins in `Cargo.toml`.

use praxis_graphlaw::BACKEND_AUTHORITIES;

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
fn eyeron_revision_matches_cargo_pin() {
    let toml = cargo_toml();
    for a in BACKEND_AUTHORITIES
        .iter()
        .filter(|a| a.authority == "eyeron")
    {
        assert!(
            toml.contains(&format!("rev = \"{}\"", a.revision)),
            "{}",
            a.capability
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
