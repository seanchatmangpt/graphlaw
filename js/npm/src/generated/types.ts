// GENERATED from registry/capability-registry.json
// registry_sha256: sha256:1c129dd6eed90bc680f8e695070dcef2eccf0eab7ff4d2b9c0462b2439c514b9
// Do not edit by hand; run `npm run gen` to regenerate.

import type { DataSpec, Term } from '../types.js';

// Structured response/request shapes documented in the ABI contract report
// (the registry types these as bare object / list<object>).
/**
 * One SHACL validation result (shacl response `results`).
 */
export interface ShaclResult {
  /** Focus node of the violation. */
  focus: string;
  /** Property path (IRI) the violation was found on. */
  path: string;
  /** offending value. */
  value: unknown;
  /** Severity IRI (e.g. sh:Violation). */
  severity: string;
  /** Constraint component IRI (e.g. sh:MinCountConstraintComponent). */
  component: string;
  /** Shape IRI that produced the result. */
  shape: string;
  /** Human-readable message, when present. */
  message?: string;
}

/**
 * One ShEx validation entry (shex response `entries`).
 */
export interface ShexEntry {
  /** Node that was validated. */
  node: Term;
  /** Shape IRI it was validated against. */
  shape: string;
  /** Validation status. */
  status: 'conformant' | 'nonconformant' | string;
  /** Failure reason, when nonconformant. */
  reason?: string;
}

/**
 * One hook firing within a round (hooks response `firings`).
 */
export interface HookFiring {
  /** Hook name. */
  hook: string;
  /** Round in which it fired. */
  round: number;
  /** Quads added by the hook. */
  added: number;
  /** Resulting row. */
  row: unknown;
}

/**
 * One Datalog rule (datalog request `rules`).
 */
export interface DatalogRule {
  /** Rule head as a [s, p, o] triple. */
  head: [string, string, string];
  /** Rule body as a list of [s, p, o] triples. */
  body: [string, string, string][];
}

/** Time-boxed authority grant for law steps. */
export interface Lease {
  /** Lease identifier. */
  id: string;
  /** Identity of the lease holder. */
  holder: string;
  /** Highest authority the lease grants. */
  ceiling: Ceiling;
  /** Step names the lease covers. */
  scope: string[];
  /** Expired when now >= expires_unix. */
  expires_unix: number;
  /** Issuer-stated issue time; 0 means not stated. */
  issued_unix: number;
}

/** A lease with its Ed25519 attestation. */
export interface SignedLease {
  /** The lease being attested. */
  lease: Lease;
  /** Signature over the lease. */
  attestation: Attestation;
}

/** Record of one executed law step. */
export interface Receipt {
  /** Id of the parent state. */
  parent: string;
  /** Id of the child state. */
  child: string;
  /** Step name, for example derive:rdfs. */
  step: string;
  /** Backend authority capability. */
  authority: string;
  /** Backend authority revision. */
  revision: string;
  /** Quads added by the step. */
  added: number;
  /** Lease id when the step ran under a lease. */
  lease_id?: string | null;
  /** Plan digest for plan steps. */
  plan_sha256?: string | null;
  /** Digest of the external subject, when bound. */
  subject_sha256?: string | null;
}

/** Ed25519 attestation over a canonical payload. */
export interface Attestation {
  /** Lowercase hex SHA-256 of the canonical payload. */
  payload_sha256: string;
  /** Identifier of the signing key. */
  key_id: string;
  /** Lowercase hex Ed25519 signature over the payload. */
  signature: string;
}

/** Ordered candidate plan and the goal it claims to reach. */
export interface Plan {
  /** Actions in replay order. */
  actions: Action;
  /** Triples that must hold after the last action. */
  goal: string;
  /** Triples that must be absent after the last action. */
  goal_not: string;
}

/** One plan action over ground triples. */
export interface Action {
  /** Action name, for example go a b. */
  name: string;
  /** Triples that must hold before the action. */
  pre: string;
  /** Triples that must be absent before the action. */
  pre_not: string;
  /** Triples added by the action. */
  add: string;
  /** Triples removed by the action. */
  del: string;
}

