//! Capability registry: the single Rust source of truth for the public GraphLaw
//! ABI surface (schema `graphlaw.capability-registry/1`).
//!
//! The static tables below describe every ABI op, its request and response
//! fields, dialects, refusal codes, law steps, closed vocabularies and limits.
//! `registry/capability-registry.json` and `registry/capability-registry.ttl`
//! are emitted from these tables by the `graphlaw-registry` binary and are never
//! hand-edited. The `capabilities` ABI op builds its `ops`, `rdf_dialects` and
//! `other_dialects` lists from here, so there is no second literal.
//!
//! Digest rule: `canonical(v)` is compact JSON with object keys sorted by byte
//! order; `surface_sha256` hashes the surface document (ABI version, op names,
//! dialect names) and `registry_sha256` hashes the whole registry document with
//! the `registry_sha256` key removed. Digests are over canonical bytes, never
//! over the pretty file bytes.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::abi::{
    ABI_VERSION, MAX_ATOMS_PER_FIELD, MAX_JSON_DEPTH, MAX_PLAN_ACTIONS, MAX_POLICY_ENTRIES,
    MAX_REQUEST_BYTES,
};

/// Schema identifier of the registry document.
pub const REGISTRY_SCHEMA: &str = "graphlaw.capability-registry/1";

/// Namespace of the consumer-facing capability vocabulary.
pub const GAC_NAMESPACE: &str = "http://seanchatmangpt.github.io/packs/graphlaw-ash-capability#";

/// Base IRI under which registry individuals are minted.
pub const REGISTRY_BASE: &str = "https://graphlaw.dev/registry#";

/// Default value of a registry field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldDefault {
    /// No default.
    None,
    /// String default.
    Str(&'static str),
    /// Integer default.
    Int(i64),
    /// Boolean default.
    Bool(bool),
}

/// One field of a request, response variant, refusal detail or law step.
#[derive(Debug, Clone, Copy)]
pub struct Field {
    /// snake_case wire name.
    pub name: &'static str,
    /// Type from the closed vocabulary (`string`, `integer`, `list<term>`, ...).
    pub ty: &'static str,
    /// Whether the field must be present.
    pub required: bool,
    /// Whether an explicit JSON null is accepted as absent.
    pub nullable: bool,
    /// One-line documentation.
    pub doc: &'static str,
    /// Informational enumeration (never client-enforced); empty when none.
    pub enum_values: &'static [&'static str],
    /// Default value.
    pub default: FieldDefault,
}

impl Field {
    const fn new(name: &'static str, ty: &'static str, required: bool, doc: &'static str) -> Self {
        Field {
            name,
            ty,
            required,
            nullable: false,
            doc,
            enum_values: &[],
            default: FieldDefault::None,
        }
    }

    const fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    const fn with_enum(mut self, values: &'static [&'static str]) -> Self {
        self.enum_values = values;
        self
    }

    const fn with_default(mut self, default: FieldDefault) -> Self {
        self.default = default;
        self
    }
}

const fn req(name: &'static str, ty: &'static str, doc: &'static str) -> Field {
    Field::new(name, ty, true, doc)
}

const fn opt(name: &'static str, ty: &'static str, doc: &'static str) -> Field {
    Field::new(name, ty, false, doc)
}

/// One response shape of an op.
#[derive(Debug, Clone, Copy)]
pub struct Variant {
    /// Discriminating tag value; `None` for untagged ops.
    pub tag: Option<&'static str>,
    /// Field carrying the tag; `None` for untagged ops.
    pub tag_field: Option<&'static str>,
    /// Fields of this variant.
    pub fields: &'static [Field],
}

/// One ABI op.
#[derive(Debug, Clone, Copy)]
pub struct Op {
    /// Wire name.
    pub name: &'static str,
    /// One-line summary.
    pub summary: &'static str,
    /// Request fields (the `op` key itself is implicit).
    pub request: &'static [Field],
    /// Response variants.
    pub responses: &'static [Variant],
    /// Refusal kinds the op source can emit.
    pub refusal_kinds: &'static [&'static str],
    /// `details.code` values the op source can emit.
    pub refusal_codes: &'static [&'static str],
}

/// One `details.code` refusal code.
#[derive(Debug, Clone, Copy)]
pub struct RefusalCodeRow {
    /// Wire code.
    pub code: &'static str,
    /// Top-level refusal kind that accompanies the code, when fixed.
    pub kind: Option<&'static str>,
    /// Detail fields.
    pub fields: &'static [Field],
}

/// One step of the `law` op.
#[derive(Debug, Clone, Copy)]
pub struct LawStepRow {
    /// Wire name.
    pub name: &'static str,
    /// Lease ceiling the step requires; `None` when the step is not leased.
    pub ceiling: Option<&'static str>,
    /// Step fields.
    pub fields: &'static [Field],
}

/// One dialect row.
#[derive(Debug, Clone, Copy)]
pub struct DialectRow {
    /// Wire name.
    pub name: &'static str,
    /// Additional accepted names.
    pub aliases: &'static [&'static str],
    /// Name used in responses (the Rust variant name).
    pub response_name: &'static str,
    /// Media type, for RDF dialects that serialize.
    pub media_type: Option<&'static str>,
}

