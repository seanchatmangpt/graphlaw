//! Document routing: RDF datasets, N3 logic rule sets, and SQL-in-RDF queries.
//!
//! A [`Dialect`] names a syntax; a [`Tier`] names the *semantics* the document
//! carries. Media-type dialects (Turtle, TriG, ...) carry an RDF dataset;
//! `Dialect::N3` carries N3 logic (rules). When a caller supplies a
//! [`DialectDecl`], routing skips sniffing entirely and dispatches on the
//! declared tier: `Tier::Rdf` parses into a dataset, `Tier::N3Logic` validates
//! and returns a [`RuleSet`] without any dataset entanglement. When no
//! declaration is supplied, sniffing behaves exactly as before — except that a
//! failure whose real cause is a tier mismatch (RDF parser given N3 logic, or
//! N3 logic sniffed without a declaration) is refused as
//! [`RefusalKind::TierMismatch`] instead of surfacing as a confusing
//! dataset-parse error.

use std::sync::Arc;

use crate::dialect::{Dialect, Refusal, RefusalKind, parse_rdf, sniff};
use purrdf::n3;

/// Stable conformance code (sql-rdf Recommendation §5, `docs/reference/sql-rdf-rec.md`):
/// `CapabilityRefusal`. Stable wire discriminator; the register is `docs/refusals.md`.
///
/// SQL-in-RDF admission compiles in this crate on every build, including the wasm
/// module, through the same `compile_turtle` path as `Dialect::SqlRdf`. The code
/// marks a query that is not document-backed (`rules::builtins`), not a missing feature.
pub const REFUSAL_04_CAPABILITY_REFUSAL: &str = "REFUSAL-04";

/// The semantics a document carries: an RDF dataset, or an N3 logic rule set.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::tier::Tier;
///
/// let t = Tier::N3Logic;
/// let is_rules = match t {
///     Tier::N3Logic => true,
///     _ => false, // required: new tiers may be added
/// };
/// assert!(is_rules);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Tier {
    /// The document denotes an RDF dataset (any media-type dialect).
    Rdf,
    /// The document denotes N3 logic: rules (and facts) owned by purrdf::n3.
    N3Logic,
    /// The document denotes a SQL-in-RDF query owned by frozen-duckdb.
    Relational,
}

impl Dialect {
    /// The tier of semantics this dialect carries.
    ///
    /// ```
    /// use graphlaw::dialect::Dialect;
    /// use graphlaw::tier::Tier;
    ///
    /// assert_eq!(Dialect::N3.tier(), Tier::N3Logic);
    /// assert_eq!(Dialect::Turtle.tier(), Tier::Rdf);
    /// ```
    pub const fn tier(self) -> Tier {
        match self {
            Dialect::N3 => Tier::N3Logic,
            Dialect::SqlRdf => Tier::Relational,
            _ => Tier::Rdf,
        }
    }
}

/// A caller-supplied declaration of what a document is, bypassing sniffing.
///
/// ```
/// use graphlaw::dialect::Dialect;
/// use graphlaw::tier::{DialectDecl, Tier};
///
/// let decl = DialectDecl {
///     tier: Tier::Rdf,
///     dialect: Dialect::Turtle,
/// };
/// assert_eq!(decl.dialect.tier(), decl.tier);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialectDecl {
    /// The tier the caller declares the document to carry.
    pub tier: Tier,
    /// The dialect the document is written in. Must agree with `tier`
    /// (`decl.dialect.tier() == decl.tier`), otherwise admission is refused.
    pub dialect: Dialect,
}

/// An N3 document validated by purrdf-n3's parse entry point.
///
/// The rules are counted but never run against a dataset here: the N3 tier
/// admits a rule set, not a state.
#[derive(Debug, Clone)]
pub struct RuleSet {
    /// The parsed N3 document (facts, rules, prefixes) as purrdf-n3 sees it.
    pub document: n3::Document,
}

impl RuleSet {
    /// Number of logic rules in the document (facts are not rules).
    pub fn rule_count(&self) -> usize {
        self.document.rules.len()
    }
}

/// What tiered admission produced.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Admitted {
    /// The document parsed as an RDF dataset (tier `Rdf`).
    Dataset(Arc<purrdf::RdfDataset>),
    /// The document validated as an N3 rule set (tier `N3Logic`).
    Rules(RuleSet),
    /// The document validated and compiled as SQL-in-RDF (tier `Relational`).
    Query(crate::sql_ast::CompiledQuery),
}

fn tier_mismatch(dialect: Dialect, message: String) -> Refusal {
    Refusal {
        kind: RefusalKind::TierMismatch,
        dialect: Some(dialect),
        engine: Some(dialect.engine()),
        message: message.into(),
    }
}

/// Validate and compile `bytes` as a SQL-in-RDF query. No dataset is admitted.
///
/// Same compiler the other dialects call for `Dialect::SqlRdf` (`compile_turtle`).
/// The wasm module links this path; it does not refuse the document for lack of a feature.
fn admit_sql_rdf(bytes: &[u8]) -> Result<crate::sql_ast::CompiledQuery, Refusal> {
    let text = std::str::from_utf8(bytes)
        .map_err(|e| Refusal::engine(Dialect::SqlRdf, format!("input is not UTF-8: {e}")))?;
    crate::sql_ast::compile_turtle(text)
        .map_err(|e| Refusal::engine(Dialect::SqlRdf, e.to_string()))
}