/** One FOND policy entry. */
export interface PolicyEntry {
  /** State the entry applies to. */
  state: string;
  /** Action chosen in the state. */
  action: string;
  /** Possible outcomes of the action. */
  outcomes: PolicyOutcome;
}

/** One possible outcome of a policy action. */
export interface PolicyOutcome {
  /** State reached by the outcome. */
  state: string;
  /** Probability in parts per million. */
  probability_ppm: number;
}

/** Registry enum Ceiling. */
export type Ceiling = "observe" | "select" | "construct";
export const Ceiling_VALUES = ["observe", "select", "construct"] as const;

/** Registry enum LeaseReason. */
export type LeaseReason = "expired" | "out_of_scope" | "ceiling" | "bad_signature" | "untrusted_key" | "clock_skew";
export const LeaseReason_VALUES = ["expired", "out_of_scope", "ceiling", "bad_signature", "untrusted_key", "clock_skew"] as const;

/** Registry enum ReceiptReason. */
export type ReceiptReason = "unattested" | "bad_signature" | "untrusted_key";
export const ReceiptReason_VALUES = ["unattested", "bad_signature", "untrusted_key"] as const;

/** Registry enum PolicyRefusalKind. */
export type PolicyRefusalKind = "MissingEntry" | "InventedOutcome" | "BadMass" | "DeadEnd" | "Malformed";
export const PolicyRefusalKind_VALUES = ["MissingEntry", "InventedOutcome", "BadMass", "DeadEnd", "Malformed"] as const;

/** Refusal kinds (error.kind). */
export type RefusalKind = "NotSemanticContent" | "Ambiguous" | "EngineRejected" | "Unsupported" | "ResourceLimit";
export const REFUSAL_KIND_VALUES = ["NotSemanticContent", "Ambiguous", "EngineRejected", "Unsupported", "ResourceLimit"] as const;

/** Refusal detail codes (error.details.code). */
export type RefusalCode = "Refused" | "ResourceLimit" | "PlanRefused" | "ReceiptRequired" | "ReceiptRefused" | "LeaseRefused" | "NotAdmitted" | "UnverifiedLeaseRefused" | "PolicyRefused";

/** Backend engines. */
export type Engine = "PurRdf" | "Eyeron";

/** Dialect wire names accepted in requests. */
export type DialectName = "turtle" | "trig" | "ntriples" | "nquads" | "rdfxml" | "jsonld" | "yamlld" | "trix" | "hextuples" | "n3" | "sparql" | "shexc" | "shexj";
/** Dialect names as reported in responses (Rust variant names). */
export type DialectResponseName = "Turtle" | "TriG" | "NTriples" | "NQuads" | "RdfXml" | "JsonLd" | "YamlLd" | "TriX" | "HexTuples" | "N3" | "Sparql" | "ShExC" | "ShExJ";

/** Entailment regimes. */
export type Regime = "simple" | "rdf" | "rdfs" | "owl-rl" | "d";

/** Report ABI version, crate version, semantic authorities, dialects, ops and registry digests. */
export interface CapabilitiesRequest {

}

/** Success response for capabilities. */
export interface CapabilitiesResponse {
  /** ABI revision (legacy alias of abi_version). */
  abi?: number;
  /** ABI revision. */
  abi_version?: number;
  /** GraphLaw crate version. */
  crate?: string;
  /** Semantic authority map: {capability, authority, revision}. */
  authorities?: Record<string, unknown>[];
  /** RDF dialect wire names. */
  rdf_dialects?: string[];
  /** Non-RDF dialect wire names. */
  other_dialects?: string[];
  /** ABI op names in registry order. */
  ops?: string[];
  /** Registry schema id; absent on engines older than v26.9.29. */
  registry_schema?: string;
  /** Registry digest; absent on engines older than v26.9.29. */
  registry_sha256?: string;
  /** Surface digest; absent on engines older than v26.9.29. */
  surface_sha256?: string;
}

/** Route text to a dialect and owning engine without parsing it. */
export interface SniffRequest {
  /** Text to route. */
  text: string;
  /** File extension or name hint. */
  hint?: string | null;
}