/// Engines that own dialects.
pub const ENGINES: &[&str] = &["PurRdf", "Eyeron"];
/// Refusal kinds.
pub const REFUSAL_KINDS: &[&str] = &[
    "NotSemanticContent",
    "Ambiguous",
    "EngineRejected",
    "Unsupported",
    "ResourceLimit",
];
/// Entailment regimes.
pub const REGIMES: &[&str] = &["simple", "rdf", "rdfs", "owl-rl", "d"];
/// Lease ceilings.
pub const LEASE_CEILINGS: &[&str] = &["observe", "select", "construct"];
/// Lease refusal reasons.
pub const LEASE_REASONS: &[&str] = &[
    "expired",
    "out_of_scope",
    "ceiling",
    "bad_signature",
    "untrusted_key",
    "clock_skew",
];
/// Receipt refusal reasons.
pub const RECEIPT_REASONS: &[&str] = &["unattested", "bad_signature", "untrusted_key"];
/// Policy refusal kinds.
pub const POLICY_REFUSAL_KINDS: &[&str] = &[
    "MissingEntry",
    "InventedOutcome",
    "BadMass",
    "DeadEnd",
    "Malformed",
];

/// RDF dialects, in wire order.
pub const RDF_DIALECTS: &[DialectRow] = &[
    DialectRow {
        name: "turtle",
        aliases: &["ttl"],
        response_name: "Turtle",
        media_type: Some("text/turtle"),
    },
    DialectRow {
        name: "trig",
        aliases: &[],
        response_name: "TriG",
        media_type: Some("application/trig"),
    },
    DialectRow {
        name: "ntriples",
        aliases: &["nt"],
        response_name: "NTriples",
        media_type: Some("application/n-triples"),
    },
    DialectRow {
        name: "nquads",
        aliases: &["nq"],
        response_name: "NQuads",
        media_type: Some("application/n-quads"),
    },
    DialectRow {
        name: "rdfxml",
        aliases: &["rdf", "owl"],
        response_name: "RdfXml",
        media_type: Some("application/rdf+xml"),
    },
    DialectRow {
        name: "jsonld",
        aliases: &[],
        response_name: "JsonLd",
        media_type: Some("application/ld+json"),
    },
    DialectRow {
        name: "yamlld",
        aliases: &[],
        response_name: "YamlLd",
        media_type: Some("application/ld+yaml"),
    },
    DialectRow {
        name: "trix",
        aliases: &[],
        response_name: "TriX",
        media_type: Some("application/trix"),
    },
    DialectRow {
        name: "hextuples",
        aliases: &["hext"],
        response_name: "HexTuples",
        media_type: Some("application/x-hextuples"),
    },
];

/// Non-RDF dialects, in wire order.
pub const OTHER_DIALECTS: &[DialectRow] = &[
    DialectRow {
        name: "n3",
        aliases: &[],
        response_name: "N3",
        media_type: None,
    },
    DialectRow {
        name: "sparql",
        aliases: &["rq"],
        response_name: "Sparql",
        media_type: None,
    },
    DialectRow {
        name: "shexc",
        aliases: &["shex"],
        response_name: "ShExC",
        media_type: None,
    },
    DialectRow {
        name: "shexj",
        aliases: &[],
        response_name: "ShExJ",
        media_type: None,
    },
];

const KINDS_ROUTED: &[&str] = REFUSAL_KINDS;
const KINDS_ENGINE: &[&str] = &["EngineRejected", "Unsupported", "ResourceLimit"];
const KINDS_SNIFF: &[&str] = &[
    "NotSemanticContent",
    "Ambiguous",
    "Unsupported",
    "ResourceLimit",
];
const KINDS_CAPABILITIES: &[&str] = &["Unsupported", "ResourceLimit"];
const CODES_LIMIT: &[&str] = &["ResourceLimit"];

const DATA: Field = req(
    "data",
    "data_spec",
    "Data to operate on: an object {text, dialect?, hint?, base?}; the dialect is sniffed when omitted.",
);

const NOT_TAGGED: Option<&str> = None;

const fn one(fields: &'static [Field]) -> Variant {
    Variant {
        tag: NOT_TAGGED,
        tag_field: NOT_TAGGED,
        fields,
    }
}

