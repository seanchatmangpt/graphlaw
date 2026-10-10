//! Docs-vs-workflow release-claim gate: the README's description of the release
//! flow must match what `release.yml` actually does, in either direction.
//!
//! Real-state (Chicago): reads the two files from disk, no mocks. If this test
//! fails, a doc describing the release pathway has drifted from the workflow.

use std::fs;
use std::path::Path;

const README: &str = "README.md";
const WORKFLOW: &str = ".github/workflows/release.yml";
const LOCAL_ONLY_MARKER: &str = "cargo cicd publish run";
const PUBLISH_STEP: &str = "cargo publish -p graphlaw";

fn read(rel: &str) -> String {
    fs::read_to_string(Path::new(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// The README text describing the release flow: from the "Releasing" heading to
/// the next `##` heading (or EOF).
fn releasing_section(readme: &str) -> &str {
    let start = readme
        .find("### Releasing")
        .or_else(|| readme.find("## Releasing"))
        .expect("README must carry a Releasing section");
    let rest = &readme[start..];
    let end = rest
        .find("\n## ")
        .map(|i| i + 1)
        .unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn readme_release_section_matches_workflow_publish_behavior() {
    let readme = read(README);
    let section = releasing_section(&readme);
    let workflow = read(WORKFLOW);
    let ci_publishes = workflow.contains(PUBLISH_STEP);
    let local_only = workflow.contains(LOCAL_ONLY_MARKER);

    if local_only {
        assert!(
            !ci_publishes,
            "release.yml both claims local-only and contains '{PUBLISH_STEP}' — workflow is self-contradictory"
        );
        assert!(
            section.contains(LOCAL_ONLY_MARKER),
            "release.yml is local-only but the README Releasing section does not mention '{LOCAL_ONLY_MARKER}'"
        );
        assert!(
            !section.contains("CARGO_REGISTRY_TOKEN"),
            "README Releasing section still claims a CI registry credential that the workflow does not use"
        );
        assert!(
            !section.contains("publishes `graphlaw-eyeron`"),
            "README Releasing section still claims CI publishes the crates"
        );
    } else if ci_publishes {
        assert!(
            section.contains("crates.io"),
            "release.yml publishes to crates.io but the README Releasing section does not describe it"
        );
    } else {
        panic!(
            "release.yml describes neither local-only publishing nor a crates.io publish step; \
             re-author this test's contract with the workflow"
        );
    }
}