/** Success response for sniff. */
export interface SniffResponse {
  /** Sniffed dialect (Rust variant name). */
  dialect?: string;
  /** Owning engine (PurRdf or Eyeron). */
  engine?: string;
}

/** Parse text with the owning engine; counts quads or validates syntax. */
export interface ParseRequest {
  /** Text to parse. */
  text: string;
  /** Dialect name; sniffed when omitted. */
  dialect?: string | null;
  /** File extension or name hint for sniffing. */
  hint?: string | null;
  /** Base IRI. */
  base?: string | null;
}

/** Success response for parse. */
export interface ParseResponse {
  /** Dialect used (Rust variant name). */
  dialect?: string;
  /** Quad count; present for RDF dialects. */
  quads?: number;
  /** Content-addressed state id; present for RDF dialects. */
  id?: string;
  /** Syntax validity; present for non-RDF dialects. */
  valid?: boolean;
}

/** Serialize parsed RDF text into another RDF dialect. */
export interface ConvertRequest {
  /** Text to convert. */
  text: string;
  /** Target RDF dialect name. */
  to: string;
  /** Source dialect name; sniffed when omitted. */
  dialect?: string | null;
  /** File extension or name hint for sniffing. */
  hint?: string | null;
  /** Base IRI. */
  base?: string | null;
}

/** Success response for convert. */
export interface ConvertResponse {
  /** Serialized output. */
  text?: string;
  /** Content-addressed id of the source state. */
  id?: string;
}

/** Canonicalize a dataset to N-Quads and return its content-addressed id. */
export interface CanonicalRequest {
  /** Data to operate on: an object {text, dialect?, hint?, base?}; the dialect is sniffed when omitted. */
  data: DataSpec;
}

/** Success response for canonical. */
export interface CanonicalResponse {
  /** Content-addressed state id. */
  id?: string;
  /** Canonical N-Quads. */
  nquads?: string;
  /** Quad count. */
  quads?: number;
}

/** Evaluate a SPARQL query over a dataset. */
export interface SparqlRequest {
  /** Data to operate on: an object {text, dialect?, hint?, base?}; the dialect is sniffed when omitted. */
  data: DataSpec;
  /** SPARQL query text. */
  query: string;
  /** Base IRI. */
  base?: string | null;
}

/** Success response union for sparql, discriminated by kind. */
export type SparqlResponse =
  | (SparqlSolutions)
  | (SparqlGraph)
  | (SparqlBoolean)
;
/** Success response for sparql (solutions). */
export interface SparqlSolutions {
  kind: "solutions";
  /** Projected variable names. */
  variables?: string[];
  /** Solution rows; null marks an unbound cell. */
  rows?: (Term | null)[][];
}

/** Success response for sparql (graph). */
export interface SparqlGraph {
  kind: "graph";
  /** Constructed graph as N-Quads. */
  nquads?: string;
  /** Quad count of the constructed graph. */
  quads?: number;
}

/** Success response for sparql (boolean). */
export interface SparqlBoolean {
  kind: "boolean";
  /** ASK result. */
  value?: boolean;
}


/** Validate a dataset against SHACL shapes. */
export interface ShaclRequest {
  /** Data to operate on: an object {text, dialect?, hint?, base?}; the dialect is sniffed when omitted. */
  data: DataSpec;
  /** SHACL shapes graph as Turtle text. */
  shapes: string;
  /** Base IRI. */
  base?: string | null;
}

/** Success response for shacl. */
export interface ShaclResponse {
  /** True when the data conforms. */
  conforms?: boolean;
  /** Validation results: {focus, path, value, severity, component, shape, message}. */
  results?: ShaclResult[];
}

/** Validate a dataset against a ShEx schema and shape map. */
export interface ShexRequest {
  /** Data to operate on: an object {text, dialect?, hint?, base?}; the dialect is sniffed when omitted. */
  data: DataSpec;
  /** ShEx schema text. */
  schema: string;
  /** Schema syntax. */
  schema_dialect?: string | null;
  /** Shape map text. */
  map: string;
  /** Base IRI. */
  base?: string | null;
}