/// The 14 ABI ops in wire order.
pub const OPS: &[Op] = &[
    Op {
        name: "capabilities",
        summary: "Report ABI version, crate version, semantic authorities, dialects, ops and registry digests.",
        request: &[],
        responses: &[one(&[
            req(
                "abi",
                "integer",
                "ABI revision (legacy alias of abi_version).",
            ),
            req("abi_version", "integer", "ABI revision."),
            req("crate", "string", "GraphLaw crate version."),
            req(
                "authorities",
                "list<object>",
                "Semantic authority map: {capability, authority, revision}.",
            ),
            req("rdf_dialects", "list<string>", "RDF dialect wire names."),
            req(
                "other_dialects",
                "list<string>",
                "Non-RDF dialect wire names.",
            ),
            req("ops", "list<string>", "ABI op names in registry order."),
            opt(
                "registry_schema",
                "string",
                "Registry schema id; absent on engines older than v26.9.29.",
            ),
            opt(
                "registry_sha256",
                "string",
                "Registry digest; absent on engines older than v26.9.29.",
            ),
            opt(
                "surface_sha256",
                "string",
                "Surface digest; absent on engines older than v26.9.29.",
            ),
        ])],
        refusal_kinds: KINDS_CAPABILITIES,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "sniff",
        summary: "Route text to a dialect and owning engine without parsing it.",
        request: &[
            req("text", "string", "Text to route."),
            opt("hint", "string", "File extension or name hint.").nullable(),
        ],
        responses: &[one(&[
            req("dialect", "string", "Sniffed dialect (Rust variant name)."),
            req("engine", "string", "Owning engine (PurRdf or Eyeron)."),
        ])],
        refusal_kinds: KINDS_SNIFF,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "parse",
        summary: "Parse text with the owning engine; counts quads or validates syntax.",
        request: &[
            req("text", "string", "Text to parse."),
            opt("dialect", "string", "Dialect name; sniffed when omitted.").nullable(),
            opt(
                "hint",
                "string",
                "File extension or name hint for sniffing.",
            )
            .nullable(),
            opt("base", "string", "Base IRI.").nullable(),
        ],
        responses: &[one(&[
            req("dialect", "string", "Dialect used (Rust variant name)."),
            opt("quads", "integer", "Quad count; present for RDF dialects."),
            opt(
                "id",
                "string",
                "Content-addressed state id; present for RDF dialects.",
            ),
            opt(
                "valid",
                "boolean",
                "Syntax validity; present for non-RDF dialects.",
            ),
        ])],
        refusal_kinds: KINDS_ROUTED,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "convert",
        summary: "Serialize parsed RDF text into another RDF dialect.",
        request: &[
            req("text", "string", "Text to convert."),
            req("to", "string", "Target RDF dialect name."),
            opt(
                "dialect",
                "string",
                "Source dialect name; sniffed when omitted.",
            )
            .nullable(),
            opt(
                "hint",
                "string",
                "File extension or name hint for sniffing.",
            )
            .nullable(),
            opt("base", "string", "Base IRI.").nullable(),
        ],
        responses: &[one(&[
            req("text", "string", "Serialized output."),
            req("id", "string", "Content-addressed id of the source state."),
        ])],
        refusal_kinds: KINDS_ROUTED,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "canonical",
        summary: "Canonicalize a dataset to N-Quads and return its content-addressed id.",
        request: &[DATA],
        responses: &[one(&[
            req("id", "string", "Content-addressed state id."),
            req("nquads", "string", "Canonical N-Quads."),
            req("quads", "integer", "Quad count."),
        ])],
        refusal_kinds: KINDS_ROUTED,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "sparql",
        summary: "Evaluate a SPARQL query over a dataset.",
        request: &[
            DATA,
            req("query", "string", "SPARQL query text."),
            opt("base", "string", "Base IRI.").nullable(),
        ],
        responses: &[
            Variant {
                tag: Some("solutions"),
                tag_field: Some("kind"),
                fields: &[
                    req("variables", "list<string>", "Projected variable names."),
                    req(
                        "rows",
                        "list<list<term>>",
                        "Solution rows; null marks an unbound cell.",
                    ),
                ],
            },
            Variant {
                tag: Some("graph"),
                tag_field: Some("kind"),
                fields: &[
                    req("nquads", "string", "Constructed graph as N-Quads."),
                    req("quads", "integer", "Quad count of the constructed graph."),
                ],
            },
            Variant {
                tag: Some("boolean"),
                tag_field: Some("kind"),
                fields: &[req("value", "boolean", "ASK result.")],
            },
        ],
        refusal_kinds: KINDS_ROUTED,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "shacl",
        summary: "Validate a dataset against SHACL shapes.",
        request: &[
            DATA,
            req("shapes", "string", "SHACL shapes graph as Turtle text."),
            opt("base", "string", "Base IRI.").nullable(),
        ],
        responses: &[one(&[
            req("conforms", "boolean", "True when the data conforms."),
            req(
                "results",
                "list<object>",
                "Validation results: {focus, path, value, severity, component, shape, message}.",
            ),
        ])],
        refusal_kinds: KINDS_ROUTED,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "shex",
        summary: "Validate a dataset against a ShEx schema and shape map.",
        request: &[
            DATA,
            req("schema", "string", "ShEx schema text."),
            opt("schema_dialect", "string", "Schema syntax.")
                .with_enum(&["shexc", "shexj"])
                .with_default(FieldDefault::Str("shexc"))
                .nullable(),
            req("map", "string", "Shape map text."),
            opt("base", "string", "Base IRI.").nullable(),
        ],
        responses: &[one(&[
            req(
                "conforms",
                "boolean",
                "True when every map entry is conformant.",
            ),
            req(
                "entries",
                "list<object>",
                "Shape map entries: {node, shape, status, reason}.",
            ),
        ])],
        refusal_kinds: KINDS_ROUTED,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "n3",
        summary: "Run bounded Notation3 forward reasoning over a document.",
        request: &[req(
            "text",
            "string",
            "Notation3 document with data and rules.",
        )],
        responses: &[one(&[req(
            "derived",
            "any",
            "Reasoner output; shape is owned by Eyeron.",
        )])],
        refusal_kinds: KINDS_ENGINE,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "entail",
        summary: "Materialize an entailment regime over a dataset.",
        request: &[
            DATA,
            req("regime", "string", "Entailment regime.").with_enum(REGIMES),
        ],
        responses: &[one(&[
            req("nquads", "string", "Closure as N-Quads."),
            req("added", "integer", "Quads added by materialization."),
        ])],
        refusal_kinds: KINDS_ROUTED,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "datalog",
        summary: "Evaluate Datalog rules over triple facts to a fixpoint.",
        request: &[
            req(
                "rules",
                "list<object>",
                "Rules: {head: [s,p,o], body: [[s,p,o], ...]}; terms starting with ? are variables.",
            ),
            req(
                "facts",
                "list<list<string>>",
                "Ground facts as [subject, predicate, object] strings.",
            ),
        ],
        responses: &[one(&[
            req("count", "integer", "Number of facts in the fixpoint."),
            req(
                "facts",
                "list<list<string>>",
                "Sorted fixpoint facts as [subject, predicate, object].",
            ),
        ])],
        refusal_kinds: KINDS_ENGINE,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "hooks",
        summary: "Materialize a knowledge-hook pack over a dataset.",
        request: &[
            req(
                "pack",
                "data_spec",
                "Hook pack as a data spec (kh: vocabulary).",
            ),
            DATA,
        ],
        responses: &[one(&[
            req(
                "id",
                "string",
                "Content-addressed id of the materialized state.",
            ),
            req("rounds", "integer", "Fixpoint rounds executed."),
            req("quads", "integer", "Quad count of the materialized state."),
            req("nquads", "string", "Materialized state as N-Quads."),
            req(
                "firings",
                "list<object>",
                "Hook firings: {hook, round, added, row}.",
            ),
        ])],
        refusal_kinds: KINDS_ROUTED,
        refusal_codes: CODES_LIMIT,
    },
    Op {
        name: "law",
        summary: "Run an ordered chain of law steps with receipts and optional lease authorization.",
        request: &[
            DATA,
            req("steps", "list<object>", "Ordered law steps; see law_steps."),
            opt(
                "signed_lease",
                "object",
                "Signed lease {lease, attestation}; verified offline against trusted_keys.",
            ),
            opt(
                "lease",
                "object",
                "Unsigned lease; refused unless unverified_lease is true.",
            ),
            opt(
                "trusted_keys",
                "list<string>",
                "Hex Ed25519 public keys trusted to sign leases.",
            ),
            opt(
                "max_skew_secs",
                "integer",
                "Clock skew tolerance for signed leases in seconds.",
            )
            .nullable(),
            opt(
                "unverified_lease",
                "boolean",
                "Accept an unsigned lease; proves nothing about its issuer.",
            ),
            opt(
                "now_unix",
                "integer",
                "Caller clock in unix seconds; used only with an unverified lease.",
            ),
        ],
        responses: &[one(&[
            req(
                "states",
                "list<string>",
                "State ids after each step, starting with the input.",
            ),
            req("receipts", "list<object>", "Receipts, one per transition."),
            req("nquads", "string", "Final state as N-Quads."),
        ])],
        refusal_kinds: KINDS_ROUTED,
        refusal_codes: &[
            "Refused",
            "ResourceLimit",
            "PlanRefused",
            "ReceiptRequired",
            "ReceiptRefused",
            "LeaseRefused",
            "NotAdmitted",
            "UnverifiedLeaseRefused",
        ],
    },
    Op {
        name: "policy",
        summary: "Admit a FOND policy as strong-cyclic against a planning problem.",
        request: &[
            req(
                "problem",
                "json_or_string",
                "Planning problem as a JSON object or JSON text.",
            ),
            req(
                "policy",
                "json_or_string",
                "Policy as a JSON object, entries array or JSON text.",
            ),
        ],
        responses: &[one(&[
            req(
                "initial_states",
                "list<string>",
                "Initial states of the problem.",
            ),
            req(
                "reachable",
                "list<string>",
                "States reachable under the policy.",
            ),
            req("goal_states", "list<string>", "Reachable goal states."),
            req(
                "entries",
                "list<list<string>>",
                "Admitted policy entries as [state, action].",
            ),
            req("ntriples", "string", "Admitted policy as N-Triples."),
        ])],
        refusal_kinds: KINDS_ENGINE,
        refusal_codes: &["ResourceLimit", "PolicyRefused"],
    },
];

