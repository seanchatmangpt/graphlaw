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
pub const ENGINES: &[&str] = &["PurRdf", "FrozenDuckdb"];
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
        name: "sql",
        aliases: &[],
        response_name: "SqlRdf",
        media_type: Some("text/turtle"),
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
                "registry_blake3",
                "string",
                "Registry digest; absent on engines older than v26.9.29.",
            ),
            opt(
                "surface_blake3",
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
            req("engine", "string", "Owning engine (PurRdf or FrozenDuckdb)."),
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
            "Reasoner output; shape is owned by purrdf::n3.",
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
        code: "DuckDb",
        kind: Some("DuckDb"),
        fields: &[req(
            "message",
            "string",
            "Diagnostic text from the DuckDB substrate (ABI op `duckdb`; dispatches inside the graphlaw-wasm unit on every target (in-module, not host-only) but stays off the hashed 14-op table for digest stability).",
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
        name: "sql",
        ceiling: Some("observe"),
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

/// Dispatch ops kept off the hashed `OPS` list so `registry_blake3` stays
/// stable until an explicit digest bump. `"duckdb"` is module-owned: it links
/// into the wasm unit and executes inside the module, but it stays unadvertised
/// from `op_names()` / `capabilities.ops` (digest stability). Absence from
/// `op_names()` / `capabilities` is not a capability refusal and is not
/// REFUSAL-04.
/// `tests/registry_dispatch.rs` composes the full native dispatch contract as
/// `op_names() + NATIVE_ONLY_OPS` and asserts it equals the parsed `fn dispatch`
/// arms.
pub const NATIVE_ONLY_OPS: &[&str] = &["duckdb"];

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