/** Success response for shex. */
export interface ShexResponse {
  /** True when every map entry is conformant. */
  conforms?: boolean;
  /** Shape map entries: {node, shape, status, reason}. */
  entries?: ShexEntry[];
}

/** Run bounded Notation3 forward reasoning over a document. */
export interface N3Request {
  /** Notation3 document with data and rules. */
  text: string;
}

/** Success response for n3. */
export interface N3Response {
  /** Reasoner output; shape is owned by Eyeron. */
  derived?: unknown;
}

/** Materialize an entailment regime over a dataset. */
export interface EntailRequest {
  /** Data to operate on: an object {text, dialect?, hint?, base?}; the dialect is sniffed when omitted. */
  data: DataSpec;
  /** Entailment regime. */
  regime: string;
}

/** Success response for entail. */
export interface EntailResponse {
  /** Closure as N-Quads. */
  nquads?: string;
  /** Quads added by materialization. */
  added?: number;
}

/** Evaluate Datalog rules over triple facts to a fixpoint. */
export interface DatalogRequest {
  /** Rules: {head: [s,p,o], body: [[s,p,o], ...]}; terms starting with ? are variables. */
  rules: DatalogRule[];
  /** Ground facts as [subject, predicate, object] strings. */
  facts: string[][];
}

/** Success response for datalog. */
export interface DatalogResponse {
  /** Number of facts in the fixpoint. */
  count?: number;
  /** Sorted fixpoint facts as [subject, predicate, object]. */
  facts?: string[][];
}

/** Materialize a knowledge-hook pack over a dataset. */
export interface HooksRequest {
  /** Hook pack as a data spec (kh: vocabulary). */
  pack: DataSpec;
  /** Data to operate on: an object {text, dialect?, hint?, base?}; the dialect is sniffed when omitted. */
  data: DataSpec;
}

/** Success response for hooks. */
export interface HooksResponse {
  /** Content-addressed id of the materialized state. */
  id?: string;
  /** Fixpoint rounds executed. */
  rounds?: number;
  /** Quad count of the materialized state. */
  quads?: number;
  /** Materialized state as N-Quads. */
  nquads?: string;
  /** Hook firings: {hook, round, added, row}. */
  firings?: HookFiring[];
}

/** Run an ordered chain of law steps with receipts and optional lease authorization. */
export interface LawRequest {
  /** Data to operate on: an object {text, dialect?, hint?, base?}; the dialect is sniffed when omitted. */
  data: DataSpec;
  /** Ordered law steps; see law_steps. */
  steps: Record<string, unknown>[];
  /** Signed lease {lease, attestation}; verified offline against trusted_keys. */
  signed_lease?: SignedLease;
  /** Unsigned lease; refused unless unverified_lease is true. */
  lease?: Lease;
  /** Hex Ed25519 public keys trusted to sign leases. */
  trusted_keys?: string[];
  /** Clock skew tolerance for signed leases in seconds. */
  max_skew_secs?: number | null;
  /** Accept an unsigned lease; proves nothing about its issuer. */
  unverified_lease?: boolean;
  /** Caller clock in unix seconds; used only with an unverified lease. */
  now_unix?: number;
}

/** Success response for law. */
export interface LawResponse {
  /** State ids after each step, starting with the input. */
  states?: string[];
  /** Receipts, one per transition. */
  receipts?: Receipt[];
  /** Final state as N-Quads. */
  nquads?: string;
}

/** Admit a FOND policy as strong-cyclic against a planning problem. */
export interface PolicyRequest {
  /** Planning problem as a JSON object or JSON text. */
  problem: string | Record<string, unknown>;
  /** Policy as a JSON object, entries array or JSON text. */
  policy: string | Record<string, unknown>;
}

/** Success response for policy. */
export interface PolicyResponse {
  /** Initial states of the problem. */
  initial_states?: string[];
  /** States reachable under the policy. */
  reachable?: string[];
  /** Reachable goal states. */
  goal_states?: string[];
  /** Admitted policy entries as [state, action]. */
  entries?: string[][];
  /** Admitted policy as N-Triples. */
  ntriples?: string;
}