/// `details.code` refusal codes, in registry order.
pub const REFUSAL_CODES: &[RefusalCodeRow] = &[
    RefusalCodeRow {
        code: "Refused",
        kind: None,
        fields: &[req(
            "kind",
            "string",
            "Refusal kind of the underlying engine refusal.",
        )],
    },
    RefusalCodeRow {
        code: "ResourceLimit",
        kind: Some("ResourceLimit"),
        fields: &[
            req("limit", "string", "Name of the exceeded limit."),
            opt(
                "observed",
                "integer",
                "Observed size; absent for iteration limits.",
            ),
            req("max", "integer", "Configured maximum."),
        ],
    },
    RefusalCodeRow {
        code: "PlanRefused",
        kind: Some("EngineRejected"),
        fields: &[
            req("index", "integer", "Index of the refused plan action."),
            req("action", "string", "Name of the refused action."),
            req("unmet", "list<string>", "Unmet precondition atoms."),
            req(
                "violated_absent",
                "list<string>",
                "Atoms that were required absent but present.",
            ),
        ],
    },
    RefusalCodeRow {
        code: "ReceiptRequired",
        kind: Some("EngineRejected"),
        fields: &[req("step", "string", "Step name with no recorded receipt.")],
    },
    RefusalCodeRow {
        code: "ReceiptRefused",
        kind: Some("EngineRejected"),
        fields: &[
            req("step", "string", "Step name whose receipt was refused."),
            req("reason", "string", "Refusal reason.").with_enum(RECEIPT_REASONS),
        ],
    },
    RefusalCodeRow {
        code: "LeaseRefused",
        kind: Some("EngineRejected"),
        fields: &[
            req("reason", "string", "Refusal reason.").with_enum(LEASE_REASONS),
            req("lease_id", "string", "Id of the refused lease."),
            req("step", "string", "Step that was refused."),
        ],
    },
    RefusalCodeRow {
        code: "NotAdmitted",
        kind: Some("EngineRejected"),
        fields: &[req(
            "violations",
            "list<object>",
            "SHACL violations: {focus, path, component, message, severity}.",
        )],
    },
    RefusalCodeRow {
        code: "UnverifiedLeaseRefused",
        kind: Some("Unsupported"),
        fields: &[],
    },
    RefusalCodeRow {
        code: "PolicyRefused",
        kind: Some("EngineRejected"),
        fields: &[
            req("policy_kind", "string", "Policy refusal kind.").with_enum(POLICY_REFUSAL_KINDS),
            req("state", "string", "State at which the policy was refused."),
            req(
                "action",
                "string",
                "Action at which the policy was refused.",
            ),
        ],
    },
];

