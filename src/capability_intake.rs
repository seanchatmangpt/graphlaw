//! CASTLE capability-donor intake for GraphLaw.
//!
//! This registry is descriptive and side-effect free. It lets GraphLaw expose
//! the exact donor subjects it may wrap or absorb while preserving GraphLaw as
//! the sole semantic-law owner.

/// Exact ggen-ecosystem projection that admitted these donor hypotheses.
pub const PROJECTION_SOURCE: &str =
    "seanchatmangpt/ggen-ecosystem@50fdfa20c84205a80c6eb94e916cffbedc4b816e";

/// GraphLaw's irreducible CASTLE capability.
pub const OWNER_CAPABILITY: &str = "SEMANTIC_LAW_DERIVATION";

/// Maximum authority this intake may manufacture.
pub const AUTHORITY_CEILING: &str = "CONSTRUCT";

/// One projected semantic-capability donor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityDonor {
    /// Source repository.
    pub repository: &'static str,
    /// Exact source commit.
    pub sha: &'static str,
    /// Projected capability.
    pub capability: &'static str,
    /// Bounded projection disposition.
    pub disposition: &'static str,
}

/// Projected donor set for this GraphLaw release.
pub const DONORS: &[CapabilityDonor] = &[
    CapabilityDonor {
        repository: "seanchatmangpt/unrdf",
        sha: "0550d9630da94b170c931f17a1580ea90091cbb4",
        capability: "RDF_KERNEL_COMPATIBILITY",
        disposition: "WRAP",
    },
    CapabilityDonor {
        repository: "seanchatmangpt/praxis",
        sha: "c78783b4ea3124fc9d1179a9e47d43c522b529f2",
        capability: "HISTORICAL_SEMANTIC_SOURCE",
        disposition: "ABSORB",
    },
    CapabilityDonor {
        repository: "seanchatmangpt/mfw",
        sha: "c5a10b00bc8cbc27052840c0b9b51607c2ff85b1",
        capability: "FORMAL_THEORY_PROJECTION",
        disposition: "CANDIDATE_WRAP",
    },
];

/// Look up a projected donor by exact repository identity.
#[must_use]
pub fn donor(repository: &str) -> Option<&'static CapabilityDonor> {
    DONORS.iter().find(|donor| donor.repository == repository)
}

/// Projected donor capability never carries GraphLaw or CASTLE DO authority.
#[must_use]
pub const fn consequence_authority(_repository: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn donor_registry_is_exact_and_non_sovereign() {
        assert_eq!(DONORS.len(), 3);
        assert_eq!(OWNER_CAPABILITY, "SEMANTIC_LAW_DERIVATION");
        assert_eq!(AUTHORITY_CEILING, "CONSTRUCT");
        let mfw = donor("seanchatmangpt/mfw").expect("mfw donor");
        assert_eq!(mfw.sha, "c5a10b00bc8cbc27052840c0b9b51607c2ff85b1");
        assert!(!consequence_authority(mfw.repository));
        assert!(donor("seanchatmangpt/unknown").is_none());
    }
}
