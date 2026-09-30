const GRAPH: &str = include_str!("../ontologies/castle-capability-intake.ttl");

#[test]
fn castle_intake_preserves_graphlaw_semantic_ownership() {
    assert!(GRAPH.contains("50fdfa20c84205a80c6eb94e916cffbedc4b816e"));
    assert!(GRAPH.contains("SEMANTIC_LAW_DERIVATION"));
    assert!(GRAPH.contains("projectionStanding \"CANDIDATE\""));
    assert!(GRAPH.contains("authorityCeiling \"CONSTRUCT\""));
    assert!(!GRAPH.contains("projectionStanding \"ALIVE\""));
    assert!(!GRAPH.contains("authorityCeiling \"DO\""));
}

#[test]
fn castle_intake_has_three_exact_non_sovereign_donors() {
    let donors = [
        (
            "seanchatmangpt/unrdf",
            "0550d9630da94b170c931f17a1580ea90091cbb4",
        ),
        (
            "seanchatmangpt/praxis",
            "c78783b4ea3124fc9d1179a9e47d43c522b529f2",
        ),
        (
            "seanchatmangpt/mfw",
            "c5a10b00bc8cbc27052840c0b9b51607c2ff85b1",
        ),
    ];

    for (repo, sha) in donors {
        assert!(GRAPH.contains(repo), "missing donor {repo}");
        assert!(GRAPH.contains(sha), "missing exact subject {repo}@{sha}");
    }

    assert_eq!(GRAPH.matches("a eco:ProjectedCapability").count(), 3);
    assert!(!GRAPH.contains("RUNTIME_CORE"));
    assert!(!GRAPH.contains("CONSEQUENCE_CROWN"));
}