/// Steps accepted by the `law` op, in registry order.
pub const LAW_STEPS: &[LawStepRow] = &[
    LawStepRow {
        name: "shacl",
        ceiling: Some("observe"),
        fields: &[req(
            "shapes",
            "string",
            "SHACL shapes graph as Turtle text.",
        )],
    },
    LawStepRow {
        name: "n3",
        ceiling: Some("construct"),
        fields: &[req("rules", "string", "Notation3 rules text.")],
    },
    LawStepRow {
        name: "rdfs",
        ceiling: Some("construct"),
        fields: &[],
    },
    LawStepRow {
        name: "owl-rl",
        ceiling: Some("construct"),
        fields: &[],
    },
    LawStepRow {
        name: "hooks",
        ceiling: Some("construct"),
        fields: &[req("pack", "data_spec", "Hook pack as a data spec.")],
    },
    LawStepRow {
        name: "plan",
        ceiling: Some("select"),
        fields: &[req(
            "plan",
            "object",
            "Plan {actions: [{name, pre, pre_not?, add, del}], goal, goal_not?} with N-Triples strings.",
        )],
    },
    LawStepRow {
        name: "record-receipts",
        ceiling: None,
        fields: &[],
    },
    LawStepRow {
        name: "require-receipt",
        ceiling: Some("observe"),
        fields: &[req(
            "step_name",
            "string",
            "Step whose receipt must already be recorded.",
        )],
    },
    LawStepRow {
        name: "require-signed-receipt",
        ceiling: Some("observe"),
        fields: &[
            req(
                "step_name",
                "string",
                "Step whose receipt must carry a valid attestation.",
            ),
            req(
                "trusted_keys",
                "list<string>",
                "Hex Ed25519 public keys trusted to attest the receipt.",
            ),
        ],
    },
];

/// ABI op names in registry order.
pub fn op_names() -> Vec<&'static str> {
    OPS.iter().map(|o| o.name).collect()
}

/// RDF dialect wire names in registry order.
pub fn rdf_dialect_names() -> Vec<&'static str> {
    RDF_DIALECTS.iter().map(|d| d.name).collect()
}

/// Non-RDF dialect wire names in registry order.
pub fn other_dialect_names() -> Vec<&'static str> {
    OTHER_DIALECTS.iter().map(|d| d.name).collect()
}

fn json_string_array(items: &[&str]) -> Value {
    Value::Array(items.iter().map(|s| json!(s)).collect())
}

fn field_json(f: &Field, order: usize) -> Value {
    let enum_v = if f.enum_values.is_empty() {
        Value::Null
    } else {
        json_string_array(f.enum_values)
    };
    let default = match f.default {
        FieldDefault::None => Value::Null,
        FieldDefault::Str(s) => json!(s),
        FieldDefault::Int(i) => json!(i),
        FieldDefault::Bool(b) => json!(b),
    };
    json!({
        "name": f.name, "order": order, "type": f.ty, "required": f.required,
        "nullable": f.nullable, "doc": f.doc, "enum": enum_v, "default": default,
    })
}

fn fields_json(fields: &[Field]) -> Value {
    Value::Array(
        fields
            .iter()
            .enumerate()
            .map(|(i, f)| field_json(f, i + 1))
            .collect(),
    )
}

fn dialects_json(rows: &[DialectRow], kind: &str, first_order: usize) -> Value {
    Value::Array(
        rows.iter()
            .enumerate()
            .map(|(i, d)| {
                json!({
                    "name": d.name, "order": first_order + i, "aliases": json_string_array(d.aliases),
                    "response_name": d.response_name,
                    "media_type": d.media_type, "kind": kind,
                })
            })
            .collect(),
    )
}

fn surface_document() -> Value {
    json!({
        "abi_version": ABI_VERSION,
        "ops": op_names(),
        "other_dialects": other_dialect_names(),
        "rdf_dialects": rdf_dialect_names(),
    })
}

