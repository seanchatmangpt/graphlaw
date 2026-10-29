# graphlaw reference

<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-BEGIN: reference body is RIGID                -->
<!-- Every row below is rendered from queries/ast_extract.rq.      -->
<!-- Agents MUST NOT add, edit, reorder, or remove any row or      -->
<!-- table cell. Prose outside the fenced slot below is refused    -->
<!-- by the doc_quality court.                                     -->
<!-- ============================================================= -->

## Modules


### @seanchatmangpt/graphlaw

| `build` | script | npm run gen && node scripts/build.mjs && tsc -p tsconfig.build.json && node scripts/assemble.mjs |  |  |  |  |

| `gen` | script | node scripts/gen-types.mjs |  |  |  |  |

| `lint:pkg` | script | publint && attw --pack . |  |  |  |  |

| `sync-version` | script | node scripts/sync-version.mjs |  |  |  |  |

| `test` | script | vitest run |  |  |  |  |

| `test:browser` | script | vitest run --config vitest.browser.config.ts |  |  |  |  |

| `typecheck` | script | tsc -p tsconfig.json --noEmit && tsc -p tsconfig.test.json --noEmit |  |  |  |  |


### GraphlawEx

| `admit` | function | admit/2 |  |  |  |  |

| `handle_call` | function | handle_call/3 |  |  |  |  |

| `init` | function | init/1 |  |  |  |  |

| `start_link` | function | start_link/1 |  |  |  |  |

| `stop` | function | stop/1 |  |  |  |  |

| `abi_not_wired` | str_key | {:error, :abi_not_wired} |  |  |  |  |

| `instance` | str_key | instance: instance |  |  |  |  |

| `module` | str_key | module: module |  |  |  |  |

| `name` | str_key | name: __MODULE__ |  |  |  |  |

| `store` | str_key | store: store |  |  |  |  |

| `wasm_load_failed` | str_key | {:error, {:wasm_load_failed, path, :enoent}} |  |  |  |  |

| `wasm_path` | str_key | wasm_path: wasm_path |  |  |  |  |


### GraphlawEx.Application

| `start` | function | start/2 |  |  |  |  |

| `name` | str_key | name: GraphlawEx.Supervisor |  |  |  |  |

| `strategy` | str_key | strategy: :one_for_one |  |  |  |  |


### benches/shacl_boundary_cliff.rs

| `../ontologies/sql-ast.ttl` | str_key | CANONICAL_SHAPES = "../ontologies/sql-ast.ttl" |  |  |  |  |

| `@prefix ex: <http://example.org/> .` | str_key | DATA_TTL = "@prefix ex: <http://example.org/> ." |  |  |  |  |

| `@prefix sh: <http://www.w3.org/ns/shacl#> .` | str_key | PREFIX = "@prefix sh: <http://www.w3.org/ns/shacl#> ." |  |  |  |  |


### benches/sieve_cliff.rs

| `urn:s/` | str_key | SUBJECT = "urn:s/" |  |  |  |  |

| `urn:v/` | str_key | VOCAB = "urn:v/" |  |  |  |  |


### benches/sql_rdf_engine.rs

| `@prefix sql: <http://graphlaw.org/sql#> .` | str_key | SELECT_TURTLE = "@prefix sql: <http://graphlaw.org/sql#> ." |  |  |  |  |

| `CREATE TABLE t(id INTEGER, name VARCHAR); ` | str_key | SEED_SQL = "CREATE TABLE t(id INTEGER, name VARCHAR); " |  |  |  |  |


### benches/statutory_boundary_50ms.rs

| `SB_DEBUG` | env_key | std::env::var_os("SB_DEBUG") |  |  |  |  |

| `@prefix : <http://example.org/sb#> .
@prefix math: <http://www.w3.org/2000/10/swap/math#> .
:trade_47 a :ProposedTrade .
:trade_47 :notional 25000000 ;
          :book :emea_deriv .
{ ?t a :ProposedTrade ;
     :notional ?n .
  ?n math:greaterThan 10000000 .
  ?t :book :emea_deriv }
  => { ?t :violates :Rule15c3_1 ; :requires :pretrade_review } .
{ ?t a :ProposedTrade ; :violates :Rule15c3_1 }
  => { :desk :halt :trade_47 } .
` | str_key | PROGRAM = "@prefix : <http://example.org/sb#> .
@prefix math: <http://www.w3.org/2000/10/swap/math#> .
:trade_47 a :ProposedTrade .
:trade_47 :notional 25000000 ;
          :book :emea_deriv .
{ ?t a :ProposedTrade ;
     :notional ?n .
  ?n math:greaterThan 10000000 .
  ?t :book :emea_deriv }
  => { ?t :violates :Rule15c3_1 ; :requires :pretrade_review } .
{ ?t a :ProposedTrade ; :violates :Rule15c3_1 }
  => { :desk :halt :trade_47 } .
" |  |  |  |  |

| `@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/sb#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
ex:TradeShape a sh:NodeShape ;
  sh:targetClass ex:Order ;
  sh:closed true ;
  sh:property [
    sh:path rdf:type ; sh:minCount 1 ; sh:maxCount 1 ; sh:hasValue ex:Order
  ] ;
  sh:property [
    sh:path ex:id ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] ;
  sh:property [
    sh:path ex:acct ; sh:minCount 1 ; sh:maxCount 2 ; sh:nodeKind sh:IRI
  ] ;
  sh:property [
    sh:path ex:qty ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] .
` | str_key | SHAPES_TTL = "@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/sb#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
ex:TradeShape a sh:NodeShape ;
  sh:targetClass ex:Order ;
  sh:closed true ;
  sh:property [
    sh:path rdf:type ; sh:minCount 1 ; sh:maxCount 1 ; sh:hasValue ex:Order
  ] ;
  sh:property [
    sh:path ex:id ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] ;
  sh:property [
    sh:path ex:acct ; sh:minCount 1 ; sh:maxCount 2 ; sh:nodeKind sh:IRI
  ] ;
  sh:property [
    sh:path ex:qty ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] .
" |  |  |  |  |


### benches/tc_hot_01.rs

| `TC_HOT_FULL` | env_key | std::env::var("TC_HOT_FULL") |  |  |  |  |

| `TC_HOT_INJECT_OUTLIER_NS` | env_key | std::env::var("TC_HOT_INJECT_OUTLIER_NS") |  |  |  |  |

| `TC_HOT_SPC_EXPORT` | env_key | std::env::var("TC_HOT_SPC_EXPORT") |  |  |  |  |


### benches/tc_hot_02.rs

| `TC_HOT_CORPUS_PATH` | env_key | std::env::var("TC_HOT_CORPUS_PATH") |  |  |  |  |

| `TC_HOT_FULL` | env_key | std::env::var("TC_HOT_FULL") |  |  |  |  |

| `thread_affinity=unavailable_stable_rust_macos (core_affinity crate not added)` | str_key | ISOLATION_DISCLOSURE = "thread_affinity=unavailable_stable_rust_macos (core_affinity crate not added)" |  |  |  |  |


### examples/allen_datalog.rs

| `p` | str_key | NAMES = "p" |  |  |  |  |


### examples/asp_tableaux_vector_gen.rs

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `3.0.1` | str_key | PURRDF = "3.0.1" |  |  |  |  |

| `purrdf-n3/3.0.1` | str_key | N3_ENGINE = "purrdf-n3/3.0.1" |  |  |  |  |

| `t` | str_key | SUBJ = "t" |  |  |  |  |

| `true` | str_key | TRUE_OBJ = "true" |  |  |  |  |


### examples/bayes_pop_vectors.rs

| `GL_DEBUG` | env_key | std::env::var_os("GL_DEBUG") |  |  |  |  |

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `3.0.1` | str_key | PURRDF = "3.0.1" |  |  |  |  |

| `purrdf-n3/3.0.1` | str_key | N3_ENGINE = "purrdf-n3/3.0.1" |  |  |  |  |


### examples/blackbox_fixture.rs

| `0123456789abcdef0123456789abcdef01234567` | str_key | SUBJECT = "0123456789abcdef0123456789abcdef01234567" |  |  |  |  |


### examples/breed_oracle_gen.rs

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `3.0.1` | str_key | PURRDF = "3.0.1" |  |  |  |  |

| `purrdf-n3/3.0.1` | str_key | N3_ENGINE = "purrdf-n3/3.0.1" |  |  |  |  |


### examples/event_calculus_vectors.rs

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `f` | env_key | var("f") |  |  |  |  |

| `t` | env_key | var("t") |  |  |  |  |

| `3.0.1` | str_key | PURRDF = "3.0.1" |  |  |  |  |

| `CARGO_PKG_VERSION` | str_key | GL_VERSION = "CARGO_PKG_VERSION" |  |  |  |  |

| `event_calculus-paper-narrative` | str_key | SCENARIOS = "event_calculus-paper-narrative" |  |  |  |  |

| `event_calculus_vectors` | str_key | GENERATOR = "event_calculus_vectors" |  |  |  |  |

| `purrdf-n3/3.0.1` | str_key | N3_ENGINE = "purrdf-n3/3.0.1" |  |  |  |  |

| `urn:ec:clipped` | str_key | P_CLIPPED = "urn:ec:clipped" |  |  |  |  |

| `urn:ec:comp` | str_key | P_COMP = "urn:ec:comp" |  |  |  |  |

| `urn:ec:happens` | str_key | P_HAPPENS = "urn:ec:happens" |  |  |  |  |

| `urn:ec:holds` | str_key | P_HOLDS = "urn:ec:holds" |  |  |  |  |

| `urn:ec:initiated` | str_key | P_INITIATED = "urn:ec:initiated" |  |  |  |  |

| `urn:ec:initiates` | str_key | P_INITIATES = "urn:ec:initiates" |  |  |  |  |

| `urn:ec:succ` | str_key | P_SUCC = "urn:ec:succ" |  |  |  |  |

| `urn:ec:terminated` | str_key | P_TERMINATED = "urn:ec:terminated" |  |  |  |  |

| `urn:ec:terminates` | str_key | P_TERMINATES = "urn:ec:terminates" |  |  |  |  |


### examples/naive_physics_vectors.rs

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw` | str_key | OUT_DIR = "/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw" |  |  |  |  |

| `https://np.graphlaw.dev/` | str_key | NS = "https://np.graphlaw.dev/" |  |  |  |  |


### examples/plan_replay_oracle_gen.rs

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `3.0.1` | str_key | PURRDF = "3.0.1" |  |  |  |  |

| `plan_replay_oracle_gen` | str_key | GENERATOR = "plan_replay_oracle_gen" |  |  |  |  |

| `purrdf-n3/3.0.1` | str_key | N3_ENGINE = "purrdf-n3/3.0.1" |  |  |  |  |


### ggen/sql-dialect/sql_extensions.rs

| `SQL_EXTENSIONS` | const | SQL_EXTENSIONS: &[(&str, &str, &str, i64)] |  |  |  |  |

| `autocomplete` | str_key | SQL_EXTENSIONS = "autocomplete" |  |  |  |  |


### ggen/sql-dialect/sql_statement_kinds.rs

| `SQL_STATEMENTS` | const | SQL_STATEMENTS: &[(&str, &str)] |  |  |  |  |

| `ExternalResourceStatement` | str_key | SQL_STATEMENTS = "ExternalResourceStatement" |  |  |  |  |


### src/abi.rs

| `ABI_VERSION` | const | ABI_VERSION: u32 |  |  |  |  |

| `MAX_ATOMS_PER_FIELD` | const | MAX_ATOMS_PER_FIELD: usize |  |  |  |  |

| `MAX_DATALOG_FACTS` | const | MAX_DATALOG_FACTS: usize |  |  |  |  |

| `MAX_DATALOG_RULES` | const | MAX_DATALOG_RULES: usize |  |  |  |  |

| `MAX_JSON_DEPTH` | const | MAX_JSON_DEPTH: usize |  |  |  |  |

| `MAX_LAW_STEPS` | const | MAX_LAW_STEPS: usize |  |  |  |  |

| `MAX_OUTSTANDING_ALLOC_BYTES` | const | MAX_OUTSTANDING_ALLOC_BYTES: usize |  |  |  |  |

| `MAX_PLAN_ACTIONS` | const | MAX_PLAN_ACTIONS: usize |  |  |  |  |

| `MAX_POLICY_ENTRIES` | const | MAX_POLICY_ENTRIES: usize |  |  |  |  |

| `MAX_REQUEST_BYTES` | const | MAX_REQUEST_BYTES: usize |  |  |  |  |

| `call` | function | call(request: &[u8]) -> Vec<u8> |  |  |  |  |

| `call_json` | function | call_json(request: &Value) -> Value |  |  |  |  |

| `dialect_by_name` | function | dialect_by_name(name: &str) -> Result<Dialect, Refusal> |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) -> Vec<u8> |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() -> Vec<u8> |  |  |  |  |

| `unknown_buffer_response` | function | unknown_buffer_response() -> Vec<u8> |  |  |  |  |

| `Fail` | struct | Fail { pub(in crate::abi) refusal: Refusal, pub(in crate::abi) details: Option<Value> } |  |  |  |  |

| `ops::plan_from_json` | use | ops::plan_from_json |  |  |  |  |


### src/abi/ops/governance.rs

| `plan_from_json` | function | plan_from_json(plan: &Value) -> Result<crate::plan::Plan, String> |  |  |  |  |


### src/abi/ops/mod.rs

| `governance::plan_from_json` | use | governance::plan_from_json |  |  |  |  |


### src/abi_manifest.rs

| `EXPORTS` | const | EXPORTS: &[Export] |  |  |  |  |

| `manifest_json_pretty` | function | manifest_json_pretty() -> String |  |  |  |  |

| `manifest_value` | function | manifest_value() -> Value |  |  |  |  |

| `write_manifest` | function | write_manifest(path: &std::path::Path) -> std::io::Result<()> |  |  |  |  |

| `gl_alloc` | str_key | EXPORTS = "gl_alloc" |  |  |  |  |

| `instruction_id` | str_key | FRAME_FIELDS = "instruction_id" |  |  |  |  |

| `Export` | struct | Export { pub name: &'static str, pub params: &'static [&'static str], pub result: &'static str, pub summary: &'static str } |  |  |  |  |


### src/attest.rs

| `GLAW_RECEIPT_V2` | const | GLAW_RECEIPT_V2: &[u8; 15] |  |  |  |  |

| `AttestError` | enum | AttestError { BadSignature, UnknownKey, Malformed(String) } |  |  |  |  |

| `canon_arrow_empty_v1` | function | canon_arrow_empty_v1() -> [u8; 32] |  |  |  |  |

| `from_bytes` | function | from_bytes(bytes: &[u8; 32]) -> Result<Self, AttestError> |  |  |  |  |

| `from_hex` | function | from_hex(hex: &str) -> Result<Self, AttestError> |  |  |  |  |

| `from_json` | function | from_json(text: &str) -> Result<Self, AttestError> |  |  |  |  |

| `from_seed` | function | from_seed(seed: [u8; 32]) -> Self |  |  |  |  |

| `from_seed_hex` | function | from_seed_hex(hex: &str) -> Result<Self, AttestError> |  |  |  |  |

| `get` | function | get(&self, key_id: &str) -> Option<&VerifyingKey> |  |  |  |  |

| `hex_decode` | function | hex_decode(s: &str) -> Option<Vec<u8>> |  |  |  |  |

| `hex_encode` | function | hex_encode(bytes: &[u8]) -> String |  |  |  |  |

| `insert` | function | insert(&mut self, key: VerifyingKey) |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `key_id` | function | key_id(&self) -> String |  |  |  |  |

| `lease_digest` | function | lease_digest(payload: &[u8]) -> String |  |  |  |  |

| `lease_payload` | function | lease_payload(l: &Lease) -> String |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `receipt_payload` | function | receipt_payload(f: &ReceiptFields<'_>) -> String |  |  |  |  |

| `receipt_v1` | function | receipt_v1( h_query: &[u8; 32], h_pre: &[u8; 32], h_post: &[u8; 32], h_triples: &[u8; 32], epoch: u64, ) -> [u8; 32] |  |  |  |  |

| `receipt_v2` | function | receipt_v2( h_query: &[u8; 32], h_pre: &[u8; 32], h_post: &[u8; 32], h_triples: &[u8; 32], h_columnar: &[u8; 32], epoch: u64, ) -> [u8; 32] |  |  |  |  |

| `sign_bytes` | function | sign_bytes(key: &SigningKey, payload: &[u8]) -> Attestation |  |  |  |  |

| `sign_lease` | function | sign_lease(key: &SigningKey, lease: Lease) -> SignedLease |  |  |  |  |

| `sign_receipt` | function | sign_receipt(key: &SigningKey, receipt: &Receipt) -> Attestation |  |  |  |  |

| `to_hex` | function | to_hex(&self) -> String |  |  |  |  |

| `to_json` | function | to_json(&self) -> String |  |  |  |  |

| `verify_bytes` | function | verify_bytes( payload: &[u8], att: &Attestation, trusted: &TrustedKeys, ) -> Result<(), AttestError> |  |  |  |  |

| `verify_lease` | function | verify_lease(signed: &SignedLease, trusted: &TrustedKeys) -> Result<(), AttestError> |  |  |  |  |

| `verify_receipt` | function | verify_receipt( receipt: &Receipt, att: &Attestation, trusted: &TrustedKeys, ) -> Result<(), AttestError> |  |  |  |  |

| `verifying_key` | function | verifying_key(&self) -> VerifyingKey |  |  |  |  |

| `GLAW_RECEIPT_V2` | str_key | GLAW_RECEIPT_V2 = "GLAW_RECEIPT_V2" |  |  |  |  |

| `Attestation` | struct | Attestation { pub payload_blake3: String, pub key_id: String, pub signature: String } |  |  |  |  |

| `ReceiptFields` | struct | ReceiptFields { pub parent: &'a str, pub child: &'a str, pub step: &'a str, pub authority: &'a str, pub revision: &'a str, pub added: u64, pub lease_id: Option<&'a str>, pub plan_blake3: Option<&'a str>, pub subject_blake3: Option<&'a str> } |  |  |  |  |

| `SigningKey` | struct | SigningKey { DalekSigning } |  |  |  |  |

| `TrustedKeys` | struct | TrustedKeys { BTreeMap<String, VerifyingKey> } |  |  |  |  |

| `VerifyingKey` | struct | VerifyingKey { DalekVerifying } |  |  |  |  |


### src/bin/dl_dualroute.rs

| `WASM4PM_FIXTURES` | env_key | std::env::var("WASM4PM_FIXTURES") |  |  |  |  |

| `https://graphlaw.dev/dl#` | str_key | DL = "https://graphlaw.dev/dl#" |  |  |  |  |


### src/bin/graphlaw_attest.rs

| `graphlaw-attest — sign or verify an arbitrary file with Ed25519 attestations.

Usage:
  graphlaw-attest sign   --key-seed-hex <hex> --in <file> [--out <attestation.json>]
  graphlaw-attest verify --in <file> --attestation <json> --trusted-key <hex> [...]
  graphlaw-attest pubkey --key-seed-hex <hex>

Commands:
  sign    Sign the file; writes (or prints) the canonical attestation JSON
  verify  Re-read the file and attestation; prints `VERIFIED` or `REFUSED: <error>`
  pubkey  Print the Ed25519 public key derived from the 32-byte seed

Flags:
  --key-seed-hex <hex>  Signing seed, 64 hex chars (32 bytes)
  --in <file>           Payload file to sign or verify
  --out <file>          Where `sign` writes the attestation (default: stdout)
  --attestation <file>  Attestation JSON for `verify`
  --trusted-key <hex>   Ed25519 public key, 64 hex chars (32 bytes); repeatable

Exit codes: 0 ok, 1 refused (signature/verification failure), 2 usage or I/O error.` | str_key | HELP = "graphlaw-attest — sign or verify an arbitrary file with Ed25519 attestations.

Usage:
  graphlaw-attest sign   --key-seed-hex <hex> --in <file> [--out <attestation.json>]
  graphlaw-attest verify --in <file> --attestation <json> --trusted-key <hex> [...]
  graphlaw-attest pubkey --key-seed-hex <hex>

Commands:
  sign    Sign the file; writes (or prints) the canonical attestation JSON
  verify  Re-read the file and attestation; prints `VERIFIED` or `REFUSED: <error>`
  pubkey  Print the Ed25519 public key derived from the 32-byte seed

Flags:
  --key-seed-hex <hex>  Signing seed, 64 hex chars (32 bytes)
  --in <file>           Payload file to sign or verify
  --out <file>          Where `sign` writes the attestation (default: stdout)
  --attestation <file>  Attestation JSON for `verify`
  --trusted-key <hex>   Ed25519 public key, 64 hex chars (32 bytes); repeatable

Exit codes: 0 ok, 1 refused (signature/verification failure), 2 usage or I/O error." |  |  |  |  |


### src/bin/graphlaw_registry.rs

| `# receipt-surface: blake3 of the canonical graphlaw.receipt-surface/1 document` | str_key | ARTIFACTS_TRAILING_COMMENTS = "# receipt-surface: blake3 of the canonical graphlaw.receipt-surface/1 document" |  |  |  |  |

| `# wasm row: never re-pinned by graphlaw-registry; re-pinned by the wasm rebuild flow.` | str_key | WASM_ROW_NOTE = "# wasm row: never re-pinned by graphlaw-registry; re-pinned by the wasm rebuild flow." |  |  |  |  |

| `ARTIFACTS.blake3` | str_key | ARTIFACTS_FILE = "ARTIFACTS.blake3" |  |  |  |  |

| `graphlaw-registry — emit or check the capability registry files.

Usage:
  graphlaw-registry --write | --check | --print-json | --print-ttl | --print-digests

Modes (exactly one per invocation):
  --write          Write registry/capability-registry.json and .ttl under the crate
                   root and re-pin the ARTIFACTS.blake3 rows this bin owns
                   (`registry`, `surface`, `receipt-surface`); the `wasm` row is
                   never touched here — it is re-pinned by the wasm rebuild flow
  --check          Compare the committed files against live output; exit 1 with a
                   first-difference summary on drift
  --print-json     Print the capability registry JSON to stdout
  --print-ttl      Print the capability registry Turtle to stdout
  --print-digests  Print the live surface/registry BLAKE3 digests

Exit codes: 0 ok, 1 drift (`--check`), 2 usage or I/O error.` | str_key | HELP = "graphlaw-registry — emit or check the capability registry files.

Usage:
  graphlaw-registry --write | --check | --print-json | --print-ttl | --print-digests

Modes (exactly one per invocation):
  --write          Write registry/capability-registry.json and .ttl under the crate
                   root and re-pin the ARTIFACTS.blake3 rows this bin owns
                   (`registry`, `surface`, `receipt-surface`); the `wasm` row is
                   never touched here — it is re-pinned by the wasm rebuild flow
  --check          Compare the committed files against live output; exit 1 with a
                   first-difference summary on drift
  --print-json     Print the capability registry JSON to stdout
  --print-ttl      Print the capability registry Turtle to stdout
  --print-digests  Print the live surface/registry BLAKE3 digests

Exit codes: 0 ok, 1 drift (`--check`), 2 usage or I/O error." |  |  |  |  |


### src/bin/graphlaw_run_eh.rs

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |


### src/bin/graphlaw_verify.rs

| `graphlaw-verify — offline third-party replay + attestation check of an admitted plan.

Usage:
  graphlaw-verify --start <state.nq|.nt|.ttl|.trig> --plan <plan.json> --receipts <dir>
                  [--subject <key>] --trusted-key <hex> [--trusted-key <hex>...]

Flags:
  --start <path>        Start state (dialect chosen by extension; default N-Quads)
  --plan <path>         Plan JSON to replay via Plan::admit
  --receipts <dir>      Receipt store root; defaults subject discovery to its only
                        subdirectory unless --subject is given
  --subject <key>       Subject key within the receipt store
  --trusted-key <hex>   Ed25519 public key, 64 hex chars (32 bytes); repeatable

Output: `ADMITTED` or `REFUSED: <code>: <detail>` on stdout.
Exit codes: 0 admitted, 1 refused, 2 usage or I/O error.` | str_key | HELP = "graphlaw-verify — offline third-party replay + attestation check of an admitted plan.

Usage:
  graphlaw-verify --start <state.nq|.nt|.ttl|.trig> --plan <plan.json> --receipts <dir>
                  [--subject <key>] --trusted-key <hex> [--trusted-key <hex>...]

Flags:
  --start <path>        Start state (dialect chosen by extension; default N-Quads)
  --plan <path>         Plan JSON to replay via Plan::admit
  --receipts <dir>      Receipt store root; defaults subject discovery to its only
                        subdirectory unless --subject is given
  --subject <key>       Subject key within the receipt store
  --trusted-key <hex>   Ed25519 public key, 64 hex chars (32 bytes); repeatable

Output: `ADMITTED` or `REFUSED: <code>: <detail>` on stdout.
Exit codes: 0 admitted, 1 refused, 2 usage or I/O error." |  |  |  |  |


### src/bin/sa2a_run14_verify.rs

| `https://chatmangpt.com/sa2a#` | str_key | V2 = "https://chatmangpt.com/sa2a#" |  |  |  |  |

| `https://sa2a.dev/ontology#` | str_key | V1 = "https://sa2a.dev/ontology#" |  |  |  |  |


### src/bin/smon_broaden_topic.rs

| `--in-ttl` | str_key | VALUE_FLAGS = "--in-ttl" |  |  |  |  |

| `smon-broaden-topic — disclosed counterfactual post-processor over an already-emitted smon: graph.
Prints a full audit of every triple it changes.

Usage:
  smon-broaden-topic --in-ttl IN.ttl --out-ttl OUT.ttl [--canonical-topic T]

Flags:
  --in-ttl <path>         Input Turtle graph (required)
  --out-ttl <path>        Output Turtle graph (required)
  --canonical-topic <t>   Topic to collapse GroundingQuestion topics to
                          (default: e2e-capability-grounding)

Exit codes: 0 ok, 1 usage, I/O or parse error.` | str_key | HELP = "smon-broaden-topic — disclosed counterfactual post-processor over an already-emitted smon: graph.
Prints a full audit of every triple it changes.

Usage:
  smon-broaden-topic --in-ttl IN.ttl --out-ttl OUT.ttl [--canonical-topic T]

Flags:
  --in-ttl <path>         Input Turtle graph (required)
  --out-ttl <path>        Output Turtle graph (required)
  --canonical-topic <t>   Topic to collapse GroundingQuestion topics to
                          (default: e2e-capability-grounding)

Exit codes: 0 ok, 1 usage, I/O or parse error." |  |  |  |  |


### src/bin/smon_transcript_to_turtle.rs

| `--assistant-only` | str_key | SWITCHES = "--assistant-only" |  |  |  |  |

| `--transcript` | str_key | VALUE_FLAGS = "--transcript" |  |  |  |  |

| `smon-transcript-to-turtle — emit smon: Turtle facts from a Claude Code session transcript (.jsonl).

Usage:
  smon-transcript-to-turtle --transcript T.jsonl --out OUT.ttl
      [--session-id ID] [--base IRI] [--assistant-only] [--topic-filter S]

Flags:
  --transcript <path>  Session transcript in JSONL format (required)
  --out <path>         Turtle output file (required)
  --session-id <id>    Session id (default: transcript file stem)
  --base <iri>         Base IRI for emitted subjects
                       (default: .../sessions/<session-id>)
  --assistant-only     Switch: keep only assistant turns
  --topic-filter <s>   Also print turns whose grounding topic contains <s>

Exit codes: 0 ok, 1 usage, I/O or parse error.` | str_key | HELP = "smon-transcript-to-turtle — emit smon: Turtle facts from a Claude Code session transcript (.jsonl).

Usage:
  smon-transcript-to-turtle --transcript T.jsonl --out OUT.ttl
      [--session-id ID] [--base IRI] [--assistant-only] [--topic-filter S]

Flags:
  --transcript <path>  Session transcript in JSONL format (required)
  --out <path>         Turtle output file (required)
  --session-id <id>    Session id (default: transcript file stem)
  --base <iri>         Base IRI for emitted subjects
                       (default: .../sessions/<session-id>)
  --assistant-only     Switch: keep only assistant turns
  --topic-filter <s>   Also print turns whose grounding topic contains <s>

Exit codes: 0 ok, 1 usage, I/O or parse error." |  |  |  |  |


### src/dialect.rs

| `Dialect` | enum | Dialect { Turtle, TriG, NTriples, NQuads, RdfXml, JsonLd, YamlLd, TriX, HexTuples, N3, ShExC, ShExJ, Sparql, SqlRdf } |  |  |  |  |

| `Engine` | enum | Engine { PurRdf, FrozenDuckdb } |  |  |  |  |

| `admit_bytes` | function | admit_bytes( bytes: &[u8], hint: Option<&str>, base: Option<&str>, ) -> Result<LawState, Refusal> |  |  |  |  |

| `check` | function | check(bytes: &[u8], dialect: Dialect, base: Option<&str>) -> Result<(), Refusal> |  |  |  |  |

| `check_typed` | function | check_typed( bytes: &[u8], dialect: Dialect, base: Option<&str>, ) -> Result<(), Box<ParseRefusal>> |  |  |  |  |

| `engine` | function | engine(self) -> Engine |  |  |  |  |

| `limit` | function | limit( dialect: Dialect, name: impl Into<String>, observed: u64, max: u64, message: impl Into<String>, ) -> Self |  |  |  |  |

| `media_type` | function | media_type(self) -> Option<&'static str> |  |  |  |  |

| `new` | function | new(dialect: Dialect, reason: ReasonClass, message: impl Into<String>) -> Self |  |  |  |  |

| `parse_rdf` | function | parse_rdf( bytes: &[u8], dialect: Dialect, base: Option<&str>, ) -> Result<Arc<purrdf::RdfDataset>, Refusal> |  |  |  |  |

| `parse_rdf_typed` | function | parse_rdf_typed( bytes: &[u8], dialect: Dialect, base: Option<&str>, ) -> Result<Arc<purrdf::RdfDataset>, Box<ParseRefusal>> |  |  |  |  |

| `ParseRefusal` | struct | ParseRefusal { pub dialect: Dialect, pub engine: Engine, pub reason: ReasonClass, pub line: Option<u32>, pub col: Option<u32>, pub byte_offset: Option<usize>, pub snippet: Option<String>, pub message: String } |  |  |  |  |

| `refusal::{ReasonClass, Refusal, RefusalKind, RefusalMessage}` | use | refusal::{ReasonClass, Refusal, RefusalKind, RefusalMessage} |  |  |  |  |

| `sniff::sniff` | use | sniff::sniff |  |  |  |  |


### src/dialect/refusal.rs

| `ReasonClass` | enum | ReasonClass { Syntax, UndefinedPrefix, BadIri, FormulaInRdfLayer, NotSemanticContent, AmbiguousDialect, RemoteContext, Limit { name: String, observed: u64, max: u64, } } |  |  |  |  |

| `RefusalKind` | enum | RefusalKind { NotSemanticContent, Ambiguous, EngineRejected, Unsupported, ResourceLimit, TierMismatch, DuckDb, BareSql, Fuse } |  |  |  |  |

| `RefusalMessage` | enum | RefusalMessage { Static(&'static str), Dynamic(String) } |  |  |  |  |

| `as_str` | function | as_str(&self) -> &str |  |  |  |  |

| `Refusal` | struct | Refusal { pub kind: RefusalKind, pub dialect: Option<Dialect>, pub engine: Option<Engine>, pub message: RefusalMessage } |  |  |  |  |


### src/dialect/sniff.rs

| `sniff` | function | sniff(bytes: &[u8], hint: Option<&str>) -> Result<Dialect, Refusal> |  |  |  |  |

| `http://graphlaw.org/sql#` | str_key | SQL_IRI = "http://graphlaw.org/sql#" |  |  |  |  |

| `sql:` | str_key | SQL = "sql:" |  |  |  |  |


### src/duckdb_eval.rs

| `DEFAULT_ROW_ENVELOPE` | const | DEFAULT_ROW_ENVELOPE: usize |  |  |  |  |

| `PARAM_BYTES_INLINE` | const | PARAM_BYTES_INLINE: usize |  |  |  |  |

| `ROW_ENVELOPE_LIMIT` | const | ROW_ENVELOPE_LIMIT: &str |  |  |  |  |

| `EvaluatorError` | enum | EvaluatorError { DuckDb(frozen_duckdb::DuckDBError), ConversionError { column: String, row: usize, details: String, }, ReceiptError(String), NoCanonicalGraph(String), BatchMemberFailed { index: usize, source: Box<EvaluatorError>, partial: BatchReceipt, }, RowEnvelopeExceeded { limit: &'static str, observed: usize, max: usize, } } |  |  |  |  |

| `ParamBytes` | enum | ParamBytes { Inline { buf: [u8; PARAM_BYTES_INLINE], len: usize, }, Digest([u8; 32]) } |  |  |  |  |

| `admitted_statement_hash` | function | admitted_statement_hash(canonical_graph: &[u8], params: &[SqlLiteral]) -> [u8; 32] |  |  |  |  |

| `canonical_ast_graph_bytes` | function | canonical_ast_graph_bytes(turtle: &str) -> Result<Vec<u8>, EvaluatorError> |  |  |  |  |

| `coarse_elapsed_ns` | function | coarse_elapsed_ns(start_ns: u64) -> u64 |  |  |  |  |

| `coarse_now_ns` | function | coarse_now_ns() -> u64 |  |  |  |  |

| `encode` | function | encode(params: &[SqlLiteral]) -> ParamBytes |  |  |  |  |

| `eval_admitted_ast` | function | eval_admitted_ast( &self, ast_turtle: &str, query_iri: &str, compiled: &CompiledQuery, subject_iri_prefix: &str, predicate_iri_prefix: &str, ) -> Result<ExecutionReceipt, EvaluatorError> |  |  |  |  |

| `eval_ast_document` | function | eval_ast_document( &self, query_iri: &str, document: &crate::sql_ast::CompiledDocument, subject_iri_prefix: &str, predicate_iri_prefix: &str, atomic: bool, ) -> Result<BatchReceipt, EvaluatorError> |  |  |  |  |

| `eval_ast_document_bounded` | function | eval_ast_document_bounded( &self, query_iri: &str, document: &crate::sql_ast::CompiledDocument, subject_iri_prefix: &str, predicate_iri_prefix: &str, atomic: bool, max_rows: usize, ) -> Result<BatchReceipt, EvaluatorError> |  |  |  |  |

| `eval_ast_query` | function | eval_ast_query( &self, query_iri: &str, compiled: &CompiledQuery, subject_iri_prefix: &str, predicate_iri_prefix: &str, ) -> Result<ExecutionReceipt, EvaluatorError> |  |  |  |  |

| `eval_ast_query_bounded` | function | eval_ast_query_bounded( &self, query_iri: &str, compiled: &CompiledQuery, subject_iri_prefix: &str, predicate_iri_prefix: &str, max_rows: usize, ) -> Result<ExecutionReceipt, EvaluatorError> |  |  |  |  |

| `explain_ast_query` | function | explain_ast_query(&self, compiled: &CompiledQuery) -> Result<String, EvaluatorError> |  |  |  |  |

| `fold` | function | fold(&self, hasher: &mut blake3::Hasher) |  |  |  |  |

| `fold_params_anchored` | function | fold_params_anchored(h_ast: &[u8; 32], params: &[SqlLiteral]) -> [u8; 32] |  |  |  |  |

| `from_connection` | function | from_connection(conn: frozen_duckdb::Connection) -> Self |  |  |  |  |

| `from_connection_with_cache` | function | from_connection_with_cache( conn: frozen_duckdb::Connection, cache: std::sync::Arc<crate::query_cache::QueryCache>, ) -> Self |  |  |  |  |

| `h_columnar_canon` | function | h_columnar_canon(triples: &[RdfTriple]) -> [u8; 32] |  |  |  |  |

| `h_triples_serterm` | function | h_triples_serterm(triples: &[RdfTriple]) -> [u8; 32] |  |  |  |  |

| `hash_param_buffer` | function | hash_param_buffer(hasher: &mut blake3::Hasher, params: &[SqlLiteral]) |  |  |  |  |

| `is_inline` | function | is_inline(&self) -> bool |  |  |  |  |

| `open_in_memory` | function | open_in_memory() -> Result<Self, EvaluatorError> |  |  |  |  |

| `open_in_memory_allowing_unsigned_extensions` | function | open_in_memory_allowing_unsigned_extensions() -> Result<Self, EvaluatorError> |  |  |  |  |

| `open_in_memory_with_cache` | function | open_in_memory_with_cache( cache: std::sync::Arc<crate::query_cache::QueryCache>, ) -> Result<Self, EvaluatorError> |  |  |  |  |

| `prepared_statement_cache_hits` | function | prepared_statement_cache_hits(&self) -> u64 |  |  |  |  |

| `prepared_statements` | function | prepared_statements(&self) -> usize |  |  |  |  |

| `raw_connection` | function | raw_connection(&self) -> &frozen_duckdb::Connection |  |  |  |  |

| `receipt_v2_state_sentinel` | function | receipt_v2_state_sentinel() -> &'static [u8; 32] |  |  |  |  |

| `register_csv_view` | function | register_csv_view( &self, view_name: &str, glob_pattern: &str, ) -> Result<(), EvaluatorError> |  |  |  |  |

| `register_parquet_view` | function | register_parquet_view( &self, view_name: &str, glob_pattern: &str, hive_partitioning: bool, ) -> Result<(), EvaluatorError> |  |  |  |  |

| `register_purrdf_parquet_views` | function | register_purrdf_parquet_views( &self, directory: &std::path::Path, ) -> Result<(), EvaluatorError> |  |  |  |  |

| `shared_cache` | function | shared_cache(&self) -> std::sync::Arc<crate::query_cache::QueryCache> |  |  |  |  |

| `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/` | str_key | TABLE = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/" |  |  |  |  |

| `duckdb_result_rows` | str_key | ROW_ENVELOPE_LIMIT = "duckdb_result_rows" |  |  |  |  |

| `seq_atomic_abort` | str_key | TABLE = "seq_atomic_abort" |  |  |  |  |

| `seq_atomic_ok` | str_key | TABLE = "seq_atomic_ok" |  |  |  |  |

| `seq_court_events` | str_key | TABLE = "seq_court_events" |  |  |  |  |

| `seq_court_partial` | str_key | TABLE = "seq_court_partial" |  |  |  |  |

| `seq_inverted` | str_key | TABLE = "seq_inverted" |  |  |  |  |

| `BatchReceipt` | struct | BatchReceipt { pub query_iri: String, pub doc_hash: [u8; 32], pub members: Vec<ExecutionReceipt>, pub rows_emitted: Vec<usize>, pub failed_at: Option<usize>, pub status: String, pub state_delta: u64 } |  |  |  |  |

| `ExecutionReceipt` | struct | ExecutionReceipt { pub query_iri: String, pub query_hash: [u8; 32], pub sql_statement: String, pub rows_emitted: usize, pub execution_ms: f64, pub result_triples: Vec<RdfTriple>, pub cached: bool, pub consult_ms: f64, pub artifact_digest_hex: Option<String>, pub artifact_bytes: Option<u64>, pub artifact_warning: Option<String>, pub columnar_digest_hex: Option<String>, pub receipt_v2_hex: Option<String> } |  |  |  |  |

| `GraphLawDuckDB` | struct | GraphLawDuckDB { conn: frozen_duckdb::Connection, cache: std::sync::Arc<crate::query_cache::QueryCache>, stmt_cache: StmtCacheSeam } |  |  |  |  |

| `RdfTriple` | struct | RdfTriple { pub subject: String, pub predicate: String, pub object: TermValue } |  |  |  |  |


### src/duckdb_op.rs

| `DUCKDB_MEMORY_LIMIT` | const | DUCKDB_MEMORY_LIMIT: &str |  |  |  |  |

| `DUCKDB_THREADS` | const | DUCKDB_THREADS: u32 |  |  |  |  |

| `MAX_CHUNK_BYTES` | const | MAX_CHUNK_BYTES: usize |  |  |  |  |

| `MAX_CHUNK_ROWS` | const | MAX_CHUNK_ROWS: usize |  |  |  |  |

| `MAX_SESSIONS` | const | MAX_SESSIONS: usize |  |  |  |  |

| `GRAPHLAW_ATTACH_ROOTS` | env_key | std::env::var("GRAPHLAW_ATTACH_ROOTS") |  |  |  |  |

| `dispatch` | function | dispatch(v: &Value) -> Result<Value, Fail> |  |  |  |  |

| `leading_sql_token` | function | leading_sql_token(sql: &str) -> Option<String> |  |  |  |  |

| `32MiB` | str_key | DUCKDB_MEMORY_LIMIT = "32MiB" |  |  |  |  |


### src/egress/arrow.rs

| `FLAGS_MASK` | const | FLAGS_MASK: u16 |  |  |  |  |

| `FLAG_BIG_ENDIAN` | const | FLAG_BIG_ENDIAN: u16 |  |  |  |  |

| `FLAG_HAS_NULLS` | const | FLAG_HAS_NULLS: u16 |  |  |  |  |

| `FLAG_STRICT_PROJECTION_ORDER` | const | FLAG_STRICT_PROJECTION_ORDER: u16 |  |  |  |  |

| `MAGIC` | const | MAGIC: [u8; 8] |  |  |  |  |

| `PAYLOAD_OFFSET` | const | PAYLOAD_OFFSET: u32 |  |  |  |  |

| `SCHEMA_VERSION` | const | SCHEMA_VERSION: u16 |  |  |  |  |

| `SIZE` | const | SIZE: usize |  |  |  |  |

| `ArrowEgressHeaderError` | enum | ArrowEgressHeaderError { BadMagic, BadSchemaVersion(u16), BadFlags(u16), BadPayloadOffset(u32), BadTotalLen { payload_byte_len: u64, total_byte_len: u64, }, ReservedNotZero } |  |  |  |  |

| `CANON_ARROW_EMPTY_V1` | function | CANON_ARROW_EMPTY_V1() -> &'static [u8; 32] |  |  |  |  |

| `canon_arrow_empty_v1` | function | canon_arrow_empty_v1() -> &'static [u8; 32] |  |  |  |  |

| `from_bytes` | function | from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self, ArrowEgressHeaderError> |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) -> [u8; Self::SIZE] |  |  |  |  |

| `GLAWARRW` | str_key | MAGIC = "GLAWARRW" |  |  |  |  |

| `ArrowEgressHeader` | struct | ArrowEgressHeader { pub magic: [u8; 8], pub schema_version: u16, pub flags: u16, pub batch_count: u32, pub payload_offset: u32, pub payload_byte_len: u64, pub total_byte_len: u64, pub triples_digest: [u8; 32], pub columnar_digest: [u8; 32], pub reserved: [u8; 24] } |  |  |  |  |


### src/egress/collector.rs

| `CanonArrowCell` | enum | CanonArrowCell { Null, Bool(bool), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64), F32(f32), F64(f64), Dec128(i128), Str(String), Bytes(Vec<u8>), Date32(i32), Time64(CanonArrowTimeUnit, i64), Timestamp(CanonArrowTimeUnit, i64), Interval { months: i32, days: i32, nanos: i64, } } |  |  |  |  |

| `CanonArrowCollectorError` | enum | CanonArrowCollectorError { ColumnIndexOutOfRange { column: usize, columns: usize, }, CellTypeMismatch { column: usize, declared: CanonArrowType, cell: &'static str, }, RaggedRows { lengths: Vec<usize>, }, Overflow { what: &'static str, } } |  |  |  |  |

| `CanonArrowTimeUnit` | enum | CanonArrowTimeUnit { Second, Millisecond, Microsecond, Nanosecond } |  |  |  |  |

| `CanonArrowType` | enum | CanonArrowType { Null, Bool, Int8, Int16, Int32, Int64, UInt8, UInt16, UInt32, UInt64, Float32, Float64, Decimal128 { precision: u8, scale: i8, }, Utf8, Binary, Date32, Time64(CanonArrowTimeUnit), Timestamp(CanonArrowTimeUnit, Option<String>), IntervalMonthDayNano } |  |  |  |  |

| `columns` | function | columns(&self) -> usize |  |  |  |  |

| `new` | function | new(schema: Vec<CanonArrowField>) -> Self |  |  |  |  |

| `push` | function | push(&mut self, column: usize, cell: CanonArrowCell) -> Result<(), CanonArrowCollectorError> |  |  |  |  |

| `schema` | function | schema(&self) -> &[CanonArrowField] |  |  |  |  |

| `CanonArrowCollector` | struct | CanonArrowCollector { schema: Vec<CanonArrowField>, columns: Vec<ColumnBuilder>, payload: Vec<u8>, hasher: blake3::Hasher, batches: u32, any_nulls: bool } |  |  |  |  |

| `CanonArrowField` | struct | CanonArrowField { pub name: String, pub ty: CanonArrowType } |  |  |  |  |


### src/evidence_ring.rs

| `RING_CAPACITY` | const | RING_CAPACITY: usize |  |  |  |  |

| `PushOutcome` | enum | PushOutcome { Buffered, DroppedNewest, RefusedTornPad } |  |  |  |  |

| `RingError` | enum | RingError { Io(String), BrokenChain { detail: String, }, Malformed { line: usize, }, TornSlot { line: usize, }, SequenceGap { detail: String, }, AlreadyInitialized } |  |  |  |  |

| `close` | function | close(&self) |  |  |  |  |

| `dropped_total` | function | dropped_total(&self) -> u64 |  |  |  |  |

| `global` | function | global() -> Option<&'static Arc<Self>> |  |  |  |  |

| `init` | function | init(dir: impl AsRef<Path>) -> Result<Arc<Self>, RingError> |  |  |  |  |

| `init_global` | function | init_global(dir: impl AsRef<Path>) -> Result<(), RingError> |  |  |  |  |

| `init_global_with_ledger` | function | init_global_with_ledger( dir: impl AsRef<Path>, ledger_dir: impl AsRef<Path>, ) -> Result<(), RingError> |  |  |  |  |

| `init_with_ledger` | function | init_with_ledger( dir: impl AsRef<Path>, ledger_dir: impl AsRef<Path>, ) -> Result<Arc<Self>, RingError> |  |  |  |  |

| `init_with_ledger_restart` | function | init_with_ledger_restart( dir: impl AsRef<Path>, ledger_dir: impl AsRef<Path>, producer_attempted: u64, ) -> Result<Arc<Self>, RingError> |  |  |  |  |

| `last_ledger_error` | function | last_ledger_error(&self) -> Option<String> |  |  |  |  |

| `last_write_error` | function | last_write_error(&self) -> Option<RingError> |  |  |  |  |

| `ledger_dir` | function | ledger_dir(&self) -> Option<&Path> |  |  |  |  |

| `ledger_path` | function | ledger_path(&self) -> PathBuf |  |  |  |  |

| `new` | function | new( h_query: [u8; 32], rows_emitted: u64, execution_ms: u32, cached: u8, timestamp_ms: u64, ) -> Self |  |  |  |  |

| `now_ms` | function | now_ms() -> u64 |  |  |  |  |

| `pads_zero` | function | pads_zero(&self) -> bool |  |  |  |  |

| `push` | function | push(&self, desc: RingDescriptor) -> PushOutcome |  |  |  |  |

| `pushed_total` | function | pushed_total(&self) -> u64 |  |  |  |  |

| `set_drainer_paused` | function | set_drainer_paused(&self, paused: bool) |  |  |  |  |

| `torn_via` | function | torn_via(&self, f: impl FnOnce(&mut [u8; 3], &mut [u8; 8])) -> Self |  |  |  |  |

| `verify` | function | verify(dir: impl AsRef<Path>) -> Result<(usize, usize, u64), RingError> |  |  |  |  |

| `with_torn_pad` | function | with_torn_pad(&self, f: impl FnOnce(&[u8; 3], &[u8; 8]) -> R) -> R |  |  |  |  |

| `0000000000000000000000000000000000000000000000000000000000000000` | str_key | GENESIS = "0000000000000000000000000000000000000000000000000000000000000000" |  |  |  |  |

| `DroppedEvidence` | struct | DroppedEvidence { pub dropped_count: u64, pub refused_h_query: [u8; 32], pub refused_at_ms: u64 } |  |  |  |  |

| `EvidenceRing` | struct | EvidenceRing { dir: PathBuf, ledger_dir: Option<PathBuf>, producer: Mutex<Option<Producer<EvidenceSlot>>>, consumer: Mutex<Option<Consumer<EvidenceSlot>>>, seq_counter: Arc<AtomicU64>, dropped: AtomicU64, paused_flag: Arc<AtomicBool>, shutdown_flag: Arc<AtomicBool>, ledger_error: Arc<Mutex<Option<String>>>, write_error: Arc<Mutex<Option<String>>>, drainer: Mutex<Option<std::thread::JoinHandle<()>>> } |  |  |  |  |

| `RingDescriptor` | struct | RingDescriptor { pub h_query: [u8; 32], pub rows_emitted: u64, pub execution_ms: u32, pub cached: u8, pad1: [u8; 3], pub timestamp_ms: u64, pad2: [u8; 8] } |  |  |  |  |


### src/hooks.rs

| `HANDLER_SPARQL_CONSTRUCT` | const | HANDLER_SPARQL_CONSTRUCT: &str |  |  |  |  |

| `KH` | const | KH: &str |  |  |  |  |

| `MAX_FIRINGS` | const | MAX_FIRINGS: usize |  |  |  |  |

| `MAX_ROUNDS` | const | MAX_ROUNDS: usize |  |  |  |  |

| `MAX_STATE_QUADS` | const | MAX_STATE_QUADS: usize |  |  |  |  |

| `Effect` | enum | Effect { EmitDelta, Refuse } |  |  |  |  |

| `Verdict` | enum | Verdict { Fired, Refuse(String) } |  |  |  |  |

| `hooks` | function | hooks(&self) -> &[Hook] |  |  |  |  |

| `load` | function | load(pack: &LawState) -> Result<Self, Refusal> |  |  |  |  |

| `materialize` | function | materialize(&self, state: &LawState) -> Result<Materialized, Refusal> |  |  |  |  |

| `http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct` | str_key | HANDLER_SPARQL_CONSTRUCT = "http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct" |  |  |  |  |

| `http://seanchatmangpt.github.io/graphlaw/kh#` | str_key | KH = "http://seanchatmangpt.github.io/graphlaw/kh#" |  |  |  |  |

| `Firing` | struct | Firing { pub hook: String, pub round: usize, pub row: Vec<(String, String)>, pub added: usize } |  |  |  |  |

| `Hook` | struct | Hook { pub iri: String, pub name: String, pub trigger: String, pub construct: String, pub priority: i64, pub effect: Effect, pub reason: Option<String> } |  |  |  |  |

| `HookPack` | struct | HookPack { hooks: Vec<Hook> } |  |  |  |  |

| `HookVerdict` | struct | HookVerdict { pub hook: String, pub round: usize, pub row: Vec<(String, String)>, pub verdict: Verdict } |  |  |  |  |

| `Materialized` | struct | Materialized { pub state: LawState, pub firings: Vec<Firing>, pub verdicts: Vec<HookVerdict>, pub rounds: usize } |  |  |  |  |


### src/law.rs

| `DEFAULT_MAX_SKEW_SECS` | const | DEFAULT_MAX_SKEW_SECS: u64 |  |  |  |  |

| `N3_MAX_BACKWARD_DEPTH` | const | N3_MAX_BACKWARD_DEPTH: usize |  |  |  |  |

| `N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL` | const | N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL: usize |  |  |  |  |

| `N3_MAX_DERIVED_FACTS` | const | N3_MAX_DERIVED_FACTS: usize |  |  |  |  |

| `N3_MAX_ITERATIONS` | const | N3_MAX_ITERATIONS: usize |  |  |  |  |

| `N3_MAX_MATCH_STEPS` | const | N3_MAX_MATCH_STEPS: usize |  |  |  |  |

| `N3_MAX_TERM_BYTES` | const | N3_MAX_TERM_BYTES: usize |  |  |  |  |

| `N3_MAX_TERM_DEPTH` | const | N3_MAX_TERM_DEPTH: usize |  |  |  |  |

| `N3_MAX_TOTAL_BYTES` | const | N3_MAX_TOTAL_BYTES: usize |  |  |  |  |

| `N3_MAX_TOTAL_STEPS` | const | N3_MAX_TOTAL_STEPS: usize |  |  |  |  |

| `REFUSAL_UNKNOWN_STEP_CAPABILITY` | const | REFUSAL_UNKNOWN_STEP_CAPABILITY: &str |  |  |  |  |

| `Ceiling` | enum | Ceiling { Observe, Select, Construct } |  |  |  |  |

| `LawError` | enum | LawError { NotAdmitted { violations: usize, results: Vec<Violation>, }, PlanRefused { index: usize, action: String, missing: Vec<String>, violated_absent: Vec<String>, }, ReceiptRequired { step: String, }, ReceiptRefused { step: String, reason: ReceiptReason, }, LeaseRefused { lease_id: String, step: String, reason: LeaseReason, }, Refused(Refusal) } |  |  |  |  |

| `LeaseReason` | enum | LeaseReason { Expired, OutOfScope, Ceiling, BadSignature, UntrustedKey, ClockSkew } |  |  |  |  |

| `N3Error` | enum | N3Error { Limit { limit: &'static str, observed: usize, max: usize, summary: String, }, Refused(Refusal), Fuse { report: String, } } |  |  |  |  |

| `ReceiptReason` | enum | ReceiptReason { Unattested, BadSignature, UntrustedKey } |  |  |  |  |

| `Step` | enum | Step { AdmitShacl { shapes_ttl: &'a str, }, DeriveN3 { rules: &'a str, ruleset: &'a crate::tier::RuleSet, }, Hooks { pack: &'a crate::hooks::HookPack, }, Plan { plan: &'a crate::plan::Plan, }, RequireReceipt { step: &'a str, }, RequireSignedReceipt { step: &'a str, trusted: &'a crate::attest::TrustedKeys, }, EntailRdfs, EntailOwlRl, AdmitSql, TestCapabilityProbe(&'static str) } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `authority_for_capability` | function | authority_for_capability(capability: &str) -> Result<BackendAuthority, LawError> |  |  |  |  |

| `authorize` | function | authorize( &self, step: &str, required: Ceiling, trusted: &crate::attest::TrustedKeys, clock: &dyn Clock, max_skew_secs: u64, ) -> Result<(), LawError> |  |  |  |  |

| `dataset` | function | dataset(&self) -> &Arc<purrdf::RdfDataset> |  |  |  |  |

| `from_dataset` | function | from_dataset(dataset: Arc<purrdf::RdfDataset>) -> Result<Self, Refusal> |  |  |  |  |

| `id` | function | id(&self) -> &str |  |  |  |  |

| `name` | function | name(&self) -> &'static str |  |  |  |  |

| `parse` | function | parse(s: &str) -> Option<Self> |  |  |  |  |

| `quad_count` | function | quad_count(&self) -> usize |  |  |  |  |

| `reason_n3_bounded` | function | reason_n3_bounded(input: &str) -> Result<String, N3Error> |  |  |  |  |

| `required_ceiling` | function | required_ceiling(&self) -> Ceiling |  |  |  |  |

| `transition` | function | transition(&self, step: &Step<'_>) -> Result<(LawState, Receipt), LawError> |  |  |  |  |

| `transition_authorized` | function | transition_authorized( &self, signed: &SignedLease, trusted: &crate::attest::TrustedKeys, clock: &dyn Clock, max_skew_secs: u64, step: &Step<'_>, ) -> Result<(LawState, Receipt), LawError> |  |  |  |  |

| `transition_leased_unverified` | function | transition_leased_unverified( &self, lease: &Lease, step: &Step<'_>, now_unix: u64, ) -> Result<(LawState, Receipt), LawError> |  |  |  |  |

| `with_subject` | function | with_subject(mut self, subject_blake3: impl Into<String>) -> Self |  |  |  |  |

| `UNKNOWN-STEP-CAPABILITY` | str_key | REFUSAL_UNKNOWN_STEP_CAPABILITY = "UNKNOWN-STEP-CAPABILITY" |  |  |  |  |

| `FixedClock` | struct | FixedClock { pub u64 } |  |  |  |  |

| `LawState` | struct | LawState { dataset: Arc<purrdf::RdfDataset>, id: String } |  |  |  |  |

| `Lease` | struct | Lease { pub id: String, pub holder: String, pub ceiling: Ceiling, pub scope: Vec<String>, pub expires_unix: u64, pub issued_unix: u64 } |  |  |  |  |

| `Receipt` | struct | Receipt { pub parent: String, pub child: String, pub step: &'static str, pub authority: BackendAuthority, pub added: usize, pub lease_id: Option<String>, pub plan_blake3: Option<String>, pub subject_blake3: Option<String> } |  |  |  |  |

| `SignedLease` | struct | SignedLease { pub lease: Lease, pub attestation: crate::attest::Attestation } |  |  |  |  |

| `SystemClock` | struct |  |  |  |  |  |

| `Violation` | struct | Violation { pub focus: String, pub path: Option<String>, pub component: String, pub message: String, pub severity: String } |  |  |  |  |

| `Clock` | trait |  |  |  |  |  |

| `ReasonerLimitCensus` | trait |  |  |  |  |  |


### src/lib.rs

| `BACKEND_AUTHORITIES` | const | BACKEND_AUTHORITIES: &[BackendAuthority] |  |  |  |  |

| `LEGACY_FALLBACK_ENABLED` | const | LEGACY_FALLBACK_ENABLED: bool |  |  |  |  |

| `RDF 1.2 / codecs / storage IR` | str_key | BACKEND_AUTHORITIES = "RDF 1.2 / codecs / storage IR" |  |  |  |  |

| `BackendAuthority` | struct | BackendAuthority { pub capability: &'static str, pub authority: &'static str, pub revision: &'static str } |  |  |  |  |

| `law::LawState` | use | law::LawState |  |  |  |  |

| `purrdf` | use | purrdf |  |  |  |  |

| `purrdf::datalog::*` | use | purrdf::datalog::* |  |  |  |  |

| `purrdf::entail::*` | use | purrdf::entail::* |  |  |  |  |

| `purrdf::events::*` | use | purrdf::events::* |  |  |  |  |

| `purrdf::geo::*` | use | purrdf::geo::* |  |  |  |  |

| `purrdf::gts::*` | use | purrdf::gts::* |  |  |  |  |

| `purrdf::n3::*` | use | purrdf::n3::* |  |  |  |  |

| `purrdf::retrieval::*` | use | purrdf::retrieval::* |  |  |  |  |

| `purrdf::shapes::*` | use | purrdf::shapes::* |  |  |  |  |

| `purrdf::shex::*` | use | purrdf::shex::* |  |  |  |  |

| `purrdf::sparql::*` | use | purrdf::sparql::* |  |  |  |  |

| `purrdf::text::*` | use | purrdf::text::* |  |  |  |  |

| `purrdf::validate::*` | use | purrdf::validate::* |  |  |  |  |

| `purrdf::{ CanonHash, RdfDataset, RdfDatasetBuilder, RdfDiagnostic, RdfLiteral, SerializeGraph, SparqlEngine, SparqlRequest, SparqlResult, TermId, TermValue, parse_dataset, parse_dataset_with, serialize_dataset, try_canonicalize, }` | use | purrdf::{ CanonHash, RdfDataset, RdfDatasetBuilder, RdfDiagnostic, RdfLiteral, SerializeGraph, SparqlEngine, SparqlRequest, SparqlResult, TermId, TermValue, parse_dataset, parse_dataset_with, serialize_dataset, try_canonicalize, } |  |  |  |  |

| `purrdf_columnar` | use | purrdf_columnar |  |  |  |  |

| `receipt_chain::{ReceiptRecord, validator::ReceiptValidator}` | use | receipt_chain::{ReceiptRecord, validator::ReceiptValidator} |  |  |  |  |


### src/plan.rs

| `MAX_PLAN_TOTAL_ATOMS` | const | MAX_PLAN_TOTAL_ATOMS: usize |  |  |  |  |

| `TripleError` | enum | TripleError { InvalidIri { role: &'static str, iri: String, }, Builder(String) } |  |  |  |  |

| `action` | function | action(mut self, name: impl Into<String>) -> Self |  |  |  |  |

| `adds` | function | adds(mut self, t: Triple) -> Self |  |  |  |  |

| `admit` | function | admit(&self, start: &LawState) -> Result<Admitted, LawError> |  |  |  |  |

| `as_str` | function | as_str(&self) -> &str |  |  |  |  |

| `build` | function | build(self) -> Action |  |  |  |  |

| `builder` | function | builder(name: impl Into<String>) -> ActionBuilder |  |  |  |  |

| `canonical_json` | function | canonical_json(&self) -> String |  |  |  |  |

| `deletes` | function | deletes(mut self, t: Triple) -> Self |  |  |  |  |

| `digest` | function | digest(&self) -> String |  |  |  |  |

| `goal` | function | goal(mut self, t: Triple) -> Self |  |  |  |  |

| `goal_not` | function | goal_not(mut self, t: Triple) -> Self |  |  |  |  |

| `iri` | function | iri(s: &str, p: &str, o: &str) -> Result<Self, TripleError> |  |  |  |  |

| `literal` | function | literal(s: &str, p: &str, text: &str) -> Result<Self, TripleError> |  |  |  |  |

| `new` | function | new(name: impl Into<String>) -> Self |  |  |  |  |

| `requires` | function | requires(mut self, t: Triple) -> Self |  |  |  |  |

| `requires_not` | function | requires_not(mut self, t: Triple) -> Self |  |  |  |  |

| `RDF 1.2 / codecs / storage IR` | str_key | CAPABILITY = "RDF 1.2 / codecs / storage IR" |  |  |  |  |

| `Action` | struct | Action { pub name: String, pub pre: String, pub pre_not: String, pub add: String, pub del: String } |  |  |  |  |

| `ActionBuilder` | struct | ActionBuilder { name: String, pre: Vec<Triple>, pre_not: Vec<Triple>, add: Vec<Triple>, del: Vec<Triple> } |  |  |  |  |

| `Admitted` | struct | Admitted { pub state: LawState, pub receipts: Vec<Receipt>, pub plan_digest: String } |  |  |  |  |

| `Plan` | struct | Plan { pub actions: Vec<Action>, pub goal: String, pub goal_not: String } |  |  |  |  |

| `PlanBuilder` | struct | PlanBuilder { done: Vec<Action>, current: Option<ActionBuilder>, goal: Vec<Triple>, goal_not: Vec<Triple>, error: Option<TripleError> } |  |  |  |  |

| `Triple` | struct | Triple { String } |  |  |  |  |


### src/policy.rs

| `PolicyRefusalKind` | enum | PolicyRefusalKind { MissingEntry, InventedOutcome, BadMass, DeadEnd, Malformed } |  |  |  |  |

| `admit` | function | admit(problem_json: &str, policy_json: &str) -> Result<PolicyAdmitted, PolicyRefused> |  |  |  |  |

| `admit_parsed` | function | admit_parsed(problem: &Problem, policy: &[Entry]) -> Result<PolicyAdmitted, PolicyRefused> |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `to_ntriples` | function | to_ntriples(&self) -> String |  |  |  |  |

| `https://graphlaw.dev/policy#` | str_key | NS = "https://graphlaw.dev/policy#" |  |  |  |  |

| `Entry` | struct | Entry { pub state: String, pub action: String, pub outcomes: Vec<Outcome> } |  |  |  |  |

| `Outcome` | struct | Outcome { pub state: String, pub probability_ppm: u64 } |  |  |  |  |

| `PolicyAdmitted` | struct | PolicyAdmitted { pub initial_states: Vec<String>, pub reachable: Vec<String>, pub goal_states: Vec<String>, pub entries: Vec<(String, String)> } |  |  |  |  |

| `PolicyRefused` | struct | PolicyRefused { pub kind: PolicyRefusalKind, pub state: String, pub action: String, pub message: String } |  |  |  |  |

| `Problem` | struct | Problem { pub states: BTreeMap<String, BTreeSet<String>>, pub initial: Vec<String>, pub goal_facts: BTreeSet<String>, pub unsafe_states: BTreeSet<String>, pub transitions: BTreeSet<(String, String, String)> } |  |  |  |  |

| `json::admit` | use | json::admit |  |  |  |  |


### src/proof_lease.rs

| `ProofLeaseError` | enum | ProofLeaseError { LeaseDigestMismatch { lease_digest: String, source_digest: String, }, Parse(String) } |  |  |  |  |

| `lease_digest_source` | function | lease_digest_source(source: &str) -> String |  |  |  |  |

| `verify_leased_proof` | function | verify_leased_proof( lease_digest: &str, source: &str, proof: &str, label: &str, ) -> Result<Report, ProofLeaseError> |  |  |  |  |

| `@prefix : <http://example.org/> .
:Socrates a :Human .
` | str_key | SOURCE = "@prefix : <http://example.org/> .
:Socrates a :Human .
" |  |  |  |  |


### src/qualification.rs

| `Standing` | enum | Standing { Unqualified, Observed, Attacked, Qualified, Refused, Unknown } |  |  |  |  |

| `compose_courts` | function | compose_courts(parent: &Value, child: &Value) -> Result<BTreeSet<String>, String> |  |  |  |  |

| `evaluate` | function | evaluate(observation: Observation) -> Standing |  |  |  |  |

| `generate_falsifier` | function | generate_falsifier(law: &Value, seed: &Value) -> Result<Value, String> |  |  |  |  |

| `temporal_rule_holds` | function | temporal_rule_holds(events: &[&str], rule: &str) -> bool |  |  |  |  |

| `Observation` | struct | Observation { pub attempt_observed: bool, pub violation_observed: bool, pub independent_observer: bool, pub postconditions_satisfied: bool, pub durable_evidence: bool, pub crash_observed: bool, pub crash_reconciled: bool } |  |  |  |  |


### src/query_cache.rs

| `bump` | function | bump(&self) -> u64 |  |  |  |  |

| `consult` | function | consult( &self, h_query: [u8; 32], grounding: Option<&[u8]>, ) -> Option<(ExecutionReceipt, u64)> |  |  |  |  |

| `epoch` | function | epoch(&self) -> u64 |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `now_ms` | function | now_ms() -> u64 |  |  |  |  |

| `store` | function | store(&self, entry: CacheEntry) |  |  |  |  |

| `with_entries` | function | with_entries(&self, f: impl FnOnce(&mut HashMap<[u8; 32], CacheEntry>) -> R) -> R |  |  |  |  |

| `CacheEntry` | struct | CacheEntry { pub h_query: [u8; 32], pub receipt: ExecutionReceipt, pub grounding: Vec<u8>, pub epoch: u64, pub stored_at: u64 } |  |  |  |  |

| `QueryCache` | struct | QueryCache { entries: Mutex<HashMap<[u8; 32], CacheEntry>>, epoch: AtomicU64 } |  |  |  |  |


### src/query_ledger.rs

| `batch::*` | use | batch::* |  |  |  |  |

| `storage::*` | use | storage::* |  |  |  |  |


### src/query_ledger/batch.rs

| `FLUSH_AGE_MS` | const | FLUSH_AGE_MS: u64 |  |  |  |  |

| `FLUSH_COUNT` | const | FLUSH_COUNT: usize |  |  |  |  |

| `GENESIS` | const | GENESIS: &str |  |  |  |  |

| `LedgerError` | enum | LedgerError { Io(String), DigestMismatch { file: String, expected: String, actual: String, }, Malformed { file: String, }, BrokenChain { detail: String, }, Fork { parent: String, }, DoubleGenesis { digest: String, }, MissingAgg { batch: String, }, AggMismatch { batch: String, }, ChainIndex { detail: String, } } |  |  |  |  |

| `content_digest` | function | content_digest(bytes: &[u8]) -> String |  |  |  |  |

| `fold_batch` | function | fold_batch(batch: &str, records: &[Descriptor]) -> BatchAgg |  |  |  |  |

| `0000000000000000000000000000000000000000000000000000000000000000` | str_key | GENESIS = "0000000000000000000000000000000000000000000000000000000000000000" |  |  |  |  |

| `Aggregate` | struct | Aggregate { pub rows_emitted_sum: BTreeMap<u64, u64>, pub saturated: bool, pub complete: bool, pub batches: usize } |  |  |  |  |

| `BatchAgg` | struct | BatchAgg { pub batch: String, pub rows_emitted_sum: BTreeMap<u64, u64>, pub saturated: bool } |  |  |  |  |

| `ChainState` | struct | ChainState { pub head: String, pub tail: String, pub batches: u64, pub next_seq: u64, pub aggs: BTreeMap<String, String> } |  |  |  |  |

| `Descriptor` | struct | Descriptor { pub h_query: [u8; 32], pub rows_emitted: u32, pub flags: u8, pub ts_unix_ms: u64, pub epoch: u64 } |  |  |  |  |


### src/query_ledger/storage.rs

| `accounted_descriptors` | function | accounted_descriptors(dir: impl AsRef<Path>) -> Result<u64, LedgerError> |  |  |  |  |

| `aggregate` | function | aggregate(&self, from_ms: u64, to_ms: u64) -> Result<Aggregate, LedgerError> |  |  |  |  |

| `chain_state` | function | chain_state(&self) -> ChainState |  |  |  |  |

| `flush` | function | flush(&mut self) -> Result<(), LedgerError> |  |  |  |  |

| `gaps` | function | gaps(&self) -> Result<Vec<(u64, u64)>, LedgerError> |  |  |  |  |

| `now_ms` | function | now_ms() -> u64 |  |  |  |  |

| `open` | function | open(dir: impl AsRef<Path>) -> Result<Self, LedgerError> |  |  |  |  |

| `open_with_producer` | function | open_with_producer( dir: impl AsRef<Path>, producer_next_seq: u64, ) -> Result<Self, LedgerError> |  |  |  |  |

| `push` | function | push(&mut self, d: Descriptor) -> Result<(), LedgerError> |  |  |  |  |

| `record_batch` | function | record_batch(&mut self, batch: &[Descriptor]) -> Result<(), LedgerError> |  |  |  |  |

| `record_gap` | function | record_gap(&self, first: u64, last: u64) -> Result<(), LedgerError> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<ChainState, LedgerError> |  |  |  |  |

| `QueryLedger` | struct | QueryLedger { dir: PathBuf, parent: String, next_seq: u64, batches: u64, head: String, tail: String, buf: Vec<Descriptor>, buf_first_seq: u64, buf_opened_ms: u64, aggs: BTreeMap<String, String>, pub last_flush_error: Option<LedgerError> } |  |  |  |  |


### src/receipt.rs

| `record` | function | record(state: &LawState, receipt: &Receipt) -> Result<LawState, LawError> |  |  |  |  |

| `record_signed` | function | record_signed( state: &LawState, receipt: &Receipt, attestation: &Attestation, ) -> Result<LawState, LawError> |  |  |  |  |

| `require` | function | require(state: &LawState, step: &str) -> Result<(), LawError> |  |  |  |  |

| `require_signed` | function | require_signed(state: &LawState, step: &str, trusted: &TrustedKeys) -> Result<(), LawError> |  |  |  |  |

| `urn:graphlaw:` | str_key | NS = "urn:graphlaw:" |  |  |  |  |


### src/receipt_chain.rs

| `MIGRATION_LAW_1_TO_2` | const | MIGRATION_LAW_1_TO_2: &str |  |  |  |  |

| `SCHEMA_V1` | const | SCHEMA_V1: &str |  |  |  |  |

| `SCHEMA_V2` | const | SCHEMA_V2: &str |  |  |  |  |

| `AdmissionDecision` | enum | AdmissionDecision { Admitted, Quarantined, Refused } |  |  |  |  |

| `AdmissionLedger` | enum | AdmissionLedger { LegacyUnrecorded, Recorded(Vec<AdmissionItem>) } |  |  |  |  |

| `AndonLevel` | enum | AndonLevel { Red, Yellow, Green } |  |  |  |  |

| `CeilingLevel` | enum | CeilingLevel { Red, LegacyObserved, Yellow, Green } |  |  |  |  |

| `CheckOutcome` | enum | CheckOutcome { Pass, Fail(String), Skip(String) } |  |  |  |  |

| `EquivalenceStatus` | enum | EquivalenceStatus { Unknown, Equivalent, Divergent(String) } |  |  |  |  |

| `ObligationCount` | enum | ObligationCount { Unknown, Tracked { required: u32, discharged: u32, } } |  |  |  |  |

| `ObservedOutcome` | enum | ObservedOutcome { Pass, Fail, Unknown } |  |  |  |  |

| `admission_item` | function | admission_item(mut self, item: AdmissionItem) -> Self |  |  |  |  |

| `all_unknown` | function | all_unknown() -> Self |  |  |  |  |

| `build` | function | build(self) -> Result<ReceiptEpochV2, CoreError> |  |  |  |  |

| `compute_ceiling` | function | compute_ceiling(prev: CeilingLevel, components: &ComponentLevels) -> CeilingLevel |  |  |  |  |

| `compute_obligation_count` | function | compute_obligation_count(ledger: &AdmissionLedger) -> ObligationCount |  |  |  |  |

| `derive_andon` | function | derive_andon(ledger: &AdmissionLedger) -> AndonLevel |  |  |  |  |

| `equivalence` | function | equivalence(mut self, map: EquivalenceMap) -> Self |  |  |  |  |

| `is_pass` | function | is_pass(&self) -> bool |  |  |  |  |

| `legacy_bounded` | function | legacy_bounded() -> Self |  |  |  |  |

| `level` | function | level(&self) -> AndonLevel |  |  |  |  |

| `new` | function | new(prev_ceiling: CeilingLevel, components: ComponentLevels) -> Self |  |  |  |  |

| `read_receipt_epoch` | function | read_receipt_epoch(record: &super::ReceiptRecord) -> Result<ReceiptEpochV2, CoreError> |  |  |  |  |

| `recoverable` | function | recoverable(prev: CeilingLevel) -> CeilingLevel |  |  |  |  |

| `remaining` | function | remaining(&self) -> Option<u32> |  |  |  |  |

| `supported` | function | supported(components: &ComponentLevels) -> CeilingLevel |  |  |  |  |

| `uniform` | function | uniform(level: AndonLevel) -> Self |  |  |  |  |

| `validate` | function | validate(records: &[ReceiptRecord], clock: &dyn Clock) -> Verdict |  |  |  |  |

| `validate_promotion` | function | validate_promotion( current_ceiling: CeilingLevel, actuator_identity: &str, witness: &PromotionWitness, ) -> Result<CeilingLevel, CoreError> |  |  |  |  |

| `with_explicit_ceiling` | function | with_explicit_ceiling(mut self, level: CeilingLevel) -> Self |  |  |  |  |

| `with_generation` | function | with_generation(mut self, generation: u64) -> Self |  |  |  |  |

| `with_verified_promotion` | function | with_verified_promotion( mut self, witness: &PromotionWitness, actuator_identity: &str, ) -> Result<Self, CoreError> |  |  |  |  |

| `M_1_to_2` | str_key | MIGRATION_LAW_1_TO_2 = "M_1_to_2" |  |  |  |  |

| `ggen-receipt/v1` | str_key | SCHEMA_V1 = "ggen-receipt/v1" |  |  |  |  |

| `ggen-receipt/v2` | str_key | SCHEMA_V2 = "ggen-receipt/v2" |  |  |  |  |

| `AdmissionItem` | struct | AdmissionItem { pub evidence_id: String, pub observed_outcome: ObservedOutcome, pub decision: AdmissionDecision, pub reason: String, pub obligations_discharged: Vec<String>, pub obligations_created: Vec<String>, pub generation: u64 } |  |  |  |  |

| `ComponentLevels` | struct | ComponentLevels { pub lint: AndonLevel, pub test: AndonLevel, pub fmt: AndonLevel, pub gate: AndonLevel } |  |  |  |  |

| `EquivalenceMap` | struct | EquivalenceMap { pub source: EquivalenceStatus, pub compiled_binary: EquivalenceStatus, pub docs: EquivalenceStatus, pub tests: EquivalenceStatus, pub receipts: EquivalenceStatus, pub evidence: EquivalenceStatus, pub gates: EquivalenceStatus, pub config: EquivalenceStatus } |  |  |  |  |

| `FixedClock` | struct | FixedClock { pub u64 } |  |  |  |  |

| `MigrationReceipt` | struct | MigrationReceipt { pub migration_law: String, pub from_schema: String, pub to_schema: String, pub final_v1_chain_hash_hex: String, pub first_v2_chain_hash_hex: String, pub carries_forward: Vec<String>, pub becomes_unknown: Vec<String>, pub resulting_ceiling: CeilingLevel } |  |  |  |  |

| `PromotionWitness` | struct | PromotionWitness { pub from_ceiling: CeilingLevel, pub to_ceiling: CeilingLevel, pub evidence_receipts: Vec<String>, pub closed_obligations: Vec<String>, pub verifier_identity: String, pub authorization_basis: String } |  |  |  |  |

| `ReceiptEpochV2` | struct | ReceiptEpochV2 { pub admission: AdmissionLedger, pub standing_ceiling: CeilingLevel, pub equivalence: EquivalenceMap, pub obligation_count: ObligationCount, pub andon: AndonLevel, pub promotion_eligible: bool, pub generation: u64 } |  |  |  |  |

| `ReceiptEpochV2Builder` | struct | ReceiptEpochV2Builder { prev_ceiling: CeilingLevel, components: ComponentLevels, admission: Vec<AdmissionItem>, equivalence: EquivalenceMap, explicit_ceiling: Option<CeilingLevel>, promoted: bool, generation: u64 } |  |  |  |  |

| `ReceiptRecordV1Legacy` | struct | ReceiptRecordV1Legacy { pub version: u32, pub instruction_id: u64, pub activity_idx: u16, pub activity: Option<String>, pub node_kind: u8, pub ts_ns: u64, pub duration_ms: Option<u64>, pub payload_hash_hex: String, pub prev_chain_hash_hex: String, pub chain_hash_hex: String, pub andon: super::Andon, pub obligation_count: u32, pub object_ids: Vec<String>, pub signature_hex: Option<String> } |  |  |  |  |

| `ReceiptValidator` | struct |  |  |  |  |  |

| `StageResult` | struct | StageResult { pub stage: String, pub outcome: CheckOutcome } |  |  |  |  |

| `SystemClock` | struct |  |  |  |  |  |

| `Verdict` | struct | Verdict { pub ok: bool, pub stages: Vec<StageResult>, pub records_checked: usize } |  |  |  |  |

| `Clock` | trait |  |  |  |  |  |

| `record::*` | use | record::* |  |  |  |  |


### src/receipt_chain/frame.rs

| `ADMITTED` | const | ADMITTED: Self |  |  |  |  |

| `AUTHORIZATION_DENIED` | const | AUTHORIZATION_DENIED: Self |  |  |  |  |

| `CONFORMANCE_GATE_FAILED` | const | CONFORMANCE_GATE_FAILED: Self |  |  |  |  |

| `OBJECT_LIFECYCLE_VIOLATION` | const | OBJECT_LIFECYCLE_VIOLATION: Self |  |  |  |  |

| `PRECONDITION_FAILED` | const | PRECONDITION_FAILED: Self |  |  |  |  |

| `RESOURCE_EXHAUSTED` | const | RESOURCE_EXHAUSTED: Self |  |  |  |  |

| `SLA_BREACH` | const | SLA_BREACH: Self |  |  |  |  |

| `WATCHDOG_DRAINED` | const | WATCHDOG_DRAINED: Self |  |  |  |  |

| `chain` | function | chain(&mut self, frame: &OcelCausalFrame) |  |  |  |  |

| `compose` | function | compose(self, other: Self) -> Self |  |  |  |  |

| `genesis` | function | genesis(run_id: [u8; 32]) -> Self |  |  |  |  |

| `is_admitted` | function | is_admitted(self) -> bool |  |  |  |  |

| `to_fired_mask` | function | to_fired_mask(self) -> u64 |  |  |  |  |

| `to_hash_bytes` | function | to_hash_bytes(&self) -> [u8; 99] |  |  |  |  |

| `DenialPolarity` | struct | DenialPolarity { pub u64 } |  |  |  |  |

| `OcelCausalFrame` | struct | OcelCausalFrame { pub instruction_id: u64, pub fired_mask: u64, pub denial: DenialPolarity, pub obj_refs: [PackedObjRef; 8], pub ts_ns: u64, pub activity_idx: u16, pub node_kind: u8, pub pad: [u8; 5], pub prior_hash: [u8; 32] } |  |  |  |  |

| `OcelCausalReceipt` | struct | OcelCausalReceipt { pub chain_hash: [u8; 32], pub frame_count: u64, pub run_id: [u8; 32], pub replay_ptr: u64 } |  |  |  |  |

| `PackedObjRef` | struct | PackedObjRef { pub u32 } |  |  |  |  |


### src/receipt_chain/record.rs

| `CHAIN_RULE_BASE` | const | CHAIN_RULE_BASE: &str |  |  |  |  |

| `CHAIN_RULE_V2_FOLD` | const | CHAIN_RULE_V2_FOLD: &str |  |  |  |  |

| `RECEIPT_RECORD_VERSION` | const | RECEIPT_RECORD_VERSION: u32 |  |  |  |  |

| `Andon` | enum | Andon { Green, Halted { unmet: Vec<Obligation>, refusals: Vec<RefusalScenario>, at: u64, }, Overridden { by: String, reason: String, at: u64, } } |  |  |  |  |

| `ChainRule` | enum | ChainRule { Base, V2Fold } |  |  |  |  |

| `ChainStanding` | enum | ChainStanding { FullyBound, LegacyV2Unbound } |  |  |  |  |

| `ChainVerification` | enum | ChainVerification { Verified(ChainStanding), Mismatch { rule: ChainRule, recomputed: [u8; 32], } } |  |  |  |  |

| `CoreError` | enum | CoreError { SerializationFailed(String), HexDecodeFailed(String), ReceiptChainRuleInvalid(String), Io(String), ReceiptSchemaPayloadMismatch(String), UnrecognizedReceiptSchema(String), CeilingExceedsMeet { requested: crate::receipt_chain::epoch::CeilingLevel, allowed: crate::receipt_chain::epoch::CeilingLevel, }, PromotionRefused { reason: String, } } |  |  |  |  |

| `Obligation` | enum | Obligation { Precondition { predicate_id: String, params_hash: [u8; 32], }, BlockingConstraint { reason: String, }, EvidenceRequired { evidence_type: String, } } |  |  |  |  |

| `RefusalCategory` | enum | RefusalCategory { Identity, Capacity, Topology, Temporal, Lifecycle, Authorization, Prerequisites, Reserved } |  |  |  |  |

| `RefusalScenario` | enum | RefusalScenario { BlockingConstraint { reason: String, }, MissingEvidence { evidence_type: String, }, UnsatisfiedPrecondition { predicate_id: String, }, WatchdogDrained, PreconditionFailed, SlaBreach, AuthorizationDenied, ResourceExhausted, ObjectLifecycleViolation, ConformanceGateFailed } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `build_admission_frame` | function | build_admission_frame( payload_hash: &[u8; 32], prev_chain_hash: &[u8; 32], meta: &ReceiptMeta, ts_ns: u64, ) -> OcelCausalFrame |  |  |  |  |

| `category` | function | category(&self) -> RefusalCategory |  |  |  |  |

| `chain_from_frame` | function | chain_from_frame(prev_chain_hash: &[u8; 32], frame: &OcelCausalFrame) -> [u8; 32] |  |  |  |  |

| `chain_hash` | function | chain_hash(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `compose_denials` | function | compose_denials( scenarios: impl IntoIterator<Item = &'a RefusalScenario>, ) -> DenialPolarity |  |  |  |  |

| `declared_chain_rule` | function | declared_chain_rule(&self) -> Result<Option<ChainRule>, CoreError> |  |  |  |  |

| `denial_lane` | function | denial_lane(scenario: &RefusalScenario) -> DenialPolarity |  |  |  |  |

| `first_declared` | function | first_declared(&self) -> Option<usize> |  |  |  |  |

| `name` | function | name(&self) -> &'static str |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `observe` | function | observe( &mut self, idx: usize, record: &ReceiptRecord, standing: ChainStanding, ) -> Result<(), CoreError> |  |  |  |  |

| `observe_declared` | function | observe_declared( &mut self, idx: usize, declared: bool, standing: ChainStanding, ) -> Result<(), CoreError> |  |  |  |  |

| `parse` | function | parse(s: &str) -> Result<Self, CoreError> |  |  |  |  |

| `payload_hash` | function | payload_hash(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `prev_chain_hash` | function | prev_chain_hash(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `recompute_chain_hash` | function | recompute_chain_hash(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `recompute_chain_hash_lawful` | function | recompute_chain_hash_lawful(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `recompute_chain_hash_under` | function | recompute_chain_hash_under(&self, rule: ChainRule) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `scenario_for_denial_lane` | function | scenario_for_denial_lane(lane: DenialPolarity) -> Option<RefusalScenario> |  |  |  |  |

| `verify_chain` | function | verify_chain(&self) -> Result<ChainVerification, CoreError> |  |  |  |  |

| `verify_chain_against` | function | verify_chain_against(&self, stored: [u8; 32]) -> Result<ChainVerification, CoreError> |  |  |  |  |

| `graphlaw-chain/base` | str_key | CHAIN_RULE_BASE = "graphlaw-chain/base" |  |  |  |  |

| `graphlaw-chain/v2-fold` | str_key | CHAIN_RULE_V2_FOLD = "graphlaw-chain/v2-fold" |  |  |  |  |

| `ChainRuleMonotonicity` | struct | ChainRuleMonotonicity { first_declared: Option<usize> } |  |  |  |  |

| `ReceiptMeta` | struct | ReceiptMeta { pub instruction_id: u64, pub activity_idx: u16, pub node_kind: u8, pub ts_ns: Option<u64>, pub denial: DenialPolarity, pub andon: Andon, pub object_ids: Vec<String>, pub obligation_count: u32 } |  |  |  |  |

| `ReceiptRecord` | struct | ReceiptRecord { pub version: u32, pub instruction_id: u64, pub activity_idx: u16, pub activity: Option<String>, pub node_kind: u8, pub ts_ns: u64, pub duration_ms: Option<u64>, pub origin: Option<String>, pub payload_hash_hex: String, pub prev_chain_hash_hex: String, pub chain_hash_hex: String, pub andon: Andon, pub obligation_count: u32, pub object_ids: Vec<String>, pub signature_hex: Option<String>, pub schema: String, pub v2: Option<epoch::ReceiptEpochV2>, pub chain_rule: Option<String> } |  |  |  |  |


### src/receipt_chain/store.rs

| `DEFAULT_RECEIPTS_DIR` | const | DEFAULT_RECEIPTS_DIR: &str |  |  |  |  |

| `GENESIS_CHAIN_HASH` | const | GENESIS_CHAIN_HASH: [u8; 32] |  |  |  |  |

| `LEDGER_FILE_NAME` | const | LEDGER_FILE_NAME: &str |  |  |  |  |

| `append` | function | append(&self, record: &ReceiptRecord) -> Result<(), CoreError> |  |  |  |  |

| `last_chain_hash` | function | last_chain_hash(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `load_all` | function | load_all(&self) -> Result<Vec<ReceiptRecord>, CoreError> |  |  |  |  |

| `open` | function | open(dir: impl AsRef<Path>) -> Result<Self, CoreError> |  |  |  |  |

| `open_default` | function | open_default() -> Result<Self, CoreError> |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `receipts` | str_key | DEFAULT_RECEIPTS_DIR = "receipts" |  |  |  |  |

| `receipts.jsonl` | str_key | LEDGER_FILE_NAME = "receipts.jsonl" |  |  |  |  |

| `ReceiptStore` | struct | ReceiptStore { path: PathBuf } |  |  |  |  |


### src/receipt_store.rs

| `StoreError` | enum | StoreError { Io(String), InvalidSubject(String), Malformed { file: String, }, DigestMismatch { file: String, expected: String, actual: String, }, Unattested { file: String, }, Attestation { file: String, error: AttestError, }, BrokenChain { detail: String, } } |  |  |  |  |

| `list` | function | list(&self, subject_sha: &str) -> Result<Vec<String>, StoreError> |  |  |  |  |

| `open` | function | open(dir: impl AsRef<Path>) -> Result<Self, StoreError> |  |  |  |  |

| `put` | function | put(&self, receipt: &Receipt, subject_sha: &str) -> Result<String, StoreError> |  |  |  |  |

| `put_signed` | function | put_signed( &self, receipt: &Receipt, subject_sha: &str, attestation: &Attestation, ) -> Result<String, StoreError> |  |  |  |  |

| `receipt_digest` | function | receipt_digest(r: &Receipt) -> String |  |  |  |  |

| `verify` | function | verify(&self, subject_sha: &str) -> Result<Vec<Receipt>, StoreError> |  |  |  |  |

| `verify_attested` | function | verify_attested( &self, subject_sha: &str, trusted: &TrustedKeys, ) -> Result<Vec<Receipt>, StoreError> |  |  |  |  |

| `ReceiptStore` | struct | ReceiptStore { dir: PathBuf } |  |  |  |  |


### src/registry.rs

| `definitions::*` | use | definitions::* |  |  |  |  |

| `emit_json::*` | use | emit_json::* |  |  |  |  |

| `emit_ttl::*` | use | emit_ttl::* |  |  |  |  |


### src/registry/definitions.rs

| `ENGINES` | const | ENGINES: &[&str] |  |  |  |  |

| `GAC_NAMESPACE` | const | GAC_NAMESPACE: &str |  |  |  |  |

| `LAW_STEPS` | const | LAW_STEPS: &[LawStepRow] |  |  |  |  |

| `LEASE_CEILINGS` | const | LEASE_CEILINGS: &[&str] |  |  |  |  |

| `LEASE_REASONS` | const | LEASE_REASONS: &[&str] |  |  |  |  |

| `NATIVE_ONLY_OPS` | const | NATIVE_ONLY_OPS: &[&str] |  |  |  |  |

| `OPS` | const | OPS: &[Op] |  |  |  |  |

| `OTHER_DIALECTS` | const | OTHER_DIALECTS: &[DialectRow] |  |  |  |  |

| `POLICY_REFUSAL_KINDS` | const | POLICY_REFUSAL_KINDS: &[&str] |  |  |  |  |

| `RDF_DIALECTS` | const | RDF_DIALECTS: &[DialectRow] |  |  |  |  |

| `RECEIPT_REASONS` | const | RECEIPT_REASONS: &[&str] |  |  |  |  |

| `REFUSAL_CODES` | const | REFUSAL_CODES: &[RefusalCodeRow] |  |  |  |  |

| `REFUSAL_KINDS` | const | REFUSAL_KINDS: &[&str] |  |  |  |  |

| `REGIMES` | const | REGIMES: &[&str] |  |  |  |  |

| `REGISTRY_BASE` | const | REGISTRY_BASE: &str |  |  |  |  |

| `REGISTRY_SCHEMA` | const | REGISTRY_SCHEMA: &str |  |  |  |  |

| `FieldDefault` | enum | FieldDefault { None, Str(&'static str), Int(i64), Bool(bool) } |  |  |  |  |

| `op_names` | function | op_names() -> Vec<&'static str> |  |  |  |  |

| `other_dialect_names` | function | other_dialect_names() -> Vec<&'static str> |  |  |  |  |

| `rdf_dialect_names` | function | rdf_dialect_names() -> Vec<&'static str> |  |  |  |  |

| `EngineRejected` | str_key | KINDS_ENGINE = "EngineRejected" |  |  |  |  |

| `MissingEntry` | str_key | POLICY_REFUSAL_KINDS = "MissingEntry" |  |  |  |  |

| `NotSemanticContent` | str_key | REFUSAL_KINDS = "NotSemanticContent" |  |  |  |  |

| `PurRdf` | str_key | ENGINES = "PurRdf" |  |  |  |  |

| `Refused` | str_key | REFUSAL_CODES = "Refused" |  |  |  |  |

| `ResourceLimit` | str_key | CODES_LIMIT = "ResourceLimit" |  |  |  |  |

| `Unsupported` | str_key | KINDS_CAPABILITIES = "Unsupported" |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `data` | str_key | DATA = "data" |  |  |  |  |

| `duckdb` | str_key | NATIVE_ONLY_OPS = "duckdb" |  |  |  |  |

| `expired` | str_key | LEASE_REASONS = "expired" |  |  |  |  |

| `graphlaw.capability-registry/1` | str_key | REGISTRY_SCHEMA = "graphlaw.capability-registry/1" |  |  |  |  |

| `http://seanchatmangpt.github.io/packs/graphlaw-ash-capability#` | str_key | GAC_NAMESPACE = "http://seanchatmangpt.github.io/packs/graphlaw-ash-capability#" |  |  |  |  |

| `https://graphlaw.dev/registry#` | str_key | REGISTRY_BASE = "https://graphlaw.dev/registry#" |  |  |  |  |

| `n3` | str_key | OTHER_DIALECTS = "n3" |  |  |  |  |

| `observe` | str_key | LEASE_CEILINGS = "observe" |  |  |  |  |

| `shacl` | str_key | LAW_STEPS = "shacl" |  |  |  |  |

| `simple` | str_key | REGIMES = "simple" |  |  |  |  |

| `turtle` | str_key | RDF_DIALECTS = "turtle" |  |  |  |  |

| `unattested` | str_key | RECEIPT_REASONS = "unattested" |  |  |  |  |

| `DialectRow` | struct | DialectRow { pub name: &'static str, pub aliases: &'static [&'static str], pub response_name: &'static str, pub media_type: Option<&'static str> } |  |  |  |  |

| `Field` | struct | Field { pub name: &'static str, pub ty: &'static str, pub required: bool, pub nullable: bool, pub doc: &'static str, pub enum_values: &'static [&'static str], pub default: FieldDefault } |  |  |  |  |

| `LawStepRow` | struct | LawStepRow { pub name: &'static str, pub ceiling: Option<&'static str>, pub fields: &'static [Field] } |  |  |  |  |

| `Op` | struct | Op { pub name: &'static str, pub summary: &'static str, pub request: &'static [Field], pub responses: &'static [Variant], pub refusal_kinds: &'static [&'static str], pub refusal_codes: &'static [&'static str] } |  |  |  |  |

| `RefusalCodeRow` | struct | RefusalCodeRow { pub code: &'static str, pub kind: Option<&'static str>, pub fields: &'static [Field] } |  |  |  |  |

| `Variant` | struct | Variant { pub tag: Option<&'static str>, pub tag_field: Option<&'static str>, pub fields: &'static [Field] } |  |  |  |  |


### src/registry/emit_json.rs

| `canonical` | function | canonical(v: &Value) -> String |  |  |  |  |

| `receipt_surface_blake3` | function | receipt_surface_blake3() -> String |  |  |  |  |

| `registry_blake3` | function | registry_blake3() -> String |  |  |  |  |

| `registry_json_pretty` | function | registry_json_pretty() -> String |  |  |  |  |

| `registry_value` | function | registry_value() -> Value |  |  |  |  |

| `surface_blake3` | function | surface_blake3() -> String |  |  |  |  |

| `Lease` | str_key | MODELS = "Lease" |  |  |  |  |

| `max_request_bytes` | str_key | LIMIT_META = "max_request_bytes" |  |  |  |  |

| `src/duckdb_eval.rs` | str_key | FILES = "src/duckdb_eval.rs" |  |  |  |  |


### src/registry/emit_ttl.rs

| `registry_turtle` | function | registry_turtle() -> String |  |  |  |  |


### src/rules/builtins.rs

| `BuiltinError` | enum | BuiltinError { Eval(EvaluatorError), InvalidArguments { expected: String, got: String }, AstError(String) } |  |  |  |  |

| `builtin_eval_sql` | function | builtin_eval_sql( args: &[&str], output_vars: &[&str], ctx: &BuiltinContext, reader: &R, ) -> Result<BuiltinResult, BuiltinError> |  |  |  |  |

| `BuiltinContext` | struct | BuiltinContext { pub db: &'a GraphLawDuckDB } |  |  |  |  |

| `BuiltinResult` | struct | BuiltinResult { pub bindings: Vec<(String, TermValue)>, pub derived_triples: Vec<RdfTriple>, pub query_hash: [u8; 32], pub rows_emitted: usize } |  |  |  |  |


### src/shacl_guard.rs

| `MAX_SHACL_DEGREE` | const | MAX_SHACL_DEGREE: usize |  |  |  |  |

| `MAX_SHACL_QUADS` | const | MAX_SHACL_QUADS: usize |  |  |  |  |

| `MAX_SHACL_SHAPES_BYTES` | const | MAX_SHACL_SHAPES_BYTES: usize |  |  |  |  |

| `MAX_SHACL_VALIDATION_DEPTH` | const | MAX_SHACL_VALIDATION_DEPTH: usize |  |  |  |  |

| `ShaclLimitExceeded` | enum | ShaclLimitExceeded { Bytes { observed: usize, }, Quads { observed: usize, }, Degree { observed: usize, }, Depth { observed: usize, } } |  |  |  |  |

| `cap` | function | cap(&self) -> usize |  |  |  |  |

| `check_shape_depth` | function | check_shape_depth(shapes_ttl: &str) -> Result<(), DepthExceeded> |  |  |  |  |

| `check_shape_limits` | function | check_shape_limits(shapes_ttl: &str) -> Result<(), ShaclLimitExceeded> |  |  |  |  |

| `limit_name` | function | limit_name(&self) -> &'static str |  |  |  |  |

| `observed` | function | observed(&self) -> usize |  |  |  |  |

| `and` | str_key | SHAPE_LIST_PREDICATES = "and" |  |  |  |  |

| `http://www.w3.org/1999/02/22-rdf-syntax-ns#first` | str_key | RDF_FIRST = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first" |  |  |  |  |

| `http://www.w3.org/1999/02/22-rdf-syntax-ns#rest` | str_key | RDF_REST = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest" |  |  |  |  |

| `http://www.w3.org/ns/shacl#` | str_key | SH = "http://www.w3.org/ns/shacl#" |  |  |  |  |

| `node` | str_key | SHAPE_LINK_PREDICATES = "node" |  |  |  |  |

| `DepthExceeded` | struct | DepthExceeded { pub observed: usize } |  |  |  |  |


### src/smon.rs

| `DCTERMS` | const | DCTERMS: &str |  |  |  |  |

| `DFL` | const | DFL: &str |  |  |  |  |

| `SMON` | const | SMON: &str |  |  |  |  |

| `broaden` | function | broaden( ds: &RdfDataset, canonical_topic: &str, ) -> Result<(Arc<RdfDataset>, Broadened), ToolError> |  |  |  |  |

| `build_dataset` | function | build_dataset( turns: &[Turn], session_id: &str, base: &str, ) -> Result<Arc<RdfDataset>, ToolError> |  |  |  |  |

| `classify_turn` | function | classify_turn(text: &str) -> (&'static str, Vec<String>) |  |  |  |  |

| `extract_topic` | function | extract_topic(text: &str) -> String |  |  |  |  |

| `from_env` | function | from_env() -> Self |  |  |  |  |

| `read_turtle` | function | read_turtle(path: &Path) -> Result<Arc<RdfDataset>, ToolError> |  |  |  |  |

| `required` | function | required(&self, flag: &str) -> Result<&str, ToolError> |  |  |  |  |

| `switch` | function | switch(&self, flag: &str) -> bool |  |  |  |  |

| `to_turtle` | function | to_turtle(ds: &RdfDataset) -> Result<Vec<u8>, ToolError> |  |  |  |  |

| `turns_from_jsonl` | function | turns_from_jsonl(raw: &str, assistant_only: bool) -> Vec<Turn> |  |  |  |  |

| `value` | function | value(&self, flag: &str) -> Option<&str> |  |  |  |  |

| `write_turtle` | function | write_turtle(ds: &RdfDataset, out: &Path) -> Result<(), ToolError> |  |  |  |  |

| `arazzo` | str_key | TOPIC_KEYWORDS = "arazzo" |  |  |  |  |

| `http://purl.org/dc/terms/` | str_key | DCTERMS = "http://purl.org/dc/terms/" |  |  |  |  |

| `http://seanchatmangpt.github.io/packs/dogfood-lifecycle#` | str_key | DFL = "http://seanchatmangpt.github.io/packs/dogfood-lifecycle#" |  |  |  |  |

| `http://seanchatmangpt.github.io/packs/self-monitoring#` | str_key | SMON = "http://seanchatmangpt.github.io/packs/self-monitoring#" |  |  |  |  |

| `http://www.w3.org/1999/02/22-rdf-syntax-ns#type` | str_key | RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" |  |  |  |  |

| `http://www.w3.org/2001/XMLSchema#integer` | str_key | XSD_INTEGER = "http://www.w3.org/2001/XMLSchema#integer" |  |  |  |  |

| `the` | str_key | STOPWORDS = "the" |  |  |  |  |

| `Args` | struct | Args { Vec<String> } |  |  |  |  |

| `Broadened` | struct | Broadened { pub audit: Vec<String>, pub rewrite1: usize, pub rewrite2: usize } |  |  |  |  |

| `Turn` | struct | Turn { pub seq: usize, pub role: &'static str, pub text: String, pub kind: &'static str, pub evidence: Vec<String>, pub topic: String } |  |  |  |  |


### src/sql_ast.rs

| `CompositeKind` | enum | CompositeKind { Struct, List, Index, Slice } |  |  |  |  |

| `compile_turtle` | function | compile_turtle(turtle: &str) -> Result<CompiledQuery, AstError> |  |  |  |  |

| `compile_turtle_document` | function | compile_turtle_document(turtle: &str) -> Result<CompiledDocument, AstError> |  |  |  |  |

| `validate_ast_shacl` | function | validate_ast_shacl(turtle_dataset: &str, shapes_ttl: &str) -> Result<(), AstError> |  |  |  |  |

| ` ;` | str_key | WHERE = " ;" |  |  |  |  |

| `@prefix sql: <http://graphlaw.org/sql#> .` | str_key | SELECT_HEAD = "@prefix sql: <http://graphlaw.org/sql#> ." |  |  |  |  |

| `[ sql:windowFunction [ sql:functionName ` | str_key | ROW_NUMBER_WINDOW = "[ sql:windowFunction [ sql:functionName " |  |  |  |  |

| `http://graphlaw.org/sql#` | str_key | SQL_NS = "http://graphlaw.org/sql#" |  |  |  |  |

| `http://graphlaw.org/sql#Columns` | str_key | COLUMNS = "http://graphlaw.org/sql#Columns" |  |  |  |  |

| `http://graphlaw.org/sql#Star` | str_key | STAR = "http://graphlaw.org/sql#Star" |  |  |  |  |

| `http://www.w3.org/1999/02/22-rdf-syntax-ns#type` | str_key | RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" |  |  |  |  |

| `http://www.w3.org/2001/XMLSchema#boolean` | str_key | XSD_BOOLEAN = "http://www.w3.org/2001/XMLSchema#boolean" |  |  |  |  |

| `statementIndex` | str_key | KNOWN_SQL_PREDICATES = "statementIndex" |  |  |  |  |

| `sum` | str_key | AGGREGATES = "sum" |  |  |  |  |

| `RdfAstGraphReader` | trait |  |  |  |  |  |

| `ast::*` | use | ast::* |  |  |  |  |

| `refusals::*` | use | refusals::* |  |  |  |  |

| `refusals::refusal_arm` | use | refusals::refusal_arm |  |  |  |  |

| `serializer::*` | use | serializer::* |  |  |  |  |


### src/sql_ast/ast.rs

| `AggregateFunction` | enum | AggregateFunction { Count, Sum, Avg, Min, Max, RowNumber } |  |  |  |  |

| `FrameBound` | enum | FrameBound { UnboundedPreceding, Preceding(u64), CurrentRow, Following(u64), UnboundedFollowing } |  |  |  |  |

| `FrameUnit` | enum | FrameUnit { Rows, Range } |  |  |  |  |

| `JoinType` | enum | JoinType { Inner, LeftOuter } |  |  |  |  |

| `Operator` | enum | Operator { Equals, NotEquals, GreaterThan, GreaterThanOrEq, LessThan, LessThanOrEq, And, Or, Like, ILike, Add, Subtract, Multiply, Divide, Modulo, JsonExtract, JsonExtractText } |  |  |  |  |

| `Scope` | enum | Scope { View, Process } |  |  |  |  |

| `ScopePos` | enum | ScopePos { SelectList, OrderBy, Having, Predicate, GroupBy, AggregateArgument, PredicateAggregateArgument, Qualify } |  |  |  |  |

| `SqlExpr` | enum | SqlExpr { ColumnRef { column: String, table_alias: Option<String>, }, Literal(SqlLiteral), Binary { op: Operator, left: Box<SqlExpr>, right: Box<SqlExpr>, }, FunctionCall { function: AggregateFunction, argument: Box<SqlExpr>, }, WindowFn { function: Box<SqlExpr>, partition_by: Vec<SqlExpr>, order_by: Vec<OrderTerm>, frame: Option<FrameSpec>, }, Alias { expr: Box<SqlExpr>, alias: String, }, StructLiteral { fields: Vec<(String, SqlExpr)>, }, ListLiteral { items: Vec<SqlExpr> }, IndexAccess { base: Box<SqlExpr>, index: Box<SqlExpr>, }, SliceAccess { base: Box<SqlExpr>, start: Box<SqlExpr>, end: Option<Box<SqlExpr>>, } } |  |  |  |  |

| `SqlLiteral` | enum | SqlLiteral { String(String), Integer(i64), Float(f64), Boolean(bool), Null } |  |  |  |  |

| `as_sql` | function | as_sql(&self) -> &'static str |  |  |  |  |

| `compile_merge` | function | compile_merge(&self) -> Result<CompiledQuery, AstError> |  |  |  |  |

| `from_iri` | function | from_iri(iri: &str) -> Result<Self, AstError> |  |  |  |  |

| `from_token` | function | from_token(token: &str) -> Result<Self, AstError> |  |  |  |  |

| `parse_token` | function | parse_token(token: &str) -> Result<Self, AstError> |  |  |  |  |

| `seal_bytes` | function | seal_bytes(&self) -> [u8; 32] |  |  |  |  |

| `validate_composite` | function | validate_composite(&self) -> Result<(), AstError> |  |  |  |  |

| `validate_identifier` | function | validate_identifier(ident: &str) -> Result<&str, AstError> |  |  |  |  |

| `validate_qualified_identifier` | function | validate_qualified_identifier(ident: &str) -> Result<(), AstError> |  |  |  |  |

| `verify_seal` | function | verify_seal(&self) -> bool |  |  |  |  |

| `CompiledDocument` | struct | CompiledDocument { pub statements: Vec<CompiledQuery>, pub doc_hash: [u8; 32], pub atomic: bool } |  |  |  |  |

| `CompiledQuery` | struct | CompiledQuery { pub sql: String, pub params: Vec<SqlLiteral>, pub ast_graph_turtle: Option<String>, pub canon_graph_bytes: Option<Vec<u8>>, pub receipt_state: Option<[u8; 32]>, pub cacheable: bool, pub(super) admission_seal: [u8; 32] } |  |  |  |  |

| `Cte` | struct | Cte { pub name: String, pub columns: Vec<String>, pub query: Box<SelectStatement>, pub recursive: bool } |  |  |  |  |

| `FrameSpec` | struct | FrameSpec { pub unit: FrameUnit, pub start: FrameBound, pub end: Option<FrameBound> } |  |  |  |  |

| `JoinClause` | struct | JoinClause { pub join_type: JoinType, pub table: TableRef, pub condition: SqlExpr } |  |  |  |  |

| `MergeStatement` | struct | MergeStatement { pub target: TableRef, pub source: SelectStatement, pub on: SqlExpr, pub assignments: Vec<(String, SqlExpr)> } |  |  |  |  |

| `OrderTerm` | struct | OrderTerm { pub expr: SqlExpr, pub descending: bool } |  |  |  |  |

| `TableRef` | struct | TableRef { pub name: String, pub alias: Option<String>, pub schema: Option<String> } |  |  |  |  |


### src/sql_ast/refusals.rs

| `AGG_IN_POSITION` | const | AGG_IN_POSITION: u16 |  |  |  |  |

| `AGG_IN_PREDICATE` | const | AGG_IN_PREDICATE: u16 |  |  |  |  |

| `ALIASED_AGG_ARG` | const | ALIASED_AGG_ARG: u16 |  |  |  |  |

| `ALIAS_IN_AGG_ARG` | const | ALIAS_IN_AGG_ARG: u16 |  |  |  |  |

| `ATOMIC_LITERAL` | const | ATOMIC_LITERAL: u16 |  |  |  |  |

| `CANONICALIZATION` | const | CANONICALIZATION: u16 |  |  |  |  |

| `COLUMNS_LAMBDA` | const | COLUMNS_LAMBDA: u16 |  |  |  |  |

| `COLUMNS_PATTERN` | const | COLUMNS_PATTERN: u16 |  |  |  |  |

| `COMPOSITE_DEPTH` | const | COMPOSITE_DEPTH: u16 |  |  |  |  |

| `COMPOSITE_MEMBER` | const | COMPOSITE_MEMBER: u16 |  |  |  |  |

| `CTE_NO_QUERY` | const | CTE_NO_QUERY: u16 |  |  |  |  |

| `DIRECTION_IRI` | const | DIRECTION_IRI: u16 |  |  |  |  |

| `DIRECTION_LEX` | const | DIRECTION_LEX: u16 |  |  |  |  |

| `DISCRIMINATORS` | const | DISCRIMINATORS: u16 |  |  |  |  |

| `DUAL_FROM` | const | DUAL_FROM: u16 |  |  |  |  |

| `DUAL_HAVING` | const | DUAL_HAVING: u16 |  |  |  |  |

| `DUAL_LEFT_OPERAND` | const | DUAL_LEFT_OPERAND: u16 |  |  |  |  |

| `DUAL_OPERATOR_FORMS` | const | DUAL_OPERATOR_FORMS: u16 |  |  |  |  |

| `DUAL_RIGHT_OPERAND` | const | DUAL_RIGHT_OPERAND: u16 |  |  |  |  |

| `DUAL_SELECT` | const | DUAL_SELECT: u16 |  |  |  |  |

| `DUAL_WHERE` | const | DUAL_WHERE: u16 |  |  |  |  |

| `DUP_CLAUSE_PREDICATE` | const | DUP_CLAUSE_PREDICATE: u16 |  |  |  |  |

| `EMPTY_FROM` | const | EMPTY_FROM: u16 |  |  |  |  |

| `EMPTY_PROJECTION` | const | EMPTY_PROJECTION: u16 |  |  |  |  |

| `EXPRESSION_WRAPPER` | const | EXPRESSION_WRAPPER: u16 |  |  |  |  |

| `FRAME_BOUND` | const | FRAME_BOUND: u16 |  |  |  |  |

| `FRAME_UNIT` | const | FRAME_UNIT: u16 |  |  |  |  |

| `INDEX_BAD_DTYPE` | const | INDEX_BAD_DTYPE: u16 |  |  |  |  |

| `INDEX_DUP` | const | INDEX_DUP: u16 |  |  |  |  |

| `INDEX_MISSING` | const | INDEX_MISSING: u16 |  |  |  |  |

| `INDEX_NEGATIVE` | const | INDEX_NEGATIVE: u16 |  |  |  |  |

| `INDEX_NOT_INT` | const | INDEX_NOT_INT: u16 |  |  |  |  |

| `INDEX_SEQUENCE_DISORDER` | const | INDEX_SEQUENCE_DISORDER: u16 |  |  |  |  |

| `JOIN_TYPE_IRI` | const | JOIN_TYPE_IRI: u16 |  |  |  |  |

| `LEGACY_MESSAGE` | const | LEGACY_MESSAGE: u16 |  |  |  |  |

| `LIMIT_DUP` | const | LIMIT_DUP: u16 |  |  |  |  |

| `LIMIT_MALFORMED` | const | LIMIT_MALFORMED: u16 |  |  |  |  |

| `LIMIT_NOT_INT` | const | LIMIT_NOT_INT: u16 |  |  |  |  |

| `MERGE_ON_NO_TARGET_COLUMN` | const | MERGE_ON_NO_TARGET_COLUMN: u16 |  |  |  |  |

| `MIXED_DOCUMENT_ROOTS` | const | MIXED_DOCUMENT_ROOTS: u16 |  |  |  |  |

| `NO_FROM` | const | NO_FROM: u16 |  |  |  |  |

| `NO_PROJECTION` | const | NO_PROJECTION: u16 |  |  |  |  |

| `NO_SQL_TEMPLATE` | const | NO_SQL_TEMPLATE: u16 |  |  |  |  |

| `OFFSET_DUP` | const | OFFSET_DUP: u16 |  |  |  |  |

| `OFFSET_MALFORMED` | const | OFFSET_MALFORMED: u16 |  |  |  |  |

| `OFFSET_NOT_INT` | const | OFFSET_NOT_INT: u16 |  |  |  |  |

| `OPERATOR_TOKEN` | const | OPERATOR_TOKEN: u16 |  |  |  |  |

| `PARSE_DATASET` | const | PARSE_DATASET: u16 |  |  |  |  |

| `PARSE_SHAPES` | const | PARSE_SHAPES: u16 |  |  |  |  |

| `PROCESS_SCOPE_IN_SELECT` | const | PROCESS_SCOPE_IN_SELECT: u16 |  |  |  |  |

| `QUALIFY_EXPR` | const | QUALIFY_EXPR: u16 |  |  |  |  |

| `REFUSAL_01_MALFORMED_AST` | const | REFUSAL_01_MALFORMED_AST: &str |  |  |  |  |

| `REFUSAL_02_UNIMPLEMENTED` | const | REFUSAL_02_UNIMPLEMENTED: &str |  |  |  |  |

| `REFUSAL_03_SCOPE_INTEGRITY` | const | REFUSAL_03_SCOPE_INTEGRITY: &str |  |  |  |  |

| `REFUSAL_05_UNBOUNDED_MUTATION` | const | REFUSAL_05_UNBOUNDED_MUTATION: &str |  |  |  |  |

| `REFUSAL_IDENT_BYTES` | const | REFUSAL_IDENT_BYTES: usize |  |  |  |  |

| `SCOPE_CARDINALITY` | const | SCOPE_CARDINALITY: u16 |  |  |  |  |

| `SCOPE_INDIVIDUAL` | const | SCOPE_INDIVIDUAL: u16 |  |  |  |  |

| `SCOPE_IN_PRED_AGG_ARG` | const | SCOPE_IN_PRED_AGG_ARG: u16 |  |  |  |  |

| `SCOPE_ON_TABLE_REF` | const | SCOPE_ON_TABLE_REF: u16 |  |  |  |  |

| `SCOPE_WRONG_NODE` | const | SCOPE_WRONG_NODE: u16 |  |  |  |  |

| `SEQUENCE_EMPTY` | const | SEQUENCE_EMPTY: u16 |  |  |  |  |

| `SET_OP_BY_NAME_KIND` | const | SET_OP_BY_NAME_KIND: u16 |  |  |  |  |

| `SET_OP_FLAG_LEX` | const | SET_OP_FLAG_LEX: u16 |  |  |  |  |

| `SET_OP_KIND_TOKEN` | const | SET_OP_KIND_TOKEN: u16 |  |  |  |  |

| `SET_OP_NO_RIGHT` | const | SET_OP_NO_RIGHT: u16 |  |  |  |  |

| `SHACL_DEPTH` | const | SHACL_DEPTH: u16 |  |  |  |  |

| `SHACL_ENGINE` | const | SHACL_ENGINE: u16 |  |  |  |  |

| `SHACL_VIOLATIONS` | const | SHACL_VIOLATIONS: u16 |  |  |  |  |

| `SQL_ALIAS_IN_AGG_ARG` | const | SQL_ALIAS_IN_AGG_ARG: u16 |  |  |  |  |

| `STAR_MODIFIER_POSITION` | const | STAR_MODIFIER_POSITION: u16 |  |  |  |  |

| `UNBOUNDED_MUTATION` | const | UNBOUNDED_MUTATION: u16 |  |  |  |  |

| `UNIMPLEMENTED_STATEMENT` | const | UNIMPLEMENTED_STATEMENT: u16 |  |  |  |  |

| `UNKNOWN_STATEMENT_PREDICATE` | const | UNKNOWN_STATEMENT_PREDICATE: u16 |  |  |  |  |

| `UNSUPPORTED_GENERIC` | const | UNSUPPORTED_GENERIC: u16 |  |  |  |  |

| `VIEW_SCOPE_IN_PREDICATE` | const | VIEW_SCOPE_IN_PREDICATE: u16 |  |  |  |  |

| `WINDOW_FN_POSITION` | const | WINDOW_FN_POSITION: u16 |  |  |  |  |

| `RefusalCode` | enum | RefusalCode { MalformedAst = 1, UnimplementedStatement = 2, ScopeIntegrity = 3, Capability = 4, InvalidIdentifier = 16, MissingProperty = 17, RecursionLimitExceeded = 18, MalformedList = 19, UnsupportedTerm = 20, MalformedNode = 21, LiteralNotInt = 36, LiteralNotFloat = 37, UnboundedMutation = 5 } |  |  |  |  |

| `InvalidIdentifier` | function | InvalidIdentifier(a: String) -> Self |  |  |  |  |

| `MalformedNode` | function | MalformedNode(a: String) -> Self |  |  |  |  |

| `MissingProperty` | function | MissingProperty(a: String, b: String) -> Self |  |  |  |  |

| `RecursionLimitExceeded` | function | RecursionLimitExceeded(n: usize) -> Self |  |  |  |  |

| `ShaclViolation` | function | ShaclViolation(msg: String) -> Self |  |  |  |  |

| `UnsupportedTerm` | function | UnsupportedTerm(a: String) -> Self |  |  |  |  |

| `as_str_lossy` | function | as_str_lossy(&self) -> std::borrow::Cow<'_, str> |  |  |  |  |

| `by_marker` | function | by_marker(name: &str) -> Option<u16> |  |  |  |  |

| `marker` | function | marker(self) -> Option<&'static str> |  |  |  |  |

| `pack` | function | pack(v: impl fmt::Display) -> Self |  |  |  |  |

| `packed2` | function | packed2(a: impl fmt::Display, b: impl fmt::Display) -> Self |  |  |  |  |

| `push` | function | push(&mut self, v: impl fmt::Display) |  |  |  |  |

| `[REFUSAL-01] MalformedAstRefusal:` | str_key | REFUSAL_01_MALFORMED_AST = "[REFUSAL-01] MalformedAstRefusal:" |  |  |  |  |

| `[REFUSAL-02] UnimplementedStatementRefusal:` | str_key | REFUSAL_02_UNIMPLEMENTED = "[REFUSAL-02] UnimplementedStatementRefusal:" |  |  |  |  |

| `[REFUSAL-03] ScopeIntegrityRefusal:` | str_key | REFUSAL_03_SCOPE_INTEGRITY = "[REFUSAL-03] ScopeIntegrityRefusal:" |  |  |  |  |

| `[REFUSAL-05] UnboundedMutationRefusal:` | str_key | REFUSAL_05_UNBOUNDED_MUTATION = "[REFUSAL-05] UnboundedMutationRefusal:" |  |  |  |  |

| `AstError` | struct | AstError { pub code: RefusalCode, pub ctx: RefusalContext } |  |  |  |  |

| `PackedIdent` | struct | PackedIdent { len: u8, bytes: [u8; REFUSAL_IDENT_BYTES] } |  |  |  |  |

| `RefusalContext` | struct | RefusalContext { pub shape_id: u16, pub predicate_idx: u16, pub num: u32, pub aux_num: u32, pub ident: PackedIdent, pub aux_ident: PackedIdent } |  |  |  |  |


### src/sql_ast/serializer.rs

| `SelectItem` | enum | SelectItem { Expr(SqlExpr), Star { exclude: Vec<String>, replace: Vec<(SqlExpr, String)>, }, Columns(String) } |  |  |  |  |

| `SetOpKind` | enum | SetOpKind { Union, Intersect, Except } |  |  |  |  |

| `as_sql` | function | as_sql(&self) -> &'static str |  |  |  |  |

| `compile` | function | compile(&self) -> Result<CompiledQuery, AstError> |  |  |  |  |

| `compile_set_operation` | function | compile_set_operation(&self) -> Result<CompiledQuery, AstError> |  |  |  |  |

| `from_token` | function | from_token(token: &str) -> Result<Self, AstError> |  |  |  |  |

| `SelectStatement` | struct | SelectStatement { pub ctes: Vec<Cte>, pub from: TableRef, pub joins: Vec<JoinClause>, pub select_list: Vec<SelectItem>, pub where_clause: Option<SqlExpr>, pub group_by: Vec<SqlExpr>, pub having: Option<SqlExpr>, pub order_by: Vec<OrderTerm>, pub limit: Option<usize>, pub offset: Option<usize>, pub qualify: Option<SqlExpr> } |  |  |  |  |

| `SetOperationStatement` | struct | SetOperationStatement { pub left: SelectStatement, pub kind: SetOpKind, pub all: bool, pub by_name: bool, pub right: SelectStatement } |  |  |  |  |


### src/tier.rs

| `REFUSAL_04_CAPABILITY_REFUSAL` | const | REFUSAL_04_CAPABILITY_REFUSAL: &str |  |  |  |  |

| `Admitted` | enum | Admitted { Dataset(Arc<purrdf::RdfDataset>), Rules(RuleSet), Query(crate::sql_ast::CompiledQuery) } |  |  |  |  |

| `Tier` | enum | Tier { Rdf, N3Logic, Relational } |  |  |  |  |

| `admit_bytes_tiered` | function | admit_bytes_tiered( bytes: &[u8], hint: Option<&str>, base: Option<&str>, decl: Option<DialectDecl>, ) -> Result<Admitted, Refusal> |  |  |  |  |

| `admit_declared` | function | admit_declared( bytes: &[u8], decl: DialectDecl, base: Option<&str>, ) -> Result<Admitted, Refusal> |  |  |  |  |

| `rule_count` | function | rule_count(&self) -> usize |  |  |  |  |

| `tier` | function | tier(self) -> Tier |  |  |  |  |

| `validate_n3_rules` | function | validate_n3_rules(bytes: &[u8], base: Option<&str>) -> Result<RuleSet, Refusal> |  |  |  |  |

| `REFUSAL-04` | str_key | REFUSAL_04_CAPABILITY_REFUSAL = "REFUSAL-04" |  |  |  |  |

| `DialectDecl` | struct | DialectDecl { pub tier: Tier, pub dialect: Dialect } |  |  |  |  |

| `RuleSet` | struct | RuleSet { pub document: n3::Document } |  |  |  |  |


### tests/abi_manifest_court.rs

| `0807060504030201887766554433221100ff0000000000000100000002000000030000000400000005000000060000000700000008000000bebafecaefbeaddeefbe2aa5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5` | str_key | GOLDEN_FRAME_HASH_HEX = "0807060504030201887766554433221100ff0000000000000100000002000000030000000400000005000000060000000700000008000000bebafecaefbeaddeefbe2aa5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5" |  |  |  |  |


### tests/attach_bypass_court.rs

| `/etc/passwd` | str_key | ATTACK_PATH = "/etc/passwd" |  |  |  |  |

| `attach path outside GRAPHLAW_ATTACH_ROOTS allowlist` | str_key | STABLE_REFUSAL = "attach path outside GRAPHLAW_ATTACH_ROOTS allowlist" |  |  |  |  |


### tests/bcbs_aggregation_court.rs

| `SUB-A` | str_key | SUBSIDIARIES = "SUB-A" |  |  |  |  |

| `bcbs-01` | str_key | SESSION = "bcbs-01" |  |  |  |  |

| `bcbs-02` | str_key | SESSION = "bcbs-02" |  |  |  |  |

| `bcbs-03` | str_key | SESSION = "bcbs-03" |  |  |  |  |


### tests/canonical_cross_dialect.rs

| `<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .` | str_key | PURE_NT = "<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> ." |  |  |  |  |

| `<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> <http://example.org/g> .` | str_key | DATASET_NQ = "<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> <http://example.org/g> ." |  |  |  |  |

| `_:p <http://example.org/knows> _:q .` | str_key | BNODE_NT_RELABELED = "_:p <http://example.org/knows> _:q ." |  |  |  |  |

| `_:x <http://example.org/knows> _:y .` | str_key | BNODE_NT = "_:x <http://example.org/knows> _:y ." |  |  |  |  |


### tests/chicago_boundary_constraints.rs

| `@prefix sh: <http://www.w3.org/ns/shacl#> .` | str_key | PREFIX = "@prefix sh: <http://www.w3.org/ns/shacl#> ." |  |  |  |  |


### tests/chicago_n3_datalog.rs

| `https://chicago.graphlaw.dev/` | str_key | NS = "https://chicago.graphlaw.dev/" |  |  |  |  |


### tests/chicago_validation.rs

| `MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAt9+KeDkWJa7SiBb7AMdXAPV2qpRunztoC2c7DtRmN9+Lq2YQm4rMQmmAQcwyxTxcK8vwPKOiE4mLMwmQMvTty9tL+buREfvg5KJ+FY3Bv05cZYsf7VuYgUDcw41zSEE5W1thh5PiafcrMn7CPF0jN0oFfVvAHB/fAM0yaZGPKs3WYqnHAHa6v/cVMeiWZ/lQORCBqqL2hii1fEiuIqw6AOEoZVsN0/lqm7GfnD899zrzLOPRuP4YFg46drL1UyNdg+4TquJ2OV9kR4CAfdGRexqHFWLmoxkTfnUxJ4yL+gkV57K1H+smEdKrDhDX68pEFM/34E2eVKZoRU/0paxDZQIDAQAB` | str_key | KEY = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAt9+KeDkWJa7SiBb7AMdXAPV2qpRunztoC2c7DtRmN9+Lq2YQm4rMQmmAQcwyxTxcK8vwPKOiE4mLMwmQMvTty9tL+buREfvg5KJ+FY3Bv05cZYsf7VuYgUDcw41zSEE5W1thh5PiafcrMn7CPF0jN0oFfVvAHB/fAM0yaZGPKs3WYqnHAHa6v/cVMeiWZ/lQORCBqqL2hii1fEiuIqw6AOEoZVsN0/lqm7GfnD899zrzLOPRuP4YFg46drL1UyNdg+4TquJ2OV9kR4CAfdGRexqHFWLmoxkTfnUxJ4yL+gkV57K1H+smEdKrDhDX68pEFM/34E2eVKZoRU/0paxDZQIDAQAB" |  |  |  |  |

| `[ a sql:BinaryExpression ; sql:operator ` | str_key | AGGREGATE_WHERE = "[ a sql:BinaryExpression ; sql:operator " |  |  |  |  |


### tests/combinatorial_eight_court.rs

| `http://example.org/crs/planar` | str_key | CRS_PLANAR = "http://example.org/crs/planar" |  |  |  |  |

| `http://example.org/geo#` | str_key | GEO = "http://example.org/geo#" |  |  |  |  |

| `http://example.org/geof/` | str_key | GEOF = "http://example.org/geof/" |  |  |  |  |

| `http://example.org/sf#` | str_key | SF = "http://example.org/sf#" |  |  |  |  |

| `http://example.org/unit/metre` | str_key | METRE = "http://example.org/unit/metre" |  |  |  |  |


### tests/common/enclosure.rs

| `CALIBRATION_FLOOR` | const | CALIBRATION_FLOOR: u64 |  |  |  |  |

| `CALIBRATION_SEED` | const | CALIBRATION_SEED: u64 |  |  |  |  |

| `EPOCH_DISARMED_TICKS` | const | EPOCH_DISARMED_TICKS: u64 |  |  |  |  |

| `UNBOUNDED_FUEL` | const | UNBOUNDED_FUEL: u64 |  |  |  |  |

| `WALL_BYTES` | const | WALL_BYTES: u64 |  |  |  |  |

| `WATCHDOG_INTERVAL` | const | WATCHDOG_INTERVAL: Duration |  |  |  |  |

| `EnclosureError` | enum | EnclosureError { Fuel { budget: u64 }, Epoch { ticks: u64 }, Trap(String), Fatal(String) } |  |  |  |  |

| `FuelPolicy` | enum | FuelPolicy { Unbounded, PerOp(BTreeMap<String, u64>) } |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `alloc_raw` | function | alloc_raw(&mut self, len: u32) -> Option<u32> |  |  |  |  |

| `artifact_path` | function | artifact_path() -> PathBuf |  |  |  |  |

| `budget` | function | budget(&self) -> Option<u64> |  |  |  |  |

| `budget_for` | function | budget_for(&self, op: &str) -> Option<u64> |  |  |  |  |

| `calibrate` | function | calibrate(&mut self, control: &Value) -> (u64, u64) |  |  |  |  |

| `call_raw` | function | call_raw(&mut self, ptr: u32, len: u32) -> (u32, u32) |  |  |  |  |

| `exec` | function | exec(&mut self, req: &Value) -> Value |  |  |  |  |

| `exec_seeded` | function | exec_seeded(&mut self, req: &Value, fuel_seed: u64) -> (Value, u64) |  |  |  |  |

| `exec_seeded_bytes` | function | exec_seeded_bytes(&mut self, req: &Value, fuel_seed: u64) -> (Vec<u8>, u64) |  |  |  |  |

| `free_raw` | function | free_raw(&mut self, ptr: u32, len: u32) |  |  |  |  |

| `grow_memory` | function | grow_memory(&mut self, delta_pages: u64) -> Result<u64, wasmtime::Error> |  |  |  |  |

| `limit_name` | function | limit_name(&self) -> Option<&'static str> |  |  |  |  |

| `memory_max_pages` | function | memory_max_pages(&mut self) -> Option<u64> |  |  |  |  |

| `memory_pages` | function | memory_pages(&mut self) -> u64 |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `next_pow2_ceil` | function | next_pow2_ceil(x: u64) -> u64 |  |  |  |  |

| `peak_wall_bytes` | function | peak_wall_bytes(&self) -> u64 |  |  |  |  |

| `peek_out` | function | peek_out(&mut self, ptr: u32, len: u32) -> Vec<u8> |  |  |  |  |

| `probe` | function | probe(&mut self, req: &Value) -> Result<Value, EnclosureError> |  |  |  |  |

| `read_out` | function | read_out(&mut self, ptr: u32, len: u32) -> Vec<u8> |  |  |  |  |

| `request_envelope` | function | request_envelope(&mut self, req: &Value) -> Vec<u8> |  |  |  |  |

| `request_envelope_leaking` | function | request_envelope_leaking(&mut self, req: &Value) -> Vec<u8> |  |  |  |  |

| `set_epoch_ticks_per_request` | function | set_epoch_ticks_per_request(&mut self, ticks: u64) |  |  |  |  |

| `synthesized_envelope` | function | synthesized_envelope(e: &EnclosureError) -> Vec<u8> |  |  |  |  |

| `write_body` | function | write_body(&mut self, ptr: u32, body: &[u8]) |  |  |  |  |

| `EnclosureHost` | struct | EnclosureHost { store: Store<EncState>, memory: Memory, alloc_fn: TypedFunc<u32, u32>, free_fn: TypedFunc<(u32, u32), ()>, call_fn: TypedFunc<(u32, u32), u64>, policy: FuelPolicy, epoch_ticks: u64, _watchdog: Watchdog } |  |  |  |  |

| `WallLimiter` | struct | WallLimiter { pub peak: u64 } |  |  |  |  |


### tests/common/mod.rs

| `ENGINE_LOCK` | const | ENGINE_LOCK: Mutex<()> |  |  |  |  |

| `HostError` | enum | HostError { Trap(String), Fatal(String) } |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `CARGO_TARGET_DIR` | env_key | std::env::var_os("CARGO_TARGET_DIR") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `close_session` | function | close_session(h: &mut Host, name: &str) |  |  |  |  |

| `engine_lock` | function | engine_lock() -> std::sync::MutexGuard<'static, ()> |  |  |  |  |

| `exec` | function | exec(&mut self, req: &Value) -> Value |  |  |  |  |

| `fail_classify` | function | fail_classify(stage: &str, e: wasmtime::Error) -> HostError |  |  |  |  |

| `host_or_skip` | function | host_or_skip(test_name: &str, preopen_tmp: bool) -> Option<Host> |  |  |  |  |

| `native` | function | native(req: &Value) -> Value |  |  |  |  |

| `native_bytes` | function | native_bytes(req: &Value) -> Vec<u8> |  |  |  |  |

| `new` | function | new(bytes: &[u8], preopen_tmp: bool) -> Result<Self, HostError> |  |  |  |  |

| `ok` | function | ok(req: &Value) -> Value |  |  |  |  |

| `open_session` | function | open_session(h: &mut Host, name: &str) |  |  |  |  |

| `read` | function | read(path: &str) -> String |  |  |  |  |

| `refused` | function | refused(req: &Value) -> Value |  |  |  |  |

| `request` | function | request(&mut self, req: &Value) -> Result<Value, HostError> |  |  |  |  |

| `request_bytes` | function | request_bytes(&mut self, body: &[u8]) -> Result<Vec<u8>, HostError> |  |  |  |  |

| `stream_all` | function | stream_all(&mut self, session: &str, cursor: &str) -> String |  |  |  |  |

| `try_host` | function | try_host(preopen_tmp: bool) -> Result<Host, String> |  |  |  |  |

| `wasm` | function | wasm(req: &Value) -> Value |  |  |  |  |

| `wasm_bytes` | function | wasm_bytes(req: &Value) -> Vec<u8> |  |  |  |  |

| `wasm_path` | function | wasm_path() -> PathBuf |  |  |  |  |

| `Host` | struct | Host { store: Store<WasiP1Ctx>, memory: Memory, alloc: TypedFunc<u32, u32>, free: TypedFunc<(u32, u32), ()>, call: TypedFunc<(u32, u32), u64> } |  |  |  |  |


### tests/corpus_conformance.rs

| `CARGO_BIN_EXE_{name}` | env_key | std::env::var("CARGO_BIN_EXE_{name}") |  |  |  |  |

| `UPDATE_ASSET_MANIFEST` | env_key | std::env::var_os("UPDATE_ASSET_MANIFEST") |  |  |  |  |

| `https://example.org/base/` | str_key | BASE = "https://example.org/base/" |  |  |  |  |

| `ontologies` | str_key | ROOTS = "ontologies" |  |  |  |  |


### tests/determinism_court.rs

| `248cd97b412685441df24bf44b52461a001f8ee8c70bd0a2c8a16e41b151a324` | str_key | GOLDEN_H_TRIPLES_SERTERM = "248cd97b412685441df24bf44b52461a001f8ee8c70bd0a2c8a16e41b151a324" |  |  |  |  |

| `4d51ca3c8cf5bc73780cb8cd3bed526293fa6b3fe34c5ad10cf3f8e8d14e8ff4` | str_key | GOLDEN_H_COLUMNAR_CANON = "4d51ca3c8cf5bc73780cb8cd3bed526293fa6b3fe34c5ad10cf3f8e8d14e8ff4" |  |  |  |  |

| `CREATE TABLE revenue (region TEXT, v BIGINT);` | str_key | P04_SEED_SQL = "CREATE TABLE revenue (region TEXT, v BIGINT);" |  |  |  |  |

| `GLAW_P04_PROBE_CHILD` | str_key | ENV_CHILD = "GLAW_P04_PROBE_CHILD" |  |  |  |  |

| `P04_PROBE_BEGIN` | str_key | PROBE_BEGIN = "P04_PROBE_BEGIN" |  |  |  |  |

| `P04_PROBE_END` | str_key | PROBE_END = "P04_PROBE_END" |  |  |  |  |

| `http://www.w3.org/2001/XMLSchema#integer` | str_key | XSD_INT = "http://www.w3.org/2001/XMLSchema#integer" |  |  |  |  |

| `urn:p04:obj:00` | str_key | OBJ_IDS = "urn:p04:obj:00" |  |  |  |  |


### tests/differential.rs

| `https://example.org/` | str_key | NS = "https://example.org/" |  |  |  |  |


### tests/dl_dualroute_vector_pin.rs

| `/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw` | str_key | OUT_DIR = "/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw" |  |  |  |  |

| `a-paper-baader-el` | str_key | PINS = "a-paper-baader-el" |  |  |  |  |

| `graphlaw_rev` | str_key | REV_LINE = "graphlaw_rev" |  |  |  |  |


### tests/duckdb_statement_kinds.rs

| `MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAt9+KeDkWJa7SiBb7AMdXAPV2qpRunztoC2c7DtRmN9+Lq2YQm4rMQmmAQcwyxTxcK8vwPKOiE4mLMwmQMvTty9tL+buREfvg5KJ+FY3Bv05cZYsf7VuYgUDcw41zSEE5W1thh5PiafcrMn7CPF0jN0oFfVvAHB/fAM0yaZGPKs3WYqnHAHa6v/cVMeiWZ/lQORCBqqL2hii1fEiuIqw6AOEoZVsN0/lqm7GfnD899zrzLOPRuP4YFg46drL1UyNdg+4TquJ2OV9kR4CAfdGRexqHFWLmoxkTfnUxJ4yL+gkV57K1H+smEdKrDhDX68pEFM/34E2eVKZoRU/0paxDZQIDAQAB` | str_key | EXPR_KEY = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAt9+KeDkWJa7SiBb7AMdXAPV2qpRunztoC2c7DtRmN9+Lq2YQm4rMQmmAQcwyxTxcK8vwPKOiE4mLMwmQMvTty9tL+buREfvg5KJ+FY3Bv05cZYsf7VuYgUDcw41zSEE5W1thh5PiafcrMn7CPF0jN0oFfVvAHB/fAM0yaZGPKs3WYqnHAHa6v/cVMeiWZ/lQORCBqqL2hii1fEiuIqw6AOEoZVsN0/lqm7GfnD899zrzLOPRuP4YFg46drL1UyNdg+4TquJ2OV9kR4CAfdGRexqHFWLmoxkTfnUxJ4yL+gkV57K1H+smEdKrDhDX68pEFM/34E2eVKZoRU/0paxDZQIDAQAB" |  |  |  |  |

| `core_functions` | str_key | CERTIFIED = "core_functions" |  |  |  |  |


### tests/dvp_settlement_court.rs

| `dvp-01` | str_key | SESSION = "dvp-01" |  |  |  |  |

| `dvp-02` | str_key | SESSION = "dvp-02" |  |  |  |  |

| `dvp-03` | str_key | SESSION = "dvp-03" |  |  |  |  |


### tests/graphlaw_run_eh_court.rs

| `GRAPHLAW_RUN_EH` | env_key | std::env::var_os("GRAPHLAW_RUN_EH") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `target/debug/graphlaw-run-eh` | str_key | DEFAULT_BIN = "target/debug/graphlaw-run-eh" |  |  |  |  |

| `target/wasm32-wasip1/wasm/graphlaw_wasm.wasm` | str_key | DEFAULT_ARTIFACT = "target/wasm32-wasip1/wasm/graphlaw_wasm.wasm" |  |  |  |  |


### tests/gws12_coldchain_cross_engine.rs

| `COLDCHAIN_CSV_DIR` | env_key | std::env::var("COLDCHAIN_CSV_DIR") |  |  |  |  |

| `DUCKDB_BIN` | env_key | std::env::var("DUCKDB_BIN") |  |  |  |  |

| `6971c912530619217b462f0b458a390eec34383a8155e0ccfcce869e4b6aeb8e` | str_key | COLD1_AUDIT_HASH = "6971c912530619217b462f0b458a390eec34383a8155e0ccfcce869e4b6aeb8e" |  |  |  |  |

| `BATCH-A1` | str_key | COLD1_QUARANTINED = "BATCH-A1" |  |  |  |  |

| `BATCH-R1` | str_key | COLD1_RELEASED = "BATCH-R1" |  |  |  |  |

| `SELECT o.object_id, MAX(e.temp_celsius) AS max_temp, SUM(e.duration_seconds) AS duration FROM ocel_events AS e JOIN ocel_event_object_rel AS r ON e.event_id = r.event_id JOIN ocel_objects AS o ON r.object_id = o.object_id WHERE e.temp_celsius > -20.0 AND r.qualifier = 'contains_product' AND o.object_type = 'bio:BiologicBatch' GROUP BY o.object_id` | str_key | CANONICAL_QUERY = "SELECT o.object_id, MAX(e.temp_celsius) AS max_temp, SUM(e.duration_seconds) AS duration FROM ocel_events AS e JOIN ocel_event_object_rel AS r ON e.event_id = r.event_id JOIN ocel_objects AS o ON r.object_id = o.object_id WHERE e.temp_celsius > -20.0 AND r.qualifier = 'contains_product' AND o.object_type = 'bio:BiologicBatch' GROUP BY o.object_id" |  |  |  |  |


### tests/gws4_ipc_injection_court.rs

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `@prefix sql: <http://graphlaw.org/sql#> .` | str_key | AGG_TURTLE = "@prefix sql: <http://graphlaw.org/sql#> ." |  |  |  |  |


### tests/insa_chain_differential.rs

| `05ea1aa096f45968aacb2c996d2b541e4b9fdef24f006e0204e1fa88d33f68a5` | str_key | INSA_KAT_HEAD_HEX = "05ea1aa096f45968aacb2c996d2b541e4b9fdef24f006e0204e1fa88d33f68a5" |  |  |  |  |

| `95163f2a6b24bebdfd19bf77cc60553640b8c9f5f5c54bc415c3b0da77e77598` | str_key | GRAPHLAW_HEAD_HEX = "95163f2a6b24bebdfd19bf77cc60553640b8c9f5f5c54bc415c3b0da77e77598" |  |  |  |  |


### tests/insa_law_order.rs

| `INSA_LAW_PACK` | env_key | std::env::var("INSA_LAW_PACK") |  |  |  |  |

| `INSA_LAW_PACK_ONTOLOGY` | env_key | std::env::var("INSA_LAW_PACK_ONTOLOGY") |  |  |  |  |

| `19d9ec90cb861e8b1e320ef01e5e0d8a54022b19357f2b4856f379417dcc79d7` | str_key | ONTOLOGY_BLAKE3 = "19d9ec90cb861e8b1e320ef01e5e0d8a54022b19357f2b4856f379417dcc79d7" |  |  |  |  |

| `@prefix insa: <urn:insa-law:> .` | str_key | LINK_RULES = "@prefix insa: <urn:insa-law:> ." |  |  |  |  |

| `SELECT ?a ?b WHERE { ` | str_key | VIOLATION_QUERY = "SELECT ?a ?b WHERE { " |  |  |  |  |

| `tx-source-pattern` | str_key | LAWFUL_EDGES = "tx-source-pattern" |  |  |  |  |


### tests/lease_authorization_differential.rs

| `admit:shacl` | str_key | STEP_NAMES = "admit:shacl" |  |  |  |  |

| `observe` | str_key | CEILING_NAMES = "observe" |  |  |  |  |


### tests/n3_boundary_fuzz.rs

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `NotSemanticContent` | str_key | REFUSAL_KINDS = "NotSemanticContent" |  |  |  |  |

| `P19-WASM-IDENTITY-SKIP` | str_key | MARKER = "P19-WASM-IDENTITY-SKIP" |  |  |  |  |

| `urn:n:a` | str_key | COPY_FOOTPRINT_OVERHEAD = "urn:n:a" |  |  |  |  |


### tests/n3_corpus_court.rs

| `/Users/sac/purrdf/corpora/n3` | str_key | CORPUS = "/Users/sac/purrdf/corpora/n3" |  |  |  |  |

| `literals-collections-conforming` | str_key | KNOWN_DIVERGENCES = "literals-collections-conforming" |  |  |  |  |


### tests/n3_fuse_court.rs

| `http://example.org/n3-fuse/` | str_key | NS = "http://example.org/n3-fuse/" |  |  |  |  |


### tests/n3_halt_latency_court.rs

| `http://example.org/n3-halt/` | str_key | NS = "http://example.org/n3-halt/" |  |  |  |  |


### tests/n3_hypothesis_isolation_court.rs

| `http://example.org/iso/` | str_key | NS = "http://example.org/iso/" |  |  |  |  |

| `http://www.w3.org/2000/10/swap/log#implies` | str_key | LOG_IMPLIES = "http://www.w3.org/2000/10/swap/log#implies" |  |  |  |  |


### tests/n3_limit_census_court.rs

| `n3_iterations` | str_key | NAMES = "n3_iterations" |  |  |  |  |


### tests/n3_proof_lease_court.rs

| `@prefix : <http://example.org/lease-court/> .

:tweety :is :chick .
:chick :is :bird .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
` | str_key | ORIGINAL_SOURCE = "@prefix : <http://example.org/lease-court/> .

:tweety :is :chick .
:chick :is :bird .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
" |  |  |  |  |

| `@prefix : <http://example.org/lease-court/> .

:tweety :is :chick .
:chick :is :finch .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
` | str_key | TAMPERED_SOURCE = "@prefix : <http://example.org/lease-court/> .

:tweety :is :chick .
:chick :is :finch .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
" |  |  |  |  |

| `n3_proof_lease_court.n3` | str_key | LABEL = "n3_proof_lease_court.n3" |  |  |  |  |


### tests/n3_proof_pipeline_court.rs

| `:daffy :is :animal` | str_key | CLAIM_TAMPERED = ":daffy :is :animal" |  |  |  |  |

| `:tweety :is :animal` | str_key | CLAIM = ":tweety :is :animal" |  |  |  |  |

| `@prefix : <http://example.org/proof-court/> .

:tweety :is :chick .
:chick :is :bird .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
` | str_key | CHAIN_SOURCE = "@prefix : <http://example.org/proof-court/> .

:tweety :is :chick .
:chick :is :bird .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
" |  |  |  |  |

| `@prefix : <http://example.org/proof-court/> .
@prefix log: <http://www.w3.org/2000/10/swap/log#> .

:t :trigger true .
:a :holds { :u :p _:q } .
{ :t :trigger true . :a :holds ?F . ?F log:skolem ?S } => { :t :out ?S } .
` | str_key | SKOLEM_SOURCE = "@prefix : <http://example.org/proof-court/> .
@prefix log: <http://www.w3.org/2000/10/swap/log#> .

:t :trigger true .
:a :holds { :u :p _:q } .
{ :t :trigger true . :a :holds ?F . ?F log:skolem ?S } => { :t :out ?S } .
" |  |  |  |  |

| `http://example.org/proof-court/` | str_key | NS = "http://example.org/proof-court/" |  |  |  |  |

| `https://eyereasoner.github.io/.well-known/genid/` | str_key | SKOLEM_NS = "https://eyereasoner.github.io/.well-known/genid/" |  |  |  |  |

| `n3_proof_pipeline_court.n3` | str_key | LABEL = "n3_proof_pipeline_court.n3" |  |  |  |  |


### tests/n3_quantifier_court.rs

| `http://example.org/q/` | str_key | NS = "http://example.org/q/" |  |  |  |  |

| `https://eyereasoner.github.io/.well-known/genid/` | str_key | GENID = "https://eyereasoner.github.io/.well-known/genid/" |  |  |  |  |


### tests/n3_transition_isolation_court.rs

| `@prefix : <urn:n:> .` | str_key | GOOD_RULES = "@prefix : <urn:n:> ." |  |  |  |  |


### tests/no_mock_markers.rs

| `mock` | str_key | BANNED = "mock" |  |  |  |  |

| `tests/no_mock_markers.rs` | str_key | ALLOWLIST = "tests/no_mock_markers.rs" |  |  |  |  |

| `tests/set_operations_court.rs` | str_key | ORACLE_ALLOWLIST = "tests/set_operations_court.rs" |  |  |  |  |

| `tests/wasm_env_isolation_court.rs` | str_key | UNWRAP_ALLOWLIST = "tests/wasm_env_isolation_court.rs" |  |  |  |  |


### tests/op_differential.rs

| `https://example.org/` | str_key | NS = "https://example.org/" |  |  |  |  |

| `turtle` | str_key | RDF_DIALECTS = "turtle" |  |  |  |  |


### tests/plan_admission.rs

| `<urn:p:at>` | str_key | AT = "<urn:p:at>" |  |  |  |  |


### tests/plan_builder.rs

| `<urn:d:door> <urn:p:is>` | str_key | DOOR = "<urn:d:door> <urn:p:is>" |  |  |  |  |


### tests/plan_negation.rs

| `req` | function | req(start: &str) -> serde_json::Value |  |  |  |  |

| `<urn:p:at>` | str_key | AT = "<urn:p:at>" |  |  |  |  |

| `<urn:p:occupied>` | str_key | OCC = "<urn:p:occupied>" |  |  |  |  |

| `blake3:e16c63901ed5012208d71405fd179ad947da8f849d8c3cb083c26d0ff6e31803` | str_key | POSITIVE_DIGEST = "blake3:e16c63901ed5012208d71405fd179ad947da8f849d8c3cb083c26d0ff6e31803" |  |  |  |  |


### tests/pn64_abi_conformance.rs

| `CARGO_MANIFEST_DIR` | str_key | PACK_DIR = "CARGO_MANIFEST_DIR" |  |  |  |  |

| `b3dbbc7c945a7883a75933dfc12699af393dac5fe711009bb5f4d694a2ac399b` | str_key | ONTOLOGY_BLAKE3 = "b3dbbc7c945a7883a75933dfc12699af393dac5fe711009bb5f4d694a2ac399b" |  |  |  |  |


### tests/purrdf_capabilities.rs

| `PREFIX cdt: <http://w3id.org/awslabs/neptune/SPARQL-CDTs/> ` | str_key | PFX = "PREFIX cdt: <http://w3id.org/awslabs/neptune/SPARQL-CDTs/> " |  |  |  |  |

| `http://example.org/crs/planar` | str_key | CRS = "http://example.org/crs/planar" |  |  |  |  |

| `http://example.org/geo#` | str_key | GEO = "http://example.org/geo#" |  |  |  |  |

| `http://example.org/geof/` | str_key | GEOF = "http://example.org/geof/" |  |  |  |  |

| `http://example.org/sf#` | str_key | SF = "http://example.org/sf#" |  |  |  |  |

| `http://example.org/unit/metre` | str_key | METRE = "http://example.org/unit/metre" |  |  |  |  |

| `http://www.w3.org/1999/02/22-rdf-syntax-ns#type` | str_key | TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" |  |  |  |  |

| `http://www.w3.org/2000/01/rdf-schema#subClassOf` | str_key | SUBCLASS = "http://www.w3.org/2000/01/rdf-schema#subClassOf" |  |  |  |  |

| `http://www.w3.org/2001/XMLSchema#double` | str_key | XSD_DOUBLE = "http://www.w3.org/2001/XMLSchema#double" |  |  |  |  |

| `https://example.org/` | str_key | EX = "https://example.org/" |  |  |  |  |

| `https://example.org/Animal` | str_key | ANIMAL = "https://example.org/Animal" |  |  |  |  |

| `https://example.org/Cat` | str_key | CAT = "https://example.org/Cat" |  |  |  |  |

| `https://example.org/knows` | str_key | KNOWS = "https://example.org/knows" |  |  |  |  |

| `https://example.org/tom` | str_key | TOM = "https://example.org/tom" |  |  |  |  |


### tests/receipt_chain_golden_test.rs

| `d52282d8c48e9021ff6f78e413d8a79dbffbc6b28e28db9a9d855f5810bb3185` | str_key | FIXTURE_BLAKE3 = "d52282d8c48e9021ff6f78e413d8a79dbffbc6b28e28db9a9d855f5810bb3185" |  |  |  |  |

| `tests/fixtures/fm_chain_009.json` | str_key | FIXTURE_PATH = "tests/fixtures/fm_chain_009.json" |  |  |  |  |


### tests/receipt_feedback.rs

| `<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .` | str_key | NT = "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> ." |  |  |  |  |

| `derive:rdfs` | str_key | GATE = "derive:rdfs" |  |  |  |  |


### tests/receipt_integrity_court.rs

| `CARGO_MANIFEST_DIR` | str_key | ROOT = "CARGO_MANIFEST_DIR" |  |  |  |  |

| `abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._/-` | str_key | PATH_CLASS = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._/-" |  |  |  |  |

| `receipts` | str_key | RECEIPTS_REL = "receipts" |  |  |  |  |

| `receipts/logs/` | str_key | LOG_PREFIX = "receipts/logs/" |  |  |  |  |


### tests/receipt_store.rs

| `0123456789abcdef0123456789abcdef01234567` | str_key | SUBJECT = "0123456789abcdef0123456789abcdef01234567" |  |  |  |  |


### tests/receipt_v2_court.rs

| `0bca2e430b2d2b856ae5b262031b0cadd1dd6ee40230ecb75d7ba3b0baf86662` | str_key | H_PRE = "0bca2e430b2d2b856ae5b262031b0cadd1dd6ee40230ecb75d7ba3b0baf86662" |  |  |  |  |

| `0ce59900a950fd16a9546ae14eed62b0b4ea63102b80812c00157a56c7a1d669` | str_key | H_TRIPLES = "0ce59900a950fd16a9546ae14eed62b0b4ea63102b80812c00157a56c7a1d669" |  |  |  |  |

| `48fc721fbbc172e0925fa27af1671de225ba927134802998b10a1568a188652b` | str_key | SETTLED_V1_INPUTS = "48fc721fbbc172e0925fa27af1671de225ba927134802998b10a1568a188652b" |  |  |  |  |

| `61054d964e2ecadcaac7d82d06fc1bd5f732a7595824376792a3f0970aae96c1` | str_key | V2_GOLDEN = "61054d964e2ecadcaac7d82d06fc1bd5f732a7595824376792a3f0970aae96c1" |  |  |  |  |

| `63bff90dee1c16beb2881e11664a1e8c0b6942abe39ba5726d9ce1f9708a8ffe` | str_key | SETTLED_V1_GOLDEN = "63bff90dee1c16beb2881e11664a1e8c0b6942abe39ba5726d9ce1f9708a8ffe" |  |  |  |  |

| `7a364272788ab0a7a56b8444e9f6f76b1f0f7d90d3e5725e3496a78e732d92fd` | str_key | H_QUERY = "7a364272788ab0a7a56b8444e9f6f76b1f0f7d90d3e5725e3496a78e732d92fd" |  |  |  |  |

| `9d79bfadd4c208bad3a87982bdd8f9d05991011f16b96de3cf1c5c33cf7c58df` | str_key | H_COLUMNAR = "9d79bfadd4c208bad3a87982bdd8f9d05991011f16b96de3cf1c5c33cf7c58df" |  |  |  |  |

| `b208929e36eafbe6511114fe40910c486b265eff351b4e7edc47f86476db6539` | str_key | H_POST = "b208929e36eafbe6511114fe40910c486b265eff351b4e7edc47f86476db6539" |  |  |  |  |

| `db2da6fc6ec068a661a7028f220524ea6fad4b3cbf7f72df336dd99fd44f7b0c` | str_key | EMPTY_GOLDEN = "db2da6fc6ec068a661a7028f220524ea6fad4b3cbf7f72df336dd99fd44f7b0c" |  |  |  |  |

| `f641cf9578e7ddf23bfbb9df055c15b79fe920b9d98910ca22be069deb20f774` | str_key | V1_GOLDEN = "f641cf9578e7ddf23bfbb9df055c15b79fe920b9d98910ca22be069deb20f774" |  |  |  |  |


### tests/receipt_v2_emission_court.rs

| `7ac44b45bd8c91a1ee1a1035ba43494bd08054231a4ea6dcb9175d8cdfc04593` | str_key | SENTINEL_PIN = "7ac44b45bd8c91a1ee1a1035ba43494bd08054231a4ea6dcb9175d8cdfc04593" |  |  |  |  |

| `d53d18c23212ea7b6300594bb89bce60218f6eff2b9d628b8cc42d3e79bbd5ab` | str_key | EMPTY_TRIPLES_PIN = "d53d18c23212ea7b6300594bb89bce60218f6eff2b9d628b8cc42d3e79bbd5ab" |  |  |  |  |

| `db2da6fc6ec068a661a7028f220524ea6fad4b3cbf7f72df336dd99fd44f7b0c` | str_key | EMPTY_COLUMNAR_PIN = "db2da6fc6ec068a661a7028f220524ea6fad4b3cbf7f72df336dd99fd44f7b0c" |  |  |  |  |


### tests/refusal_crosswalk_differential.rs

| `REFUSAL_VOCAB_PACK` | env_key | std::env::var("REFUSAL_VOCAB_PACK") |  |  |  |  |

| `malformed_frame` | str_key | VALVE_EXPECTED = "malformed_frame" |  |  |  |  |

| `ontology.ttl` | str_key | PINNED = "ontology.ttl" |  |  |  |  |

| `tests/fixtures/refusal_crosswalk` | str_key | FIXTURE_ROOT = "tests/fixtures/refusal_crosswalk" |  |  |  |  |


### tests/refusal_details.rs

| `SHAPES` | const | SHAPES: &str |  |  |  |  |

| `TWO_BAD` | const | TWO_BAD: &str |  |  |  |  |

| `<https://e/a> a <https://e/T> .` | str_key | TWO_BAD = "<https://e/a> a <https://e/T> ." |  |  |  |  |

| `@prefix sh: <http://www.w3.org/ns/shacl#> .` | str_key | SHAPES = "@prefix sh: <http://www.w3.org/ns/shacl#> ." |  |  |  |  |


### tests/registry_artifacts.rs

| `GRAPHLAW_REQUIRE_WASM_PIN` | env_key | std::env::var_os("GRAPHLAW_REQUIRE_WASM_PIN") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var("GRAPHLAW_WASM") |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `http://seanchatmangpt.github.io/packs/graphlaw-ash-capability#` | str_key | GAC = "http://seanchatmangpt.github.io/packs/graphlaw-ash-capability#" |  |  |  |  |


### tests/registry_dispatch.rs

| `<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .` | str_key | NT_DATA = "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> ." |  |  |  |  |


### tests/registry_limits.rs

| `max_request_bytes` | str_key | EXPECTED = "max_request_bytes" |  |  |  |  |


### tests/rehypothecation_gate_court.rs

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `rehy-01` | str_key | SESSION = "rehy-01" |  |  |  |  |

| `rehy-02` | str_key | SESSION = "rehy-02" |  |  |  |  |

| `rehy-03` | str_key | SESSION = "rehy-03" |  |  |  |  |


### tests/sa2a_run14.rs

| `CARGO_BIN_EXE_sa2a-run14-verify` | str_key | BIN = "CARGO_BIN_EXE_sa2a-run14-verify" |  |  |  |  |


### tests/shacl_boundary_cliff_court.rs

| `../ontologies/sql-ast.ttl` | str_key | CANONICAL_SHAPES = "../ontologies/sql-ast.ttl" |  |  |  |  |

| `@prefix sh: <http://www.w3.org/ns/shacl#> .` | str_key | PREFIX = "@prefix sh: <http://www.w3.org/ns/shacl#> ." |  |  |  |  |

| `@prefix sql: <http://graphlaw.org/sql#> .` | str_key | AST_DOC = "@prefix sql: <http://graphlaw.org/sql#> ." |  |  |  |  |


### tests/sieve_court.rs

| `urn:s/` | str_key | SUBJECT = "urn:s/" |  |  |  |  |

| `urn:v/` | str_key | VOCAB = "urn:v/" |  |  |  |  |


### tests/sql_rdf_conformance.rs

| `CONFORMANCE_REPORT` | env_key | std::env::var("CONFORMANCE_REPORT") |  |  |  |  |

| `../docs/reference/sql-rdf-rec.md` | str_key | REC = "../docs/reference/sql-rdf-rec.md" |  |  |  |  |

| `../docs/refusals.md` | str_key | REFUSALS_MD = "../docs/refusals.md" |  |  |  |  |

| `../ontologies/sql-ast.ttl` | str_key | SQL_AST_TTL = "../ontologies/sql-ast.ttl" |  |  |  |  |

| `../src/sql_ast/refusals.rs` | str_key | REFUSALS_RS = "../src/sql_ast/refusals.rs" |  |  |  |  |

| `@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .` | str_key | B = "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> ." |  |  |  |  |

| `@prefix sql: <http://graphlaw.org/sql#> .` | str_key | A = "@prefix sql: <http://graphlaw.org/sql#> ." |  |  |  |  |

| `ALIASED_AGG_ARG` | str_key | PINNED_GAPS = "ALIASED_AGG_ARG" |  |  |  |  |

| `OPERATOR_TOKEN` | str_key | REGISTERED_ARMS = "OPERATOR_TOKEN" |  |  |  |  |

| `core_functions` | str_key | CERTIFIED_EXTENSION_RESULTS = "core_functions" |  |  |  |  |

| `§1` | str_key | CLAUSE_TABLE = "§1" |  |  |  |  |


### tests/strata_gate_differential.rs

| `hash` | str_key | GATES = "hash" |  |  |  |  |

| `tests/fixtures/strata_gate_differential` | str_key | FIXTURE_ROOT = "tests/fixtures/strata_gate_differential" |  |  |  |  |


### tests/tc_mega_antivacuity.rs

| `docs/jira/v26.10.22/mega/tc-mega-fibo-01` | str_key | MEGA = "docs/jira/v26.10.22/mega/tc-mega-fibo-01" |  |  |  |  |


### tests/temprun_framing.rs

| `cd3cde66a2a9a3440240e8ff005ca7258dfd995744f36b061102db522e33f30e` | str_key | PIN_BLAKE3 = "cd3cde66a2a9a3440240e8ff005ca7258dfd995744f36b061102db522e33f30e" |  |  |  |  |

| `https://strata.chatman.ai/ontology#` | str_key | ST = "https://strata.chatman.ai/ontology#" |  |  |  |  |

| `tests/fixtures/strata-temprun-ontology.ttl` | str_key | PINNED = "tests/fixtures/strata-temprun-ontology.ttl" |  |  |  |  |


### tests/tier_court.rs

| `@prefix : <https://e/> .` | str_key | N3_RULES = "@prefix : <https://e/> ." |  |  |  |  |


### tests/ubo_sanctions_court.rs

| `http://example.org/ubo#` | str_key | NS = "http://example.org/ubo#" |  |  |  |  |

| `http://graphlaw.org/sql#` | str_key | SQL = "http://graphlaw.org/sql#" |  |  |  |  |

| `http://www.w3.org/2001/XMLSchema#` | str_key | XSD = "http://www.w3.org/2001/XMLSchema#" |  |  |  |  |

| `ubo-01` | str_key | SESSION = "ubo-01" |  |  |  |  |

| `ubo-02` | str_key | SESSION = "ubo-02" |  |  |  |  |

| `ubo-03` | str_key | SESSION = "ubo-03" |  |  |  |  |


### tests/verify_cli.rs

| `0123456789abcdef0123456789abcdef01234567` | str_key | SUBJECT = "0123456789abcdef0123456789abcdef01234567" |  |  |  |  |


### tests/w3c_conformance.rs

| `rdf-n-triples/nt-syntax-bad-string-04.nt` | str_key | KNOWN_DIVERGENCES = "rdf-n-triples/nt-syntax-bad-string-04.nt" |  |  |  |  |

| `rdf-turtle/turtle-syntax-number-11.ttl` | str_key | KNOWN_POSITIVE_DIVERGENCES = "rdf-turtle/turtle-syntax-number-11.ttl" |  |  |  |  |


### tests/w3c_manifest.rs

| `ENTRIES` | const | ENTRIES: &[(Dialect, &str, Kind)] |  |  |  |  |

| `Dialect` | enum | Dialect { Turtle, NTriples, NQuads, Trig, RdfXml } |  |  |  |  |

| `Kind` | enum | Kind { Positive, Negative } |  |  |  |  |

| `rdf-turtle/turtle-syntax-file-01.ttl` | str_key | ENTRIES = "rdf-turtle/turtle-syntax-file-01.ttl" |  |  |  |  |


### tests/wasm_abi.rs

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `@prefix ex: <https://e/> . ex:s ex:p ex:o, ` | str_key | SAMPLE = "@prefix ex: <https://e/> . ex:s ex:p ex:o, " |  |  |  |  |


### tests/wasm_boundary_fuzz.rs

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `'; DROP TABLE cold_chain_events; --` | str_key | SQL_TOKENS = "'; DROP TABLE cold_chain_events; --" |  |  |  |  |

| `NotSemanticContent` | str_key | REFUSAL_KINDS = "NotSemanticContent" |  |  |  |  |


### tests/wasm_coldchain_e2e.rs

| `GWS10_DEBUG` | env_key | std::env::var("GWS10_DEBUG") |  |  |  |  |

| `BATCH-A1` | str_key | COLD1_QUARANTINED = "BATCH-A1" |  |  |  |  |

| `BATCH-R1` | str_key | COLD1_RELEASED = "BATCH-R1" |  |  |  |  |


### tests/wasm_duckdb_lifecycle_eh.rs

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `f149433c6e1a84ec95e961ac92b16b65b28e15d6fd8546beb1779d3b54a02e7e` | str_key | LEGACY_DIGEST = "f149433c6e1a84ec95e961ac92b16b65b28e15d6fd8546beb1779d3b54a02e7e" |  |  |  |  |

| `fibo_accounts` | str_key | FIBO_TABLE = "fibo_accounts" |  |  |  |  |

| `lifecycle-a` | str_key | SESSION = "lifecycle-a" |  |  |  |  |

| `lifecycle-b` | str_key | SESSION = "lifecycle-b" |  |  |  |  |


### tests/wasm_duckdb_op.rs

| `SELECT batch_id, SUM(cost) AS total_cost FROM telemetry WHERE (latency > $1) GROUP BY batch_id` | str_key | AGG_SQL = "SELECT batch_id, SUM(cost) AS total_cost FROM telemetry WHERE (latency > $1) GROUP BY batch_id" |  |  |  |  |

| `abi-court` | str_key | SESSION = "abi-court" |  |  |  |  |

| `batch_id,cost,latency` | str_key | TELEMETRY_HEADER = "batch_id,cost,latency" |  |  |  |  |

| `https://example.org/data` | str_key | SUBJECT = "https://example.org/data" |  |  |  |  |

| `https://example.org/queries` | str_key | Q_PREFIX = "https://example.org/queries" |  |  |  |  |

| `https://example.org/queries/q1` | str_key | Q1 = "https://example.org/queries/q1" |  |  |  |  |

| `https://example.org/vocab` | str_key | VOCAB = "https://example.org/vocab" |  |  |  |  |


### tests/wasm_enclosure_court.rs

| `CARGO_TARGET_DIR` | env_key | std::env::var_os("CARGO_TARGET_DIR") |  |  |  |  |

| `GRAPHLAW_RUN_EH` | env_key | std::env::var_os("GRAPHLAW_RUN_EH") |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `debug/graphlaw-run-eh` | str_key | EH_BIN_REL = "debug/graphlaw-run-eh" |  |  |  |  |


### tests/wasm_env_isolation_court.rs

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `CARGO_TARGET_DIR` | env_key | std::env::var_os("CARGO_TARGET_DIR") |  |  |  |  |

| `GRAPHLAW_RUN_EH` | env_key | std::env::var_os("GRAPHLAW_RUN_EH") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/` | str_key | ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/" |  |  |  |  |

| `SECRET_TEST_TOKEN` | str_key | TOKEN_NAME = "SECRET_TEST_TOKEN" |  |  |  |  |

| `gl-env-court-canary-6f19e2ac` | str_key | TOKEN_VALUE = "gl-env-court-canary-6f19e2ac" |  |  |  |  |


### tests/wasm_fibo_egress_eh.rs

| `GRAPHLAW_DIAG_NO_EH` | env_key | std::env::var("GRAPHLAW_DIAG_NO_EH") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `FD_ALLOCATE` | str_key | RIGHTS = "FD_ALLOCATE" |  |  |  |  |

| `fibo-08` | str_key | SESSION = "fibo-08" |  |  |  |  |

| `http://example.org/fibo#stmt1` | str_key | FIBO_SUBJECT = "http://example.org/fibo#stmt1" |  |  |  |  |


### tests/wasm_fibo_fixtures_eh.rs

| `GRAPHLAW_DIAG_NO_EH` | env_key | std::env::var("GRAPHLAW_DIAG_NO_EH") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `FD_ALLOCATE` | str_key | RIGHTS = "FD_ALLOCATE" |  |  |  |  |

| `fibo-01` | str_key | SESSION = "fibo-01" |  |  |  |  |

| `fibo-02` | str_key | SESSION = "fibo-02" |  |  |  |  |

| `fibo-03` | str_key | SESSION = "fibo-03" |  |  |  |  |

| `http://example.org/fibo#stmt1` | str_key | FIBO_SUBJECT = "http://example.org/fibo#stmt1" |  |  |  |  |


### tests/wasm_frame_bound_court.rs

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `CARGO_TARGET_DIR` | env_key | std::env::var_os("CARGO_TARGET_DIR") |  |  |  |  |


### tests/wasm_memory_wall.rs

| `@prefix ex: <https://e/> . ex:s ex:p ex:o .` | str_key | SAMPLE = "@prefix ex: <https://e/> . ex:s ex:p ex:o ." |  |  |  |  |


### tests/wasm_native_differential.rs

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `_:p <http://example.org/knows> _:q .` | str_key | BNQ_PQ = "_:p <http://example.org/knows> _:q ." |  |  |  |  |

| `_:x <http://example.org/knows> _:y .` | str_key | BNQ_AB = "_:x <http://example.org/knows> _:y ." |  |  |  |  |

| `packs/self-monitoring-pack/fixtures/session-real-broad-topic.ttl` | str_key | HOOK_SESSION = "packs/self-monitoring-pack/fixtures/session-real-broad-topic.ttl" |  |  |  |  |

| `packs/self-monitoring-pack/hook.ttl` | str_key | HOOK_PACK = "packs/self-monitoring-pack/hook.ttl" |  |  |  |  |


### tests/wasm_size_court.rs

| `GRAPHLAW_WASM` | env_key | std::env::var("GRAPHLAW_WASM") |  |  |  |  |


### tests/wire_completeness_court.rs

| `Ambiguous` | str_key | UNWITNESSED_KINDS = "Ambiguous" |  |  |  |  |

| `NotSemanticContent` | str_key | REFUSAL_KINDS = "NotSemanticContent" |  |  |  |  |

| `RDF 1.2 / codecs / storage IR` | str_key | CAPABILITY_OPS = "RDF 1.2 / codecs / storage IR" |  |  |  |  |

| `ResourceLimit` | str_key | REFUSAL_CODES = "ResourceLimit" |  |  |  |  |

| `https://graphlaw.dev/wire-court#` | str_key | NS = "https://graphlaw.dev/wire-court#" |  |  |  |  |


### wasm/src/abi_meta.rs

| `ABI_VERSION` | const | ABI_VERSION: u32 |  |  |  |  |

| `CRATE_NAME` | const | CRATE_NAME: &str |  |  |  |  |

| `ERROR_CODES` | const | ERROR_CODES: &[&str] |  |  |  |  |

| `EXPORT_PREFIX` | const | EXPORT_PREFIX: &str |  |  |  |  |

| `MAX_JSON_DEPTH` | const | MAX_JSON_DEPTH: usize |  |  |  |  |

| `MAX_OUTSTANDING_BYTES` | const | MAX_OUTSTANDING_BYTES: usize |  |  |  |  |

| `MAX_REQUEST_BYTES` | const | MAX_REQUEST_BYTES: usize |  |  |  |  |

| `OPS` | const | OPS: &[&str] |  |  |  |  |

| `Refused` | str_key | ERROR_CODES = "Refused" |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `gl` | str_key | EXPORT_PREFIX = "gl" |  |  |  |  |

| `graphlaw-wasm` | str_key | CRATE_NAME = "graphlaw-wasm" |  |  |  |  |


### wasm/src/abi_meta.rs

| `ABI_VERSION` | const | ABI_VERSION: u32 |  |  |  |  |

| `CRATE_NAME` | const | CRATE_NAME: &str |  |  |  |  |

| `ERROR_CODES` | const | ERROR_CODES: &[&str] |  |  |  |  |

| `EXPORT_PREFIX` | const | EXPORT_PREFIX: &str |  |  |  |  |

| `MAX_JSON_DEPTH` | const | MAX_JSON_DEPTH: usize |  |  |  |  |

| `MAX_OUTSTANDING_BYTES` | const | MAX_OUTSTANDING_BYTES: usize |  |  |  |  |

| `MAX_REQUEST_BYTES` | const | MAX_REQUEST_BYTES: usize |  |  |  |  |

| `OPS` | const | OPS: &[&str] |  |  |  |  |

| `Refused` | str_key | ERROR_CODES = "Refused" |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `gl` | str_key | EXPORT_PREFIX = "gl" |  |  |  |  |

| `graphlaw-wasm` | str_key | CRATE_NAME = "graphlaw-wasm" |  |  |  |  |


### wasm/src/ffi.rs

| `alloc_buf` | function | alloc_buf(len: u32) -> *mut u8 |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) -> Vec<u8> |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `gl_alloc` | function | gl_alloc(len: u32) -> *mut u8 |  |  |  |  |

| `gl_call` | function | gl_call(ptr: *mut u8, len: u32) -> u64 |  |  |  |  |

| `gl_free` | function | gl_free(ptr: *mut u8, len: u32) |  |  |  |  |


### wasm/src/ffi.rs

| `alloc_buf` | function | alloc_buf(len: u32) -> *mut u8 |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) -> Vec<u8> |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `gl_alloc` | function | gl_alloc(len: u32) -> *mut u8 |  |  |  |  |

| `gl_call` | function | gl_call(ptr: *mut u8, len: u32) -> u64 |  |  |  |  |

| `gl_free` | function | gl_free(ptr: *mut u8, len: u32) |  |  |  |  |


### wasm/src/lib.rs

| `__cxa_atexit` | function | __cxa_atexit( _func: unsafe extern "C" fn(*mut c_void), _arg: *mut c_void, _dso: *mut c_void, ) -> i32 |  |  |  |  |

| `__cxa_finalize` | function | __cxa_finalize(_d: *mut c_void) |  |  |  |  |

| `__funcs_on_exit` | function | __funcs_on_exit() |  |  |  |  |

| `atexit` | function | atexit(_func: unsafe extern "C" fn()) -> i32 |  |  |  |  |

| `graphlaw_wasm_register_static_extensions` | function | graphlaw_wasm_register_static_extensions() -> i32 |  |  |  |  |

| `C` | str_key | REGISTER = "C" |  |  |  |  |


### wasm/src/lib.rs

| `__cxa_atexit` | function | __cxa_atexit( _func: unsafe extern "C" fn(*mut c_void), _arg: *mut c_void, _dso: *mut c_void, ) -> i32 |  |  |  |  |

| `__cxa_finalize` | function | __cxa_finalize(_d: *mut c_void) |  |  |  |  |

| `__funcs_on_exit` | function | __funcs_on_exit() |  |  |  |  |

| `atexit` | function | atexit(_func: unsafe extern "C" fn()) -> i32 |  |  |  |  |

| `graphlaw_wasm_register_static_extensions` | function | graphlaw_wasm_register_static_extensions() -> i32 |  |  |  |  |

| `C` | str_key | REGISTER = "C" |  |  |  |  |



<!-- AGENT-FORBIDDEN-END -->

## Signature/type/default/errors table

<!-- RIGID table: header order is fixed; rows come only from the query. -->

| Item | Type | Signature | Params | Defaults | Errors | Invariants |
|------|------|-----------|--------|----------|--------|------------|

| `build` | script | npm run gen && node scripts/build.mjs && tsc -p tsconfig.build.json && node scripts/assemble.mjs |  |  |  |  |

| `gen` | script | node scripts/gen-types.mjs |  |  |  |  |

| `lint:pkg` | script | publint && attw --pack . |  |  |  |  |

| `sync-version` | script | node scripts/sync-version.mjs |  |  |  |  |

| `test` | script | vitest run |  |  |  |  |

| `test:browser` | script | vitest run --config vitest.browser.config.ts |  |  |  |  |

| `typecheck` | script | tsc -p tsconfig.json --noEmit && tsc -p tsconfig.test.json --noEmit |  |  |  |  |

| `admit` | function | admit/2 |  |  |  |  |

| `handle_call` | function | handle_call/3 |  |  |  |  |

| `init` | function | init/1 |  |  |  |  |

| `start_link` | function | start_link/1 |  |  |  |  |

| `stop` | function | stop/1 |  |  |  |  |

| `abi_not_wired` | str_key | {:error, :abi_not_wired} |  |  |  |  |

| `instance` | str_key | instance: instance |  |  |  |  |

| `module` | str_key | module: module |  |  |  |  |

| `name` | str_key | name: __MODULE__ |  |  |  |  |

| `store` | str_key | store: store |  |  |  |  |

| `wasm_load_failed` | str_key | {:error, {:wasm_load_failed, path, :enoent}} |  |  |  |  |

| `wasm_path` | str_key | wasm_path: wasm_path |  |  |  |  |

| `start` | function | start/2 |  |  |  |  |

| `name` | str_key | name: GraphlawEx.Supervisor |  |  |  |  |

| `strategy` | str_key | strategy: :one_for_one |  |  |  |  |

| `../ontologies/sql-ast.ttl` | str_key | CANONICAL_SHAPES = "../ontologies/sql-ast.ttl" |  |  |  |  |

| `@prefix ex: <http://example.org/> .` | str_key | DATA_TTL = "@prefix ex: <http://example.org/> ." |  |  |  |  |

| `@prefix sh: <http://www.w3.org/ns/shacl#> .` | str_key | PREFIX = "@prefix sh: <http://www.w3.org/ns/shacl#> ." |  |  |  |  |

| `urn:s/` | str_key | SUBJECT = "urn:s/" |  |  |  |  |

| `urn:v/` | str_key | VOCAB = "urn:v/" |  |  |  |  |

| `@prefix sql: <http://graphlaw.org/sql#> .` | str_key | SELECT_TURTLE = "@prefix sql: <http://graphlaw.org/sql#> ." |  |  |  |  |

| `CREATE TABLE t(id INTEGER, name VARCHAR); ` | str_key | SEED_SQL = "CREATE TABLE t(id INTEGER, name VARCHAR); " |  |  |  |  |

| `SB_DEBUG` | env_key | std::env::var_os("SB_DEBUG") |  |  |  |  |

| `@prefix : <http://example.org/sb#> .
@prefix math: <http://www.w3.org/2000/10/swap/math#> .
:trade_47 a :ProposedTrade .
:trade_47 :notional 25000000 ;
          :book :emea_deriv .
{ ?t a :ProposedTrade ;
     :notional ?n .
  ?n math:greaterThan 10000000 .
  ?t :book :emea_deriv }
  => { ?t :violates :Rule15c3_1 ; :requires :pretrade_review } .
{ ?t a :ProposedTrade ; :violates :Rule15c3_1 }
  => { :desk :halt :trade_47 } .
` | str_key | PROGRAM = "@prefix : <http://example.org/sb#> .
@prefix math: <http://www.w3.org/2000/10/swap/math#> .
:trade_47 a :ProposedTrade .
:trade_47 :notional 25000000 ;
          :book :emea_deriv .
{ ?t a :ProposedTrade ;
     :notional ?n .
  ?n math:greaterThan 10000000 .
  ?t :book :emea_deriv }
  => { ?t :violates :Rule15c3_1 ; :requires :pretrade_review } .
{ ?t a :ProposedTrade ; :violates :Rule15c3_1 }
  => { :desk :halt :trade_47 } .
" |  |  |  |  |

| `@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/sb#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
ex:TradeShape a sh:NodeShape ;
  sh:targetClass ex:Order ;
  sh:closed true ;
  sh:property [
    sh:path rdf:type ; sh:minCount 1 ; sh:maxCount 1 ; sh:hasValue ex:Order
  ] ;
  sh:property [
    sh:path ex:id ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] ;
  sh:property [
    sh:path ex:acct ; sh:minCount 1 ; sh:maxCount 2 ; sh:nodeKind sh:IRI
  ] ;
  sh:property [
    sh:path ex:qty ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] .
` | str_key | SHAPES_TTL = "@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/sb#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
ex:TradeShape a sh:NodeShape ;
  sh:targetClass ex:Order ;
  sh:closed true ;
  sh:property [
    sh:path rdf:type ; sh:minCount 1 ; sh:maxCount 1 ; sh:hasValue ex:Order
  ] ;
  sh:property [
    sh:path ex:id ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] ;
  sh:property [
    sh:path ex:acct ; sh:minCount 1 ; sh:maxCount 2 ; sh:nodeKind sh:IRI
  ] ;
  sh:property [
    sh:path ex:qty ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] .
" |  |  |  |  |

| `TC_HOT_FULL` | env_key | std::env::var("TC_HOT_FULL") |  |  |  |  |

| `TC_HOT_INJECT_OUTLIER_NS` | env_key | std::env::var("TC_HOT_INJECT_OUTLIER_NS") |  |  |  |  |

| `TC_HOT_SPC_EXPORT` | env_key | std::env::var("TC_HOT_SPC_EXPORT") |  |  |  |  |

| `TC_HOT_CORPUS_PATH` | env_key | std::env::var("TC_HOT_CORPUS_PATH") |  |  |  |  |

| `TC_HOT_FULL` | env_key | std::env::var("TC_HOT_FULL") |  |  |  |  |

| `thread_affinity=unavailable_stable_rust_macos (core_affinity crate not added)` | str_key | ISOLATION_DISCLOSURE = "thread_affinity=unavailable_stable_rust_macos (core_affinity crate not added)" |  |  |  |  |

| `p` | str_key | NAMES = "p" |  |  |  |  |

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `3.0.1` | str_key | PURRDF = "3.0.1" |  |  |  |  |

| `purrdf-n3/3.0.1` | str_key | N3_ENGINE = "purrdf-n3/3.0.1" |  |  |  |  |

| `t` | str_key | SUBJ = "t" |  |  |  |  |

| `true` | str_key | TRUE_OBJ = "true" |  |  |  |  |

| `GL_DEBUG` | env_key | std::env::var_os("GL_DEBUG") |  |  |  |  |

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `3.0.1` | str_key | PURRDF = "3.0.1" |  |  |  |  |

| `purrdf-n3/3.0.1` | str_key | N3_ENGINE = "purrdf-n3/3.0.1" |  |  |  |  |

| `0123456789abcdef0123456789abcdef01234567` | str_key | SUBJECT = "0123456789abcdef0123456789abcdef01234567" |  |  |  |  |

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `3.0.1` | str_key | PURRDF = "3.0.1" |  |  |  |  |

| `purrdf-n3/3.0.1` | str_key | N3_ENGINE = "purrdf-n3/3.0.1" |  |  |  |  |

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `f` | env_key | var("f") |  |  |  |  |

| `t` | env_key | var("t") |  |  |  |  |

| `3.0.1` | str_key | PURRDF = "3.0.1" |  |  |  |  |

| `CARGO_PKG_VERSION` | str_key | GL_VERSION = "CARGO_PKG_VERSION" |  |  |  |  |

| `event_calculus-paper-narrative` | str_key | SCENARIOS = "event_calculus-paper-narrative" |  |  |  |  |

| `event_calculus_vectors` | str_key | GENERATOR = "event_calculus_vectors" |  |  |  |  |

| `purrdf-n3/3.0.1` | str_key | N3_ENGINE = "purrdf-n3/3.0.1" |  |  |  |  |

| `urn:ec:clipped` | str_key | P_CLIPPED = "urn:ec:clipped" |  |  |  |  |

| `urn:ec:comp` | str_key | P_COMP = "urn:ec:comp" |  |  |  |  |

| `urn:ec:happens` | str_key | P_HAPPENS = "urn:ec:happens" |  |  |  |  |

| `urn:ec:holds` | str_key | P_HOLDS = "urn:ec:holds" |  |  |  |  |

| `urn:ec:initiated` | str_key | P_INITIATED = "urn:ec:initiated" |  |  |  |  |

| `urn:ec:initiates` | str_key | P_INITIATES = "urn:ec:initiates" |  |  |  |  |

| `urn:ec:succ` | str_key | P_SUCC = "urn:ec:succ" |  |  |  |  |

| `urn:ec:terminated` | str_key | P_TERMINATED = "urn:ec:terminated" |  |  |  |  |

| `urn:ec:terminates` | str_key | P_TERMINATES = "urn:ec:terminates" |  |  |  |  |

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw` | str_key | OUT_DIR = "/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw" |  |  |  |  |

| `https://np.graphlaw.dev/` | str_key | NS = "https://np.graphlaw.dev/" |  |  |  |  |

| `GL_VECDIR` | env_key | std::env::var_os("GL_VECDIR") |  |  |  |  |

| `3.0.1` | str_key | PURRDF = "3.0.1" |  |  |  |  |

| `plan_replay_oracle_gen` | str_key | GENERATOR = "plan_replay_oracle_gen" |  |  |  |  |

| `purrdf-n3/3.0.1` | str_key | N3_ENGINE = "purrdf-n3/3.0.1" |  |  |  |  |

| `SQL_EXTENSIONS` | const | SQL_EXTENSIONS: &[(&str, &str, &str, i64)] |  |  |  |  |

| `autocomplete` | str_key | SQL_EXTENSIONS = "autocomplete" |  |  |  |  |

| `SQL_STATEMENTS` | const | SQL_STATEMENTS: &[(&str, &str)] |  |  |  |  |

| `ExternalResourceStatement` | str_key | SQL_STATEMENTS = "ExternalResourceStatement" |  |  |  |  |

| `ABI_VERSION` | const | ABI_VERSION: u32 |  |  |  |  |

| `MAX_ATOMS_PER_FIELD` | const | MAX_ATOMS_PER_FIELD: usize |  |  |  |  |

| `MAX_DATALOG_FACTS` | const | MAX_DATALOG_FACTS: usize |  |  |  |  |

| `MAX_DATALOG_RULES` | const | MAX_DATALOG_RULES: usize |  |  |  |  |

| `MAX_JSON_DEPTH` | const | MAX_JSON_DEPTH: usize |  |  |  |  |

| `MAX_LAW_STEPS` | const | MAX_LAW_STEPS: usize |  |  |  |  |

| `MAX_OUTSTANDING_ALLOC_BYTES` | const | MAX_OUTSTANDING_ALLOC_BYTES: usize |  |  |  |  |

| `MAX_PLAN_ACTIONS` | const | MAX_PLAN_ACTIONS: usize |  |  |  |  |

| `MAX_POLICY_ENTRIES` | const | MAX_POLICY_ENTRIES: usize |  |  |  |  |

| `MAX_REQUEST_BYTES` | const | MAX_REQUEST_BYTES: usize |  |  |  |  |

| `call` | function | call(request: &[u8]) -> Vec<u8> |  |  |  |  |

| `call_json` | function | call_json(request: &Value) -> Value |  |  |  |  |

| `dialect_by_name` | function | dialect_by_name(name: &str) -> Result<Dialect, Refusal> |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) -> Vec<u8> |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() -> Vec<u8> |  |  |  |  |

| `unknown_buffer_response` | function | unknown_buffer_response() -> Vec<u8> |  |  |  |  |

| `Fail` | struct | Fail { pub(in crate::abi) refusal: Refusal, pub(in crate::abi) details: Option<Value> } |  |  |  |  |

| `ops::plan_from_json` | use | ops::plan_from_json |  |  |  |  |

| `plan_from_json` | function | plan_from_json(plan: &Value) -> Result<crate::plan::Plan, String> |  |  |  |  |

| `governance::plan_from_json` | use | governance::plan_from_json |  |  |  |  |

| `EXPORTS` | const | EXPORTS: &[Export] |  |  |  |  |

| `manifest_json_pretty` | function | manifest_json_pretty() -> String |  |  |  |  |

| `manifest_value` | function | manifest_value() -> Value |  |  |  |  |

| `write_manifest` | function | write_manifest(path: &std::path::Path) -> std::io::Result<()> |  |  |  |  |

| `gl_alloc` | str_key | EXPORTS = "gl_alloc" |  |  |  |  |

| `instruction_id` | str_key | FRAME_FIELDS = "instruction_id" |  |  |  |  |

| `Export` | struct | Export { pub name: &'static str, pub params: &'static [&'static str], pub result: &'static str, pub summary: &'static str } |  |  |  |  |

| `GLAW_RECEIPT_V2` | const | GLAW_RECEIPT_V2: &[u8; 15] |  |  |  |  |

| `AttestError` | enum | AttestError { BadSignature, UnknownKey, Malformed(String) } |  |  |  |  |

| `canon_arrow_empty_v1` | function | canon_arrow_empty_v1() -> [u8; 32] |  |  |  |  |

| `from_bytes` | function | from_bytes(bytes: &[u8; 32]) -> Result<Self, AttestError> |  |  |  |  |

| `from_hex` | function | from_hex(hex: &str) -> Result<Self, AttestError> |  |  |  |  |

| `from_json` | function | from_json(text: &str) -> Result<Self, AttestError> |  |  |  |  |

| `from_seed` | function | from_seed(seed: [u8; 32]) -> Self |  |  |  |  |

| `from_seed_hex` | function | from_seed_hex(hex: &str) -> Result<Self, AttestError> |  |  |  |  |

| `get` | function | get(&self, key_id: &str) -> Option<&VerifyingKey> |  |  |  |  |

| `hex_decode` | function | hex_decode(s: &str) -> Option<Vec<u8>> |  |  |  |  |

| `hex_encode` | function | hex_encode(bytes: &[u8]) -> String |  |  |  |  |

| `insert` | function | insert(&mut self, key: VerifyingKey) |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `key_id` | function | key_id(&self) -> String |  |  |  |  |

| `lease_digest` | function | lease_digest(payload: &[u8]) -> String |  |  |  |  |

| `lease_payload` | function | lease_payload(l: &Lease) -> String |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `receipt_payload` | function | receipt_payload(f: &ReceiptFields<'_>) -> String |  |  |  |  |

| `receipt_v1` | function | receipt_v1( h_query: &[u8; 32], h_pre: &[u8; 32], h_post: &[u8; 32], h_triples: &[u8; 32], epoch: u64, ) -> [u8; 32] |  |  |  |  |

| `receipt_v2` | function | receipt_v2( h_query: &[u8; 32], h_pre: &[u8; 32], h_post: &[u8; 32], h_triples: &[u8; 32], h_columnar: &[u8; 32], epoch: u64, ) -> [u8; 32] |  |  |  |  |

| `sign_bytes` | function | sign_bytes(key: &SigningKey, payload: &[u8]) -> Attestation |  |  |  |  |

| `sign_lease` | function | sign_lease(key: &SigningKey, lease: Lease) -> SignedLease |  |  |  |  |

| `sign_receipt` | function | sign_receipt(key: &SigningKey, receipt: &Receipt) -> Attestation |  |  |  |  |

| `to_hex` | function | to_hex(&self) -> String |  |  |  |  |

| `to_json` | function | to_json(&self) -> String |  |  |  |  |

| `verify_bytes` | function | verify_bytes( payload: &[u8], att: &Attestation, trusted: &TrustedKeys, ) -> Result<(), AttestError> |  |  |  |  |

| `verify_lease` | function | verify_lease(signed: &SignedLease, trusted: &TrustedKeys) -> Result<(), AttestError> |  |  |  |  |

| `verify_receipt` | function | verify_receipt( receipt: &Receipt, att: &Attestation, trusted: &TrustedKeys, ) -> Result<(), AttestError> |  |  |  |  |

| `verifying_key` | function | verifying_key(&self) -> VerifyingKey |  |  |  |  |

| `GLAW_RECEIPT_V2` | str_key | GLAW_RECEIPT_V2 = "GLAW_RECEIPT_V2" |  |  |  |  |

| `Attestation` | struct | Attestation { pub payload_blake3: String, pub key_id: String, pub signature: String } |  |  |  |  |

| `ReceiptFields` | struct | ReceiptFields { pub parent: &'a str, pub child: &'a str, pub step: &'a str, pub authority: &'a str, pub revision: &'a str, pub added: u64, pub lease_id: Option<&'a str>, pub plan_blake3: Option<&'a str>, pub subject_blake3: Option<&'a str> } |  |  |  |  |

| `SigningKey` | struct | SigningKey { DalekSigning } |  |  |  |  |

| `TrustedKeys` | struct | TrustedKeys { BTreeMap<String, VerifyingKey> } |  |  |  |  |

| `VerifyingKey` | struct | VerifyingKey { DalekVerifying } |  |  |  |  |

| `WASM4PM_FIXTURES` | env_key | std::env::var("WASM4PM_FIXTURES") |  |  |  |  |

| `https://graphlaw.dev/dl#` | str_key | DL = "https://graphlaw.dev/dl#" |  |  |  |  |

| `graphlaw-attest — sign or verify an arbitrary file with Ed25519 attestations.

Usage:
  graphlaw-attest sign   --key-seed-hex <hex> --in <file> [--out <attestation.json>]
  graphlaw-attest verify --in <file> --attestation <json> --trusted-key <hex> [...]
  graphlaw-attest pubkey --key-seed-hex <hex>

Commands:
  sign    Sign the file; writes (or prints) the canonical attestation JSON
  verify  Re-read the file and attestation; prints `VERIFIED` or `REFUSED: <error>`
  pubkey  Print the Ed25519 public key derived from the 32-byte seed

Flags:
  --key-seed-hex <hex>  Signing seed, 64 hex chars (32 bytes)
  --in <file>           Payload file to sign or verify
  --out <file>          Where `sign` writes the attestation (default: stdout)
  --attestation <file>  Attestation JSON for `verify`
  --trusted-key <hex>   Ed25519 public key, 64 hex chars (32 bytes); repeatable

Exit codes: 0 ok, 1 refused (signature/verification failure), 2 usage or I/O error.` | str_key | HELP = "graphlaw-attest — sign or verify an arbitrary file with Ed25519 attestations.

Usage:
  graphlaw-attest sign   --key-seed-hex <hex> --in <file> [--out <attestation.json>]
  graphlaw-attest verify --in <file> --attestation <json> --trusted-key <hex> [...]
  graphlaw-attest pubkey --key-seed-hex <hex>

Commands:
  sign    Sign the file; writes (or prints) the canonical attestation JSON
  verify  Re-read the file and attestation; prints `VERIFIED` or `REFUSED: <error>`
  pubkey  Print the Ed25519 public key derived from the 32-byte seed

Flags:
  --key-seed-hex <hex>  Signing seed, 64 hex chars (32 bytes)
  --in <file>           Payload file to sign or verify
  --out <file>          Where `sign` writes the attestation (default: stdout)
  --attestation <file>  Attestation JSON for `verify`
  --trusted-key <hex>   Ed25519 public key, 64 hex chars (32 bytes); repeatable

Exit codes: 0 ok, 1 refused (signature/verification failure), 2 usage or I/O error." |  |  |  |  |

| `# receipt-surface: blake3 of the canonical graphlaw.receipt-surface/1 document` | str_key | ARTIFACTS_TRAILING_COMMENTS = "# receipt-surface: blake3 of the canonical graphlaw.receipt-surface/1 document" |  |  |  |  |

| `# wasm row: never re-pinned by graphlaw-registry; re-pinned by the wasm rebuild flow.` | str_key | WASM_ROW_NOTE = "# wasm row: never re-pinned by graphlaw-registry; re-pinned by the wasm rebuild flow." |  |  |  |  |

| `ARTIFACTS.blake3` | str_key | ARTIFACTS_FILE = "ARTIFACTS.blake3" |  |  |  |  |

| `graphlaw-registry — emit or check the capability registry files.

Usage:
  graphlaw-registry --write | --check | --print-json | --print-ttl | --print-digests

Modes (exactly one per invocation):
  --write          Write registry/capability-registry.json and .ttl under the crate
                   root and re-pin the ARTIFACTS.blake3 rows this bin owns
                   (`registry`, `surface`, `receipt-surface`); the `wasm` row is
                   never touched here — it is re-pinned by the wasm rebuild flow
  --check          Compare the committed files against live output; exit 1 with a
                   first-difference summary on drift
  --print-json     Print the capability registry JSON to stdout
  --print-ttl      Print the capability registry Turtle to stdout
  --print-digests  Print the live surface/registry BLAKE3 digests

Exit codes: 0 ok, 1 drift (`--check`), 2 usage or I/O error.` | str_key | HELP = "graphlaw-registry — emit or check the capability registry files.

Usage:
  graphlaw-registry --write | --check | --print-json | --print-ttl | --print-digests

Modes (exactly one per invocation):
  --write          Write registry/capability-registry.json and .ttl under the crate
                   root and re-pin the ARTIFACTS.blake3 rows this bin owns
                   (`registry`, `surface`, `receipt-surface`); the `wasm` row is
                   never touched here — it is re-pinned by the wasm rebuild flow
  --check          Compare the committed files against live output; exit 1 with a
                   first-difference summary on drift
  --print-json     Print the capability registry JSON to stdout
  --print-ttl      Print the capability registry Turtle to stdout
  --print-digests  Print the live surface/registry BLAKE3 digests

Exit codes: 0 ok, 1 drift (`--check`), 2 usage or I/O error." |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `graphlaw-verify — offline third-party replay + attestation check of an admitted plan.

Usage:
  graphlaw-verify --start <state.nq|.nt|.ttl|.trig> --plan <plan.json> --receipts <dir>
                  [--subject <key>] --trusted-key <hex> [--trusted-key <hex>...]

Flags:
  --start <path>        Start state (dialect chosen by extension; default N-Quads)
  --plan <path>         Plan JSON to replay via Plan::admit
  --receipts <dir>      Receipt store root; defaults subject discovery to its only
                        subdirectory unless --subject is given
  --subject <key>       Subject key within the receipt store
  --trusted-key <hex>   Ed25519 public key, 64 hex chars (32 bytes); repeatable

Output: `ADMITTED` or `REFUSED: <code>: <detail>` on stdout.
Exit codes: 0 admitted, 1 refused, 2 usage or I/O error.` | str_key | HELP = "graphlaw-verify — offline third-party replay + attestation check of an admitted plan.

Usage:
  graphlaw-verify --start <state.nq|.nt|.ttl|.trig> --plan <plan.json> --receipts <dir>
                  [--subject <key>] --trusted-key <hex> [--trusted-key <hex>...]

Flags:
  --start <path>        Start state (dialect chosen by extension; default N-Quads)
  --plan <path>         Plan JSON to replay via Plan::admit
  --receipts <dir>      Receipt store root; defaults subject discovery to its only
                        subdirectory unless --subject is given
  --subject <key>       Subject key within the receipt store
  --trusted-key <hex>   Ed25519 public key, 64 hex chars (32 bytes); repeatable

Output: `ADMITTED` or `REFUSED: <code>: <detail>` on stdout.
Exit codes: 0 admitted, 1 refused, 2 usage or I/O error." |  |  |  |  |

| `https://chatmangpt.com/sa2a#` | str_key | V2 = "https://chatmangpt.com/sa2a#" |  |  |  |  |

| `https://sa2a.dev/ontology#` | str_key | V1 = "https://sa2a.dev/ontology#" |  |  |  |  |

| `--in-ttl` | str_key | VALUE_FLAGS = "--in-ttl" |  |  |  |  |

| `smon-broaden-topic — disclosed counterfactual post-processor over an already-emitted smon: graph.
Prints a full audit of every triple it changes.

Usage:
  smon-broaden-topic --in-ttl IN.ttl --out-ttl OUT.ttl [--canonical-topic T]

Flags:
  --in-ttl <path>         Input Turtle graph (required)
  --out-ttl <path>        Output Turtle graph (required)
  --canonical-topic <t>   Topic to collapse GroundingQuestion topics to
                          (default: e2e-capability-grounding)

Exit codes: 0 ok, 1 usage, I/O or parse error.` | str_key | HELP = "smon-broaden-topic — disclosed counterfactual post-processor over an already-emitted smon: graph.
Prints a full audit of every triple it changes.

Usage:
  smon-broaden-topic --in-ttl IN.ttl --out-ttl OUT.ttl [--canonical-topic T]

Flags:
  --in-ttl <path>         Input Turtle graph (required)
  --out-ttl <path>        Output Turtle graph (required)
  --canonical-topic <t>   Topic to collapse GroundingQuestion topics to
                          (default: e2e-capability-grounding)

Exit codes: 0 ok, 1 usage, I/O or parse error." |  |  |  |  |

| `--assistant-only` | str_key | SWITCHES = "--assistant-only" |  |  |  |  |

| `--transcript` | str_key | VALUE_FLAGS = "--transcript" |  |  |  |  |

| `smon-transcript-to-turtle — emit smon: Turtle facts from a Claude Code session transcript (.jsonl).

Usage:
  smon-transcript-to-turtle --transcript T.jsonl --out OUT.ttl
      [--session-id ID] [--base IRI] [--assistant-only] [--topic-filter S]

Flags:
  --transcript <path>  Session transcript in JSONL format (required)
  --out <path>         Turtle output file (required)
  --session-id <id>    Session id (default: transcript file stem)
  --base <iri>         Base IRI for emitted subjects
                       (default: .../sessions/<session-id>)
  --assistant-only     Switch: keep only assistant turns
  --topic-filter <s>   Also print turns whose grounding topic contains <s>

Exit codes: 0 ok, 1 usage, I/O or parse error.` | str_key | HELP = "smon-transcript-to-turtle — emit smon: Turtle facts from a Claude Code session transcript (.jsonl).

Usage:
  smon-transcript-to-turtle --transcript T.jsonl --out OUT.ttl
      [--session-id ID] [--base IRI] [--assistant-only] [--topic-filter S]

Flags:
  --transcript <path>  Session transcript in JSONL format (required)
  --out <path>         Turtle output file (required)
  --session-id <id>    Session id (default: transcript file stem)
  --base <iri>         Base IRI for emitted subjects
                       (default: .../sessions/<session-id>)
  --assistant-only     Switch: keep only assistant turns
  --topic-filter <s>   Also print turns whose grounding topic contains <s>

Exit codes: 0 ok, 1 usage, I/O or parse error." |  |  |  |  |

| `Dialect` | enum | Dialect { Turtle, TriG, NTriples, NQuads, RdfXml, JsonLd, YamlLd, TriX, HexTuples, N3, ShExC, ShExJ, Sparql, SqlRdf } |  |  |  |  |

| `Engine` | enum | Engine { PurRdf, FrozenDuckdb } |  |  |  |  |

| `admit_bytes` | function | admit_bytes( bytes: &[u8], hint: Option<&str>, base: Option<&str>, ) -> Result<LawState, Refusal> |  |  |  |  |

| `check` | function | check(bytes: &[u8], dialect: Dialect, base: Option<&str>) -> Result<(), Refusal> |  |  |  |  |

| `check_typed` | function | check_typed( bytes: &[u8], dialect: Dialect, base: Option<&str>, ) -> Result<(), Box<ParseRefusal>> |  |  |  |  |

| `engine` | function | engine(self) -> Engine |  |  |  |  |

| `limit` | function | limit( dialect: Dialect, name: impl Into<String>, observed: u64, max: u64, message: impl Into<String>, ) -> Self |  |  |  |  |

| `media_type` | function | media_type(self) -> Option<&'static str> |  |  |  |  |

| `new` | function | new(dialect: Dialect, reason: ReasonClass, message: impl Into<String>) -> Self |  |  |  |  |

| `parse_rdf` | function | parse_rdf( bytes: &[u8], dialect: Dialect, base: Option<&str>, ) -> Result<Arc<purrdf::RdfDataset>, Refusal> |  |  |  |  |

| `parse_rdf_typed` | function | parse_rdf_typed( bytes: &[u8], dialect: Dialect, base: Option<&str>, ) -> Result<Arc<purrdf::RdfDataset>, Box<ParseRefusal>> |  |  |  |  |

| `ParseRefusal` | struct | ParseRefusal { pub dialect: Dialect, pub engine: Engine, pub reason: ReasonClass, pub line: Option<u32>, pub col: Option<u32>, pub byte_offset: Option<usize>, pub snippet: Option<String>, pub message: String } |  |  |  |  |

| `refusal::{ReasonClass, Refusal, RefusalKind, RefusalMessage}` | use | refusal::{ReasonClass, Refusal, RefusalKind, RefusalMessage} |  |  |  |  |

| `sniff::sniff` | use | sniff::sniff |  |  |  |  |

| `ReasonClass` | enum | ReasonClass { Syntax, UndefinedPrefix, BadIri, FormulaInRdfLayer, NotSemanticContent, AmbiguousDialect, RemoteContext, Limit { name: String, observed: u64, max: u64, } } |  |  |  |  |

| `RefusalKind` | enum | RefusalKind { NotSemanticContent, Ambiguous, EngineRejected, Unsupported, ResourceLimit, TierMismatch, DuckDb, BareSql, Fuse } |  |  |  |  |

| `RefusalMessage` | enum | RefusalMessage { Static(&'static str), Dynamic(String) } |  |  |  |  |

| `as_str` | function | as_str(&self) -> &str |  |  |  |  |

| `Refusal` | struct | Refusal { pub kind: RefusalKind, pub dialect: Option<Dialect>, pub engine: Option<Engine>, pub message: RefusalMessage } |  |  |  |  |

| `sniff` | function | sniff(bytes: &[u8], hint: Option<&str>) -> Result<Dialect, Refusal> |  |  |  |  |

| `http://graphlaw.org/sql#` | str_key | SQL_IRI = "http://graphlaw.org/sql#" |  |  |  |  |

| `sql:` | str_key | SQL = "sql:" |  |  |  |  |

| `DEFAULT_ROW_ENVELOPE` | const | DEFAULT_ROW_ENVELOPE: usize |  |  |  |  |

| `PARAM_BYTES_INLINE` | const | PARAM_BYTES_INLINE: usize |  |  |  |  |

| `ROW_ENVELOPE_LIMIT` | const | ROW_ENVELOPE_LIMIT: &str |  |  |  |  |

| `EvaluatorError` | enum | EvaluatorError { DuckDb(frozen_duckdb::DuckDBError), ConversionError { column: String, row: usize, details: String, }, ReceiptError(String), NoCanonicalGraph(String), BatchMemberFailed { index: usize, source: Box<EvaluatorError>, partial: BatchReceipt, }, RowEnvelopeExceeded { limit: &'static str, observed: usize, max: usize, } } |  |  |  |  |

| `ParamBytes` | enum | ParamBytes { Inline { buf: [u8; PARAM_BYTES_INLINE], len: usize, }, Digest([u8; 32]) } |  |  |  |  |

| `admitted_statement_hash` | function | admitted_statement_hash(canonical_graph: &[u8], params: &[SqlLiteral]) -> [u8; 32] |  |  |  |  |

| `canonical_ast_graph_bytes` | function | canonical_ast_graph_bytes(turtle: &str) -> Result<Vec<u8>, EvaluatorError> |  |  |  |  |

| `coarse_elapsed_ns` | function | coarse_elapsed_ns(start_ns: u64) -> u64 |  |  |  |  |

| `coarse_now_ns` | function | coarse_now_ns() -> u64 |  |  |  |  |

| `encode` | function | encode(params: &[SqlLiteral]) -> ParamBytes |  |  |  |  |

| `eval_admitted_ast` | function | eval_admitted_ast( &self, ast_turtle: &str, query_iri: &str, compiled: &CompiledQuery, subject_iri_prefix: &str, predicate_iri_prefix: &str, ) -> Result<ExecutionReceipt, EvaluatorError> |  |  |  |  |

| `eval_ast_document` | function | eval_ast_document( &self, query_iri: &str, document: &crate::sql_ast::CompiledDocument, subject_iri_prefix: &str, predicate_iri_prefix: &str, atomic: bool, ) -> Result<BatchReceipt, EvaluatorError> |  |  |  |  |

| `eval_ast_document_bounded` | function | eval_ast_document_bounded( &self, query_iri: &str, document: &crate::sql_ast::CompiledDocument, subject_iri_prefix: &str, predicate_iri_prefix: &str, atomic: bool, max_rows: usize, ) -> Result<BatchReceipt, EvaluatorError> |  |  |  |  |

| `eval_ast_query` | function | eval_ast_query( &self, query_iri: &str, compiled: &CompiledQuery, subject_iri_prefix: &str, predicate_iri_prefix: &str, ) -> Result<ExecutionReceipt, EvaluatorError> |  |  |  |  |

| `eval_ast_query_bounded` | function | eval_ast_query_bounded( &self, query_iri: &str, compiled: &CompiledQuery, subject_iri_prefix: &str, predicate_iri_prefix: &str, max_rows: usize, ) -> Result<ExecutionReceipt, EvaluatorError> |  |  |  |  |

| `explain_ast_query` | function | explain_ast_query(&self, compiled: &CompiledQuery) -> Result<String, EvaluatorError> |  |  |  |  |

| `fold` | function | fold(&self, hasher: &mut blake3::Hasher) |  |  |  |  |

| `fold_params_anchored` | function | fold_params_anchored(h_ast: &[u8; 32], params: &[SqlLiteral]) -> [u8; 32] |  |  |  |  |

| `from_connection` | function | from_connection(conn: frozen_duckdb::Connection) -> Self |  |  |  |  |

| `from_connection_with_cache` | function | from_connection_with_cache( conn: frozen_duckdb::Connection, cache: std::sync::Arc<crate::query_cache::QueryCache>, ) -> Self |  |  |  |  |

| `h_columnar_canon` | function | h_columnar_canon(triples: &[RdfTriple]) -> [u8; 32] |  |  |  |  |

| `h_triples_serterm` | function | h_triples_serterm(triples: &[RdfTriple]) -> [u8; 32] |  |  |  |  |

| `hash_param_buffer` | function | hash_param_buffer(hasher: &mut blake3::Hasher, params: &[SqlLiteral]) |  |  |  |  |

| `is_inline` | function | is_inline(&self) -> bool |  |  |  |  |

| `open_in_memory` | function | open_in_memory() -> Result<Self, EvaluatorError> |  |  |  |  |

| `open_in_memory_allowing_unsigned_extensions` | function | open_in_memory_allowing_unsigned_extensions() -> Result<Self, EvaluatorError> |  |  |  |  |

| `open_in_memory_with_cache` | function | open_in_memory_with_cache( cache: std::sync::Arc<crate::query_cache::QueryCache>, ) -> Result<Self, EvaluatorError> |  |  |  |  |

| `prepared_statement_cache_hits` | function | prepared_statement_cache_hits(&self) -> u64 |  |  |  |  |

| `prepared_statements` | function | prepared_statements(&self) -> usize |  |  |  |  |

| `raw_connection` | function | raw_connection(&self) -> &frozen_duckdb::Connection |  |  |  |  |

| `receipt_v2_state_sentinel` | function | receipt_v2_state_sentinel() -> &'static [u8; 32] |  |  |  |  |

| `register_csv_view` | function | register_csv_view( &self, view_name: &str, glob_pattern: &str, ) -> Result<(), EvaluatorError> |  |  |  |  |

| `register_parquet_view` | function | register_parquet_view( &self, view_name: &str, glob_pattern: &str, hive_partitioning: bool, ) -> Result<(), EvaluatorError> |  |  |  |  |

| `register_purrdf_parquet_views` | function | register_purrdf_parquet_views( &self, directory: &std::path::Path, ) -> Result<(), EvaluatorError> |  |  |  |  |

| `shared_cache` | function | shared_cache(&self) -> std::sync::Arc<crate::query_cache::QueryCache> |  |  |  |  |

| `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/` | str_key | TABLE = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/" |  |  |  |  |

| `duckdb_result_rows` | str_key | ROW_ENVELOPE_LIMIT = "duckdb_result_rows" |  |  |  |  |

| `seq_atomic_abort` | str_key | TABLE = "seq_atomic_abort" |  |  |  |  |

| `seq_atomic_ok` | str_key | TABLE = "seq_atomic_ok" |  |  |  |  |

| `seq_court_events` | str_key | TABLE = "seq_court_events" |  |  |  |  |

| `seq_court_partial` | str_key | TABLE = "seq_court_partial" |  |  |  |  |

| `seq_inverted` | str_key | TABLE = "seq_inverted" |  |  |  |  |

| `BatchReceipt` | struct | BatchReceipt { pub query_iri: String, pub doc_hash: [u8; 32], pub members: Vec<ExecutionReceipt>, pub rows_emitted: Vec<usize>, pub failed_at: Option<usize>, pub status: String, pub state_delta: u64 } |  |  |  |  |

| `ExecutionReceipt` | struct | ExecutionReceipt { pub query_iri: String, pub query_hash: [u8; 32], pub sql_statement: String, pub rows_emitted: usize, pub execution_ms: f64, pub result_triples: Vec<RdfTriple>, pub cached: bool, pub consult_ms: f64, pub artifact_digest_hex: Option<String>, pub artifact_bytes: Option<u64>, pub artifact_warning: Option<String>, pub columnar_digest_hex: Option<String>, pub receipt_v2_hex: Option<String> } |  |  |  |  |

| `GraphLawDuckDB` | struct | GraphLawDuckDB { conn: frozen_duckdb::Connection, cache: std::sync::Arc<crate::query_cache::QueryCache>, stmt_cache: StmtCacheSeam } |  |  |  |  |

| `RdfTriple` | struct | RdfTriple { pub subject: String, pub predicate: String, pub object: TermValue } |  |  |  |  |

| `DUCKDB_MEMORY_LIMIT` | const | DUCKDB_MEMORY_LIMIT: &str |  |  |  |  |

| `DUCKDB_THREADS` | const | DUCKDB_THREADS: u32 |  |  |  |  |

| `MAX_CHUNK_BYTES` | const | MAX_CHUNK_BYTES: usize |  |  |  |  |

| `MAX_CHUNK_ROWS` | const | MAX_CHUNK_ROWS: usize |  |  |  |  |

| `MAX_SESSIONS` | const | MAX_SESSIONS: usize |  |  |  |  |

| `GRAPHLAW_ATTACH_ROOTS` | env_key | std::env::var("GRAPHLAW_ATTACH_ROOTS") |  |  |  |  |

| `dispatch` | function | dispatch(v: &Value) -> Result<Value, Fail> |  |  |  |  |

| `leading_sql_token` | function | leading_sql_token(sql: &str) -> Option<String> |  |  |  |  |

| `32MiB` | str_key | DUCKDB_MEMORY_LIMIT = "32MiB" |  |  |  |  |

| `FLAGS_MASK` | const | FLAGS_MASK: u16 |  |  |  |  |

| `FLAG_BIG_ENDIAN` | const | FLAG_BIG_ENDIAN: u16 |  |  |  |  |

| `FLAG_HAS_NULLS` | const | FLAG_HAS_NULLS: u16 |  |  |  |  |

| `FLAG_STRICT_PROJECTION_ORDER` | const | FLAG_STRICT_PROJECTION_ORDER: u16 |  |  |  |  |

| `MAGIC` | const | MAGIC: [u8; 8] |  |  |  |  |

| `PAYLOAD_OFFSET` | const | PAYLOAD_OFFSET: u32 |  |  |  |  |

| `SCHEMA_VERSION` | const | SCHEMA_VERSION: u16 |  |  |  |  |

| `SIZE` | const | SIZE: usize |  |  |  |  |

| `ArrowEgressHeaderError` | enum | ArrowEgressHeaderError { BadMagic, BadSchemaVersion(u16), BadFlags(u16), BadPayloadOffset(u32), BadTotalLen { payload_byte_len: u64, total_byte_len: u64, }, ReservedNotZero } |  |  |  |  |

| `CANON_ARROW_EMPTY_V1` | function | CANON_ARROW_EMPTY_V1() -> &'static [u8; 32] |  |  |  |  |

| `canon_arrow_empty_v1` | function | canon_arrow_empty_v1() -> &'static [u8; 32] |  |  |  |  |

| `from_bytes` | function | from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self, ArrowEgressHeaderError> |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) -> [u8; Self::SIZE] |  |  |  |  |

| `GLAWARRW` | str_key | MAGIC = "GLAWARRW" |  |  |  |  |

| `ArrowEgressHeader` | struct | ArrowEgressHeader { pub magic: [u8; 8], pub schema_version: u16, pub flags: u16, pub batch_count: u32, pub payload_offset: u32, pub payload_byte_len: u64, pub total_byte_len: u64, pub triples_digest: [u8; 32], pub columnar_digest: [u8; 32], pub reserved: [u8; 24] } |  |  |  |  |

| `CanonArrowCell` | enum | CanonArrowCell { Null, Bool(bool), I8(i8), I16(i16), I32(i32), I64(i64), U8(u8), U16(u16), U32(u32), U64(u64), F32(f32), F64(f64), Dec128(i128), Str(String), Bytes(Vec<u8>), Date32(i32), Time64(CanonArrowTimeUnit, i64), Timestamp(CanonArrowTimeUnit, i64), Interval { months: i32, days: i32, nanos: i64, } } |  |  |  |  |

| `CanonArrowCollectorError` | enum | CanonArrowCollectorError { ColumnIndexOutOfRange { column: usize, columns: usize, }, CellTypeMismatch { column: usize, declared: CanonArrowType, cell: &'static str, }, RaggedRows { lengths: Vec<usize>, }, Overflow { what: &'static str, } } |  |  |  |  |

| `CanonArrowTimeUnit` | enum | CanonArrowTimeUnit { Second, Millisecond, Microsecond, Nanosecond } |  |  |  |  |

| `CanonArrowType` | enum | CanonArrowType { Null, Bool, Int8, Int16, Int32, Int64, UInt8, UInt16, UInt32, UInt64, Float32, Float64, Decimal128 { precision: u8, scale: i8, }, Utf8, Binary, Date32, Time64(CanonArrowTimeUnit), Timestamp(CanonArrowTimeUnit, Option<String>), IntervalMonthDayNano } |  |  |  |  |

| `columns` | function | columns(&self) -> usize |  |  |  |  |

| `new` | function | new(schema: Vec<CanonArrowField>) -> Self |  |  |  |  |

| `push` | function | push(&mut self, column: usize, cell: CanonArrowCell) -> Result<(), CanonArrowCollectorError> |  |  |  |  |

| `schema` | function | schema(&self) -> &[CanonArrowField] |  |  |  |  |

| `CanonArrowCollector` | struct | CanonArrowCollector { schema: Vec<CanonArrowField>, columns: Vec<ColumnBuilder>, payload: Vec<u8>, hasher: blake3::Hasher, batches: u32, any_nulls: bool } |  |  |  |  |

| `CanonArrowField` | struct | CanonArrowField { pub name: String, pub ty: CanonArrowType } |  |  |  |  |

| `RING_CAPACITY` | const | RING_CAPACITY: usize |  |  |  |  |

| `PushOutcome` | enum | PushOutcome { Buffered, DroppedNewest, RefusedTornPad } |  |  |  |  |

| `RingError` | enum | RingError { Io(String), BrokenChain { detail: String, }, Malformed { line: usize, }, TornSlot { line: usize, }, SequenceGap { detail: String, }, AlreadyInitialized } |  |  |  |  |

| `close` | function | close(&self) |  |  |  |  |

| `dropped_total` | function | dropped_total(&self) -> u64 |  |  |  |  |

| `global` | function | global() -> Option<&'static Arc<Self>> |  |  |  |  |

| `init` | function | init(dir: impl AsRef<Path>) -> Result<Arc<Self>, RingError> |  |  |  |  |

| `init_global` | function | init_global(dir: impl AsRef<Path>) -> Result<(), RingError> |  |  |  |  |

| `init_global_with_ledger` | function | init_global_with_ledger( dir: impl AsRef<Path>, ledger_dir: impl AsRef<Path>, ) -> Result<(), RingError> |  |  |  |  |

| `init_with_ledger` | function | init_with_ledger( dir: impl AsRef<Path>, ledger_dir: impl AsRef<Path>, ) -> Result<Arc<Self>, RingError> |  |  |  |  |

| `init_with_ledger_restart` | function | init_with_ledger_restart( dir: impl AsRef<Path>, ledger_dir: impl AsRef<Path>, producer_attempted: u64, ) -> Result<Arc<Self>, RingError> |  |  |  |  |

| `last_ledger_error` | function | last_ledger_error(&self) -> Option<String> |  |  |  |  |

| `last_write_error` | function | last_write_error(&self) -> Option<RingError> |  |  |  |  |

| `ledger_dir` | function | ledger_dir(&self) -> Option<&Path> |  |  |  |  |

| `ledger_path` | function | ledger_path(&self) -> PathBuf |  |  |  |  |

| `new` | function | new( h_query: [u8; 32], rows_emitted: u64, execution_ms: u32, cached: u8, timestamp_ms: u64, ) -> Self |  |  |  |  |

| `now_ms` | function | now_ms() -> u64 |  |  |  |  |

| `pads_zero` | function | pads_zero(&self) -> bool |  |  |  |  |

| `push` | function | push(&self, desc: RingDescriptor) -> PushOutcome |  |  |  |  |

| `pushed_total` | function | pushed_total(&self) -> u64 |  |  |  |  |

| `set_drainer_paused` | function | set_drainer_paused(&self, paused: bool) |  |  |  |  |

| `torn_via` | function | torn_via(&self, f: impl FnOnce(&mut [u8; 3], &mut [u8; 8])) -> Self |  |  |  |  |

| `verify` | function | verify(dir: impl AsRef<Path>) -> Result<(usize, usize, u64), RingError> |  |  |  |  |

| `with_torn_pad` | function | with_torn_pad(&self, f: impl FnOnce(&[u8; 3], &[u8; 8]) -> R) -> R |  |  |  |  |

| `0000000000000000000000000000000000000000000000000000000000000000` | str_key | GENESIS = "0000000000000000000000000000000000000000000000000000000000000000" |  |  |  |  |

| `DroppedEvidence` | struct | DroppedEvidence { pub dropped_count: u64, pub refused_h_query: [u8; 32], pub refused_at_ms: u64 } |  |  |  |  |

| `EvidenceRing` | struct | EvidenceRing { dir: PathBuf, ledger_dir: Option<PathBuf>, producer: Mutex<Option<Producer<EvidenceSlot>>>, consumer: Mutex<Option<Consumer<EvidenceSlot>>>, seq_counter: Arc<AtomicU64>, dropped: AtomicU64, paused_flag: Arc<AtomicBool>, shutdown_flag: Arc<AtomicBool>, ledger_error: Arc<Mutex<Option<String>>>, write_error: Arc<Mutex<Option<String>>>, drainer: Mutex<Option<std::thread::JoinHandle<()>>> } |  |  |  |  |

| `RingDescriptor` | struct | RingDescriptor { pub h_query: [u8; 32], pub rows_emitted: u64, pub execution_ms: u32, pub cached: u8, pad1: [u8; 3], pub timestamp_ms: u64, pad2: [u8; 8] } |  |  |  |  |

| `HANDLER_SPARQL_CONSTRUCT` | const | HANDLER_SPARQL_CONSTRUCT: &str |  |  |  |  |

| `KH` | const | KH: &str |  |  |  |  |

| `MAX_FIRINGS` | const | MAX_FIRINGS: usize |  |  |  |  |

| `MAX_ROUNDS` | const | MAX_ROUNDS: usize |  |  |  |  |

| `MAX_STATE_QUADS` | const | MAX_STATE_QUADS: usize |  |  |  |  |

| `Effect` | enum | Effect { EmitDelta, Refuse } |  |  |  |  |

| `Verdict` | enum | Verdict { Fired, Refuse(String) } |  |  |  |  |

| `hooks` | function | hooks(&self) -> &[Hook] |  |  |  |  |

| `load` | function | load(pack: &LawState) -> Result<Self, Refusal> |  |  |  |  |

| `materialize` | function | materialize(&self, state: &LawState) -> Result<Materialized, Refusal> |  |  |  |  |

| `http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct` | str_key | HANDLER_SPARQL_CONSTRUCT = "http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct" |  |  |  |  |

| `http://seanchatmangpt.github.io/graphlaw/kh#` | str_key | KH = "http://seanchatmangpt.github.io/graphlaw/kh#" |  |  |  |  |

| `Firing` | struct | Firing { pub hook: String, pub round: usize, pub row: Vec<(String, String)>, pub added: usize } |  |  |  |  |

| `Hook` | struct | Hook { pub iri: String, pub name: String, pub trigger: String, pub construct: String, pub priority: i64, pub effect: Effect, pub reason: Option<String> } |  |  |  |  |

| `HookPack` | struct | HookPack { hooks: Vec<Hook> } |  |  |  |  |

| `HookVerdict` | struct | HookVerdict { pub hook: String, pub round: usize, pub row: Vec<(String, String)>, pub verdict: Verdict } |  |  |  |  |

| `Materialized` | struct | Materialized { pub state: LawState, pub firings: Vec<Firing>, pub verdicts: Vec<HookVerdict>, pub rounds: usize } |  |  |  |  |

| `DEFAULT_MAX_SKEW_SECS` | const | DEFAULT_MAX_SKEW_SECS: u64 |  |  |  |  |

| `N3_MAX_BACKWARD_DEPTH` | const | N3_MAX_BACKWARD_DEPTH: usize |  |  |  |  |

| `N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL` | const | N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL: usize |  |  |  |  |

| `N3_MAX_DERIVED_FACTS` | const | N3_MAX_DERIVED_FACTS: usize |  |  |  |  |

| `N3_MAX_ITERATIONS` | const | N3_MAX_ITERATIONS: usize |  |  |  |  |

| `N3_MAX_MATCH_STEPS` | const | N3_MAX_MATCH_STEPS: usize |  |  |  |  |

| `N3_MAX_TERM_BYTES` | const | N3_MAX_TERM_BYTES: usize |  |  |  |  |

| `N3_MAX_TERM_DEPTH` | const | N3_MAX_TERM_DEPTH: usize |  |  |  |  |

| `N3_MAX_TOTAL_BYTES` | const | N3_MAX_TOTAL_BYTES: usize |  |  |  |  |

| `N3_MAX_TOTAL_STEPS` | const | N3_MAX_TOTAL_STEPS: usize |  |  |  |  |

| `REFUSAL_UNKNOWN_STEP_CAPABILITY` | const | REFUSAL_UNKNOWN_STEP_CAPABILITY: &str |  |  |  |  |

| `Ceiling` | enum | Ceiling { Observe, Select, Construct } |  |  |  |  |

| `LawError` | enum | LawError { NotAdmitted { violations: usize, results: Vec<Violation>, }, PlanRefused { index: usize, action: String, missing: Vec<String>, violated_absent: Vec<String>, }, ReceiptRequired { step: String, }, ReceiptRefused { step: String, reason: ReceiptReason, }, LeaseRefused { lease_id: String, step: String, reason: LeaseReason, }, Refused(Refusal) } |  |  |  |  |

| `LeaseReason` | enum | LeaseReason { Expired, OutOfScope, Ceiling, BadSignature, UntrustedKey, ClockSkew } |  |  |  |  |

| `N3Error` | enum | N3Error { Limit { limit: &'static str, observed: usize, max: usize, summary: String, }, Refused(Refusal), Fuse { report: String, } } |  |  |  |  |

| `ReceiptReason` | enum | ReceiptReason { Unattested, BadSignature, UntrustedKey } |  |  |  |  |

| `Step` | enum | Step { AdmitShacl { shapes_ttl: &'a str, }, DeriveN3 { rules: &'a str, ruleset: &'a crate::tier::RuleSet, }, Hooks { pack: &'a crate::hooks::HookPack, }, Plan { plan: &'a crate::plan::Plan, }, RequireReceipt { step: &'a str, }, RequireSignedReceipt { step: &'a str, trusted: &'a crate::attest::TrustedKeys, }, EntailRdfs, EntailOwlRl, AdmitSql, TestCapabilityProbe(&'static str) } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `authority_for_capability` | function | authority_for_capability(capability: &str) -> Result<BackendAuthority, LawError> |  |  |  |  |

| `authorize` | function | authorize( &self, step: &str, required: Ceiling, trusted: &crate::attest::TrustedKeys, clock: &dyn Clock, max_skew_secs: u64, ) -> Result<(), LawError> |  |  |  |  |

| `dataset` | function | dataset(&self) -> &Arc<purrdf::RdfDataset> |  |  |  |  |

| `from_dataset` | function | from_dataset(dataset: Arc<purrdf::RdfDataset>) -> Result<Self, Refusal> |  |  |  |  |

| `id` | function | id(&self) -> &str |  |  |  |  |

| `name` | function | name(&self) -> &'static str |  |  |  |  |

| `parse` | function | parse(s: &str) -> Option<Self> |  |  |  |  |

| `quad_count` | function | quad_count(&self) -> usize |  |  |  |  |

| `reason_n3_bounded` | function | reason_n3_bounded(input: &str) -> Result<String, N3Error> |  |  |  |  |

| `required_ceiling` | function | required_ceiling(&self) -> Ceiling |  |  |  |  |

| `transition` | function | transition(&self, step: &Step<'_>) -> Result<(LawState, Receipt), LawError> |  |  |  |  |

| `transition_authorized` | function | transition_authorized( &self, signed: &SignedLease, trusted: &crate::attest::TrustedKeys, clock: &dyn Clock, max_skew_secs: u64, step: &Step<'_>, ) -> Result<(LawState, Receipt), LawError> |  |  |  |  |

| `transition_leased_unverified` | function | transition_leased_unverified( &self, lease: &Lease, step: &Step<'_>, now_unix: u64, ) -> Result<(LawState, Receipt), LawError> |  |  |  |  |

| `with_subject` | function | with_subject(mut self, subject_blake3: impl Into<String>) -> Self |  |  |  |  |

| `UNKNOWN-STEP-CAPABILITY` | str_key | REFUSAL_UNKNOWN_STEP_CAPABILITY = "UNKNOWN-STEP-CAPABILITY" |  |  |  |  |

| `FixedClock` | struct | FixedClock { pub u64 } |  |  |  |  |

| `LawState` | struct | LawState { dataset: Arc<purrdf::RdfDataset>, id: String } |  |  |  |  |

| `Lease` | struct | Lease { pub id: String, pub holder: String, pub ceiling: Ceiling, pub scope: Vec<String>, pub expires_unix: u64, pub issued_unix: u64 } |  |  |  |  |

| `Receipt` | struct | Receipt { pub parent: String, pub child: String, pub step: &'static str, pub authority: BackendAuthority, pub added: usize, pub lease_id: Option<String>, pub plan_blake3: Option<String>, pub subject_blake3: Option<String> } |  |  |  |  |

| `SignedLease` | struct | SignedLease { pub lease: Lease, pub attestation: crate::attest::Attestation } |  |  |  |  |

| `SystemClock` | struct |  |  |  |  |  |

| `Violation` | struct | Violation { pub focus: String, pub path: Option<String>, pub component: String, pub message: String, pub severity: String } |  |  |  |  |

| `Clock` | trait |  |  |  |  |  |

| `ReasonerLimitCensus` | trait |  |  |  |  |  |

| `BACKEND_AUTHORITIES` | const | BACKEND_AUTHORITIES: &[BackendAuthority] |  |  |  |  |

| `LEGACY_FALLBACK_ENABLED` | const | LEGACY_FALLBACK_ENABLED: bool |  |  |  |  |

| `RDF 1.2 / codecs / storage IR` | str_key | BACKEND_AUTHORITIES = "RDF 1.2 / codecs / storage IR" |  |  |  |  |

| `BackendAuthority` | struct | BackendAuthority { pub capability: &'static str, pub authority: &'static str, pub revision: &'static str } |  |  |  |  |

| `law::LawState` | use | law::LawState |  |  |  |  |

| `purrdf` | use | purrdf |  |  |  |  |

| `purrdf::datalog::*` | use | purrdf::datalog::* |  |  |  |  |

| `purrdf::entail::*` | use | purrdf::entail::* |  |  |  |  |

| `purrdf::events::*` | use | purrdf::events::* |  |  |  |  |

| `purrdf::geo::*` | use | purrdf::geo::* |  |  |  |  |

| `purrdf::gts::*` | use | purrdf::gts::* |  |  |  |  |

| `purrdf::n3::*` | use | purrdf::n3::* |  |  |  |  |

| `purrdf::retrieval::*` | use | purrdf::retrieval::* |  |  |  |  |

| `purrdf::shapes::*` | use | purrdf::shapes::* |  |  |  |  |

| `purrdf::shex::*` | use | purrdf::shex::* |  |  |  |  |

| `purrdf::sparql::*` | use | purrdf::sparql::* |  |  |  |  |

| `purrdf::text::*` | use | purrdf::text::* |  |  |  |  |

| `purrdf::validate::*` | use | purrdf::validate::* |  |  |  |  |

| `purrdf::{ CanonHash, RdfDataset, RdfDatasetBuilder, RdfDiagnostic, RdfLiteral, SerializeGraph, SparqlEngine, SparqlRequest, SparqlResult, TermId, TermValue, parse_dataset, parse_dataset_with, serialize_dataset, try_canonicalize, }` | use | purrdf::{ CanonHash, RdfDataset, RdfDatasetBuilder, RdfDiagnostic, RdfLiteral, SerializeGraph, SparqlEngine, SparqlRequest, SparqlResult, TermId, TermValue, parse_dataset, parse_dataset_with, serialize_dataset, try_canonicalize, } |  |  |  |  |

| `purrdf_columnar` | use | purrdf_columnar |  |  |  |  |

| `receipt_chain::{ReceiptRecord, validator::ReceiptValidator}` | use | receipt_chain::{ReceiptRecord, validator::ReceiptValidator} |  |  |  |  |

| `MAX_PLAN_TOTAL_ATOMS` | const | MAX_PLAN_TOTAL_ATOMS: usize |  |  |  |  |

| `TripleError` | enum | TripleError { InvalidIri { role: &'static str, iri: String, }, Builder(String) } |  |  |  |  |

| `action` | function | action(mut self, name: impl Into<String>) -> Self |  |  |  |  |

| `adds` | function | adds(mut self, t: Triple) -> Self |  |  |  |  |

| `admit` | function | admit(&self, start: &LawState) -> Result<Admitted, LawError> |  |  |  |  |

| `as_str` | function | as_str(&self) -> &str |  |  |  |  |

| `build` | function | build(self) -> Action |  |  |  |  |

| `builder` | function | builder(name: impl Into<String>) -> ActionBuilder |  |  |  |  |

| `canonical_json` | function | canonical_json(&self) -> String |  |  |  |  |

| `deletes` | function | deletes(mut self, t: Triple) -> Self |  |  |  |  |

| `digest` | function | digest(&self) -> String |  |  |  |  |

| `goal` | function | goal(mut self, t: Triple) -> Self |  |  |  |  |

| `goal_not` | function | goal_not(mut self, t: Triple) -> Self |  |  |  |  |

| `iri` | function | iri(s: &str, p: &str, o: &str) -> Result<Self, TripleError> |  |  |  |  |

| `literal` | function | literal(s: &str, p: &str, text: &str) -> Result<Self, TripleError> |  |  |  |  |

| `new` | function | new(name: impl Into<String>) -> Self |  |  |  |  |

| `requires` | function | requires(mut self, t: Triple) -> Self |  |  |  |  |

| `requires_not` | function | requires_not(mut self, t: Triple) -> Self |  |  |  |  |

| `RDF 1.2 / codecs / storage IR` | str_key | CAPABILITY = "RDF 1.2 / codecs / storage IR" |  |  |  |  |

| `Action` | struct | Action { pub name: String, pub pre: String, pub pre_not: String, pub add: String, pub del: String } |  |  |  |  |

| `ActionBuilder` | struct | ActionBuilder { name: String, pre: Vec<Triple>, pre_not: Vec<Triple>, add: Vec<Triple>, del: Vec<Triple> } |  |  |  |  |

| `Admitted` | struct | Admitted { pub state: LawState, pub receipts: Vec<Receipt>, pub plan_digest: String } |  |  |  |  |

| `Plan` | struct | Plan { pub actions: Vec<Action>, pub goal: String, pub goal_not: String } |  |  |  |  |

| `PlanBuilder` | struct | PlanBuilder { done: Vec<Action>, current: Option<ActionBuilder>, goal: Vec<Triple>, goal_not: Vec<Triple>, error: Option<TripleError> } |  |  |  |  |

| `Triple` | struct | Triple { String } |  |  |  |  |

| `PolicyRefusalKind` | enum | PolicyRefusalKind { MissingEntry, InventedOutcome, BadMass, DeadEnd, Malformed } |  |  |  |  |

| `admit` | function | admit(problem_json: &str, policy_json: &str) -> Result<PolicyAdmitted, PolicyRefused> |  |  |  |  |

| `admit_parsed` | function | admit_parsed(problem: &Problem, policy: &[Entry]) -> Result<PolicyAdmitted, PolicyRefused> |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `to_ntriples` | function | to_ntriples(&self) -> String |  |  |  |  |

| `https://graphlaw.dev/policy#` | str_key | NS = "https://graphlaw.dev/policy#" |  |  |  |  |

| `Entry` | struct | Entry { pub state: String, pub action: String, pub outcomes: Vec<Outcome> } |  |  |  |  |

| `Outcome` | struct | Outcome { pub state: String, pub probability_ppm: u64 } |  |  |  |  |

| `PolicyAdmitted` | struct | PolicyAdmitted { pub initial_states: Vec<String>, pub reachable: Vec<String>, pub goal_states: Vec<String>, pub entries: Vec<(String, String)> } |  |  |  |  |

| `PolicyRefused` | struct | PolicyRefused { pub kind: PolicyRefusalKind, pub state: String, pub action: String, pub message: String } |  |  |  |  |

| `Problem` | struct | Problem { pub states: BTreeMap<String, BTreeSet<String>>, pub initial: Vec<String>, pub goal_facts: BTreeSet<String>, pub unsafe_states: BTreeSet<String>, pub transitions: BTreeSet<(String, String, String)> } |  |  |  |  |

| `json::admit` | use | json::admit |  |  |  |  |

| `ProofLeaseError` | enum | ProofLeaseError { LeaseDigestMismatch { lease_digest: String, source_digest: String, }, Parse(String) } |  |  |  |  |

| `lease_digest_source` | function | lease_digest_source(source: &str) -> String |  |  |  |  |

| `verify_leased_proof` | function | verify_leased_proof( lease_digest: &str, source: &str, proof: &str, label: &str, ) -> Result<Report, ProofLeaseError> |  |  |  |  |

| `@prefix : <http://example.org/> .
:Socrates a :Human .
` | str_key | SOURCE = "@prefix : <http://example.org/> .
:Socrates a :Human .
" |  |  |  |  |

| `Standing` | enum | Standing { Unqualified, Observed, Attacked, Qualified, Refused, Unknown } |  |  |  |  |

| `compose_courts` | function | compose_courts(parent: &Value, child: &Value) -> Result<BTreeSet<String>, String> |  |  |  |  |

| `evaluate` | function | evaluate(observation: Observation) -> Standing |  |  |  |  |

| `generate_falsifier` | function | generate_falsifier(law: &Value, seed: &Value) -> Result<Value, String> |  |  |  |  |

| `temporal_rule_holds` | function | temporal_rule_holds(events: &[&str], rule: &str) -> bool |  |  |  |  |

| `Observation` | struct | Observation { pub attempt_observed: bool, pub violation_observed: bool, pub independent_observer: bool, pub postconditions_satisfied: bool, pub durable_evidence: bool, pub crash_observed: bool, pub crash_reconciled: bool } |  |  |  |  |

| `bump` | function | bump(&self) -> u64 |  |  |  |  |

| `consult` | function | consult( &self, h_query: [u8; 32], grounding: Option<&[u8]>, ) -> Option<(ExecutionReceipt, u64)> |  |  |  |  |

| `epoch` | function | epoch(&self) -> u64 |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `now_ms` | function | now_ms() -> u64 |  |  |  |  |

| `store` | function | store(&self, entry: CacheEntry) |  |  |  |  |

| `with_entries` | function | with_entries(&self, f: impl FnOnce(&mut HashMap<[u8; 32], CacheEntry>) -> R) -> R |  |  |  |  |

| `CacheEntry` | struct | CacheEntry { pub h_query: [u8; 32], pub receipt: ExecutionReceipt, pub grounding: Vec<u8>, pub epoch: u64, pub stored_at: u64 } |  |  |  |  |

| `QueryCache` | struct | QueryCache { entries: Mutex<HashMap<[u8; 32], CacheEntry>>, epoch: AtomicU64 } |  |  |  |  |

| `batch::*` | use | batch::* |  |  |  |  |

| `storage::*` | use | storage::* |  |  |  |  |

| `FLUSH_AGE_MS` | const | FLUSH_AGE_MS: u64 |  |  |  |  |

| `FLUSH_COUNT` | const | FLUSH_COUNT: usize |  |  |  |  |

| `GENESIS` | const | GENESIS: &str |  |  |  |  |

| `LedgerError` | enum | LedgerError { Io(String), DigestMismatch { file: String, expected: String, actual: String, }, Malformed { file: String, }, BrokenChain { detail: String, }, Fork { parent: String, }, DoubleGenesis { digest: String, }, MissingAgg { batch: String, }, AggMismatch { batch: String, }, ChainIndex { detail: String, } } |  |  |  |  |

| `content_digest` | function | content_digest(bytes: &[u8]) -> String |  |  |  |  |

| `fold_batch` | function | fold_batch(batch: &str, records: &[Descriptor]) -> BatchAgg |  |  |  |  |

| `0000000000000000000000000000000000000000000000000000000000000000` | str_key | GENESIS = "0000000000000000000000000000000000000000000000000000000000000000" |  |  |  |  |

| `Aggregate` | struct | Aggregate { pub rows_emitted_sum: BTreeMap<u64, u64>, pub saturated: bool, pub complete: bool, pub batches: usize } |  |  |  |  |

| `BatchAgg` | struct | BatchAgg { pub batch: String, pub rows_emitted_sum: BTreeMap<u64, u64>, pub saturated: bool } |  |  |  |  |

| `ChainState` | struct | ChainState { pub head: String, pub tail: String, pub batches: u64, pub next_seq: u64, pub aggs: BTreeMap<String, String> } |  |  |  |  |

| `Descriptor` | struct | Descriptor { pub h_query: [u8; 32], pub rows_emitted: u32, pub flags: u8, pub ts_unix_ms: u64, pub epoch: u64 } |  |  |  |  |

| `accounted_descriptors` | function | accounted_descriptors(dir: impl AsRef<Path>) -> Result<u64, LedgerError> |  |  |  |  |

| `aggregate` | function | aggregate(&self, from_ms: u64, to_ms: u64) -> Result<Aggregate, LedgerError> |  |  |  |  |

| `chain_state` | function | chain_state(&self) -> ChainState |  |  |  |  |

| `flush` | function | flush(&mut self) -> Result<(), LedgerError> |  |  |  |  |

| `gaps` | function | gaps(&self) -> Result<Vec<(u64, u64)>, LedgerError> |  |  |  |  |

| `now_ms` | function | now_ms() -> u64 |  |  |  |  |

| `open` | function | open(dir: impl AsRef<Path>) -> Result<Self, LedgerError> |  |  |  |  |

| `open_with_producer` | function | open_with_producer( dir: impl AsRef<Path>, producer_next_seq: u64, ) -> Result<Self, LedgerError> |  |  |  |  |

| `push` | function | push(&mut self, d: Descriptor) -> Result<(), LedgerError> |  |  |  |  |

| `record_batch` | function | record_batch(&mut self, batch: &[Descriptor]) -> Result<(), LedgerError> |  |  |  |  |

| `record_gap` | function | record_gap(&self, first: u64, last: u64) -> Result<(), LedgerError> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<ChainState, LedgerError> |  |  |  |  |

| `QueryLedger` | struct | QueryLedger { dir: PathBuf, parent: String, next_seq: u64, batches: u64, head: String, tail: String, buf: Vec<Descriptor>, buf_first_seq: u64, buf_opened_ms: u64, aggs: BTreeMap<String, String>, pub last_flush_error: Option<LedgerError> } |  |  |  |  |

| `record` | function | record(state: &LawState, receipt: &Receipt) -> Result<LawState, LawError> |  |  |  |  |

| `record_signed` | function | record_signed( state: &LawState, receipt: &Receipt, attestation: &Attestation, ) -> Result<LawState, LawError> |  |  |  |  |

| `require` | function | require(state: &LawState, step: &str) -> Result<(), LawError> |  |  |  |  |

| `require_signed` | function | require_signed(state: &LawState, step: &str, trusted: &TrustedKeys) -> Result<(), LawError> |  |  |  |  |

| `urn:graphlaw:` | str_key | NS = "urn:graphlaw:" |  |  |  |  |

| `MIGRATION_LAW_1_TO_2` | const | MIGRATION_LAW_1_TO_2: &str |  |  |  |  |

| `SCHEMA_V1` | const | SCHEMA_V1: &str |  |  |  |  |

| `SCHEMA_V2` | const | SCHEMA_V2: &str |  |  |  |  |

| `AdmissionDecision` | enum | AdmissionDecision { Admitted, Quarantined, Refused } |  |  |  |  |

| `AdmissionLedger` | enum | AdmissionLedger { LegacyUnrecorded, Recorded(Vec<AdmissionItem>) } |  |  |  |  |

| `AndonLevel` | enum | AndonLevel { Red, Yellow, Green } |  |  |  |  |

| `CeilingLevel` | enum | CeilingLevel { Red, LegacyObserved, Yellow, Green } |  |  |  |  |

| `CheckOutcome` | enum | CheckOutcome { Pass, Fail(String), Skip(String) } |  |  |  |  |

| `EquivalenceStatus` | enum | EquivalenceStatus { Unknown, Equivalent, Divergent(String) } |  |  |  |  |

| `ObligationCount` | enum | ObligationCount { Unknown, Tracked { required: u32, discharged: u32, } } |  |  |  |  |

| `ObservedOutcome` | enum | ObservedOutcome { Pass, Fail, Unknown } |  |  |  |  |

| `admission_item` | function | admission_item(mut self, item: AdmissionItem) -> Self |  |  |  |  |

| `all_unknown` | function | all_unknown() -> Self |  |  |  |  |

| `build` | function | build(self) -> Result<ReceiptEpochV2, CoreError> |  |  |  |  |

| `compute_ceiling` | function | compute_ceiling(prev: CeilingLevel, components: &ComponentLevels) -> CeilingLevel |  |  |  |  |

| `compute_obligation_count` | function | compute_obligation_count(ledger: &AdmissionLedger) -> ObligationCount |  |  |  |  |

| `derive_andon` | function | derive_andon(ledger: &AdmissionLedger) -> AndonLevel |  |  |  |  |

| `equivalence` | function | equivalence(mut self, map: EquivalenceMap) -> Self |  |  |  |  |

| `is_pass` | function | is_pass(&self) -> bool |  |  |  |  |

| `legacy_bounded` | function | legacy_bounded() -> Self |  |  |  |  |

| `level` | function | level(&self) -> AndonLevel |  |  |  |  |

| `new` | function | new(prev_ceiling: CeilingLevel, components: ComponentLevels) -> Self |  |  |  |  |

| `read_receipt_epoch` | function | read_receipt_epoch(record: &super::ReceiptRecord) -> Result<ReceiptEpochV2, CoreError> |  |  |  |  |

| `recoverable` | function | recoverable(prev: CeilingLevel) -> CeilingLevel |  |  |  |  |

| `remaining` | function | remaining(&self) -> Option<u32> |  |  |  |  |

| `supported` | function | supported(components: &ComponentLevels) -> CeilingLevel |  |  |  |  |

| `uniform` | function | uniform(level: AndonLevel) -> Self |  |  |  |  |

| `validate` | function | validate(records: &[ReceiptRecord], clock: &dyn Clock) -> Verdict |  |  |  |  |

| `validate_promotion` | function | validate_promotion( current_ceiling: CeilingLevel, actuator_identity: &str, witness: &PromotionWitness, ) -> Result<CeilingLevel, CoreError> |  |  |  |  |

| `with_explicit_ceiling` | function | with_explicit_ceiling(mut self, level: CeilingLevel) -> Self |  |  |  |  |

| `with_generation` | function | with_generation(mut self, generation: u64) -> Self |  |  |  |  |

| `with_verified_promotion` | function | with_verified_promotion( mut self, witness: &PromotionWitness, actuator_identity: &str, ) -> Result<Self, CoreError> |  |  |  |  |

| `M_1_to_2` | str_key | MIGRATION_LAW_1_TO_2 = "M_1_to_2" |  |  |  |  |

| `ggen-receipt/v1` | str_key | SCHEMA_V1 = "ggen-receipt/v1" |  |  |  |  |

| `ggen-receipt/v2` | str_key | SCHEMA_V2 = "ggen-receipt/v2" |  |  |  |  |

| `AdmissionItem` | struct | AdmissionItem { pub evidence_id: String, pub observed_outcome: ObservedOutcome, pub decision: AdmissionDecision, pub reason: String, pub obligations_discharged: Vec<String>, pub obligations_created: Vec<String>, pub generation: u64 } |  |  |  |  |

| `ComponentLevels` | struct | ComponentLevels { pub lint: AndonLevel, pub test: AndonLevel, pub fmt: AndonLevel, pub gate: AndonLevel } |  |  |  |  |

| `EquivalenceMap` | struct | EquivalenceMap { pub source: EquivalenceStatus, pub compiled_binary: EquivalenceStatus, pub docs: EquivalenceStatus, pub tests: EquivalenceStatus, pub receipts: EquivalenceStatus, pub evidence: EquivalenceStatus, pub gates: EquivalenceStatus, pub config: EquivalenceStatus } |  |  |  |  |

| `FixedClock` | struct | FixedClock { pub u64 } |  |  |  |  |

| `MigrationReceipt` | struct | MigrationReceipt { pub migration_law: String, pub from_schema: String, pub to_schema: String, pub final_v1_chain_hash_hex: String, pub first_v2_chain_hash_hex: String, pub carries_forward: Vec<String>, pub becomes_unknown: Vec<String>, pub resulting_ceiling: CeilingLevel } |  |  |  |  |

| `PromotionWitness` | struct | PromotionWitness { pub from_ceiling: CeilingLevel, pub to_ceiling: CeilingLevel, pub evidence_receipts: Vec<String>, pub closed_obligations: Vec<String>, pub verifier_identity: String, pub authorization_basis: String } |  |  |  |  |

| `ReceiptEpochV2` | struct | ReceiptEpochV2 { pub admission: AdmissionLedger, pub standing_ceiling: CeilingLevel, pub equivalence: EquivalenceMap, pub obligation_count: ObligationCount, pub andon: AndonLevel, pub promotion_eligible: bool, pub generation: u64 } |  |  |  |  |

| `ReceiptEpochV2Builder` | struct | ReceiptEpochV2Builder { prev_ceiling: CeilingLevel, components: ComponentLevels, admission: Vec<AdmissionItem>, equivalence: EquivalenceMap, explicit_ceiling: Option<CeilingLevel>, promoted: bool, generation: u64 } |  |  |  |  |

| `ReceiptRecordV1Legacy` | struct | ReceiptRecordV1Legacy { pub version: u32, pub instruction_id: u64, pub activity_idx: u16, pub activity: Option<String>, pub node_kind: u8, pub ts_ns: u64, pub duration_ms: Option<u64>, pub payload_hash_hex: String, pub prev_chain_hash_hex: String, pub chain_hash_hex: String, pub andon: super::Andon, pub obligation_count: u32, pub object_ids: Vec<String>, pub signature_hex: Option<String> } |  |  |  |  |

| `ReceiptValidator` | struct |  |  |  |  |  |

| `StageResult` | struct | StageResult { pub stage: String, pub outcome: CheckOutcome } |  |  |  |  |

| `SystemClock` | struct |  |  |  |  |  |

| `Verdict` | struct | Verdict { pub ok: bool, pub stages: Vec<StageResult>, pub records_checked: usize } |  |  |  |  |

| `Clock` | trait |  |  |  |  |  |

| `record::*` | use | record::* |  |  |  |  |

| `ADMITTED` | const | ADMITTED: Self |  |  |  |  |

| `AUTHORIZATION_DENIED` | const | AUTHORIZATION_DENIED: Self |  |  |  |  |

| `CONFORMANCE_GATE_FAILED` | const | CONFORMANCE_GATE_FAILED: Self |  |  |  |  |

| `OBJECT_LIFECYCLE_VIOLATION` | const | OBJECT_LIFECYCLE_VIOLATION: Self |  |  |  |  |

| `PRECONDITION_FAILED` | const | PRECONDITION_FAILED: Self |  |  |  |  |

| `RESOURCE_EXHAUSTED` | const | RESOURCE_EXHAUSTED: Self |  |  |  |  |

| `SLA_BREACH` | const | SLA_BREACH: Self |  |  |  |  |

| `WATCHDOG_DRAINED` | const | WATCHDOG_DRAINED: Self |  |  |  |  |

| `chain` | function | chain(&mut self, frame: &OcelCausalFrame) |  |  |  |  |

| `compose` | function | compose(self, other: Self) -> Self |  |  |  |  |

| `genesis` | function | genesis(run_id: [u8; 32]) -> Self |  |  |  |  |

| `is_admitted` | function | is_admitted(self) -> bool |  |  |  |  |

| `to_fired_mask` | function | to_fired_mask(self) -> u64 |  |  |  |  |

| `to_hash_bytes` | function | to_hash_bytes(&self) -> [u8; 99] |  |  |  |  |

| `DenialPolarity` | struct | DenialPolarity { pub u64 } |  |  |  |  |

| `OcelCausalFrame` | struct | OcelCausalFrame { pub instruction_id: u64, pub fired_mask: u64, pub denial: DenialPolarity, pub obj_refs: [PackedObjRef; 8], pub ts_ns: u64, pub activity_idx: u16, pub node_kind: u8, pub pad: [u8; 5], pub prior_hash: [u8; 32] } |  |  |  |  |

| `OcelCausalReceipt` | struct | OcelCausalReceipt { pub chain_hash: [u8; 32], pub frame_count: u64, pub run_id: [u8; 32], pub replay_ptr: u64 } |  |  |  |  |

| `PackedObjRef` | struct | PackedObjRef { pub u32 } |  |  |  |  |

| `CHAIN_RULE_BASE` | const | CHAIN_RULE_BASE: &str |  |  |  |  |

| `CHAIN_RULE_V2_FOLD` | const | CHAIN_RULE_V2_FOLD: &str |  |  |  |  |

| `RECEIPT_RECORD_VERSION` | const | RECEIPT_RECORD_VERSION: u32 |  |  |  |  |

| `Andon` | enum | Andon { Green, Halted { unmet: Vec<Obligation>, refusals: Vec<RefusalScenario>, at: u64, }, Overridden { by: String, reason: String, at: u64, } } |  |  |  |  |

| `ChainRule` | enum | ChainRule { Base, V2Fold } |  |  |  |  |

| `ChainStanding` | enum | ChainStanding { FullyBound, LegacyV2Unbound } |  |  |  |  |

| `ChainVerification` | enum | ChainVerification { Verified(ChainStanding), Mismatch { rule: ChainRule, recomputed: [u8; 32], } } |  |  |  |  |

| `CoreError` | enum | CoreError { SerializationFailed(String), HexDecodeFailed(String), ReceiptChainRuleInvalid(String), Io(String), ReceiptSchemaPayloadMismatch(String), UnrecognizedReceiptSchema(String), CeilingExceedsMeet { requested: crate::receipt_chain::epoch::CeilingLevel, allowed: crate::receipt_chain::epoch::CeilingLevel, }, PromotionRefused { reason: String, } } |  |  |  |  |

| `Obligation` | enum | Obligation { Precondition { predicate_id: String, params_hash: [u8; 32], }, BlockingConstraint { reason: String, }, EvidenceRequired { evidence_type: String, } } |  |  |  |  |

| `RefusalCategory` | enum | RefusalCategory { Identity, Capacity, Topology, Temporal, Lifecycle, Authorization, Prerequisites, Reserved } |  |  |  |  |

| `RefusalScenario` | enum | RefusalScenario { BlockingConstraint { reason: String, }, MissingEvidence { evidence_type: String, }, UnsatisfiedPrecondition { predicate_id: String, }, WatchdogDrained, PreconditionFailed, SlaBreach, AuthorizationDenied, ResourceExhausted, ObjectLifecycleViolation, ConformanceGateFailed } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `build_admission_frame` | function | build_admission_frame( payload_hash: &[u8; 32], prev_chain_hash: &[u8; 32], meta: &ReceiptMeta, ts_ns: u64, ) -> OcelCausalFrame |  |  |  |  |

| `category` | function | category(&self) -> RefusalCategory |  |  |  |  |

| `chain_from_frame` | function | chain_from_frame(prev_chain_hash: &[u8; 32], frame: &OcelCausalFrame) -> [u8; 32] |  |  |  |  |

| `chain_hash` | function | chain_hash(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `compose_denials` | function | compose_denials( scenarios: impl IntoIterator<Item = &'a RefusalScenario>, ) -> DenialPolarity |  |  |  |  |

| `declared_chain_rule` | function | declared_chain_rule(&self) -> Result<Option<ChainRule>, CoreError> |  |  |  |  |

| `denial_lane` | function | denial_lane(scenario: &RefusalScenario) -> DenialPolarity |  |  |  |  |

| `first_declared` | function | first_declared(&self) -> Option<usize> |  |  |  |  |

| `name` | function | name(&self) -> &'static str |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `observe` | function | observe( &mut self, idx: usize, record: &ReceiptRecord, standing: ChainStanding, ) -> Result<(), CoreError> |  |  |  |  |

| `observe_declared` | function | observe_declared( &mut self, idx: usize, declared: bool, standing: ChainStanding, ) -> Result<(), CoreError> |  |  |  |  |

| `parse` | function | parse(s: &str) -> Result<Self, CoreError> |  |  |  |  |

| `payload_hash` | function | payload_hash(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `prev_chain_hash` | function | prev_chain_hash(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `recompute_chain_hash` | function | recompute_chain_hash(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `recompute_chain_hash_lawful` | function | recompute_chain_hash_lawful(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `recompute_chain_hash_under` | function | recompute_chain_hash_under(&self, rule: ChainRule) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `scenario_for_denial_lane` | function | scenario_for_denial_lane(lane: DenialPolarity) -> Option<RefusalScenario> |  |  |  |  |

| `verify_chain` | function | verify_chain(&self) -> Result<ChainVerification, CoreError> |  |  |  |  |

| `verify_chain_against` | function | verify_chain_against(&self, stored: [u8; 32]) -> Result<ChainVerification, CoreError> |  |  |  |  |

| `graphlaw-chain/base` | str_key | CHAIN_RULE_BASE = "graphlaw-chain/base" |  |  |  |  |

| `graphlaw-chain/v2-fold` | str_key | CHAIN_RULE_V2_FOLD = "graphlaw-chain/v2-fold" |  |  |  |  |

| `ChainRuleMonotonicity` | struct | ChainRuleMonotonicity { first_declared: Option<usize> } |  |  |  |  |

| `ReceiptMeta` | struct | ReceiptMeta { pub instruction_id: u64, pub activity_idx: u16, pub node_kind: u8, pub ts_ns: Option<u64>, pub denial: DenialPolarity, pub andon: Andon, pub object_ids: Vec<String>, pub obligation_count: u32 } |  |  |  |  |

| `ReceiptRecord` | struct | ReceiptRecord { pub version: u32, pub instruction_id: u64, pub activity_idx: u16, pub activity: Option<String>, pub node_kind: u8, pub ts_ns: u64, pub duration_ms: Option<u64>, pub origin: Option<String>, pub payload_hash_hex: String, pub prev_chain_hash_hex: String, pub chain_hash_hex: String, pub andon: Andon, pub obligation_count: u32, pub object_ids: Vec<String>, pub signature_hex: Option<String>, pub schema: String, pub v2: Option<epoch::ReceiptEpochV2>, pub chain_rule: Option<String> } |  |  |  |  |

| `DEFAULT_RECEIPTS_DIR` | const | DEFAULT_RECEIPTS_DIR: &str |  |  |  |  |

| `GENESIS_CHAIN_HASH` | const | GENESIS_CHAIN_HASH: [u8; 32] |  |  |  |  |

| `LEDGER_FILE_NAME` | const | LEDGER_FILE_NAME: &str |  |  |  |  |

| `append` | function | append(&self, record: &ReceiptRecord) -> Result<(), CoreError> |  |  |  |  |

| `last_chain_hash` | function | last_chain_hash(&self) -> Result<[u8; 32], CoreError> |  |  |  |  |

| `load_all` | function | load_all(&self) -> Result<Vec<ReceiptRecord>, CoreError> |  |  |  |  |

| `open` | function | open(dir: impl AsRef<Path>) -> Result<Self, CoreError> |  |  |  |  |

| `open_default` | function | open_default() -> Result<Self, CoreError> |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `receipts` | str_key | DEFAULT_RECEIPTS_DIR = "receipts" |  |  |  |  |

| `receipts.jsonl` | str_key | LEDGER_FILE_NAME = "receipts.jsonl" |  |  |  |  |

| `ReceiptStore` | struct | ReceiptStore { path: PathBuf } |  |  |  |  |

| `StoreError` | enum | StoreError { Io(String), InvalidSubject(String), Malformed { file: String, }, DigestMismatch { file: String, expected: String, actual: String, }, Unattested { file: String, }, Attestation { file: String, error: AttestError, }, BrokenChain { detail: String, } } |  |  |  |  |

| `list` | function | list(&self, subject_sha: &str) -> Result<Vec<String>, StoreError> |  |  |  |  |

| `open` | function | open(dir: impl AsRef<Path>) -> Result<Self, StoreError> |  |  |  |  |

| `put` | function | put(&self, receipt: &Receipt, subject_sha: &str) -> Result<String, StoreError> |  |  |  |  |

| `put_signed` | function | put_signed( &self, receipt: &Receipt, subject_sha: &str, attestation: &Attestation, ) -> Result<String, StoreError> |  |  |  |  |

| `receipt_digest` | function | receipt_digest(r: &Receipt) -> String |  |  |  |  |

| `verify` | function | verify(&self, subject_sha: &str) -> Result<Vec<Receipt>, StoreError> |  |  |  |  |

| `verify_attested` | function | verify_attested( &self, subject_sha: &str, trusted: &TrustedKeys, ) -> Result<Vec<Receipt>, StoreError> |  |  |  |  |

| `ReceiptStore` | struct | ReceiptStore { dir: PathBuf } |  |  |  |  |

| `definitions::*` | use | definitions::* |  |  |  |  |

| `emit_json::*` | use | emit_json::* |  |  |  |  |

| `emit_ttl::*` | use | emit_ttl::* |  |  |  |  |

| `ENGINES` | const | ENGINES: &[&str] |  |  |  |  |

| `GAC_NAMESPACE` | const | GAC_NAMESPACE: &str |  |  |  |  |

| `LAW_STEPS` | const | LAW_STEPS: &[LawStepRow] |  |  |  |  |

| `LEASE_CEILINGS` | const | LEASE_CEILINGS: &[&str] |  |  |  |  |

| `LEASE_REASONS` | const | LEASE_REASONS: &[&str] |  |  |  |  |

| `NATIVE_ONLY_OPS` | const | NATIVE_ONLY_OPS: &[&str] |  |  |  |  |

| `OPS` | const | OPS: &[Op] |  |  |  |  |

| `OTHER_DIALECTS` | const | OTHER_DIALECTS: &[DialectRow] |  |  |  |  |

| `POLICY_REFUSAL_KINDS` | const | POLICY_REFUSAL_KINDS: &[&str] |  |  |  |  |

| `RDF_DIALECTS` | const | RDF_DIALECTS: &[DialectRow] |  |  |  |  |

| `RECEIPT_REASONS` | const | RECEIPT_REASONS: &[&str] |  |  |  |  |

| `REFUSAL_CODES` | const | REFUSAL_CODES: &[RefusalCodeRow] |  |  |  |  |

| `REFUSAL_KINDS` | const | REFUSAL_KINDS: &[&str] |  |  |  |  |

| `REGIMES` | const | REGIMES: &[&str] |  |  |  |  |

| `REGISTRY_BASE` | const | REGISTRY_BASE: &str |  |  |  |  |

| `REGISTRY_SCHEMA` | const | REGISTRY_SCHEMA: &str |  |  |  |  |

| `FieldDefault` | enum | FieldDefault { None, Str(&'static str), Int(i64), Bool(bool) } |  |  |  |  |

| `op_names` | function | op_names() -> Vec<&'static str> |  |  |  |  |

| `other_dialect_names` | function | other_dialect_names() -> Vec<&'static str> |  |  |  |  |

| `rdf_dialect_names` | function | rdf_dialect_names() -> Vec<&'static str> |  |  |  |  |

| `EngineRejected` | str_key | KINDS_ENGINE = "EngineRejected" |  |  |  |  |

| `MissingEntry` | str_key | POLICY_REFUSAL_KINDS = "MissingEntry" |  |  |  |  |

| `NotSemanticContent` | str_key | REFUSAL_KINDS = "NotSemanticContent" |  |  |  |  |

| `PurRdf` | str_key | ENGINES = "PurRdf" |  |  |  |  |

| `Refused` | str_key | REFUSAL_CODES = "Refused" |  |  |  |  |

| `ResourceLimit` | str_key | CODES_LIMIT = "ResourceLimit" |  |  |  |  |

| `Unsupported` | str_key | KINDS_CAPABILITIES = "Unsupported" |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `data` | str_key | DATA = "data" |  |  |  |  |

| `duckdb` | str_key | NATIVE_ONLY_OPS = "duckdb" |  |  |  |  |

| `expired` | str_key | LEASE_REASONS = "expired" |  |  |  |  |

| `graphlaw.capability-registry/1` | str_key | REGISTRY_SCHEMA = "graphlaw.capability-registry/1" |  |  |  |  |

| `http://seanchatmangpt.github.io/packs/graphlaw-ash-capability#` | str_key | GAC_NAMESPACE = "http://seanchatmangpt.github.io/packs/graphlaw-ash-capability#" |  |  |  |  |

| `https://graphlaw.dev/registry#` | str_key | REGISTRY_BASE = "https://graphlaw.dev/registry#" |  |  |  |  |

| `n3` | str_key | OTHER_DIALECTS = "n3" |  |  |  |  |

| `observe` | str_key | LEASE_CEILINGS = "observe" |  |  |  |  |

| `shacl` | str_key | LAW_STEPS = "shacl" |  |  |  |  |

| `simple` | str_key | REGIMES = "simple" |  |  |  |  |

| `turtle` | str_key | RDF_DIALECTS = "turtle" |  |  |  |  |

| `unattested` | str_key | RECEIPT_REASONS = "unattested" |  |  |  |  |

| `DialectRow` | struct | DialectRow { pub name: &'static str, pub aliases: &'static [&'static str], pub response_name: &'static str, pub media_type: Option<&'static str> } |  |  |  |  |

| `Field` | struct | Field { pub name: &'static str, pub ty: &'static str, pub required: bool, pub nullable: bool, pub doc: &'static str, pub enum_values: &'static [&'static str], pub default: FieldDefault } |  |  |  |  |

| `LawStepRow` | struct | LawStepRow { pub name: &'static str, pub ceiling: Option<&'static str>, pub fields: &'static [Field] } |  |  |  |  |

| `Op` | struct | Op { pub name: &'static str, pub summary: &'static str, pub request: &'static [Field], pub responses: &'static [Variant], pub refusal_kinds: &'static [&'static str], pub refusal_codes: &'static [&'static str] } |  |  |  |  |

| `RefusalCodeRow` | struct | RefusalCodeRow { pub code: &'static str, pub kind: Option<&'static str>, pub fields: &'static [Field] } |  |  |  |  |

| `Variant` | struct | Variant { pub tag: Option<&'static str>, pub tag_field: Option<&'static str>, pub fields: &'static [Field] } |  |  |  |  |

| `canonical` | function | canonical(v: &Value) -> String |  |  |  |  |

| `receipt_surface_blake3` | function | receipt_surface_blake3() -> String |  |  |  |  |

| `registry_blake3` | function | registry_blake3() -> String |  |  |  |  |

| `registry_json_pretty` | function | registry_json_pretty() -> String |  |  |  |  |

| `registry_value` | function | registry_value() -> Value |  |  |  |  |

| `surface_blake3` | function | surface_blake3() -> String |  |  |  |  |

| `Lease` | str_key | MODELS = "Lease" |  |  |  |  |

| `max_request_bytes` | str_key | LIMIT_META = "max_request_bytes" |  |  |  |  |

| `src/duckdb_eval.rs` | str_key | FILES = "src/duckdb_eval.rs" |  |  |  |  |

| `registry_turtle` | function | registry_turtle() -> String |  |  |  |  |

| `BuiltinError` | enum | BuiltinError { Eval(EvaluatorError), InvalidArguments { expected: String, got: String }, AstError(String) } |  |  |  |  |

| `builtin_eval_sql` | function | builtin_eval_sql( args: &[&str], output_vars: &[&str], ctx: &BuiltinContext, reader: &R, ) -> Result<BuiltinResult, BuiltinError> |  |  |  |  |

| `BuiltinContext` | struct | BuiltinContext { pub db: &'a GraphLawDuckDB } |  |  |  |  |

| `BuiltinResult` | struct | BuiltinResult { pub bindings: Vec<(String, TermValue)>, pub derived_triples: Vec<RdfTriple>, pub query_hash: [u8; 32], pub rows_emitted: usize } |  |  |  |  |

| `MAX_SHACL_DEGREE` | const | MAX_SHACL_DEGREE: usize |  |  |  |  |

| `MAX_SHACL_QUADS` | const | MAX_SHACL_QUADS: usize |  |  |  |  |

| `MAX_SHACL_SHAPES_BYTES` | const | MAX_SHACL_SHAPES_BYTES: usize |  |  |  |  |

| `MAX_SHACL_VALIDATION_DEPTH` | const | MAX_SHACL_VALIDATION_DEPTH: usize |  |  |  |  |

| `ShaclLimitExceeded` | enum | ShaclLimitExceeded { Bytes { observed: usize, }, Quads { observed: usize, }, Degree { observed: usize, }, Depth { observed: usize, } } |  |  |  |  |

| `cap` | function | cap(&self) -> usize |  |  |  |  |

| `check_shape_depth` | function | check_shape_depth(shapes_ttl: &str) -> Result<(), DepthExceeded> |  |  |  |  |

| `check_shape_limits` | function | check_shape_limits(shapes_ttl: &str) -> Result<(), ShaclLimitExceeded> |  |  |  |  |

| `limit_name` | function | limit_name(&self) -> &'static str |  |  |  |  |

| `observed` | function | observed(&self) -> usize |  |  |  |  |

| `and` | str_key | SHAPE_LIST_PREDICATES = "and" |  |  |  |  |

| `http://www.w3.org/1999/02/22-rdf-syntax-ns#first` | str_key | RDF_FIRST = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first" |  |  |  |  |

| `http://www.w3.org/1999/02/22-rdf-syntax-ns#rest` | str_key | RDF_REST = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest" |  |  |  |  |

| `http://www.w3.org/ns/shacl#` | str_key | SH = "http://www.w3.org/ns/shacl#" |  |  |  |  |

| `node` | str_key | SHAPE_LINK_PREDICATES = "node" |  |  |  |  |

| `DepthExceeded` | struct | DepthExceeded { pub observed: usize } |  |  |  |  |

| `DCTERMS` | const | DCTERMS: &str |  |  |  |  |

| `DFL` | const | DFL: &str |  |  |  |  |

| `SMON` | const | SMON: &str |  |  |  |  |

| `broaden` | function | broaden( ds: &RdfDataset, canonical_topic: &str, ) -> Result<(Arc<RdfDataset>, Broadened), ToolError> |  |  |  |  |

| `build_dataset` | function | build_dataset( turns: &[Turn], session_id: &str, base: &str, ) -> Result<Arc<RdfDataset>, ToolError> |  |  |  |  |

| `classify_turn` | function | classify_turn(text: &str) -> (&'static str, Vec<String>) |  |  |  |  |

| `extract_topic` | function | extract_topic(text: &str) -> String |  |  |  |  |

| `from_env` | function | from_env() -> Self |  |  |  |  |

| `read_turtle` | function | read_turtle(path: &Path) -> Result<Arc<RdfDataset>, ToolError> |  |  |  |  |

| `required` | function | required(&self, flag: &str) -> Result<&str, ToolError> |  |  |  |  |

| `switch` | function | switch(&self, flag: &str) -> bool |  |  |  |  |

| `to_turtle` | function | to_turtle(ds: &RdfDataset) -> Result<Vec<u8>, ToolError> |  |  |  |  |

| `turns_from_jsonl` | function | turns_from_jsonl(raw: &str, assistant_only: bool) -> Vec<Turn> |  |  |  |  |

| `value` | function | value(&self, flag: &str) -> Option<&str> |  |  |  |  |

| `write_turtle` | function | write_turtle(ds: &RdfDataset, out: &Path) -> Result<(), ToolError> |  |  |  |  |

| `arazzo` | str_key | TOPIC_KEYWORDS = "arazzo" |  |  |  |  |

| `http://purl.org/dc/terms/` | str_key | DCTERMS = "http://purl.org/dc/terms/" |  |  |  |  |

| `http://seanchatmangpt.github.io/packs/dogfood-lifecycle#` | str_key | DFL = "http://seanchatmangpt.github.io/packs/dogfood-lifecycle#" |  |  |  |  |

| `http://seanchatmangpt.github.io/packs/self-monitoring#` | str_key | SMON = "http://seanchatmangpt.github.io/packs/self-monitoring#" |  |  |  |  |

| `http://www.w3.org/1999/02/22-rdf-syntax-ns#type` | str_key | RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" |  |  |  |  |

| `http://www.w3.org/2001/XMLSchema#integer` | str_key | XSD_INTEGER = "http://www.w3.org/2001/XMLSchema#integer" |  |  |  |  |

| `the` | str_key | STOPWORDS = "the" |  |  |  |  |

| `Args` | struct | Args { Vec<String> } |  |  |  |  |

| `Broadened` | struct | Broadened { pub audit: Vec<String>, pub rewrite1: usize, pub rewrite2: usize } |  |  |  |  |

| `Turn` | struct | Turn { pub seq: usize, pub role: &'static str, pub text: String, pub kind: &'static str, pub evidence: Vec<String>, pub topic: String } |  |  |  |  |

| `CompositeKind` | enum | CompositeKind { Struct, List, Index, Slice } |  |  |  |  |

| `compile_turtle` | function | compile_turtle(turtle: &str) -> Result<CompiledQuery, AstError> |  |  |  |  |

| `compile_turtle_document` | function | compile_turtle_document(turtle: &str) -> Result<CompiledDocument, AstError> |  |  |  |  |

| `validate_ast_shacl` | function | validate_ast_shacl(turtle_dataset: &str, shapes_ttl: &str) -> Result<(), AstError> |  |  |  |  |

| ` ;` | str_key | WHERE = " ;" |  |  |  |  |

| `@prefix sql: <http://graphlaw.org/sql#> .` | str_key | SELECT_HEAD = "@prefix sql: <http://graphlaw.org/sql#> ." |  |  |  |  |

| `[ sql:windowFunction [ sql:functionName ` | str_key | ROW_NUMBER_WINDOW = "[ sql:windowFunction [ sql:functionName " |  |  |  |  |

| `http://graphlaw.org/sql#` | str_key | SQL_NS = "http://graphlaw.org/sql#" |  |  |  |  |

| `http://graphlaw.org/sql#Columns` | str_key | COLUMNS = "http://graphlaw.org/sql#Columns" |  |  |  |  |

| `http://graphlaw.org/sql#Star` | str_key | STAR = "http://graphlaw.org/sql#Star" |  |  |  |  |

| `http://www.w3.org/1999/02/22-rdf-syntax-ns#type` | str_key | RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" |  |  |  |  |

| `http://www.w3.org/2001/XMLSchema#boolean` | str_key | XSD_BOOLEAN = "http://www.w3.org/2001/XMLSchema#boolean" |  |  |  |  |

| `statementIndex` | str_key | KNOWN_SQL_PREDICATES = "statementIndex" |  |  |  |  |

| `sum` | str_key | AGGREGATES = "sum" |  |  |  |  |

| `RdfAstGraphReader` | trait |  |  |  |  |  |

| `ast::*` | use | ast::* |  |  |  |  |

| `refusals::*` | use | refusals::* |  |  |  |  |

| `refusals::refusal_arm` | use | refusals::refusal_arm |  |  |  |  |

| `serializer::*` | use | serializer::* |  |  |  |  |

| `AggregateFunction` | enum | AggregateFunction { Count, Sum, Avg, Min, Max, RowNumber } |  |  |  |  |

| `FrameBound` | enum | FrameBound { UnboundedPreceding, Preceding(u64), CurrentRow, Following(u64), UnboundedFollowing } |  |  |  |  |

| `FrameUnit` | enum | FrameUnit { Rows, Range } |  |  |  |  |

| `JoinType` | enum | JoinType { Inner, LeftOuter } |  |  |  |  |

| `Operator` | enum | Operator { Equals, NotEquals, GreaterThan, GreaterThanOrEq, LessThan, LessThanOrEq, And, Or, Like, ILike, Add, Subtract, Multiply, Divide, Modulo, JsonExtract, JsonExtractText } |  |  |  |  |

| `Scope` | enum | Scope { View, Process } |  |  |  |  |

| `ScopePos` | enum | ScopePos { SelectList, OrderBy, Having, Predicate, GroupBy, AggregateArgument, PredicateAggregateArgument, Qualify } |  |  |  |  |

| `SqlExpr` | enum | SqlExpr { ColumnRef { column: String, table_alias: Option<String>, }, Literal(SqlLiteral), Binary { op: Operator, left: Box<SqlExpr>, right: Box<SqlExpr>, }, FunctionCall { function: AggregateFunction, argument: Box<SqlExpr>, }, WindowFn { function: Box<SqlExpr>, partition_by: Vec<SqlExpr>, order_by: Vec<OrderTerm>, frame: Option<FrameSpec>, }, Alias { expr: Box<SqlExpr>, alias: String, }, StructLiteral { fields: Vec<(String, SqlExpr)>, }, ListLiteral { items: Vec<SqlExpr> }, IndexAccess { base: Box<SqlExpr>, index: Box<SqlExpr>, }, SliceAccess { base: Box<SqlExpr>, start: Box<SqlExpr>, end: Option<Box<SqlExpr>>, } } |  |  |  |  |

| `SqlLiteral` | enum | SqlLiteral { String(String), Integer(i64), Float(f64), Boolean(bool), Null } |  |  |  |  |

| `as_sql` | function | as_sql(&self) -> &'static str |  |  |  |  |

| `compile_merge` | function | compile_merge(&self) -> Result<CompiledQuery, AstError> |  |  |  |  |

| `from_iri` | function | from_iri(iri: &str) -> Result<Self, AstError> |  |  |  |  |

| `from_token` | function | from_token(token: &str) -> Result<Self, AstError> |  |  |  |  |

| `parse_token` | function | parse_token(token: &str) -> Result<Self, AstError> |  |  |  |  |

| `seal_bytes` | function | seal_bytes(&self) -> [u8; 32] |  |  |  |  |

| `validate_composite` | function | validate_composite(&self) -> Result<(), AstError> |  |  |  |  |

| `validate_identifier` | function | validate_identifier(ident: &str) -> Result<&str, AstError> |  |  |  |  |

| `validate_qualified_identifier` | function | validate_qualified_identifier(ident: &str) -> Result<(), AstError> |  |  |  |  |

| `verify_seal` | function | verify_seal(&self) -> bool |  |  |  |  |

| `CompiledDocument` | struct | CompiledDocument { pub statements: Vec<CompiledQuery>, pub doc_hash: [u8; 32], pub atomic: bool } |  |  |  |  |

| `CompiledQuery` | struct | CompiledQuery { pub sql: String, pub params: Vec<SqlLiteral>, pub ast_graph_turtle: Option<String>, pub canon_graph_bytes: Option<Vec<u8>>, pub receipt_state: Option<[u8; 32]>, pub cacheable: bool, pub(super) admission_seal: [u8; 32] } |  |  |  |  |

| `Cte` | struct | Cte { pub name: String, pub columns: Vec<String>, pub query: Box<SelectStatement>, pub recursive: bool } |  |  |  |  |

| `FrameSpec` | struct | FrameSpec { pub unit: FrameUnit, pub start: FrameBound, pub end: Option<FrameBound> } |  |  |  |  |

| `JoinClause` | struct | JoinClause { pub join_type: JoinType, pub table: TableRef, pub condition: SqlExpr } |  |  |  |  |

| `MergeStatement` | struct | MergeStatement { pub target: TableRef, pub source: SelectStatement, pub on: SqlExpr, pub assignments: Vec<(String, SqlExpr)> } |  |  |  |  |

| `OrderTerm` | struct | OrderTerm { pub expr: SqlExpr, pub descending: bool } |  |  |  |  |

| `TableRef` | struct | TableRef { pub name: String, pub alias: Option<String>, pub schema: Option<String> } |  |  |  |  |

| `AGG_IN_POSITION` | const | AGG_IN_POSITION: u16 |  |  |  |  |

| `AGG_IN_PREDICATE` | const | AGG_IN_PREDICATE: u16 |  |  |  |  |

| `ALIASED_AGG_ARG` | const | ALIASED_AGG_ARG: u16 |  |  |  |  |

| `ALIAS_IN_AGG_ARG` | const | ALIAS_IN_AGG_ARG: u16 |  |  |  |  |

| `ATOMIC_LITERAL` | const | ATOMIC_LITERAL: u16 |  |  |  |  |

| `CANONICALIZATION` | const | CANONICALIZATION: u16 |  |  |  |  |

| `COLUMNS_LAMBDA` | const | COLUMNS_LAMBDA: u16 |  |  |  |  |

| `COLUMNS_PATTERN` | const | COLUMNS_PATTERN: u16 |  |  |  |  |

| `COMPOSITE_DEPTH` | const | COMPOSITE_DEPTH: u16 |  |  |  |  |

| `COMPOSITE_MEMBER` | const | COMPOSITE_MEMBER: u16 |  |  |  |  |

| `CTE_NO_QUERY` | const | CTE_NO_QUERY: u16 |  |  |  |  |

| `DIRECTION_IRI` | const | DIRECTION_IRI: u16 |  |  |  |  |

| `DIRECTION_LEX` | const | DIRECTION_LEX: u16 |  |  |  |  |

| `DISCRIMINATORS` | const | DISCRIMINATORS: u16 |  |  |  |  |

| `DUAL_FROM` | const | DUAL_FROM: u16 |  |  |  |  |

| `DUAL_HAVING` | const | DUAL_HAVING: u16 |  |  |  |  |

| `DUAL_LEFT_OPERAND` | const | DUAL_LEFT_OPERAND: u16 |  |  |  |  |

| `DUAL_OPERATOR_FORMS` | const | DUAL_OPERATOR_FORMS: u16 |  |  |  |  |

| `DUAL_RIGHT_OPERAND` | const | DUAL_RIGHT_OPERAND: u16 |  |  |  |  |

| `DUAL_SELECT` | const | DUAL_SELECT: u16 |  |  |  |  |

| `DUAL_WHERE` | const | DUAL_WHERE: u16 |  |  |  |  |

| `DUP_CLAUSE_PREDICATE` | const | DUP_CLAUSE_PREDICATE: u16 |  |  |  |  |

| `EMPTY_FROM` | const | EMPTY_FROM: u16 |  |  |  |  |

| `EMPTY_PROJECTION` | const | EMPTY_PROJECTION: u16 |  |  |  |  |

| `EXPRESSION_WRAPPER` | const | EXPRESSION_WRAPPER: u16 |  |  |  |  |

| `FRAME_BOUND` | const | FRAME_BOUND: u16 |  |  |  |  |

| `FRAME_UNIT` | const | FRAME_UNIT: u16 |  |  |  |  |

| `INDEX_BAD_DTYPE` | const | INDEX_BAD_DTYPE: u16 |  |  |  |  |

| `INDEX_DUP` | const | INDEX_DUP: u16 |  |  |  |  |

| `INDEX_MISSING` | const | INDEX_MISSING: u16 |  |  |  |  |

| `INDEX_NEGATIVE` | const | INDEX_NEGATIVE: u16 |  |  |  |  |

| `INDEX_NOT_INT` | const | INDEX_NOT_INT: u16 |  |  |  |  |

| `INDEX_SEQUENCE_DISORDER` | const | INDEX_SEQUENCE_DISORDER: u16 |  |  |  |  |

| `JOIN_TYPE_IRI` | const | JOIN_TYPE_IRI: u16 |  |  |  |  |

| `LEGACY_MESSAGE` | const | LEGACY_MESSAGE: u16 |  |  |  |  |

| `LIMIT_DUP` | const | LIMIT_DUP: u16 |  |  |  |  |

| `LIMIT_MALFORMED` | const | LIMIT_MALFORMED: u16 |  |  |  |  |

| `LIMIT_NOT_INT` | const | LIMIT_NOT_INT: u16 |  |  |  |  |

| `MERGE_ON_NO_TARGET_COLUMN` | const | MERGE_ON_NO_TARGET_COLUMN: u16 |  |  |  |  |

| `MIXED_DOCUMENT_ROOTS` | const | MIXED_DOCUMENT_ROOTS: u16 |  |  |  |  |

| `NO_FROM` | const | NO_FROM: u16 |  |  |  |  |

| `NO_PROJECTION` | const | NO_PROJECTION: u16 |  |  |  |  |

| `NO_SQL_TEMPLATE` | const | NO_SQL_TEMPLATE: u16 |  |  |  |  |

| `OFFSET_DUP` | const | OFFSET_DUP: u16 |  |  |  |  |

| `OFFSET_MALFORMED` | const | OFFSET_MALFORMED: u16 |  |  |  |  |

| `OFFSET_NOT_INT` | const | OFFSET_NOT_INT: u16 |  |  |  |  |

| `OPERATOR_TOKEN` | const | OPERATOR_TOKEN: u16 |  |  |  |  |

| `PARSE_DATASET` | const | PARSE_DATASET: u16 |  |  |  |  |

| `PARSE_SHAPES` | const | PARSE_SHAPES: u16 |  |  |  |  |

| `PROCESS_SCOPE_IN_SELECT` | const | PROCESS_SCOPE_IN_SELECT: u16 |  |  |  |  |

| `QUALIFY_EXPR` | const | QUALIFY_EXPR: u16 |  |  |  |  |

| `REFUSAL_01_MALFORMED_AST` | const | REFUSAL_01_MALFORMED_AST: &str |  |  |  |  |

| `REFUSAL_02_UNIMPLEMENTED` | const | REFUSAL_02_UNIMPLEMENTED: &str |  |  |  |  |

| `REFUSAL_03_SCOPE_INTEGRITY` | const | REFUSAL_03_SCOPE_INTEGRITY: &str |  |  |  |  |

| `REFUSAL_05_UNBOUNDED_MUTATION` | const | REFUSAL_05_UNBOUNDED_MUTATION: &str |  |  |  |  |

| `REFUSAL_IDENT_BYTES` | const | REFUSAL_IDENT_BYTES: usize |  |  |  |  |

| `SCOPE_CARDINALITY` | const | SCOPE_CARDINALITY: u16 |  |  |  |  |

| `SCOPE_INDIVIDUAL` | const | SCOPE_INDIVIDUAL: u16 |  |  |  |  |

| `SCOPE_IN_PRED_AGG_ARG` | const | SCOPE_IN_PRED_AGG_ARG: u16 |  |  |  |  |

| `SCOPE_ON_TABLE_REF` | const | SCOPE_ON_TABLE_REF: u16 |  |  |  |  |

| `SCOPE_WRONG_NODE` | const | SCOPE_WRONG_NODE: u16 |  |  |  |  |

| `SEQUENCE_EMPTY` | const | SEQUENCE_EMPTY: u16 |  |  |  |  |

| `SET_OP_BY_NAME_KIND` | const | SET_OP_BY_NAME_KIND: u16 |  |  |  |  |

| `SET_OP_FLAG_LEX` | const | SET_OP_FLAG_LEX: u16 |  |  |  |  |

| `SET_OP_KIND_TOKEN` | const | SET_OP_KIND_TOKEN: u16 |  |  |  |  |

| `SET_OP_NO_RIGHT` | const | SET_OP_NO_RIGHT: u16 |  |  |  |  |

| `SHACL_DEPTH` | const | SHACL_DEPTH: u16 |  |  |  |  |

| `SHACL_ENGINE` | const | SHACL_ENGINE: u16 |  |  |  |  |

| `SHACL_VIOLATIONS` | const | SHACL_VIOLATIONS: u16 |  |  |  |  |

| `SQL_ALIAS_IN_AGG_ARG` | const | SQL_ALIAS_IN_AGG_ARG: u16 |  |  |  |  |

| `STAR_MODIFIER_POSITION` | const | STAR_MODIFIER_POSITION: u16 |  |  |  |  |

| `UNBOUNDED_MUTATION` | const | UNBOUNDED_MUTATION: u16 |  |  |  |  |

| `UNIMPLEMENTED_STATEMENT` | const | UNIMPLEMENTED_STATEMENT: u16 |  |  |  |  |

| `UNKNOWN_STATEMENT_PREDICATE` | const | UNKNOWN_STATEMENT_PREDICATE: u16 |  |  |  |  |

| `UNSUPPORTED_GENERIC` | const | UNSUPPORTED_GENERIC: u16 |  |  |  |  |

| `VIEW_SCOPE_IN_PREDICATE` | const | VIEW_SCOPE_IN_PREDICATE: u16 |  |  |  |  |

| `WINDOW_FN_POSITION` | const | WINDOW_FN_POSITION: u16 |  |  |  |  |

| `RefusalCode` | enum | RefusalCode { MalformedAst = 1, UnimplementedStatement = 2, ScopeIntegrity = 3, Capability = 4, InvalidIdentifier = 16, MissingProperty = 17, RecursionLimitExceeded = 18, MalformedList = 19, UnsupportedTerm = 20, MalformedNode = 21, LiteralNotInt = 36, LiteralNotFloat = 37, UnboundedMutation = 5 } |  |  |  |  |

| `InvalidIdentifier` | function | InvalidIdentifier(a: String) -> Self |  |  |  |  |

| `MalformedNode` | function | MalformedNode(a: String) -> Self |  |  |  |  |

| `MissingProperty` | function | MissingProperty(a: String, b: String) -> Self |  |  |  |  |

| `RecursionLimitExceeded` | function | RecursionLimitExceeded(n: usize) -> Self |  |  |  |  |

| `ShaclViolation` | function | ShaclViolation(msg: String) -> Self |  |  |  |  |

| `UnsupportedTerm` | function | UnsupportedTerm(a: String) -> Self |  |  |  |  |

| `as_str_lossy` | function | as_str_lossy(&self) -> std::borrow::Cow<'_, str> |  |  |  |  |

| `by_marker` | function | by_marker(name: &str) -> Option<u16> |  |  |  |  |

| `marker` | function | marker(self) -> Option<&'static str> |  |  |  |  |

| `pack` | function | pack(v: impl fmt::Display) -> Self |  |  |  |  |

| `packed2` | function | packed2(a: impl fmt::Display, b: impl fmt::Display) -> Self |  |  |  |  |

| `push` | function | push(&mut self, v: impl fmt::Display) |  |  |  |  |

| `[REFUSAL-01] MalformedAstRefusal:` | str_key | REFUSAL_01_MALFORMED_AST = "[REFUSAL-01] MalformedAstRefusal:" |  |  |  |  |

| `[REFUSAL-02] UnimplementedStatementRefusal:` | str_key | REFUSAL_02_UNIMPLEMENTED = "[REFUSAL-02] UnimplementedStatementRefusal:" |  |  |  |  |

| `[REFUSAL-03] ScopeIntegrityRefusal:` | str_key | REFUSAL_03_SCOPE_INTEGRITY = "[REFUSAL-03] ScopeIntegrityRefusal:" |  |  |  |  |

| `[REFUSAL-05] UnboundedMutationRefusal:` | str_key | REFUSAL_05_UNBOUNDED_MUTATION = "[REFUSAL-05] UnboundedMutationRefusal:" |  |  |  |  |

| `AstError` | struct | AstError { pub code: RefusalCode, pub ctx: RefusalContext } |  |  |  |  |

| `PackedIdent` | struct | PackedIdent { len: u8, bytes: [u8; REFUSAL_IDENT_BYTES] } |  |  |  |  |

| `RefusalContext` | struct | RefusalContext { pub shape_id: u16, pub predicate_idx: u16, pub num: u32, pub aux_num: u32, pub ident: PackedIdent, pub aux_ident: PackedIdent } |  |  |  |  |

| `SelectItem` | enum | SelectItem { Expr(SqlExpr), Star { exclude: Vec<String>, replace: Vec<(SqlExpr, String)>, }, Columns(String) } |  |  |  |  |

| `SetOpKind` | enum | SetOpKind { Union, Intersect, Except } |  |  |  |  |

| `as_sql` | function | as_sql(&self) -> &'static str |  |  |  |  |

| `compile` | function | compile(&self) -> Result<CompiledQuery, AstError> |  |  |  |  |

| `compile_set_operation` | function | compile_set_operation(&self) -> Result<CompiledQuery, AstError> |  |  |  |  |

| `from_token` | function | from_token(token: &str) -> Result<Self, AstError> |  |  |  |  |

| `SelectStatement` | struct | SelectStatement { pub ctes: Vec<Cte>, pub from: TableRef, pub joins: Vec<JoinClause>, pub select_list: Vec<SelectItem>, pub where_clause: Option<SqlExpr>, pub group_by: Vec<SqlExpr>, pub having: Option<SqlExpr>, pub order_by: Vec<OrderTerm>, pub limit: Option<usize>, pub offset: Option<usize>, pub qualify: Option<SqlExpr> } |  |  |  |  |

| `SetOperationStatement` | struct | SetOperationStatement { pub left: SelectStatement, pub kind: SetOpKind, pub all: bool, pub by_name: bool, pub right: SelectStatement } |  |  |  |  |

| `REFUSAL_04_CAPABILITY_REFUSAL` | const | REFUSAL_04_CAPABILITY_REFUSAL: &str |  |  |  |  |

| `Admitted` | enum | Admitted { Dataset(Arc<purrdf::RdfDataset>), Rules(RuleSet), Query(crate::sql_ast::CompiledQuery) } |  |  |  |  |

| `Tier` | enum | Tier { Rdf, N3Logic, Relational } |  |  |  |  |

| `admit_bytes_tiered` | function | admit_bytes_tiered( bytes: &[u8], hint: Option<&str>, base: Option<&str>, decl: Option<DialectDecl>, ) -> Result<Admitted, Refusal> |  |  |  |  |

| `admit_declared` | function | admit_declared( bytes: &[u8], decl: DialectDecl, base: Option<&str>, ) -> Result<Admitted, Refusal> |  |  |  |  |

| `rule_count` | function | rule_count(&self) -> usize |  |  |  |  |

| `tier` | function | tier(self) -> Tier |  |  |  |  |

| `validate_n3_rules` | function | validate_n3_rules(bytes: &[u8], base: Option<&str>) -> Result<RuleSet, Refusal> |  |  |  |  |

| `REFUSAL-04` | str_key | REFUSAL_04_CAPABILITY_REFUSAL = "REFUSAL-04" |  |  |  |  |

| `DialectDecl` | struct | DialectDecl { pub tier: Tier, pub dialect: Dialect } |  |  |  |  |

| `RuleSet` | struct | RuleSet { pub document: n3::Document } |  |  |  |  |

| `0807060504030201887766554433221100ff0000000000000100000002000000030000000400000005000000060000000700000008000000bebafecaefbeaddeefbe2aa5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5` | str_key | GOLDEN_FRAME_HASH_HEX = "0807060504030201887766554433221100ff0000000000000100000002000000030000000400000005000000060000000700000008000000bebafecaefbeaddeefbe2aa5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5" |  |  |  |  |

| `/etc/passwd` | str_key | ATTACK_PATH = "/etc/passwd" |  |  |  |  |

| `attach path outside GRAPHLAW_ATTACH_ROOTS allowlist` | str_key | STABLE_REFUSAL = "attach path outside GRAPHLAW_ATTACH_ROOTS allowlist" |  |  |  |  |

| `SUB-A` | str_key | SUBSIDIARIES = "SUB-A" |  |  |  |  |

| `bcbs-01` | str_key | SESSION = "bcbs-01" |  |  |  |  |

| `bcbs-02` | str_key | SESSION = "bcbs-02" |  |  |  |  |

| `bcbs-03` | str_key | SESSION = "bcbs-03" |  |  |  |  |

| `<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .` | str_key | PURE_NT = "<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> ." |  |  |  |  |

| `<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> <http://example.org/g> .` | str_key | DATASET_NQ = "<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> <http://example.org/g> ." |  |  |  |  |

| `_:p <http://example.org/knows> _:q .` | str_key | BNODE_NT_RELABELED = "_:p <http://example.org/knows> _:q ." |  |  |  |  |

| `_:x <http://example.org/knows> _:y .` | str_key | BNODE_NT = "_:x <http://example.org/knows> _:y ." |  |  |  |  |

| `@prefix sh: <http://www.w3.org/ns/shacl#> .` | str_key | PREFIX = "@prefix sh: <http://www.w3.org/ns/shacl#> ." |  |  |  |  |

| `https://chicago.graphlaw.dev/` | str_key | NS = "https://chicago.graphlaw.dev/" |  |  |  |  |

| `MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAt9+KeDkWJa7SiBb7AMdXAPV2qpRunztoC2c7DtRmN9+Lq2YQm4rMQmmAQcwyxTxcK8vwPKOiE4mLMwmQMvTty9tL+buREfvg5KJ+FY3Bv05cZYsf7VuYgUDcw41zSEE5W1thh5PiafcrMn7CPF0jN0oFfVvAHB/fAM0yaZGPKs3WYqnHAHa6v/cVMeiWZ/lQORCBqqL2hii1fEiuIqw6AOEoZVsN0/lqm7GfnD899zrzLOPRuP4YFg46drL1UyNdg+4TquJ2OV9kR4CAfdGRexqHFWLmoxkTfnUxJ4yL+gkV57K1H+smEdKrDhDX68pEFM/34E2eVKZoRU/0paxDZQIDAQAB` | str_key | KEY = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAt9+KeDkWJa7SiBb7AMdXAPV2qpRunztoC2c7DtRmN9+Lq2YQm4rMQmmAQcwyxTxcK8vwPKOiE4mLMwmQMvTty9tL+buREfvg5KJ+FY3Bv05cZYsf7VuYgUDcw41zSEE5W1thh5PiafcrMn7CPF0jN0oFfVvAHB/fAM0yaZGPKs3WYqnHAHa6v/cVMeiWZ/lQORCBqqL2hii1fEiuIqw6AOEoZVsN0/lqm7GfnD899zrzLOPRuP4YFg46drL1UyNdg+4TquJ2OV9kR4CAfdGRexqHFWLmoxkTfnUxJ4yL+gkV57K1H+smEdKrDhDX68pEFM/34E2eVKZoRU/0paxDZQIDAQAB" |  |  |  |  |

| `[ a sql:BinaryExpression ; sql:operator ` | str_key | AGGREGATE_WHERE = "[ a sql:BinaryExpression ; sql:operator " |  |  |  |  |

| `http://example.org/crs/planar` | str_key | CRS_PLANAR = "http://example.org/crs/planar" |  |  |  |  |

| `http://example.org/geo#` | str_key | GEO = "http://example.org/geo#" |  |  |  |  |

| `http://example.org/geof/` | str_key | GEOF = "http://example.org/geof/" |  |  |  |  |

| `http://example.org/sf#` | str_key | SF = "http://example.org/sf#" |  |  |  |  |

| `http://example.org/unit/metre` | str_key | METRE = "http://example.org/unit/metre" |  |  |  |  |

| `CALIBRATION_FLOOR` | const | CALIBRATION_FLOOR: u64 |  |  |  |  |

| `CALIBRATION_SEED` | const | CALIBRATION_SEED: u64 |  |  |  |  |

| `EPOCH_DISARMED_TICKS` | const | EPOCH_DISARMED_TICKS: u64 |  |  |  |  |

| `UNBOUNDED_FUEL` | const | UNBOUNDED_FUEL: u64 |  |  |  |  |

| `WALL_BYTES` | const | WALL_BYTES: u64 |  |  |  |  |

| `WATCHDOG_INTERVAL` | const | WATCHDOG_INTERVAL: Duration |  |  |  |  |

| `EnclosureError` | enum | EnclosureError { Fuel { budget: u64 }, Epoch { ticks: u64 }, Trap(String), Fatal(String) } |  |  |  |  |

| `FuelPolicy` | enum | FuelPolicy { Unbounded, PerOp(BTreeMap<String, u64>) } |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `alloc_raw` | function | alloc_raw(&mut self, len: u32) -> Option<u32> |  |  |  |  |

| `artifact_path` | function | artifact_path() -> PathBuf |  |  |  |  |

| `budget` | function | budget(&self) -> Option<u64> |  |  |  |  |

| `budget_for` | function | budget_for(&self, op: &str) -> Option<u64> |  |  |  |  |

| `calibrate` | function | calibrate(&mut self, control: &Value) -> (u64, u64) |  |  |  |  |

| `call_raw` | function | call_raw(&mut self, ptr: u32, len: u32) -> (u32, u32) |  |  |  |  |

| `exec` | function | exec(&mut self, req: &Value) -> Value |  |  |  |  |

| `exec_seeded` | function | exec_seeded(&mut self, req: &Value, fuel_seed: u64) -> (Value, u64) |  |  |  |  |

| `exec_seeded_bytes` | function | exec_seeded_bytes(&mut self, req: &Value, fuel_seed: u64) -> (Vec<u8>, u64) |  |  |  |  |

| `free_raw` | function | free_raw(&mut self, ptr: u32, len: u32) |  |  |  |  |

| `grow_memory` | function | grow_memory(&mut self, delta_pages: u64) -> Result<u64, wasmtime::Error> |  |  |  |  |

| `limit_name` | function | limit_name(&self) -> Option<&'static str> |  |  |  |  |

| `memory_max_pages` | function | memory_max_pages(&mut self) -> Option<u64> |  |  |  |  |

| `memory_pages` | function | memory_pages(&mut self) -> u64 |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `next_pow2_ceil` | function | next_pow2_ceil(x: u64) -> u64 |  |  |  |  |

| `peak_wall_bytes` | function | peak_wall_bytes(&self) -> u64 |  |  |  |  |

| `peek_out` | function | peek_out(&mut self, ptr: u32, len: u32) -> Vec<u8> |  |  |  |  |

| `probe` | function | probe(&mut self, req: &Value) -> Result<Value, EnclosureError> |  |  |  |  |

| `read_out` | function | read_out(&mut self, ptr: u32, len: u32) -> Vec<u8> |  |  |  |  |

| `request_envelope` | function | request_envelope(&mut self, req: &Value) -> Vec<u8> |  |  |  |  |

| `request_envelope_leaking` | function | request_envelope_leaking(&mut self, req: &Value) -> Vec<u8> |  |  |  |  |

| `set_epoch_ticks_per_request` | function | set_epoch_ticks_per_request(&mut self, ticks: u64) |  |  |  |  |

| `synthesized_envelope` | function | synthesized_envelope(e: &EnclosureError) -> Vec<u8> |  |  |  |  |

| `write_body` | function | write_body(&mut self, ptr: u32, body: &[u8]) |  |  |  |  |

| `EnclosureHost` | struct | EnclosureHost { store: Store<EncState>, memory: Memory, alloc_fn: TypedFunc<u32, u32>, free_fn: TypedFunc<(u32, u32), ()>, call_fn: TypedFunc<(u32, u32), u64>, policy: FuelPolicy, epoch_ticks: u64, _watchdog: Watchdog } |  |  |  |  |

| `WallLimiter` | struct | WallLimiter { pub peak: u64 } |  |  |  |  |

| `ENGINE_LOCK` | const | ENGINE_LOCK: Mutex<()> |  |  |  |  |

| `HostError` | enum | HostError { Trap(String), Fatal(String) } |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `CARGO_TARGET_DIR` | env_key | std::env::var_os("CARGO_TARGET_DIR") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `close_session` | function | close_session(h: &mut Host, name: &str) |  |  |  |  |

| `engine_lock` | function | engine_lock() -> std::sync::MutexGuard<'static, ()> |  |  |  |  |

| `exec` | function | exec(&mut self, req: &Value) -> Value |  |  |  |  |

| `fail_classify` | function | fail_classify(stage: &str, e: wasmtime::Error) -> HostError |  |  |  |  |

| `host_or_skip` | function | host_or_skip(test_name: &str, preopen_tmp: bool) -> Option<Host> |  |  |  |  |

| `native` | function | native(req: &Value) -> Value |  |  |  |  |

| `native_bytes` | function | native_bytes(req: &Value) -> Vec<u8> |  |  |  |  |

| `new` | function | new(bytes: &[u8], preopen_tmp: bool) -> Result<Self, HostError> |  |  |  |  |

| `ok` | function | ok(req: &Value) -> Value |  |  |  |  |

| `open_session` | function | open_session(h: &mut Host, name: &str) |  |  |  |  |

| `read` | function | read(path: &str) -> String |  |  |  |  |

| `refused` | function | refused(req: &Value) -> Value |  |  |  |  |

| `request` | function | request(&mut self, req: &Value) -> Result<Value, HostError> |  |  |  |  |

| `request_bytes` | function | request_bytes(&mut self, body: &[u8]) -> Result<Vec<u8>, HostError> |  |  |  |  |

| `stream_all` | function | stream_all(&mut self, session: &str, cursor: &str) -> String |  |  |  |  |

| `try_host` | function | try_host(preopen_tmp: bool) -> Result<Host, String> |  |  |  |  |

| `wasm` | function | wasm(req: &Value) -> Value |  |  |  |  |

| `wasm_bytes` | function | wasm_bytes(req: &Value) -> Vec<u8> |  |  |  |  |

| `wasm_path` | function | wasm_path() -> PathBuf |  |  |  |  |

| `Host` | struct | Host { store: Store<WasiP1Ctx>, memory: Memory, alloc: TypedFunc<u32, u32>, free: TypedFunc<(u32, u32), ()>, call: TypedFunc<(u32, u32), u64> } |  |  |  |  |

| `CARGO_BIN_EXE_{name}` | env_key | std::env::var("CARGO_BIN_EXE_{name}") |  |  |  |  |

| `UPDATE_ASSET_MANIFEST` | env_key | std::env::var_os("UPDATE_ASSET_MANIFEST") |  |  |  |  |

| `https://example.org/base/` | str_key | BASE = "https://example.org/base/" |  |  |  |  |

| `ontologies` | str_key | ROOTS = "ontologies" |  |  |  |  |

| `248cd97b412685441df24bf44b52461a001f8ee8c70bd0a2c8a16e41b151a324` | str_key | GOLDEN_H_TRIPLES_SERTERM = "248cd97b412685441df24bf44b52461a001f8ee8c70bd0a2c8a16e41b151a324" |  |  |  |  |

| `4d51ca3c8cf5bc73780cb8cd3bed526293fa6b3fe34c5ad10cf3f8e8d14e8ff4` | str_key | GOLDEN_H_COLUMNAR_CANON = "4d51ca3c8cf5bc73780cb8cd3bed526293fa6b3fe34c5ad10cf3f8e8d14e8ff4" |  |  |  |  |

| `CREATE TABLE revenue (region TEXT, v BIGINT);` | str_key | P04_SEED_SQL = "CREATE TABLE revenue (region TEXT, v BIGINT);" |  |  |  |  |

| `GLAW_P04_PROBE_CHILD` | str_key | ENV_CHILD = "GLAW_P04_PROBE_CHILD" |  |  |  |  |

| `P04_PROBE_BEGIN` | str_key | PROBE_BEGIN = "P04_PROBE_BEGIN" |  |  |  |  |

| `P04_PROBE_END` | str_key | PROBE_END = "P04_PROBE_END" |  |  |  |  |

| `http://www.w3.org/2001/XMLSchema#integer` | str_key | XSD_INT = "http://www.w3.org/2001/XMLSchema#integer" |  |  |  |  |

| `urn:p04:obj:00` | str_key | OBJ_IDS = "urn:p04:obj:00" |  |  |  |  |

| `https://example.org/` | str_key | NS = "https://example.org/" |  |  |  |  |

| `/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw` | str_key | OUT_DIR = "/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw" |  |  |  |  |

| `a-paper-baader-el` | str_key | PINS = "a-paper-baader-el" |  |  |  |  |

| `graphlaw_rev` | str_key | REV_LINE = "graphlaw_rev" |  |  |  |  |

| `MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAt9+KeDkWJa7SiBb7AMdXAPV2qpRunztoC2c7DtRmN9+Lq2YQm4rMQmmAQcwyxTxcK8vwPKOiE4mLMwmQMvTty9tL+buREfvg5KJ+FY3Bv05cZYsf7VuYgUDcw41zSEE5W1thh5PiafcrMn7CPF0jN0oFfVvAHB/fAM0yaZGPKs3WYqnHAHa6v/cVMeiWZ/lQORCBqqL2hii1fEiuIqw6AOEoZVsN0/lqm7GfnD899zrzLOPRuP4YFg46drL1UyNdg+4TquJ2OV9kR4CAfdGRexqHFWLmoxkTfnUxJ4yL+gkV57K1H+smEdKrDhDX68pEFM/34E2eVKZoRU/0paxDZQIDAQAB` | str_key | EXPR_KEY = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAt9+KeDkWJa7SiBb7AMdXAPV2qpRunztoC2c7DtRmN9+Lq2YQm4rMQmmAQcwyxTxcK8vwPKOiE4mLMwmQMvTty9tL+buREfvg5KJ+FY3Bv05cZYsf7VuYgUDcw41zSEE5W1thh5PiafcrMn7CPF0jN0oFfVvAHB/fAM0yaZGPKs3WYqnHAHa6v/cVMeiWZ/lQORCBqqL2hii1fEiuIqw6AOEoZVsN0/lqm7GfnD899zrzLOPRuP4YFg46drL1UyNdg+4TquJ2OV9kR4CAfdGRexqHFWLmoxkTfnUxJ4yL+gkV57K1H+smEdKrDhDX68pEFM/34E2eVKZoRU/0paxDZQIDAQAB" |  |  |  |  |

| `core_functions` | str_key | CERTIFIED = "core_functions" |  |  |  |  |

| `dvp-01` | str_key | SESSION = "dvp-01" |  |  |  |  |

| `dvp-02` | str_key | SESSION = "dvp-02" |  |  |  |  |

| `dvp-03` | str_key | SESSION = "dvp-03" |  |  |  |  |

| `GRAPHLAW_RUN_EH` | env_key | std::env::var_os("GRAPHLAW_RUN_EH") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `target/debug/graphlaw-run-eh` | str_key | DEFAULT_BIN = "target/debug/graphlaw-run-eh" |  |  |  |  |

| `target/wasm32-wasip1/wasm/graphlaw_wasm.wasm` | str_key | DEFAULT_ARTIFACT = "target/wasm32-wasip1/wasm/graphlaw_wasm.wasm" |  |  |  |  |

| `COLDCHAIN_CSV_DIR` | env_key | std::env::var("COLDCHAIN_CSV_DIR") |  |  |  |  |

| `DUCKDB_BIN` | env_key | std::env::var("DUCKDB_BIN") |  |  |  |  |

| `6971c912530619217b462f0b458a390eec34383a8155e0ccfcce869e4b6aeb8e` | str_key | COLD1_AUDIT_HASH = "6971c912530619217b462f0b458a390eec34383a8155e0ccfcce869e4b6aeb8e" |  |  |  |  |

| `BATCH-A1` | str_key | COLD1_QUARANTINED = "BATCH-A1" |  |  |  |  |

| `BATCH-R1` | str_key | COLD1_RELEASED = "BATCH-R1" |  |  |  |  |

| `SELECT o.object_id, MAX(e.temp_celsius) AS max_temp, SUM(e.duration_seconds) AS duration FROM ocel_events AS e JOIN ocel_event_object_rel AS r ON e.event_id = r.event_id JOIN ocel_objects AS o ON r.object_id = o.object_id WHERE e.temp_celsius > -20.0 AND r.qualifier = 'contains_product' AND o.object_type = 'bio:BiologicBatch' GROUP BY o.object_id` | str_key | CANONICAL_QUERY = "SELECT o.object_id, MAX(e.temp_celsius) AS max_temp, SUM(e.duration_seconds) AS duration FROM ocel_events AS e JOIN ocel_event_object_rel AS r ON e.event_id = r.event_id JOIN ocel_objects AS o ON r.object_id = o.object_id WHERE e.temp_celsius > -20.0 AND r.qualifier = 'contains_product' AND o.object_type = 'bio:BiologicBatch' GROUP BY o.object_id" |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `@prefix sql: <http://graphlaw.org/sql#> .` | str_key | AGG_TURTLE = "@prefix sql: <http://graphlaw.org/sql#> ." |  |  |  |  |

| `05ea1aa096f45968aacb2c996d2b541e4b9fdef24f006e0204e1fa88d33f68a5` | str_key | INSA_KAT_HEAD_HEX = "05ea1aa096f45968aacb2c996d2b541e4b9fdef24f006e0204e1fa88d33f68a5" |  |  |  |  |

| `95163f2a6b24bebdfd19bf77cc60553640b8c9f5f5c54bc415c3b0da77e77598` | str_key | GRAPHLAW_HEAD_HEX = "95163f2a6b24bebdfd19bf77cc60553640b8c9f5f5c54bc415c3b0da77e77598" |  |  |  |  |

| `INSA_LAW_PACK` | env_key | std::env::var("INSA_LAW_PACK") |  |  |  |  |

| `INSA_LAW_PACK_ONTOLOGY` | env_key | std::env::var("INSA_LAW_PACK_ONTOLOGY") |  |  |  |  |

| `19d9ec90cb861e8b1e320ef01e5e0d8a54022b19357f2b4856f379417dcc79d7` | str_key | ONTOLOGY_BLAKE3 = "19d9ec90cb861e8b1e320ef01e5e0d8a54022b19357f2b4856f379417dcc79d7" |  |  |  |  |

| `@prefix insa: <urn:insa-law:> .` | str_key | LINK_RULES = "@prefix insa: <urn:insa-law:> ." |  |  |  |  |

| `SELECT ?a ?b WHERE { ` | str_key | VIOLATION_QUERY = "SELECT ?a ?b WHERE { " |  |  |  |  |

| `tx-source-pattern` | str_key | LAWFUL_EDGES = "tx-source-pattern" |  |  |  |  |

| `admit:shacl` | str_key | STEP_NAMES = "admit:shacl" |  |  |  |  |

| `observe` | str_key | CEILING_NAMES = "observe" |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `NotSemanticContent` | str_key | REFUSAL_KINDS = "NotSemanticContent" |  |  |  |  |

| `P19-WASM-IDENTITY-SKIP` | str_key | MARKER = "P19-WASM-IDENTITY-SKIP" |  |  |  |  |

| `urn:n:a` | str_key | COPY_FOOTPRINT_OVERHEAD = "urn:n:a" |  |  |  |  |

| `/Users/sac/purrdf/corpora/n3` | str_key | CORPUS = "/Users/sac/purrdf/corpora/n3" |  |  |  |  |

| `literals-collections-conforming` | str_key | KNOWN_DIVERGENCES = "literals-collections-conforming" |  |  |  |  |

| `http://example.org/n3-fuse/` | str_key | NS = "http://example.org/n3-fuse/" |  |  |  |  |

| `http://example.org/n3-halt/` | str_key | NS = "http://example.org/n3-halt/" |  |  |  |  |

| `http://example.org/iso/` | str_key | NS = "http://example.org/iso/" |  |  |  |  |

| `http://www.w3.org/2000/10/swap/log#implies` | str_key | LOG_IMPLIES = "http://www.w3.org/2000/10/swap/log#implies" |  |  |  |  |

| `n3_iterations` | str_key | NAMES = "n3_iterations" |  |  |  |  |

| `@prefix : <http://example.org/lease-court/> .

:tweety :is :chick .
:chick :is :bird .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
` | str_key | ORIGINAL_SOURCE = "@prefix : <http://example.org/lease-court/> .

:tweety :is :chick .
:chick :is :bird .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
" |  |  |  |  |

| `@prefix : <http://example.org/lease-court/> .

:tweety :is :chick .
:chick :is :finch .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
` | str_key | TAMPERED_SOURCE = "@prefix : <http://example.org/lease-court/> .

:tweety :is :chick .
:chick :is :finch .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
" |  |  |  |  |

| `n3_proof_lease_court.n3` | str_key | LABEL = "n3_proof_lease_court.n3" |  |  |  |  |

| `:daffy :is :animal` | str_key | CLAIM_TAMPERED = ":daffy :is :animal" |  |  |  |  |

| `:tweety :is :animal` | str_key | CLAIM = ":tweety :is :animal" |  |  |  |  |

| `@prefix : <http://example.org/proof-court/> .

:tweety :is :chick .
:chick :is :bird .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
` | str_key | CHAIN_SOURCE = "@prefix : <http://example.org/proof-court/> .

:tweety :is :chick .
:chick :is :bird .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
" |  |  |  |  |

| `@prefix : <http://example.org/proof-court/> .
@prefix log: <http://www.w3.org/2000/10/swap/log#> .

:t :trigger true .
:a :holds { :u :p _:q } .
{ :t :trigger true . :a :holds ?F . ?F log:skolem ?S } => { :t :out ?S } .
` | str_key | SKOLEM_SOURCE = "@prefix : <http://example.org/proof-court/> .
@prefix log: <http://www.w3.org/2000/10/swap/log#> .

:t :trigger true .
:a :holds { :u :p _:q } .
{ :t :trigger true . :a :holds ?F . ?F log:skolem ?S } => { :t :out ?S } .
" |  |  |  |  |

| `http://example.org/proof-court/` | str_key | NS = "http://example.org/proof-court/" |  |  |  |  |

| `https://eyereasoner.github.io/.well-known/genid/` | str_key | SKOLEM_NS = "https://eyereasoner.github.io/.well-known/genid/" |  |  |  |  |

| `n3_proof_pipeline_court.n3` | str_key | LABEL = "n3_proof_pipeline_court.n3" |  |  |  |  |

| `http://example.org/q/` | str_key | NS = "http://example.org/q/" |  |  |  |  |

| `https://eyereasoner.github.io/.well-known/genid/` | str_key | GENID = "https://eyereasoner.github.io/.well-known/genid/" |  |  |  |  |

| `@prefix : <urn:n:> .` | str_key | GOOD_RULES = "@prefix : <urn:n:> ." |  |  |  |  |

| `mock` | str_key | BANNED = "mock" |  |  |  |  |

| `tests/no_mock_markers.rs` | str_key | ALLOWLIST = "tests/no_mock_markers.rs" |  |  |  |  |

| `tests/set_operations_court.rs` | str_key | ORACLE_ALLOWLIST = "tests/set_operations_court.rs" |  |  |  |  |

| `tests/wasm_env_isolation_court.rs` | str_key | UNWRAP_ALLOWLIST = "tests/wasm_env_isolation_court.rs" |  |  |  |  |

| `https://example.org/` | str_key | NS = "https://example.org/" |  |  |  |  |

| `turtle` | str_key | RDF_DIALECTS = "turtle" |  |  |  |  |

| `<urn:p:at>` | str_key | AT = "<urn:p:at>" |  |  |  |  |

| `<urn:d:door> <urn:p:is>` | str_key | DOOR = "<urn:d:door> <urn:p:is>" |  |  |  |  |

| `req` | function | req(start: &str) -> serde_json::Value |  |  |  |  |

| `<urn:p:at>` | str_key | AT = "<urn:p:at>" |  |  |  |  |

| `<urn:p:occupied>` | str_key | OCC = "<urn:p:occupied>" |  |  |  |  |

| `blake3:e16c63901ed5012208d71405fd179ad947da8f849d8c3cb083c26d0ff6e31803` | str_key | POSITIVE_DIGEST = "blake3:e16c63901ed5012208d71405fd179ad947da8f849d8c3cb083c26d0ff6e31803" |  |  |  |  |

| `CARGO_MANIFEST_DIR` | str_key | PACK_DIR = "CARGO_MANIFEST_DIR" |  |  |  |  |

| `b3dbbc7c945a7883a75933dfc12699af393dac5fe711009bb5f4d694a2ac399b` | str_key | ONTOLOGY_BLAKE3 = "b3dbbc7c945a7883a75933dfc12699af393dac5fe711009bb5f4d694a2ac399b" |  |  |  |  |

| `PREFIX cdt: <http://w3id.org/awslabs/neptune/SPARQL-CDTs/> ` | str_key | PFX = "PREFIX cdt: <http://w3id.org/awslabs/neptune/SPARQL-CDTs/> " |  |  |  |  |

| `http://example.org/crs/planar` | str_key | CRS = "http://example.org/crs/planar" |  |  |  |  |

| `http://example.org/geo#` | str_key | GEO = "http://example.org/geo#" |  |  |  |  |

| `http://example.org/geof/` | str_key | GEOF = "http://example.org/geof/" |  |  |  |  |

| `http://example.org/sf#` | str_key | SF = "http://example.org/sf#" |  |  |  |  |

| `http://example.org/unit/metre` | str_key | METRE = "http://example.org/unit/metre" |  |  |  |  |

| `http://www.w3.org/1999/02/22-rdf-syntax-ns#type` | str_key | TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" |  |  |  |  |

| `http://www.w3.org/2000/01/rdf-schema#subClassOf` | str_key | SUBCLASS = "http://www.w3.org/2000/01/rdf-schema#subClassOf" |  |  |  |  |

| `http://www.w3.org/2001/XMLSchema#double` | str_key | XSD_DOUBLE = "http://www.w3.org/2001/XMLSchema#double" |  |  |  |  |

| `https://example.org/` | str_key | EX = "https://example.org/" |  |  |  |  |

| `https://example.org/Animal` | str_key | ANIMAL = "https://example.org/Animal" |  |  |  |  |

| `https://example.org/Cat` | str_key | CAT = "https://example.org/Cat" |  |  |  |  |

| `https://example.org/knows` | str_key | KNOWS = "https://example.org/knows" |  |  |  |  |

| `https://example.org/tom` | str_key | TOM = "https://example.org/tom" |  |  |  |  |

| `d52282d8c48e9021ff6f78e413d8a79dbffbc6b28e28db9a9d855f5810bb3185` | str_key | FIXTURE_BLAKE3 = "d52282d8c48e9021ff6f78e413d8a79dbffbc6b28e28db9a9d855f5810bb3185" |  |  |  |  |

| `tests/fixtures/fm_chain_009.json` | str_key | FIXTURE_PATH = "tests/fixtures/fm_chain_009.json" |  |  |  |  |

| `<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .` | str_key | NT = "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> ." |  |  |  |  |

| `derive:rdfs` | str_key | GATE = "derive:rdfs" |  |  |  |  |

| `CARGO_MANIFEST_DIR` | str_key | ROOT = "CARGO_MANIFEST_DIR" |  |  |  |  |

| `abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._/-` | str_key | PATH_CLASS = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._/-" |  |  |  |  |

| `receipts` | str_key | RECEIPTS_REL = "receipts" |  |  |  |  |

| `receipts/logs/` | str_key | LOG_PREFIX = "receipts/logs/" |  |  |  |  |

| `0123456789abcdef0123456789abcdef01234567` | str_key | SUBJECT = "0123456789abcdef0123456789abcdef01234567" |  |  |  |  |

| `0bca2e430b2d2b856ae5b262031b0cadd1dd6ee40230ecb75d7ba3b0baf86662` | str_key | H_PRE = "0bca2e430b2d2b856ae5b262031b0cadd1dd6ee40230ecb75d7ba3b0baf86662" |  |  |  |  |

| `0ce59900a950fd16a9546ae14eed62b0b4ea63102b80812c00157a56c7a1d669` | str_key | H_TRIPLES = "0ce59900a950fd16a9546ae14eed62b0b4ea63102b80812c00157a56c7a1d669" |  |  |  |  |

| `48fc721fbbc172e0925fa27af1671de225ba927134802998b10a1568a188652b` | str_key | SETTLED_V1_INPUTS = "48fc721fbbc172e0925fa27af1671de225ba927134802998b10a1568a188652b" |  |  |  |  |

| `61054d964e2ecadcaac7d82d06fc1bd5f732a7595824376792a3f0970aae96c1` | str_key | V2_GOLDEN = "61054d964e2ecadcaac7d82d06fc1bd5f732a7595824376792a3f0970aae96c1" |  |  |  |  |

| `63bff90dee1c16beb2881e11664a1e8c0b6942abe39ba5726d9ce1f9708a8ffe` | str_key | SETTLED_V1_GOLDEN = "63bff90dee1c16beb2881e11664a1e8c0b6942abe39ba5726d9ce1f9708a8ffe" |  |  |  |  |

| `7a364272788ab0a7a56b8444e9f6f76b1f0f7d90d3e5725e3496a78e732d92fd` | str_key | H_QUERY = "7a364272788ab0a7a56b8444e9f6f76b1f0f7d90d3e5725e3496a78e732d92fd" |  |  |  |  |

| `9d79bfadd4c208bad3a87982bdd8f9d05991011f16b96de3cf1c5c33cf7c58df` | str_key | H_COLUMNAR = "9d79bfadd4c208bad3a87982bdd8f9d05991011f16b96de3cf1c5c33cf7c58df" |  |  |  |  |

| `b208929e36eafbe6511114fe40910c486b265eff351b4e7edc47f86476db6539` | str_key | H_POST = "b208929e36eafbe6511114fe40910c486b265eff351b4e7edc47f86476db6539" |  |  |  |  |

| `db2da6fc6ec068a661a7028f220524ea6fad4b3cbf7f72df336dd99fd44f7b0c` | str_key | EMPTY_GOLDEN = "db2da6fc6ec068a661a7028f220524ea6fad4b3cbf7f72df336dd99fd44f7b0c" |  |  |  |  |

| `f641cf9578e7ddf23bfbb9df055c15b79fe920b9d98910ca22be069deb20f774` | str_key | V1_GOLDEN = "f641cf9578e7ddf23bfbb9df055c15b79fe920b9d98910ca22be069deb20f774" |  |  |  |  |

| `7ac44b45bd8c91a1ee1a1035ba43494bd08054231a4ea6dcb9175d8cdfc04593` | str_key | SENTINEL_PIN = "7ac44b45bd8c91a1ee1a1035ba43494bd08054231a4ea6dcb9175d8cdfc04593" |  |  |  |  |

| `d53d18c23212ea7b6300594bb89bce60218f6eff2b9d628b8cc42d3e79bbd5ab` | str_key | EMPTY_TRIPLES_PIN = "d53d18c23212ea7b6300594bb89bce60218f6eff2b9d628b8cc42d3e79bbd5ab" |  |  |  |  |

| `db2da6fc6ec068a661a7028f220524ea6fad4b3cbf7f72df336dd99fd44f7b0c` | str_key | EMPTY_COLUMNAR_PIN = "db2da6fc6ec068a661a7028f220524ea6fad4b3cbf7f72df336dd99fd44f7b0c" |  |  |  |  |

| `REFUSAL_VOCAB_PACK` | env_key | std::env::var("REFUSAL_VOCAB_PACK") |  |  |  |  |

| `malformed_frame` | str_key | VALVE_EXPECTED = "malformed_frame" |  |  |  |  |

| `ontology.ttl` | str_key | PINNED = "ontology.ttl" |  |  |  |  |

| `tests/fixtures/refusal_crosswalk` | str_key | FIXTURE_ROOT = "tests/fixtures/refusal_crosswalk" |  |  |  |  |

| `SHAPES` | const | SHAPES: &str |  |  |  |  |

| `TWO_BAD` | const | TWO_BAD: &str |  |  |  |  |

| `<https://e/a> a <https://e/T> .` | str_key | TWO_BAD = "<https://e/a> a <https://e/T> ." |  |  |  |  |

| `@prefix sh: <http://www.w3.org/ns/shacl#> .` | str_key | SHAPES = "@prefix sh: <http://www.w3.org/ns/shacl#> ." |  |  |  |  |

| `GRAPHLAW_REQUIRE_WASM_PIN` | env_key | std::env::var_os("GRAPHLAW_REQUIRE_WASM_PIN") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var("GRAPHLAW_WASM") |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `http://seanchatmangpt.github.io/packs/graphlaw-ash-capability#` | str_key | GAC = "http://seanchatmangpt.github.io/packs/graphlaw-ash-capability#" |  |  |  |  |

| `<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .` | str_key | NT_DATA = "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> ." |  |  |  |  |

| `max_request_bytes` | str_key | EXPECTED = "max_request_bytes" |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `rehy-01` | str_key | SESSION = "rehy-01" |  |  |  |  |

| `rehy-02` | str_key | SESSION = "rehy-02" |  |  |  |  |

| `rehy-03` | str_key | SESSION = "rehy-03" |  |  |  |  |

| `CARGO_BIN_EXE_sa2a-run14-verify` | str_key | BIN = "CARGO_BIN_EXE_sa2a-run14-verify" |  |  |  |  |

| `../ontologies/sql-ast.ttl` | str_key | CANONICAL_SHAPES = "../ontologies/sql-ast.ttl" |  |  |  |  |

| `@prefix sh: <http://www.w3.org/ns/shacl#> .` | str_key | PREFIX = "@prefix sh: <http://www.w3.org/ns/shacl#> ." |  |  |  |  |

| `@prefix sql: <http://graphlaw.org/sql#> .` | str_key | AST_DOC = "@prefix sql: <http://graphlaw.org/sql#> ." |  |  |  |  |

| `urn:s/` | str_key | SUBJECT = "urn:s/" |  |  |  |  |

| `urn:v/` | str_key | VOCAB = "urn:v/" |  |  |  |  |

| `CONFORMANCE_REPORT` | env_key | std::env::var("CONFORMANCE_REPORT") |  |  |  |  |

| `../docs/reference/sql-rdf-rec.md` | str_key | REC = "../docs/reference/sql-rdf-rec.md" |  |  |  |  |

| `../docs/refusals.md` | str_key | REFUSALS_MD = "../docs/refusals.md" |  |  |  |  |

| `../ontologies/sql-ast.ttl` | str_key | SQL_AST_TTL = "../ontologies/sql-ast.ttl" |  |  |  |  |

| `../src/sql_ast/refusals.rs` | str_key | REFUSALS_RS = "../src/sql_ast/refusals.rs" |  |  |  |  |

| `@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .` | str_key | B = "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> ." |  |  |  |  |

| `@prefix sql: <http://graphlaw.org/sql#> .` | str_key | A = "@prefix sql: <http://graphlaw.org/sql#> ." |  |  |  |  |

| `ALIASED_AGG_ARG` | str_key | PINNED_GAPS = "ALIASED_AGG_ARG" |  |  |  |  |

| `OPERATOR_TOKEN` | str_key | REGISTERED_ARMS = "OPERATOR_TOKEN" |  |  |  |  |

| `core_functions` | str_key | CERTIFIED_EXTENSION_RESULTS = "core_functions" |  |  |  |  |

| `§1` | str_key | CLAUSE_TABLE = "§1" |  |  |  |  |

| `hash` | str_key | GATES = "hash" |  |  |  |  |

| `tests/fixtures/strata_gate_differential` | str_key | FIXTURE_ROOT = "tests/fixtures/strata_gate_differential" |  |  |  |  |

| `docs/jira/v26.10.22/mega/tc-mega-fibo-01` | str_key | MEGA = "docs/jira/v26.10.22/mega/tc-mega-fibo-01" |  |  |  |  |

| `cd3cde66a2a9a3440240e8ff005ca7258dfd995744f36b061102db522e33f30e` | str_key | PIN_BLAKE3 = "cd3cde66a2a9a3440240e8ff005ca7258dfd995744f36b061102db522e33f30e" |  |  |  |  |

| `https://strata.chatman.ai/ontology#` | str_key | ST = "https://strata.chatman.ai/ontology#" |  |  |  |  |

| `tests/fixtures/strata-temprun-ontology.ttl` | str_key | PINNED = "tests/fixtures/strata-temprun-ontology.ttl" |  |  |  |  |

| `@prefix : <https://e/> .` | str_key | N3_RULES = "@prefix : <https://e/> ." |  |  |  |  |

| `http://example.org/ubo#` | str_key | NS = "http://example.org/ubo#" |  |  |  |  |

| `http://graphlaw.org/sql#` | str_key | SQL = "http://graphlaw.org/sql#" |  |  |  |  |

| `http://www.w3.org/2001/XMLSchema#` | str_key | XSD = "http://www.w3.org/2001/XMLSchema#" |  |  |  |  |

| `ubo-01` | str_key | SESSION = "ubo-01" |  |  |  |  |

| `ubo-02` | str_key | SESSION = "ubo-02" |  |  |  |  |

| `ubo-03` | str_key | SESSION = "ubo-03" |  |  |  |  |

| `0123456789abcdef0123456789abcdef01234567` | str_key | SUBJECT = "0123456789abcdef0123456789abcdef01234567" |  |  |  |  |

| `rdf-n-triples/nt-syntax-bad-string-04.nt` | str_key | KNOWN_DIVERGENCES = "rdf-n-triples/nt-syntax-bad-string-04.nt" |  |  |  |  |

| `rdf-turtle/turtle-syntax-number-11.ttl` | str_key | KNOWN_POSITIVE_DIVERGENCES = "rdf-turtle/turtle-syntax-number-11.ttl" |  |  |  |  |

| `ENTRIES` | const | ENTRIES: &[(Dialect, &str, Kind)] |  |  |  |  |

| `Dialect` | enum | Dialect { Turtle, NTriples, NQuads, Trig, RdfXml } |  |  |  |  |

| `Kind` | enum | Kind { Positive, Negative } |  |  |  |  |

| `rdf-turtle/turtle-syntax-file-01.ttl` | str_key | ENTRIES = "rdf-turtle/turtle-syntax-file-01.ttl" |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `@prefix ex: <https://e/> . ex:s ex:p ex:o, ` | str_key | SAMPLE = "@prefix ex: <https://e/> . ex:s ex:p ex:o, " |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `'; DROP TABLE cold_chain_events; --` | str_key | SQL_TOKENS = "'; DROP TABLE cold_chain_events; --" |  |  |  |  |

| `NotSemanticContent` | str_key | REFUSAL_KINDS = "NotSemanticContent" |  |  |  |  |

| `GWS10_DEBUG` | env_key | std::env::var("GWS10_DEBUG") |  |  |  |  |

| `BATCH-A1` | str_key | COLD1_QUARANTINED = "BATCH-A1" |  |  |  |  |

| `BATCH-R1` | str_key | COLD1_RELEASED = "BATCH-R1" |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `f149433c6e1a84ec95e961ac92b16b65b28e15d6fd8546beb1779d3b54a02e7e` | str_key | LEGACY_DIGEST = "f149433c6e1a84ec95e961ac92b16b65b28e15d6fd8546beb1779d3b54a02e7e" |  |  |  |  |

| `fibo_accounts` | str_key | FIBO_TABLE = "fibo_accounts" |  |  |  |  |

| `lifecycle-a` | str_key | SESSION = "lifecycle-a" |  |  |  |  |

| `lifecycle-b` | str_key | SESSION = "lifecycle-b" |  |  |  |  |

| `SELECT batch_id, SUM(cost) AS total_cost FROM telemetry WHERE (latency > $1) GROUP BY batch_id` | str_key | AGG_SQL = "SELECT batch_id, SUM(cost) AS total_cost FROM telemetry WHERE (latency > $1) GROUP BY batch_id" |  |  |  |  |

| `abi-court` | str_key | SESSION = "abi-court" |  |  |  |  |

| `batch_id,cost,latency` | str_key | TELEMETRY_HEADER = "batch_id,cost,latency" |  |  |  |  |

| `https://example.org/data` | str_key | SUBJECT = "https://example.org/data" |  |  |  |  |

| `https://example.org/queries` | str_key | Q_PREFIX = "https://example.org/queries" |  |  |  |  |

| `https://example.org/queries/q1` | str_key | Q1 = "https://example.org/queries/q1" |  |  |  |  |

| `https://example.org/vocab` | str_key | VOCAB = "https://example.org/vocab" |  |  |  |  |

| `CARGO_TARGET_DIR` | env_key | std::env::var_os("CARGO_TARGET_DIR") |  |  |  |  |

| `GRAPHLAW_RUN_EH` | env_key | std::env::var_os("GRAPHLAW_RUN_EH") |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `debug/graphlaw-run-eh` | str_key | EH_BIN_REL = "debug/graphlaw-run-eh" |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `CARGO_TARGET_DIR` | env_key | std::env::var_os("CARGO_TARGET_DIR") |  |  |  |  |

| `GRAPHLAW_RUN_EH` | env_key | std::env::var_os("GRAPHLAW_RUN_EH") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/` | str_key | ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/" |  |  |  |  |

| `SECRET_TEST_TOKEN` | str_key | TOKEN_NAME = "SECRET_TEST_TOKEN" |  |  |  |  |

| `gl-env-court-canary-6f19e2ac` | str_key | TOKEN_VALUE = "gl-env-court-canary-6f19e2ac" |  |  |  |  |

| `GRAPHLAW_DIAG_NO_EH` | env_key | std::env::var("GRAPHLAW_DIAG_NO_EH") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `FD_ALLOCATE` | str_key | RIGHTS = "FD_ALLOCATE" |  |  |  |  |

| `fibo-08` | str_key | SESSION = "fibo-08" |  |  |  |  |

| `http://example.org/fibo#stmt1` | str_key | FIBO_SUBJECT = "http://example.org/fibo#stmt1" |  |  |  |  |

| `GRAPHLAW_DIAG_NO_EH` | env_key | std::env::var("GRAPHLAW_DIAG_NO_EH") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `FD_ALLOCATE` | str_key | RIGHTS = "FD_ALLOCATE" |  |  |  |  |

| `fibo-01` | str_key | SESSION = "fibo-01" |  |  |  |  |

| `fibo-02` | str_key | SESSION = "fibo-02" |  |  |  |  |

| `fibo-03` | str_key | SESSION = "fibo-03" |  |  |  |  |

| `http://example.org/fibo#stmt1` | str_key | FIBO_SUBJECT = "http://example.org/fibo#stmt1" |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `CARGO_TARGET_DIR` | env_key | std::env::var_os("CARGO_TARGET_DIR") |  |  |  |  |

| `@prefix ex: <https://e/> . ex:s ex:p ex:o .` | str_key | SAMPLE = "@prefix ex: <https://e/> . ex:s ex:p ex:o ." |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var_os("GRAPHLAW_WASM") |  |  |  |  |

| `_:p <http://example.org/knows> _:q .` | str_key | BNQ_PQ = "_:p <http://example.org/knows> _:q ." |  |  |  |  |

| `_:x <http://example.org/knows> _:y .` | str_key | BNQ_AB = "_:x <http://example.org/knows> _:y ." |  |  |  |  |

| `packs/self-monitoring-pack/fixtures/session-real-broad-topic.ttl` | str_key | HOOK_SESSION = "packs/self-monitoring-pack/fixtures/session-real-broad-topic.ttl" |  |  |  |  |

| `packs/self-monitoring-pack/hook.ttl` | str_key | HOOK_PACK = "packs/self-monitoring-pack/hook.ttl" |  |  |  |  |

| `GRAPHLAW_WASM` | env_key | std::env::var("GRAPHLAW_WASM") |  |  |  |  |

| `Ambiguous` | str_key | UNWITNESSED_KINDS = "Ambiguous" |  |  |  |  |

| `NotSemanticContent` | str_key | REFUSAL_KINDS = "NotSemanticContent" |  |  |  |  |

| `RDF 1.2 / codecs / storage IR` | str_key | CAPABILITY_OPS = "RDF 1.2 / codecs / storage IR" |  |  |  |  |

| `ResourceLimit` | str_key | REFUSAL_CODES = "ResourceLimit" |  |  |  |  |

| `https://graphlaw.dev/wire-court#` | str_key | NS = "https://graphlaw.dev/wire-court#" |  |  |  |  |

| `ABI_VERSION` | const | ABI_VERSION: u32 |  |  |  |  |

| `CRATE_NAME` | const | CRATE_NAME: &str |  |  |  |  |

| `ERROR_CODES` | const | ERROR_CODES: &[&str] |  |  |  |  |

| `EXPORT_PREFIX` | const | EXPORT_PREFIX: &str |  |  |  |  |

| `MAX_JSON_DEPTH` | const | MAX_JSON_DEPTH: usize |  |  |  |  |

| `MAX_OUTSTANDING_BYTES` | const | MAX_OUTSTANDING_BYTES: usize |  |  |  |  |

| `MAX_REQUEST_BYTES` | const | MAX_REQUEST_BYTES: usize |  |  |  |  |

| `OPS` | const | OPS: &[&str] |  |  |  |  |

| `Refused` | str_key | ERROR_CODES = "Refused" |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `gl` | str_key | EXPORT_PREFIX = "gl" |  |  |  |  |

| `graphlaw-wasm` | str_key | CRATE_NAME = "graphlaw-wasm" |  |  |  |  |

| `ABI_VERSION` | const | ABI_VERSION: u32 |  |  |  |  |

| `CRATE_NAME` | const | CRATE_NAME: &str |  |  |  |  |

| `ERROR_CODES` | const | ERROR_CODES: &[&str] |  |  |  |  |

| `EXPORT_PREFIX` | const | EXPORT_PREFIX: &str |  |  |  |  |

| `MAX_JSON_DEPTH` | const | MAX_JSON_DEPTH: usize |  |  |  |  |

| `MAX_OUTSTANDING_BYTES` | const | MAX_OUTSTANDING_BYTES: usize |  |  |  |  |

| `MAX_REQUEST_BYTES` | const | MAX_REQUEST_BYTES: usize |  |  |  |  |

| `OPS` | const | OPS: &[&str] |  |  |  |  |

| `Refused` | str_key | ERROR_CODES = "Refused" |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `gl` | str_key | EXPORT_PREFIX = "gl" |  |  |  |  |

| `graphlaw-wasm` | str_key | CRATE_NAME = "graphlaw-wasm" |  |  |  |  |

| `alloc_buf` | function | alloc_buf(len: u32) -> *mut u8 |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) -> Vec<u8> |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `gl_alloc` | function | gl_alloc(len: u32) -> *mut u8 |  |  |  |  |

| `gl_call` | function | gl_call(ptr: *mut u8, len: u32) -> u64 |  |  |  |  |

| `gl_free` | function | gl_free(ptr: *mut u8, len: u32) |  |  |  |  |

| `alloc_buf` | function | alloc_buf(len: u32) -> *mut u8 |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) -> Vec<u8> |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `gl_alloc` | function | gl_alloc(len: u32) -> *mut u8 |  |  |  |  |

| `gl_call` | function | gl_call(ptr: *mut u8, len: u32) -> u64 |  |  |  |  |

| `gl_free` | function | gl_free(ptr: *mut u8, len: u32) |  |  |  |  |

| `__cxa_atexit` | function | __cxa_atexit( _func: unsafe extern "C" fn(*mut c_void), _arg: *mut c_void, _dso: *mut c_void, ) -> i32 |  |  |  |  |

| `__cxa_finalize` | function | __cxa_finalize(_d: *mut c_void) |  |  |  |  |

| `__funcs_on_exit` | function | __funcs_on_exit() |  |  |  |  |

| `atexit` | function | atexit(_func: unsafe extern "C" fn()) -> i32 |  |  |  |  |

| `graphlaw_wasm_register_static_extensions` | function | graphlaw_wasm_register_static_extensions() -> i32 |  |  |  |  |

| `C` | str_key | REGISTER = "C" |  |  |  |  |

| `__cxa_atexit` | function | __cxa_atexit( _func: unsafe extern "C" fn(*mut c_void), _arg: *mut c_void, _dso: *mut c_void, ) -> i32 |  |  |  |  |

| `__cxa_finalize` | function | __cxa_finalize(_d: *mut c_void) |  |  |  |  |

| `__funcs_on_exit` | function | __funcs_on_exit() |  |  |  |  |

| `atexit` | function | atexit(_func: unsafe extern "C" fn()) -> i32 |  |  |  |  |

| `graphlaw_wasm_register_static_extensions` | function | graphlaw_wasm_register_static_extensions() -> i32 |  |  |  |  |

| `C` | str_key | REGISTER = "C" |  |  |  |  |


<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-END: nothing below this line may describe     -->
<!-- code behavior.                                                -->
<!-- ============================================================= -->
