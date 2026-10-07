//! ABI metadata for the `graphlaw-wasm` WASI JSON-ABI module.
//
// Consumed query columns (meta.rq): crate_name, export_prefix, abi_version,
// max_request_bytes, max_json_depth, ops, error_codes.
// Optional column (meta.rq): max_outstanding_bytes.
// Rendered by ggen (rust-wasi-wasmex-pack) from the wja: graph.
// Edit the ontology and re-render; never edit this file by hand.
#![allow(dead_code)]

/// Name of the crate hosting the module.
pub const CRATE_NAME: &str = "graphlaw-wasm";

/// Prefix of the exported symbols (`<prefix>_abi_version/_alloc/_free/_call`).
pub const EXPORT_PREFIX: &str = "gl";

/// Value returned by `<prefix>_abi_version`.
pub const ABI_VERSION: u32 = 1;

/// Upper bound on request size in bytes; alloc returns null above it.
pub const MAX_REQUEST_BYTES: usize = 16777216;

/// Upper bound on accepted JSON nesting depth.
pub const MAX_JSON_DEPTH: usize = 64;

/// Upper bound on bytes handed to the host and not yet freed; alloc returns null above it.
pub const MAX_OUTSTANDING_BYTES: usize = 268435456;

/// Op table, ordered by `wja:opOrder`.
pub const OPS: &[&str] = &[
    "capabilities",
    "sniff",
    "parse",
    "convert",
    "canonical",
    "sparql",
    "shacl",
    "shex",
    "n3",
    "entail",
    "datalog",
    "hooks",
    "law",
    "policy",
];

/// Typed error codes, ordered by `wja:codeOrder`.
pub const ERROR_CODES: &[&str] = &[
    "Refused",
    "ResourceLimit",
    "PlanRefused",
    "ReceiptRequired",
    "ReceiptRefused",
    "LeaseRefused",
    "NotAdmitted",
    "UnverifiedLeaseRefused",
    "PolicyRefused",
];