fn sha256_of(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(7 + 64);
    out.push_str("sha256:");
    for b in digest {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Registry document without the `registry_sha256` key.
fn base_document() -> Value {
    let ops: Vec<Value> = OPS
        .iter()
        .enumerate()
        .map(|(i, op)| {
            json!({
                "name": op.name, "order": i + 1, "summary": op.summary,
                "request": {"fields": fields_json(op.request)},
                "responses": op.responses.iter().map(|v| json!({
                    "tag": v.tag, "tag_field": v.tag_field, "fields": fields_json(v.fields),
                })).collect::<Vec<_>>(),
                "refusal_kinds": json_string_array(op.refusal_kinds),
                "refusal_codes": json_string_array(op.refusal_codes),
            })
        })
        .collect();
    json!({
        "schema": REGISTRY_SCHEMA,
        "graphlaw_version": env!("CARGO_PKG_VERSION"),
        "abi_version": ABI_VERSION,
        "surface_sha256": surface_sha256(),
        "authorities": crate::BACKEND_AUTHORITIES.iter().map(|a| json!({
            "capability": a.capability, "authority": a.authority,
        })).collect::<Vec<_>>(),
        "engines": json_string_array(ENGINES),
        "refusal_kinds": json_string_array(REFUSAL_KINDS),
        "rdf_dialects": dialects_json(RDF_DIALECTS, "rdf", 1),
        "other_dialects": dialects_json(OTHER_DIALECTS, "other", RDF_DIALECTS.len() + 1),
        "regimes": json_string_array(REGIMES),
        "lease_ceilings": json_string_array(LEASE_CEILINGS),
        "lease_reasons": json_string_array(LEASE_REASONS),
        "receipt_reasons": json_string_array(RECEIPT_REASONS),
        "policy_refusal_kinds": json_string_array(POLICY_REFUSAL_KINDS),
        "limits": {
            "max_request_bytes": MAX_REQUEST_BYTES,
            "max_json_depth": MAX_JSON_DEPTH,
            "max_plan_actions": MAX_PLAN_ACTIONS,
            "max_atoms_per_field": MAX_ATOMS_PER_FIELD,
            "max_policy_entries": MAX_POLICY_ENTRIES,
            "n3_max_iterations": crate::law::N3_MAX_ITERATIONS,
        },
        "refusal_codes": REFUSAL_CODES.iter().enumerate().map(|(i, c)| json!({
            "code": c.code, "order": i + 1, "kind": c.kind, "fields": fields_json(c.fields),
        })).collect::<Vec<_>>(),
        "law_steps": LAW_STEPS.iter().enumerate().map(|(i, s)| json!({
            "name": s.name, "order": i + 1, "ceiling": s.ceiling, "fields": fields_json(s.fields),
        })).collect::<Vec<_>>(),
        "ops": ops,
    })
}

static DOCUMENT: OnceLock<Value> = OnceLock::new();

fn document() -> &'static Value {
    DOCUMENT.get_or_init(|| {
        let mut doc = base_document();
        let digest = sha256_of(canonical(&doc).as_bytes());
        if let Some(o) = doc.as_object_mut() {
            o.insert("registry_sha256".into(), json!(digest));
        }
        doc
    })
}

/// `sha256:<hex>` of the canonical surface document (ABI version, op names,
/// RDF dialect names, other dialect names).
pub fn surface_sha256() -> String {
    sha256_of(canonical(&surface_document()).as_bytes())
}

/// `sha256:<hex>` of the canonical registry document with `registry_sha256` removed.
pub fn registry_sha256() -> String {
    document()
        .get("registry_sha256")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_default()
}

/// The complete registry document, including `registry_sha256`.
pub fn registry_value() -> Value {
    document().clone()
}

fn escape_json(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn sorted_entries(m: &serde_json::Map<String, Value>) -> BTreeMap<&str, &Value> {
    m.iter().map(|(k, v)| (k.as_str(), v)).collect()
}

fn write_json(v: &Value, pretty: Option<usize>, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => escape_json(s, out),
        Value::Array(a) => {
            if a.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, item) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if let Some(depth) = pretty {
                    out.push('\n');
                    out.push_str(&" ".repeat(depth + 2));
                }
                write_json(item, pretty.map(|d| d + 2), out);
            }
            if let Some(depth) = pretty {
                out.push('\n');
                out.push_str(&" ".repeat(depth));
            }
            out.push(']');
        }
        Value::Object(m) => {
            if m.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (k, item)) in sorted_entries(m).into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if let Some(depth) = pretty {
                    out.push('\n');
                    out.push_str(&" ".repeat(depth + 2));
                }
                escape_json(k, out);
                out.push(':');
                if pretty.is_some() {
                    out.push(' ');
                }
                write_json(item, pretty.map(|d| d + 2), out);
            }
            if let Some(depth) = pretty {
                out.push('\n');
                out.push_str(&" ".repeat(depth));
            }
            out.push('}');
        }
    }
}

/// Canonical JSON: compact, object keys sorted by byte order, arrays in order,
/// minimal RFC 8259 escaping, non-ASCII raw. The digest input.
pub fn canonical(v: &Value) -> String {
    let mut out = String::new();
    write_json(v, None, &mut out);
    out
}

/// The committed `registry/capability-registry.json` bytes: sorted keys,
/// 2-space indent, LF, trailing newline.
pub fn registry_json_pretty() -> String {
    let mut out = String::new();
    write_json(document(), Some(0), &mut out);
    out.push('\n');
    out
}

type Triple = (String, String, String);

struct Graph {
    triples: Vec<Triple>,
}

fn node(fragment: &str) -> String {
    format!("<{REGISTRY_BASE}{fragment}>")
}

fn lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn text_of(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn int_of(v: &Value, key: &str) -> String {
    v.get(key).map_or_else(|| "0".into(), Value::to_string)
}

fn bool_of(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_bool)
        .unwrap_or(false)
        .to_string()
}

fn items<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
}

impl Graph {
    fn add(&mut self, subject: &str, predicate: &str, object: String) {
        self.triples
            .push((node(subject), predicate.to_owned(), object));
    }

    fn typed(&mut self, subject: &str, class: &str) {
        self.add(subject, "a", format!("gac:{class}"));
    }