/// Validate `bytes` as an N3 logic document via purrdf-n3's parse entry point.
///
/// No dataset is built; the result is a [`RuleSet`] for rule consumers.
pub fn validate_n3_rules(bytes: &[u8], base: Option<&str>) -> Result<RuleSet, Refusal> {
    let text = std::str::from_utf8(bytes)
        .map_err(|e| Refusal::engine(Dialect::N3, format!("input is not UTF-8: {e}")))?;
    let document = n3::parse_n3(text, base).map_err(|e| Refusal::engine(Dialect::N3, e))?;
    Ok(RuleSet { document })
}

/// Parse `bytes` as RDF, upgrading the refusal to [`RefusalKind::TierMismatch`]
/// when the failure is really a tier mismatch (the document is N3 logic).
fn parse_rdf_tier_checked(
    bytes: &[u8],
    dialect: Dialect,
    base: Option<&str>,
) -> Result<Arc<purrdf::RdfDataset>, Refusal> {
    parse_rdf(bytes, dialect, base).map_err(|e| {
        if validate_n3_rules(bytes, base).is_ok() {
            tier_mismatch(
                dialect,
                format!(
                    "declared {dialect:?} ({:?}) but the document validates as N3 logic; \
                     declare `Tier::N3Logic` to load it as rules",
                    dialect.tier()
                ),
            )
        } else {
            e
        }
    })
}

/// Admit `bytes` under a caller-supplied [`DialectDecl`]: no sniffing, tier
/// dispatch only.
///
/// `Tier::Rdf` parses into a dataset; `Tier::N3Logic` validates via purrdf-n3 and
/// returns a [`RuleSet`]. A declaration whose `dialect` disagrees with its
/// `tier` is refused as `TierMismatch`.
pub fn admit_declared(
    bytes: &[u8],
    decl: DialectDecl,
    base: Option<&str>,
) -> Result<Admitted, Refusal> {
    if decl.dialect.tier() != decl.tier {
        return Err(tier_mismatch(
            decl.dialect,
            format!(
                "declaration is inconsistent: {decl:?} carries {:?}, not {:?}",
                decl.dialect.tier(),
                decl.tier
            ),
        ));
    }
    match decl.tier {
        Tier::Rdf => parse_rdf_tier_checked(bytes, decl.dialect, base).map(Admitted::Dataset),
        Tier::N3Logic => validate_n3_rules(bytes, base).map(Admitted::Rules),
        Tier::Relational => admit_sql_rdf(bytes).map(Admitted::Query),
    }
}

/// The tiered admit dispatch: declared routing when a [`DialectDecl`] is
/// supplied, otherwise the historical sniff behavior with typed tier-mismatch
/// refusals on failure.
pub fn admit_bytes_tiered(
    bytes: &[u8],
    hint: Option<&str>,
    base: Option<&str>,
    decl: Option<DialectDecl>,
) -> Result<Admitted, Refusal> {
    match decl {
        Some(decl) => admit_declared(bytes, decl, base),
        None => {
            let dialect = sniff(bytes, hint)?;
            if dialect.tier() == Tier::Relational {
                return admit_sql_rdf(bytes).map(Admitted::Query);
            }
            match parse_rdf(bytes, dialect, base) {
                Ok(dataset) => Ok(Admitted::Dataset(dataset)),
                Err(e) => {
                    if validate_n3_rules(bytes, base).is_ok() {
                        Err(tier_mismatch(
                            dialect,
                            format!(
                                "sniffed {dialect:?} ({:?}) but the document validates as \
                                 N3 logic; supply a `DialectDecl {{ tier: Tier::N3Logic, .. }}` \
                                 to load it as rules",
                                dialect.tier()
                            ),
                        ))
                    } else {
                        Err(e)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQL_DOC: &str = r#"
        @prefix sql: <http://graphlaw.org/sql#> .
        @prefix ex: <http://example.org/queries#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        ex:Q a sql:SelectStatement ;
            sql:statementIndex 1 ;
            sql:fromTable [ sql:tableName "telemetry_events" ] ;
            sql:selectList ( [ sql:columnName "batch_id" ] ) ;
            sql:whereClause [
                sql:operator sql:GreaterThan ;
                sql:leftOperand [ sql:columnName "latency_ms" ] ;
                sql:rightOperand [ sql:literalValue "400"^^xsd:integer ]
            ] ."#;

    /// SQL-in-RDF admits through the same compiler on every build of this crate,
    /// including the graphlaw linked into the wasm module.
    #[test]
    fn sql_document_admitted() {
        let admitted = admit_declared(
            SQL_DOC.as_bytes(),
            DialectDecl {
                tier: Tier::Relational,
                dialect: Dialect::SqlRdf,
            },
            None,
        )
        .unwrap();
        assert!(matches!(admitted, Admitted::Query(_)));
    }
}