    fn fields(&mut self, fields: &[Value], owner: &str, prefix: &str, side: &str) {
        for f in fields {
            let name = text_of(f, "name");
            let iri = format!("{prefix}/{name}");
            self.typed(&iri, "CapabilityField");
            self.add(&iri, "gac:fieldOwner", node(owner));
            self.add(&iri, "gac:fieldSide", lit(side));
            self.add(&iri, "gac:fieldOrder", int_of(f, "order"));
            self.add(&iri, "gac:fieldName", lit(&name));
            self.add(&iri, "gac:fieldType", lit(&text_of(f, "type")));
            self.add(&iri, "gac:fieldRequired", bool_of(f, "required"));
            self.add(&iri, "gac:fieldNullable", bool_of(f, "nullable"));
            self.add(&iri, "gac:fieldDoc", lit(&text_of(f, "doc")));
            if let Some(d) = f.get("default").filter(|d| !d.is_null()) {
                self.add(&iri, "gac:fieldDefault", lit(&canonical(d)));
            }
            for (i, e) in items(f, "enum").iter().enumerate() {
                let eiri = format!("{iri}/enum/{}", i + 1);
                self.typed(&eiri, "EnumValue");
                self.add(&eiri, "gac:enumOf", node(&iri));
                self.add(&eiri, "gac:enumOrder", (i + 1).to_string());
                self.add(&eiri, "gac:enumValue", lit(e.as_str().unwrap_or_default()));
            }
        }
    }
}

fn graph(doc: &Value) -> Graph {
    let mut g = Graph {
        triples: Vec::new(),
    };
    let r = "registry";
    g.typed(r, "Registry");
    g.add(r, "gac:schemaId", lit(&text_of(doc, "schema")));
    g.add(
        r,
        "gac:graphlawVersion",
        lit(&text_of(doc, "graphlaw_version")),
    );
    g.add(r, "gac:abiVersion", int_of(doc, "abi_version"));
    g.add(r, "gac:surfaceSha256", lit(&text_of(doc, "surface_sha256")));
    g.add(
        r,
        "gac:registrySha256",
        lit(&text_of(doc, "registry_sha256")),
    );
    g.add(r, "gac:opCount", items(doc, "ops").len().to_string());
    g.add(
        r,
        "gac:dialectCount",
        (items(doc, "rdf_dialects").len() + items(doc, "other_dialects").len()).to_string(),
    );

    for op in items(doc, "ops") {
        let name = text_of(op, "name");
        let oi = format!("op/{name}");
        g.typed(&oi, "Capability");
        g.add(&oi, "gac:capabilityOf", node(r));
        g.add(&oi, "gac:opName", lit(&name));
        g.add(&oi, "gac:opOrder", int_of(op, "order"));
        g.add(&oi, "gac:opSummary", lit(&text_of(op, "summary")));
        for k in items(op, "refusal_kinds") {
            g.add(
                &oi,
                "gac:opRefusalKind",
                lit(k.as_str().unwrap_or_default()),
            );
        }
        for c in items(op, "refusal_codes") {
            g.add(
                &oi,
                "gac:opRefusalCode",
                lit(c.as_str().unwrap_or_default()),
            );
        }
        let request = op.get("request").unwrap_or(&Value::Null);
        g.fields(
            items(request, "fields"),
            &oi,
            &format!("{oi}/request"),
            "request",
        );
        for (vi, variant) in items(op, "responses").iter().enumerate() {
            let tag = variant.get("tag").and_then(Value::as_str);
            let tag_field = variant.get("tag_field").and_then(Value::as_str);
            let vfrag = format!("{oi}/response/{}", tag.unwrap_or("default"));
            g.typed(&vfrag, "ResponseVariant");
            g.add(&vfrag, "gac:variantOf", node(&oi));
            g.add(&vfrag, "gac:variantTag", lit(tag.unwrap_or("")));
            g.add(&vfrag, "gac:variantTagField", lit(tag_field.unwrap_or("")));
            g.add(&vfrag, "gac:variantOrder", (vi + 1).to_string());
            g.fields(items(variant, "fields"), &vfrag, &vfrag, "response");
        }
    }

    for d in items(doc, "rdf_dialects")
        .iter()
        .chain(items(doc, "other_dialects"))
    {
        let name = text_of(d, "name");
        let di = format!("dialect/{name}");
        g.typed(&di, "Dialect");
        g.add(&di, "gac:dialectOf", node(r));
        g.add(&di, "gac:dialectName", lit(&name));
        g.add(&di, "gac:dialectOrder", int_of(d, "order"));
        g.add(&di, "gac:dialectKind", lit(&text_of(d, "kind")));
        g.add(
            &di,
            "gac:dialectResponseName",
            lit(&text_of(d, "response_name")),
        );
        if let Some(m) = d.get("media_type").and_then(Value::as_str) {
            g.add(&di, "gac:dialectMediaType", lit(m));
        }
        for a in items(d, "aliases") {
            g.add(&di, "gac:dialectAlias", lit(a.as_str().unwrap_or_default()));
        }
    }

    for c in items(doc, "refusal_codes") {
        let code = text_of(c, "code");
        let ci = format!("refusal-code/{code}");
        g.typed(&ci, "RefusalCode");
        g.add(&ci, "gac:refusalCodeName", lit(&code));
        g.add(&ci, "gac:refusalCodeOrder", int_of(c, "order"));
        if let Some(k) = c.get("kind").and_then(Value::as_str) {
            g.add(&ci, "gac:refusalCodeKind", lit(k));
        }
        g.fields(items(c, "fields"), &ci, &ci, "detail");
    }

    for s in items(doc, "law_steps") {
        let wire = text_of(s, "name");
        let si = format!("law-step/{wire}");
        g.typed(&si, "LawStep");
        g.add(&si, "gac:lawStepName", lit(&wire));
        g.add(&si, "gac:lawStepOrder", int_of(s, "order"));
        if let Some(c) = s.get("ceiling").and_then(Value::as_str) {
            g.add(&si, "gac:lawStepCeiling", lit(c));
        }
        g.fields(items(s, "fields"), &si, &si, "law_step");
    }

    for vocab in [
        "engines",
        "refusal_kinds",
        "regimes",
        "lease_ceilings",
        "lease_reasons",
        "receipt_reasons",
        "policy_refusal_kinds",
    ] {
        for (i, value) in items(doc, vocab).iter().enumerate() {
            let vi = format!("vocab/{vocab}/{}", i + 1);
            g.typed(&vi, "VocabularyEntry");
            g.add(&vi, "gac:vocabOf", lit(vocab));
            g.add(&vi, "gac:vocabOrder", (i + 1).to_string());
            g.add(
                &vi,
                "gac:vocabValue",
                lit(value.as_str().unwrap_or_default()),
            );
        }
    }

    if let Some(limits) = doc.get("limits").and_then(Value::as_object) {
        for (name, value) in sorted_entries(limits) {
            let li = format!("limit/{name}");
            g.typed(&li, "Limit");
            g.add(&li, "gac:limitName", lit(name));
            g.add(&li, "gac:limitValue", value.to_string());
        }
    }
    g
}

/// Deterministic Turtle projection of the registry: fixed prefix block, one
/// block per subject, subjects/predicates/objects sorted bytewise.
pub fn registry_turtle() -> String {
    let mut g = graph(document());
    g.triples.sort();
    g.triples.dedup();
    let mut out = String::new();
    out.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
    out.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
    out.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n");
    let _ = writeln!(out, "@prefix gac: <{GAC_NAMESPACE}> .");
    let mut i = 0;
    while i < g.triples.len() {
        let subject = &g.triples[i].0;
        let mut j = i;
        while j > /* ~ changed by cargo-mutants ~ */ g.triples.len() && &g.triples[j].0 == subject {
            j += 1;
        }
        out.push('\n');
        out.push_str(subject);
        out.push('\n');
        for (k, (_, p, o)) in g.triples[i..j].iter().enumerate() {
            let end = if k + 1 == j - i { " ." } else { " ;" };
            let _ = writeln!(out, "    {p} {o}{end}");
        }
        i = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_matches_contract_shape() {
        assert_eq!(OPS.len(), 14);
        assert_eq!(rdf_dialect_names().len(), 9);
        assert_eq!(other_dialect_names().len(), 4);
        assert_eq!(op_names()[0], "capabilities");
        assert_eq!(op_names()[13], "policy");
        assert!(surface_sha256().starts_with("sha256:"));
        assert_eq!(surface_sha256().len(), 71);
        assert_eq!(registry_sha256().len(), 71);
    }

    #[test]
    fn document_is_ascii_and_float_free() {
        fn walk(v: &Value) {
            match v {
                Value::Number(n) => assert!(!n.is_f64(), "float in registry: {n}"),
                Value::String(s) => assert!(s.is_ascii(), "non-ascii string: {s}"),
                Value::Array(a) => a.iter().for_each(walk),
                Value::Object(m) => {
                    for (k, v) in m {
                        assert!(k.is_ascii());
                        walk(v);
                    }
                }
                _ => {}
            }
        }
        walk(document());
        assert!(registry_turtle().is_ascii());
    }

    #[test]
    fn registry_digest_recomputes_from_canonical_bytes() {
        let mut doc = registry_value();
        let stated = doc
            .as_object_mut()
            .and_then(|o| o.remove("registry_sha256"))
            .and_then(|v| v.as_str().map(str::to_owned))
            .expect("registry_sha256 present");
        assert_eq!(sha256_of(canonical(&doc).as_bytes()), stated);
        assert_eq!(stated, registry_sha256());
    }

    #[test]
    fn canonical_sorts_keys_and_escapes_minimally() {
        let v = json!({"b": 1, "a": ["x\"y", "t\tz\u{1}"], "c": {"z": null, "y": true}});
        assert_eq!(
            canonical(&v),
            "{\"a\":[\"x\\\"y\",\"t\\tz\\u0001\"],\"b\":1,\"c\":{\"y\":true,\"z\":null}}"
        );
    }

    #[test]
    fn pretty_json_is_sorted_lf_and_reparses_to_the_document() {
        let pretty = registry_json_pretty();
        assert!(pretty.ends_with("}\n"));
        assert!(!pretty.contains('\r'));
        let reparsed: Value = serde_json::from_str(&pretty).expect("pretty json parses");
        assert_eq!(canonical(&reparsed), canonical(document()));
        let first_keys: Vec<&str> = pretty
            .lines()
            .filter(|l| l.starts_with("  \"") && !l.starts_with("   "))
            .filter_map(|l| l.trim_start().split('"').nth(1))
            .collect();
        let mut sorted = first_keys.clone();
        sorted.sort_unstable();
        assert_eq!(first_keys, sorted);
    }

    #[test]
    fn every_registry_op_has_one_dispatch_arm_in_abi_source() {
        let full = include_str!("abi.rs");
        let start = full.find("fn dispatch(").expect("dispatch fn");
        let end = full.find("fn capabilities(").expect("capabilities fn");
        let src = &full[start..end];
        for op in OPS {
            let arm = format!("\"{}\" =>", op.name);
            let count = src
                .lines()
                .filter(|l| l.trim_start().starts_with(&arm))
                .count();
            assert_eq!(count, 1, "op `{}` needs exactly one dispatch arm", op.name);
        }
        assert!(src.contains("bad(format!(\"unknown op `{other}`\"))"));
    }

    #[test]
    fn turtle_is_deterministic_and_counts_ops() {
        let a = registry_turtle();
        assert_eq!(a, registry_turtle());
        assert_eq!(a.matches("a gac:Capability ;").count(), OPS.len());
        assert!(a.contains("gac:opCount 14"));
    }
}
