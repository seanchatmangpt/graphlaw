# graphlaw reference

<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-BEGIN: reference body is RIGID                -->
<!-- Every row below is rendered from queries/ast_extract.rq.      -->
<!-- Agents MUST NOT add, edit, reorder, or remove any row or      -->
<!-- table cell. Prose outside the fenced slot below is refused    -->
<!-- by the doc_quality court.                                     -->
<!-- ============================================================= -->

## Modules


### crates/graphlaw-eyeron/src/ast.rs

| `Term` | enum |  |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `blank` | function | blank(value: impl Into<Name>) |  |  |  |  |

| `default_prefixes` | function | default_prefixes() |  |  |  |  |

| `formula` | function | formula(triples: Vec<Triple>) |  |  |  |  |

| `fuse` | function | fuse(premise: Vec<Triple>) |  |  |  |  |

| `iri` | function | iri(value: impl Into<Name>) |  |  |  |  |

| `is_ground` | function | is_ground(&self) |  |  |  |  |

| `is_variable` | function | is_variable(&self) |  |  |  |  |

| `list` | function | list(items: Vec<Term>) |  |  |  |  |

| `literal` | function | literal(value: impl Into<Name>) |  |  |  |  |

| `merge` | function | merge(&mut self, other: Document) |  |  |  |  |

| `new` | function | new(s: Term, p: Term, o: Term) |  |  |  |  |

| `plain` | function | plain(value: impl Into<Name>) |  |  |  |  |

| `var` | function | var(value: impl Into<Name>) |  |  |  |  |

| `with_fuse` | function | with_fuse(mut self, is_fuse: bool) |  |  |  |  |

| `with_query` | function | with_query(mut self, is_query: bool) |  |  |  |  |

| `with_source` | function | with_source(mut self, source: Option<SourceRef>) |  |  |  |  |

| `Document` | struct |  |  |  |  |  |

| `Literal` | struct |  |  |  |  |  |

| `Name` | struct |  |  |  |  |  |

| `Rule` | struct |  |  |  |  |  |

| `SourceRef` | struct |  |  |  |  |  |

| `Triple` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/ast.rs

| `Term` | enum |  |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `blank` | function | blank(value: impl Into<Name>) |  |  |  |  |

| `default_prefixes` | function | default_prefixes() |  |  |  |  |

| `formula` | function | formula(triples: Vec<Triple>) |  |  |  |  |

| `fuse` | function | fuse(premise: Vec<Triple>) |  |  |  |  |

| `iri` | function | iri(value: impl Into<Name>) |  |  |  |  |

| `is_ground` | function | is_ground(&self) |  |  |  |  |

| `is_variable` | function | is_variable(&self) |  |  |  |  |

| `list` | function | list(items: Vec<Term>) |  |  |  |  |

| `literal` | function | literal(value: impl Into<Name>) |  |  |  |  |

| `merge` | function | merge(&mut self, other: Document) |  |  |  |  |

| `new` | function | new(s: Term, p: Term, o: Term) |  |  |  |  |

| `plain` | function | plain(value: impl Into<Name>) |  |  |  |  |

| `var` | function | var(value: impl Into<Name>) |  |  |  |  |

| `with_fuse` | function | with_fuse(mut self, is_fuse: bool) |  |  |  |  |

| `with_query` | function | with_query(mut self, is_query: bool) |  |  |  |  |

| `with_source` | function | with_source(mut self, source: Option<SourceRef>) |  |  |  |  |

| `Document` | struct |  |  |  |  |  |

| `Literal` | struct |  |  |  |  |  |

| `Name` | struct |  |  |  |  |  |

| `Rule` | struct |  |  |  |  |  |

| `SourceRef` | struct |  |  |  |  |  |

| `Triple` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/error.rs

| `at` | function | at(message: impl Into<String>, offset: usize) |  |  |  |  |

| `new` | function | new(message: impl Into<String>) |  |  |  |  |

| `with_source_location` | function | with_source_location(&self, source: &str, label: &str) |  |  |  |  |

| `EyeronError` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/error.rs

| `at` | function | at(message: impl Into<String>, offset: usize) |  |  |  |  |

| `new` | function | new(message: impl Into<String>) |  |  |  |  |

| `with_source_location` | function | with_source_location(&self, source: &str, label: &str) |  |  |  |  |

| `EyeronError` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/lexer.rs

| `TokenKind` | enum |  |  |  |  |  |

| `lex` | function | lex(input: &str) |  |  |  |  |

| `Token` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/lexer.rs

| `TokenKind` | enum |  |  |  |  |  |

| `lex` | function | lex(input: &str) |  |  |  |  |

| `Token` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/lib.rs

| `reason` | function | reason(input: &str) |  |  |  |  |


### crates/graphlaw-eyeron/src/lib.rs

| `reason` | function | reason(input: &str) |  |  |  |  |


### crates/graphlaw-eyeron/src/parser.rs

| `is_rdf_message_log` | function | is_rdf_message_log(input: &str) |  |  |  |  |

| `parse_n3` | function | parse_n3(input: &str, base_iri: Option<&str>) |  |  |  |  |

| `parse_n3_with_source` | function | parse_n3_with_source(
    input: &str,
    base_iri: Option<&str>,
    source_label: Option<&str>,
) |  |  |  |  |

| `parse_rdf_message_log` | function | parse_rdf_message_log(input: &str, base_iri: Option<&str>) |  |  |  |  |


### crates/graphlaw-eyeron/src/parser.rs

| `is_rdf_message_log` | function | is_rdf_message_log(input: &str) |  |  |  |  |

| `parse_n3` | function | parse_n3(input: &str, base_iri: Option<&str>) |  |  |  |  |

| `parse_n3_with_source` | function | parse_n3_with_source(
    input: &str,
    base_iri: Option<&str>,
    source_label: Option<&str>,
) |  |  |  |  |

| `parse_rdf_message_log` | function | parse_rdf_message_log(input: &str, base_iri: Option<&str>) |  |  |  |  |


### crates/graphlaw-eyeron/src/printing.rs

| `document_debug` | function | document_debug(doc: &Document) |  |  |  |  |

| `fuse_report` | function | fuse_report(prefixes: &BTreeMap<String, String>, fuse: &FiredFuse) |  |  |  |  |

| `rdf12_json` | function | rdf12_json(doc: &Document) |  |  |  |  |

| `rdf_result_to_string` | function | rdf_result_to_string(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `result_to_string` | function | result_to_string(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `term_to_n3_object` | function | term_to_n3_object(term: &Term, prefixes: &BTreeMap<String, String>) |  |  |  |  |

| `term_to_n3_predicate` | function | term_to_n3_predicate(term: &Term, prefixes: &BTreeMap<String, String>) |  |  |  |  |

| `triple_to_n3` | function | triple_to_n3(prefixes: &BTreeMap<String, String>, t: &Triple) |  |  |  |  |

| `triples_to_n3` | function | triples_to_n3(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `triples_to_trig` | function | triples_to_trig(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |


### crates/graphlaw-eyeron/src/printing.rs

| `document_debug` | function | document_debug(doc: &Document) |  |  |  |  |

| `fuse_report` | function | fuse_report(prefixes: &BTreeMap<String, String>, fuse: &FiredFuse) |  |  |  |  |

| `rdf12_json` | function | rdf12_json(doc: &Document) |  |  |  |  |

| `rdf_result_to_string` | function | rdf_result_to_string(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `result_to_string` | function | result_to_string(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `term_to_n3_object` | function | term_to_n3_object(term: &Term, prefixes: &BTreeMap<String, String>) |  |  |  |  |

| `term_to_n3_predicate` | function | term_to_n3_predicate(term: &Term, prefixes: &BTreeMap<String, String>) |  |  |  |  |

| `triple_to_n3` | function | triple_to_n3(prefixes: &BTreeMap<String, String>, t: &Triple) |  |  |  |  |

| `triples_to_n3` | function | triples_to_n3(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `triples_to_trig` | function | triples_to_trig(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |


### crates/graphlaw-eyeron/src/proof_check.rs

| `Checked` | enum |  |  |  |  |  |

| `Kind` | enum |  |  |  |  |  |

| `Resolution` | enum |  |  |  |  |  |

| `check` | function | check(document: &dyn Document) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `valid` | function | valid(&self) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `Failure` | struct |  |  |  |  |  |

| `Obligation` | struct |  |  |  |  |  |

| `Report` | struct |  |  |  |  |  |

| `Document` | trait |  |  |  |  |  |


### crates/graphlaw-eyeron/src/proof_check.rs

| `Checked` | enum |  |  |  |  |  |

| `Kind` | enum |  |  |  |  |  |

| `Resolution` | enum |  |  |  |  |  |

| `check` | function | check(document: &dyn Document) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `valid` | function | valid(&self) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `Failure` | struct |  |  |  |  |  |

| `Obligation` | struct |  |  |  |  |  |

| `Report` | struct |  |  |  |  |  |

| `Document` | trait |  |  |  |  |  |


### crates/graphlaw-eyeron/src/proof_check_n3.rs

| `check_proof` | function | check_proof(source: &str, proof: &str, label: &str) |  |  |  |  |

| `check_proof_document` | function | check_proof_document(source: &crate::ast::Document, proof: &str) |  |  |  |  |

| `read_document` | function | read_document(document: &crate::ast::Document, proof: &str) |  |  |  |  |

| `N3Proof` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/proof_check_n3.rs

| `check_proof` | function | check_proof(source: &str, proof: &str, label: &str) |  |  |  |  |

| `check_proof_document` | function | check_proof_document(source: &crate::ast::Document, proof: &str) |  |  |  |  |

| `read_document` | function | read_document(document: &crate::ast::Document, proof: &str) |  |  |  |  |

| `N3Proof` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/proof_writer.rs

| `proof_to_n3` | function | proof_to_n3(prefixes: &BTreeMap<String, String>, result: &ReasonerResult) |  |  |  |  |

| `rule_statement` | function | rule_statement(rule: &Rule) |  |  |  |  |


### crates/graphlaw-eyeron/src/proof_writer.rs

| `proof_to_n3` | function | proof_to_n3(prefixes: &BTreeMap<String, String>, result: &ReasonerResult) |  |  |  |  |

| `rule_statement` | function | rule_statement(rule: &Rule) |  |  |  |  |


### crates/graphlaw-eyeron/src/rdf_compat.rs

| `RdfFormat` | enum |  |  |  |  |  |

| `parse` | function | parse(value: &str) |  |  |  |  |

| `parse_rdf12` | function | parse_rdf12(input: &str, base_iri: Option<&str>, format: RdfFormat) |  |  |  |  |


### crates/graphlaw-eyeron/src/rdf_compat.rs

| `RdfFormat` | enum |  |  |  |  |  |

| `parse` | function | parse(value: &str) |  |  |  |  |

| `parse_rdf12` | function | parse_rdf12(input: &str, base_iri: Option<&str>, format: RdfFormat) |  |  |  |  |


### crates/graphlaw-eyeron/src/reasoner.rs

| `BackwardStep` | enum |  |  |  |  |  |

| `CompletionStatus` | enum |  |  |  |  |  |

| `ProofNode` | enum |  |  |  |  |  |

| `ReasonerError` | enum |  |  |  |  |  |

| `ReasonerLimit` | enum |  |  |  |  |  |

| `builtin_reads_outside_its_triple` | function | builtin_reads_outside_its_triple(predicate: &Term) |  |  |  |  |

| `explain_backward` | function | explain_backward(
    goal: &Triple,
    facts: &[Triple],
    given: &BTreeSet<Triple>,
    rules: &[Rule],
    max_depth: usize,
    emit: &mut dyn FnMut(BackwardStep) |  |  |  |  |

| `find_backward_proof_for_goal` | function | find_backward_proof_for_goal(
    goal: &Triple,
    facts: &[Triple],
    rules: &[Rule],
    max_depth: usize,
) |  |  |  |  |

| `incomplete_summary` | function | incomplete_summary(&self) |  |  |  |  |

| `is_complete` | function | is_complete(&self) |  |  |  |  |

| `new` | function | new(program: Document) |  |  |  |  |

| `program` | function | program(&self) |  |  |  |  |

| `reason` | function | reason(&self, data: &Document, options: &ReasonerOptions) |  |  |  |  |

| `verify_builtin_triple` | function | verify_builtin_triple(goal: &Triple) |  |  |  |  |

| `DerivedFact` | struct |  |  |  |  |  |

| `FiredFuse` | struct |  |  |  |  |  |

| `PreparedReasoner` | struct |  |  |  |  |  |

| `ReasonerOptions` | struct |  |  |  |  |  |

| `ReasonerResult` | struct |  |  |  |  |  |

| `ReasonerStatistics` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/reasoner.rs

| `BackwardStep` | enum |  |  |  |  |  |

| `CompletionStatus` | enum |  |  |  |  |  |

| `ProofNode` | enum |  |  |  |  |  |

| `ReasonerError` | enum |  |  |  |  |  |

| `ReasonerLimit` | enum |  |  |  |  |  |

| `builtin_reads_outside_its_triple` | function | builtin_reads_outside_its_triple(predicate: &Term) |  |  |  |  |

| `explain_backward` | function | explain_backward(
    goal: &Triple,
    facts: &[Triple],
    given: &BTreeSet<Triple>,
    rules: &[Rule],
    max_depth: usize,
    emit: &mut dyn FnMut(BackwardStep) |  |  |  |  |

| `find_backward_proof_for_goal` | function | find_backward_proof_for_goal(
    goal: &Triple,
    facts: &[Triple],
    rules: &[Rule],
    max_depth: usize,
) |  |  |  |  |

| `incomplete_summary` | function | incomplete_summary(&self) |  |  |  |  |

| `is_complete` | function | is_complete(&self) |  |  |  |  |

| `new` | function | new(program: Document) |  |  |  |  |

| `program` | function | program(&self) |  |  |  |  |

| `reason` | function | reason(&self, data: &Document, options: &ReasonerOptions) |  |  |  |  |

| `verify_builtin_triple` | function | verify_builtin_triple(goal: &Triple) |  |  |  |  |

| `DerivedFact` | struct |  |  |  |  |  |

| `FiredFuse` | struct |  |  |  |  |  |

| `PreparedReasoner` | struct |  |  |  |  |  |

| `ReasonerOptions` | struct |  |  |  |  |  |

| `ReasonerResult` | struct |  |  |  |  |  |

| `ReasonerStatistics` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/sudoku.rs

| `solve_sudoku_string` | function | solve_sudoku_string(puzzle: &str) |  |  |  |  |


### crates/graphlaw-eyeron/src/sudoku.rs

| `solve_sudoku_string` | function | solve_sudoku_string(puzzle: &str) |  |  |  |  |


### crates/graphlaw-eyeron/src/wasm.rs

| `new` | function | new(program: &str, proof: bool) |  |  |  |  |

| `program_facts` | function | program_facts(&self) |  |  |  |  |

| `program_rules` | function | program_rules(&self) |  |  |  |  |

| `reason` | function | reason(input: &str) |  |  |  |  |

| `reason_report` | function | reason_report(&self, data: &str, rdf: bool, rdf_format: &str) |  |  |  |  |

| `reason_with_data` | function | reason_with_data(
    program: &str,
    data: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `reason_with_data_report` | function | reason_with_data_report(
    program: &str,
    data: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `reason_with_options` | function | reason_with_options(
    input: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `version` | function | version() |  |  |  |  |

| `EyeronSession` | struct |  |  |  |  |  |


### crates/graphlaw-eyeron/src/wasm.rs

| `new` | function | new(program: &str, proof: bool) |  |  |  |  |

| `program_facts` | function | program_facts(&self) |  |  |  |  |

| `program_rules` | function | program_rules(&self) |  |  |  |  |

| `reason` | function | reason(input: &str) |  |  |  |  |

| `reason_report` | function | reason_report(&self, data: &str, rdf: bool, rdf_format: &str) |  |  |  |  |

| `reason_with_data` | function | reason_with_data(
    program: &str,
    data: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `reason_with_data_report` | function | reason_with_data_report(
    program: &str,
    data: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `reason_with_options` | function | reason_with_options(
    input: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `version` | function | version() |  |  |  |  |

| `EyeronSession` | struct |  |  |  |  |  |


### src/abi.rs

| `call` | function | call(request: &[u8]) |  |  |  |  |

| `call_json` | function | call_json(request: &Value) |  |  |  |  |

| `dialect_by_name` | function | dialect_by_name(name: &str) |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() |  |  |  |  |

| `plan_from_json` | function | plan_from_json(plan: &Value) |  |  |  |  |

| `Fail` | struct |  |  |  |  |  |


### src/attest.rs

| `AttestError` | enum |  |  |  |  |  |

| `from_bytes` | function | from_bytes(bytes: &[u8; 32]) |  |  |  |  |

| `from_hex` | function | from_hex(hex: &str) |  |  |  |  |

| `from_json` | function | from_json(text: &str) |  |  |  |  |

| `from_seed` | function | from_seed(seed: [u8; 32]) |  |  |  |  |

| `from_seed_hex` | function | from_seed_hex(hex: &str) |  |  |  |  |

| `get` | function | get(&self, key_id: &str) |  |  |  |  |

| `hex_decode` | function | hex_decode(s: &str) |  |  |  |  |

| `hex_encode` | function | hex_encode(bytes: &[u8]) |  |  |  |  |

| `insert` | function | insert(&mut self, key: VerifyingKey) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `key_id` | function | key_id(&self) |  |  |  |  |

| `lease_payload` | function | lease_payload(l: &Lease) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `receipt_payload` | function | receipt_payload(f: &ReceiptFields<'_>) |  |  |  |  |

| `sign_bytes` | function | sign_bytes(key: &SigningKey, payload: &[u8]) |  |  |  |  |

| `sign_lease` | function | sign_lease(key: &SigningKey, lease: Lease) |  |  |  |  |

| `sign_receipt` | function | sign_receipt(key: &SigningKey, receipt: &Receipt) |  |  |  |  |

| `to_hex` | function | to_hex(&self) |  |  |  |  |

| `to_json` | function | to_json(&self) |  |  |  |  |

| `verify_bytes` | function | verify_bytes(
    payload: &[u8],
    att: &Attestation,
    trusted: &TrustedKeys,
) |  |  |  |  |

| `verify_lease` | function | verify_lease(signed: &SignedLease, trusted: &TrustedKeys) |  |  |  |  |

| `verify_receipt` | function | verify_receipt(
    receipt: &Receipt,
    att: &Attestation,
    trusted: &TrustedKeys,
) |  |  |  |  |

| `verifying_key` | function | verifying_key(&self) |  |  |  |  |

| `Attestation` | struct |  |  |  |  |  |

| `ReceiptFields` | struct |  |  |  |  |  |

| `SigningKey` | struct |  |  |  |  |  |

| `TrustedKeys` | struct |  |  |  |  |  |

| `VerifyingKey` | struct |  |  |  |  |  |


### src/capability_intake.rs

| `consequence_authority` | function | consequence_authority(_repository: &str) |  |  |  |  |

| `donor` | function | donor(repository: &str) |  |  |  |  |

| `CapabilityDonor` | struct |  |  |  |  |  |


### src/dialect.rs

| `Dialect` | enum |  |  |  |  |  |

| `Engine` | enum |  |  |  |  |  |

| `RefusalKind` | enum |  |  |  |  |  |

| `admit_bytes` | function | admit_bytes(
    bytes: &[u8],
    hint: Option<&str>,
    base: Option<&str>,
) |  |  |  |  |

| `check` | function | check(bytes: &[u8], dialect: Dialect, base: Option<&str>) |  |  |  |  |

| `engine` | function | engine(self) |  |  |  |  |

| `media_type` | function | media_type(self) |  |  |  |  |

| `parse_rdf` | function | parse_rdf(
    bytes: &[u8],
    dialect: Dialect,
    base: Option<&str>,
) |  |  |  |  |

| `sniff` | function | sniff(bytes: &[u8], hint: Option<&str>) |  |  |  |  |

| `Refusal` | struct |  |  |  |  |  |


### src/hooks.rs

| `hooks` | function | hooks(&self) |  |  |  |  |

| `load` | function | load(pack: &LawState) |  |  |  |  |

| `materialize` | function | materialize(&self, state: &LawState) |  |  |  |  |

| `Firing` | struct |  |  |  |  |  |

| `Hook` | struct |  |  |  |  |  |

| `HookPack` | struct |  |  |  |  |  |

| `Materialized` | struct |  |  |  |  |  |


### src/law.rs

| `Ceiling` | enum |  |  |  |  |  |

| `LawError` | enum |  |  |  |  |  |

| `LeaseReason` | enum |  |  |  |  |  |

| `N3Error` | enum |  |  |  |  |  |

| `ReceiptReason` | enum |  |  |  |  |  |

| `Step` | enum |  |  |  |  |  |

| `as_str` | function | as_str(self) |  |  |  |  |

| `authorize` | function | authorize(
        &self,
        step: &str,
        required: Ceiling,
        trusted: &crate::attest::TrustedKeys,
        clock: &dyn Clock,
        max_skew_secs: u64,
    ) |  |  |  |  |

| `dataset` | function | dataset(&self) |  |  |  |  |

| `from_dataset` | function | from_dataset(dataset: Arc<purrdf::RdfDataset>) |  |  |  |  |

| `id` | function | id(&self) |  |  |  |  |

| `name` | function | name(&self) |  |  |  |  |

| `parse` | function | parse(s: &str) |  |  |  |  |

| `quad_count` | function | quad_count(&self) |  |  |  |  |

| `reason_n3_bounded` | function | reason_n3_bounded(input: &str) |  |  |  |  |

| `required_ceiling` | function | required_ceiling(&self) |  |  |  |  |

| `transition` | function | transition(&self, step: &Step<'_>) |  |  |  |  |

| `transition_authorized` | function | transition_authorized(
        &self,
        signed: &SignedLease,
        trusted: &crate::attest::TrustedKeys,
        clock: &dyn Clock,
        max_skew_secs: u64,
        step: &Step<'_>,
    ) |  |  |  |  |

| `transition_leased_unverified` | function | transition_leased_unverified(
        &self,
        lease: &Lease,
        step: &Step<'_>,
        now_unix: u64,
    ) |  |  |  |  |

| `with_subject` | function | with_subject(mut self, subject_sha256: impl Into<String>) |  |  |  |  |

| `FixedClock` | struct |  |  |  |  |  |

| `LawState` | struct |  |  |  |  |  |

| `Lease` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `SignedLease` | struct |  |  |  |  |  |

| `SystemClock` | struct |  |  |  |  |  |

| `Violation` | struct |  |  |  |  |  |

| `Clock` | trait |  |  |  |  |  |


### src/lib.rs

| `BackendAuthority` | struct |  |  |  |  |  |


### src/plan.rs

| `TripleError` | enum |  |  |  |  |  |

| `action` | function | action(mut self, name: impl Into<String>) |  |  |  |  |

| `adds` | function | adds(mut self, t: Triple) |  |  |  |  |

| `admit` | function | admit(&self, start: &LawState) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `build` | function | build(self) |  |  |  |  |

| `builder` | function | builder(name: impl Into<String>) |  |  |  |  |

| `canonical_json` | function | canonical_json(&self) |  |  |  |  |

| `deletes` | function | deletes(mut self, t: Triple) |  |  |  |  |

| `digest` | function | digest(&self) |  |  |  |  |

| `goal` | function | goal(mut self, t: Triple) |  |  |  |  |

| `goal_not` | function | goal_not(mut self, t: Triple) |  |  |  |  |

| `iri` | function | iri(s: &str, p: &str, o: &str) |  |  |  |  |

| `literal` | function | literal(s: &str, p: &str, text: &str) |  |  |  |  |

| `new` | function | new(name: impl Into<String>) |  |  |  |  |

| `requires` | function | requires(mut self, t: Triple) |  |  |  |  |

| `requires_not` | function | requires_not(mut self, t: Triple) |  |  |  |  |

| `Action` | struct |  |  |  |  |  |

| `ActionBuilder` | struct |  |  |  |  |  |

| `Admitted` | struct |  |  |  |  |  |

| `Plan` | struct |  |  |  |  |  |

| `PlanBuilder` | struct |  |  |  |  |  |

| `Triple` | struct |  |  |  |  |  |


### src/policy.rs

| `PolicyRefusalKind` | enum |  |  |  |  |  |

| `admit` | function | admit(problem_json: &str, policy_json: &str) |  |  |  |  |

| `admit_parsed` | function | admit_parsed(problem: &Problem, policy: &[Entry]) |  |  |  |  |

| `as_str` | function | as_str(self) |  |  |  |  |

| `to_ntriples` | function | to_ntriples(&self) |  |  |  |  |

| `Entry` | struct |  |  |  |  |  |

| `Outcome` | struct |  |  |  |  |  |

| `PolicyAdmitted` | struct |  |  |  |  |  |

| `PolicyRefused` | struct |  |  |  |  |  |

| `Problem` | struct |  |  |  |  |  |


### src/qualification.rs

| `Standing` | enum |  |  |  |  |  |

| `compose_courts` | function | compose_courts(parent: &Value, child: &Value) |  |  |  |  |

| `evaluate` | function | evaluate(observation: Observation) |  |  |  |  |

| `generate_falsifier` | function | generate_falsifier(law: &Value, seed: &Value) |  |  |  |  |

| `temporal_rule_holds` | function | temporal_rule_holds(events: &[&str], rule: &str) |  |  |  |  |

| `Observation` | struct |  |  |  |  |  |


### src/receipt.rs

| `record` | function | record(state: &LawState, receipt: &Receipt) |  |  |  |  |

| `record_signed` | function | record_signed(
    state: &LawState,
    receipt: &Receipt,
    attestation: &Attestation,
) |  |  |  |  |

| `require` | function | require(state: &LawState, step: &str) |  |  |  |  |

| `require_signed` | function | require_signed(state: &LawState, step: &str, trusted: &TrustedKeys) |  |  |  |  |


### src/receipt_store.rs

| `StoreError` | enum |  |  |  |  |  |

| `list` | function | list(&self, subject_sha: &str) |  |  |  |  |

| `open` | function | open(dir: impl AsRef<Path>) |  |  |  |  |

| `put` | function | put(&self, receipt: &Receipt, subject_sha: &str) |  |  |  |  |

| `put_signed` | function | put_signed(
        &self,
        receipt: &Receipt,
        subject_sha: &str,
        attestation: &Attestation,
    ) |  |  |  |  |

| `receipt_digest` | function | receipt_digest(r: &Receipt) |  |  |  |  |

| `verify` | function | verify(&self, subject_sha: &str) |  |  |  |  |

| `verify_attested` | function | verify_attested(
        &self,
        subject_sha: &str,
        trusted: &TrustedKeys,
    ) |  |  |  |  |

| `ReceiptStore` | struct |  |  |  |  |  |


### src/registry.rs

| `FieldDefault` | enum |  |  |  |  |  |

| `canonical` | function | canonical(v: &Value) |  |  |  |  |

| `op_names` | function | op_names() |  |  |  |  |

| `other_dialect_names` | function | other_dialect_names() |  |  |  |  |

| `rdf_dialect_names` | function | rdf_dialect_names() |  |  |  |  |

| `registry_json_pretty` | function | registry_json_pretty() |  |  |  |  |

| `registry_sha256` | function | registry_sha256() |  |  |  |  |

| `registry_turtle` | function | registry_turtle() |  |  |  |  |

| `registry_value` | function | registry_value() |  |  |  |  |

| `surface_sha256` | function | surface_sha256() |  |  |  |  |

| `DialectRow` | struct |  |  |  |  |  |

| `Field` | struct |  |  |  |  |  |

| `LawStepRow` | struct |  |  |  |  |  |

| `Op` | struct |  |  |  |  |  |

| `RefusalCodeRow` | struct |  |  |  |  |  |

| `Variant` | struct |  |  |  |  |  |


### src/smon.rs

| `broaden` | function | broaden(
    ds: &RdfDataset,
    canonical_topic: &str,
) |  |  |  |  |

| `build_dataset` | function | build_dataset(
    turns: &[Turn],
    session_id: &str,
    base: &str,
) |  |  |  |  |

| `classify_turn` | function | classify_turn(text: &str) |  |  |  |  |

| `extract_topic` | function | extract_topic(text: &str) |  |  |  |  |

| `from_env` | function | from_env() |  |  |  |  |

| `read_turtle` | function | read_turtle(path: &Path) |  |  |  |  |

| `required` | function | required(&self, flag: &str) |  |  |  |  |

| `switch` | function | switch(&self, flag: &str) |  |  |  |  |

| `to_turtle` | function | to_turtle(ds: &RdfDataset) |  |  |  |  |

| `turns_from_jsonl` | function | turns_from_jsonl(raw: &str, assistant_only: bool) |  |  |  |  |

| `value` | function | value(&self, flag: &str) |  |  |  |  |

| `write_turtle` | function | write_turtle(ds: &RdfDataset, out: &Path) |  |  |  |  |

| `Args` | struct |  |  |  |  |  |

| `Broadened` | struct |  |  |  |  |  |

| `Turn` | struct |  |  |  |  |  |


### tests/common/mod.rs

| `native` | function | native(req: &Value) |  |  |  |  |

| `native_bytes` | function | native_bytes(req: &Value) |  |  |  |  |

| `ok` | function | ok(req: &Value) |  |  |  |  |

| `read` | function | read(path: &str) |  |  |  |  |

| `refused` | function | refused(req: &Value) |  |  |  |  |

| `wasm` | function | wasm(req: &Value) |  |  |  |  |

| `wasm_bytes` | function | wasm_bytes(req: &Value) |  |  |  |  |

| `wasm_path` | function | wasm_path() |  |  |  |  |


### tests/plan_negation.rs

| `req` | function | req(start: &str) |  |  |  |  |


### vendor/purrdf-sparql-eval/src/agg_fn.rs

| `AlgebraicClass` | enum |  |  |  |  |  |

| `ScalarvalKind` | enum |  |  |  |  |  |

| `describe` | function | describe(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(name: &'static str, kind: ScalarvalKind) |  |  |  |  |

| `register` | function | register(&mut self, iri: impl Into<String>, aggregate: Arc<dyn CustomAggregate>) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `AggDescriptor` | struct |  |  |  |  |  |

| `AggregateRegistry` | struct |  |  |  |  |  |

| `ScalarvalSpec` | struct |  |  |  |  |  |

| `AggregateAccumulator` | trait |  |  |  |  |  |

| `CustomAggregate` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/agg_fn.rs

| `AlgebraicClass` | enum |  |  |  |  |  |

| `ScalarvalKind` | enum |  |  |  |  |  |

| `describe` | function | describe(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(name: &'static str, kind: ScalarvalKind) |  |  |  |  |

| `register` | function | register(&mut self, iri: impl Into<String>, aggregate: Arc<dyn CustomAggregate>) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `AggDescriptor` | struct |  |  |  |  |  |

| `AggregateRegistry` | struct |  |  |  |  |  |

| `ScalarvalSpec` | struct |  |  |  |  |  |

| `AggregateAccumulator` | trait |  |  |  |  |  |

| `CustomAggregate` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/engine.rs

| `ShaclPrebinding` | enum |  |  |  |  |  |

| `cached_plan_count` | function | cached_plan_count(&self) |  |  |  |  |

| `explain_query` | function | explain_query(
        &self,
        dataset: &Arc<RdfDataset>,
        query_text: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `explain_query_with_options` | function | explain_query_with_options(
        &self,
        dataset: &Arc<RdfDataset>,
        query_text: &str,
        base_iri: Option<&str>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `memory_observer` | function | memory_observer(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `order_cache_stats` | function | order_cache_stats(&self) |  |  |  |  |

| `plan_cache_stats` | function | plan_cache_stats(&self) |  |  |  |  |

| `plan_memory_observer` | function | plan_memory_observer(&self) |  |  |  |  |

| `prepare` | function | prepare(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `prepare_algebra` | function | prepare_algebra(
        &self,
        query: Query,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `prepare_query` | function | prepare_query(
        &self,
        query: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `prepare_query_with_options` | function | prepare_query_with_options(
        &self,
        query: &str,
        base_iri: Option<&str>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `prepare_with` | function | prepare_with(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
        options: &ParserOptions,
    ) |  |  |  |  |

| `prepare_with_relations` | function | prepare_with_relations(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
        options: &ParserOptions,
        relations: &crate::property_fn::PropertyFunctionRegistry,
        aggregates: &crate::agg_fn::AggregateRegistry,
    ) |  |  |  |  |

| `query_governed` | function | query_governed(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
        governors: &QueryGovernors,
    ) |  |  |  |  |

| `query_governed_with_source` | function | query_governed_with_source(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        source: &(dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `query_prepared` | function | query_prepared(
        &self,
        dataset: &Arc<RdfDataset>,
        prepared: &PreparedQuery,
        substitutions: &[(String, TermValue) |  |  |  |  |

| `query_with_source` | function | query_with_source(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        source: &(dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `retained_size_bytes` | function | retained_size_bytes(&self) |  |  |  |  |

| `rewritten` | function | rewritten(query: Query, options: QueryOptions<'_>) |  |  |  |  |

| `stats` | function | stats(&self) |  |  |  |  |

| `update_governed` | function | update_governed(
        &self,
        dataset: &mut Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
        governors: &QueryGovernors,
    ) |  |  |  |  |

| `update_with_options` | function | update_with_options(
        &self,
        dataset: &mut Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `with_eval_options` | function | with_eval_options(mut self, options: EvalOptions) |  |  |  |  |

| `with_limits` | function | with_limits(limits: CacheLimits) |  |  |  |  |

| `with_loss_vocabulary` | function | with_loss_vocabulary(mut self, vocab: LossVocabulary) |  |  |  |  |

| `with_order_cache_limits` | function | with_order_cache_limits(mut self, limits: CacheLimits) |  |  |  |  |

| `with_parser_options` | function | with_parser_options(mut self, options: ParserOptions) |  |  |  |  |

| `with_plan_cache_limits` | function | with_plan_cache_limits(mut self, limits: CacheLimits) |  |  |  |  |

| `with_resolver` | function | with_resolver(mut self, resolver: Arc<dyn GraphResolver>) |  |  |  |  |

| `with_standpoint_predicates` | function | with_standpoint_predicates(mut self, predicates: StandpointPredicates) |  |  |  |  |

| `NativeSparqlEngine` | struct |  |  |  |  |  |

| `PlanCache` | struct |  |  |  |  |  |

| `PreparedQuery` | struct |  |  |  |  |  |

| `QueryOptions` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/engine.rs

| `ShaclPrebinding` | enum |  |  |  |  |  |

| `cached_plan_count` | function | cached_plan_count(&self) |  |  |  |  |

| `explain_query` | function | explain_query(
        &self,
        dataset: &Arc<RdfDataset>,
        query_text: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `explain_query_with_options` | function | explain_query_with_options(
        &self,
        dataset: &Arc<RdfDataset>,
        query_text: &str,
        base_iri: Option<&str>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `memory_observer` | function | memory_observer(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `order_cache_stats` | function | order_cache_stats(&self) |  |  |  |  |

| `plan_cache_stats` | function | plan_cache_stats(&self) |  |  |  |  |

| `plan_memory_observer` | function | plan_memory_observer(&self) |  |  |  |  |

| `prepare` | function | prepare(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `prepare_algebra` | function | prepare_algebra(
        &self,
        query: Query,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `prepare_query` | function | prepare_query(
        &self,
        query: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `prepare_query_with_options` | function | prepare_query_with_options(
        &self,
        query: &str,
        base_iri: Option<&str>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `prepare_with` | function | prepare_with(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
        options: &ParserOptions,
    ) |  |  |  |  |

| `prepare_with_relations` | function | prepare_with_relations(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
        options: &ParserOptions,
        relations: &crate::property_fn::PropertyFunctionRegistry,
        aggregates: &crate::agg_fn::AggregateRegistry,
    ) |  |  |  |  |

| `query_governed` | function | query_governed(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
        governors: &QueryGovernors,
    ) |  |  |  |  |

| `query_governed_with_source` | function | query_governed_with_source(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        source: &(dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `query_prepared` | function | query_prepared(
        &self,
        dataset: &Arc<RdfDataset>,
        prepared: &PreparedQuery,
        substitutions: &[(String, TermValue) |  |  |  |  |

| `query_with_source` | function | query_with_source(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        source: &(dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `retained_size_bytes` | function | retained_size_bytes(&self) |  |  |  |  |

| `rewritten` | function | rewritten(query: Query, options: QueryOptions<'_>) |  |  |  |  |

| `stats` | function | stats(&self) |  |  |  |  |

| `update_governed` | function | update_governed(
        &self,
        dataset: &mut Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
        governors: &QueryGovernors,
    ) |  |  |  |  |

| `update_with_options` | function | update_with_options(
        &self,
        dataset: &mut Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `with_eval_options` | function | with_eval_options(mut self, options: EvalOptions) |  |  |  |  |

| `with_limits` | function | with_limits(limits: CacheLimits) |  |  |  |  |

| `with_loss_vocabulary` | function | with_loss_vocabulary(mut self, vocab: LossVocabulary) |  |  |  |  |

| `with_order_cache_limits` | function | with_order_cache_limits(mut self, limits: CacheLimits) |  |  |  |  |

| `with_parser_options` | function | with_parser_options(mut self, options: ParserOptions) |  |  |  |  |

| `with_plan_cache_limits` | function | with_plan_cache_limits(mut self, limits: CacheLimits) |  |  |  |  |

| `with_resolver` | function | with_resolver(mut self, resolver: Arc<dyn GraphResolver>) |  |  |  |  |

| `with_standpoint_predicates` | function | with_standpoint_predicates(mut self, predicates: StandpointPredicates) |  |  |  |  |

| `NativeSparqlEngine` | struct |  |  |  |  |  |

| `PlanCache` | struct |  |  |  |  |  |

| `PreparedQuery` | struct |  |  |  |  |  |

| `QueryOptions` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/engine/graph_build.rs

| `GraphBuildError` | enum |  |  |  |  |  |

| `GraphBuildStats` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/engine/graph_build.rs

| `GraphBuildError` | enum |  |  |  |  |  |

| `GraphBuildStats` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/error.rs

| `EvalError` | enum |  |  |  |  |  |

| `UnsupportedKind` | enum |  |  |  |  |  |

| `code` | function | code(self) |  |  |  |  |

| `config` | function | config(what: impl Into<String>) |  |  |  |  |

| `data` | function | data(what: impl Into<String>) |  |  |  |  |

| `diagnostic_code` | function | diagnostic_code(&self) |  |  |  |  |

| `function` | function | function(what: impl Into<String>) |  |  |  |  |

| `internal` | function | internal(what: impl Into<String>) |  |  |  |  |

| `remote` | function | remote(what: impl Into<String>) |  |  |  |  |

| `unsupported` | function | unsupported(what: impl Into<String>) |  |  |  |  |


### vendor/purrdf-sparql-eval/src/error.rs

| `EvalError` | enum |  |  |  |  |  |

| `UnsupportedKind` | enum |  |  |  |  |  |

| `code` | function | code(self) |  |  |  |  |

| `config` | function | config(what: impl Into<String>) |  |  |  |  |

| `data` | function | data(what: impl Into<String>) |  |  |  |  |

| `diagnostic_code` | function | diagnostic_code(&self) |  |  |  |  |

| `function` | function | function(what: impl Into<String>) |  |  |  |  |

| `internal` | function | internal(what: impl Into<String>) |  |  |  |  |

| `remote` | function | remote(what: impl Into<String>) |  |  |  |  |

| `unsupported` | function | unsupported(what: impl Into<String>) |  |  |  |  |


### vendor/purrdf-sparql-eval/src/eval.rs

| `Outcome` | enum |  |  |  |  |  |

| `new` | function | new(according_to: impl Into<String>, sharpens: impl Into<String>) |  |  |  |  |

| `with_aggregates` | function | with_aggregates(mut self, registry: &'d crate::agg_fn::AggregateRegistry) |  |  |  |  |

| `with_bnode_mint_prefix` | function | with_bnode_mint_prefix(mut self, prefix: &str) |  |  |  |  |

| `with_call_depth` | function | with_call_depth(mut self, depth: u32) |  |  |  |  |

| `with_eval_options` | function | with_eval_options(mut self, options: EvalOptions) |  |  |  |  |

| `with_focus_graph` | function | with_focus_graph(mut self, graph: &'d Arc<RdfDataset>) |  |  |  |  |

| `with_governors` | function | with_governors(mut self, governors: Arc<GovernorState>) |  |  |  |  |

| `with_loss_vocabulary` | function | with_loss_vocabulary(mut self, vocab: LossVocabulary) |  |  |  |  |

| `with_order_cache` | function | with_order_cache(mut self, cache: &'d BgpOrderCache) |  |  |  |  |

| `with_property_functions` | function | with_property_functions(
        mut self,
        registry: &'d crate::property_fn::PropertyFunctionRegistry,
    ) |  |  |  |  |

| `with_remote` | function | with_remote(mut self, source: &'d (dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `with_standpoint_predicates` | function | with_standpoint_predicates(mut self, predicates: StandpointPredicates) |  |  |  |  |

| `with_user_functions` | function | with_user_functions(
        mut self,
        registry: &'d crate::user_fn::UserFunctionRegistry,
    ) |  |  |  |  |

| `EvalCtx` | struct |  |  |  |  |  |

| `EvalOptions` | struct |  |  |  |  |  |

| `LossVocabulary` | struct |  |  |  |  |  |

| `StandpointPredicates` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/eval.rs

| `Outcome` | enum |  |  |  |  |  |

| `new` | function | new(according_to: impl Into<String>, sharpens: impl Into<String>) |  |  |  |  |

| `with_aggregates` | function | with_aggregates(mut self, registry: &'d crate::agg_fn::AggregateRegistry) |  |  |  |  |

| `with_bnode_mint_prefix` | function | with_bnode_mint_prefix(mut self, prefix: &str) |  |  |  |  |

| `with_call_depth` | function | with_call_depth(mut self, depth: u32) |  |  |  |  |

| `with_eval_options` | function | with_eval_options(mut self, options: EvalOptions) |  |  |  |  |

| `with_focus_graph` | function | with_focus_graph(mut self, graph: &'d Arc<RdfDataset>) |  |  |  |  |

| `with_governors` | function | with_governors(mut self, governors: Arc<GovernorState>) |  |  |  |  |

| `with_loss_vocabulary` | function | with_loss_vocabulary(mut self, vocab: LossVocabulary) |  |  |  |  |

| `with_order_cache` | function | with_order_cache(mut self, cache: &'d BgpOrderCache) |  |  |  |  |

| `with_property_functions` | function | with_property_functions(
        mut self,
        registry: &'d crate::property_fn::PropertyFunctionRegistry,
    ) |  |  |  |  |

| `with_remote` | function | with_remote(mut self, source: &'d (dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `with_standpoint_predicates` | function | with_standpoint_predicates(mut self, predicates: StandpointPredicates) |  |  |  |  |

| `with_user_functions` | function | with_user_functions(
        mut self,
        registry: &'d crate::user_fn::UserFunctionRegistry,
    ) |  |  |  |  |

| `EvalCtx` | struct |  |  |  |  |  |

| `EvalOptions` | struct |  |  |  |  |  |

| `LossVocabulary` | struct |  |  |  |  |  |

| `StandpointPredicates` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/fallible.rs

| `FallibleSparqlError` | enum |  |  |  |  |  |

| `diagnostic` | function | diagnostic(&self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `into_parts` | function | into_parts(self) |  |  |  |  |

| `operational_error` | function | operational_error(&self) |  |  |  |  |

| `partial_answers` | function | partial_answers(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `CompleteSparqlResult` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/fallible.rs

| `FallibleSparqlError` | enum |  |  |  |  |  |

| `diagnostic` | function | diagnostic(&self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `into_parts` | function | into_parts(self) |  |  |  |  |

| `operational_error` | function | operational_error(&self) |  |  |  |  |

| `partial_answers` | function | partial_answers(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `CompleteSparqlResult` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/governed.rs

| `GovernedOutcome` | enum |  |  |  |  |  |

| `GovernedUpdateOutcome` | enum |  |  |  |  |  |

| `PartialAnswers` | enum |  |  |  |  |  |

| `barrier` | function | barrier(&self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `exhausted` | function | exhausted(&self) |  |  |  |  |

| `into_complete` | function | into_complete(self) |  |  |  |  |

| `into_result` | function | into_result(self) |  |  |  |  |

| `is_applied` | function | is_applied(&self) |  |  |  |  |

| `is_certain` | function | is_certain(&self) |  |  |  |  |

| `is_complete` | function | is_complete(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_positional_prefix` | function | is_positional_prefix(&self) |  |  |  |  |

| `relations` | function | relations(&self) |  |  |  |  |

| `result` | function | result(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `withholding_blank_nodes` | function | withholding_blank_nodes(self, mut withhold: impl FnMut(&str) |  |  |  |  |

| `BudgetExhausted` | struct |  |  |  |  |  |

| `GovernedEvidence` | struct |  |  |  |  |  |

| `PartialSparqlResult` | struct |  |  |  |  |  |

| `RelationIdentity` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/governed.rs

| `GovernedOutcome` | enum |  |  |  |  |  |

| `GovernedUpdateOutcome` | enum |  |  |  |  |  |

| `PartialAnswers` | enum |  |  |  |  |  |

| `barrier` | function | barrier(&self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `exhausted` | function | exhausted(&self) |  |  |  |  |

| `into_complete` | function | into_complete(self) |  |  |  |  |

| `into_result` | function | into_result(self) |  |  |  |  |

| `is_applied` | function | is_applied(&self) |  |  |  |  |

| `is_certain` | function | is_certain(&self) |  |  |  |  |

| `is_complete` | function | is_complete(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_positional_prefix` | function | is_positional_prefix(&self) |  |  |  |  |

| `relations` | function | relations(&self) |  |  |  |  |

| `result` | function | result(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `withholding_blank_nodes` | function | withholding_blank_nodes(self, mut withhold: impl FnMut(&str) |  |  |  |  |

| `BudgetExhausted` | struct |  |  |  |  |  |

| `GovernedEvidence` | struct |  |  |  |  |  |

| `PartialSparqlResult` | struct |  |  |  |  |  |

| `RelationIdentity` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/governor/ledger.rs

| `aggregates` | function | aggregates(&self) |  |  |  |  |

| `current` | function | current() |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `fuel_at` | function | fuel_at(&self, point: ChargePoint) |  |  |  |  |

| `fuel_total` | function | fuel_total(&self) |  |  |  |  |

| `join_orders` | function | join_orders(&self) |  |  |  |  |

| `ledger` | function | ledger(&self) |  |  |  |  |

| `peak_cells` | function | peak_cells(&self) |  |  |  |  |

| `profile` | function | profile(&self) |  |  |  |  |

| `relations` | function | relations(&self) |  |  |  |  |

| `render` | function | render(&self) |  |  |  |  |

| `NodeCharges` | struct |  |  |  |  |  |

| `PlanEstimate` | struct |  |  |  |  |  |

| `ProfileIdentity` | struct |  |  |  |  |  |

| `QueryExplanation` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/governor/ledger.rs

| `aggregates` | function | aggregates(&self) |  |  |  |  |

| `current` | function | current() |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `fuel_at` | function | fuel_at(&self, point: ChargePoint) |  |  |  |  |

| `fuel_total` | function | fuel_total(&self) |  |  |  |  |

| `join_orders` | function | join_orders(&self) |  |  |  |  |

| `ledger` | function | ledger(&self) |  |  |  |  |

| `peak_cells` | function | peak_cells(&self) |  |  |  |  |

| `profile` | function | profile(&self) |  |  |  |  |

| `relations` | function | relations(&self) |  |  |  |  |

| `render` | function | render(&self) |  |  |  |  |

| `NodeCharges` | struct |  |  |  |  |  |

| `PlanEstimate` | struct |  |  |  |  |  |

| `ProfileIdentity` | struct |  |  |  |  |  |

| `QueryExplanation` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/governor/lift.rs

| `operator` | function | operator(self) |  |  |  |  |

| `NonMonotoneBarrier` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/governor/lift.rs

| `operator` | function | operator(self) |  |  |  |  |

| `NonMonotoneBarrier` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/governor/mod.rs

| `ChargePoint` | enum |  |  |  |  |  |

| `after` | function | after(budget: Duration) |  |  |  |  |

| `cancel` | function | cancel(&self) |  |  |  |  |

| `charge` | function | charge(&self, dimension: ResourceDimension, amount: u64) |  |  |  |  |

| `charge_if_engaged` | function | charge_if_engaged(
        &self,
        dimension: ResourceDimension,
        amount: u64,
    ) |  |  |  |  |

| `charge_point` | function | charge_point(&self, point: ChargePoint) |  |  |  |  |

| `charge_point_if_engaged` | function | charge_point_if_engaged(&self, point: ChargePoint) |  |  |  |  |

| `commit_ordered_items` | function | commit_ordered_items(
        &self,
        per_item: &[ItemCharge],
    ) |  |  |  |  |

| `consumed_in` | function | consumed_in(&self, dimension: ResourceDimension) |  |  |  |  |

| `cost` | function | cost(self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `has_expired` | function | has_expired(&self) |  |  |  |  |

| `is_cancelled` | function | is_cancelled(&self) |  |  |  |  |

| `is_engaged` | function | is_engaged(&self) |  |  |  |  |

| `is_engaged_in` | function | is_engaged_in(&self, dimension: ResourceDimension) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `limits` | function | limits(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `observe_peak` | function | observe_peak(
        &self,
        dimension: ResourceDimension,
        observed: u64,
    ) |  |  |  |  |

| `observe_peak_if_engaged` | function | observe_peak_if_engaged(
        &self,
        dimension: ResourceDimension,
        observed: u64,
    ) |  |  |  |  |

| `poll_stop` | function | poll_stop(&self) |  |  |  |  |

| `record_trip` | function | record_trip(&self, candidate: TrippedGovernor) |  |  |  |  |

| `schedule_index` | function | schedule_index(self) |  |  |  |  |

| `should_abandon` | function | should_abandon(&self) |  |  |  |  |

| `stop_signal` | function | stop_signal(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `with_fuel` | function | with_fuel(mut self, fuel: u64) |  |  |  |  |

| `with_max_answers` | function | with_max_answers(mut self, rows: u64) |  |  |  |  |

| `with_max_intermediate_cells` | function | with_max_intermediate_cells(mut self, cells: u64) |  |  |  |  |

| `with_max_remote_requests` | function | with_max_remote_requests(mut self, requests: u64) |  |  |  |  |

| `with_max_scratch_bytes` | function | with_max_scratch_bytes(mut self, bytes: u64) |  |  |  |  |

| `with_stop_signal` | function | with_stop_signal(mut self, signal: Arc<dyn StopSignal>) |  |  |  |  |

| `CancellationFlag` | struct |  |  |  |  |  |

| `GovernorState` | struct |  |  |  |  |  |

| `ItemCharge` | struct |  |  |  |  |  |

| `QueryGovernors` | struct |  |  |  |  |  |

| `WallDeadline` | struct |  |  |  |  |  |

| `StopSignal` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/governor/mod.rs

| `ChargePoint` | enum |  |  |  |  |  |

| `after` | function | after(budget: Duration) |  |  |  |  |

| `cancel` | function | cancel(&self) |  |  |  |  |

| `charge` | function | charge(&self, dimension: ResourceDimension, amount: u64) |  |  |  |  |

| `charge_if_engaged` | function | charge_if_engaged(
        &self,
        dimension: ResourceDimension,
        amount: u64,
    ) |  |  |  |  |

| `charge_point` | function | charge_point(&self, point: ChargePoint) |  |  |  |  |

| `charge_point_if_engaged` | function | charge_point_if_engaged(&self, point: ChargePoint) |  |  |  |  |

| `commit_ordered_items` | function | commit_ordered_items(
        &self,
        per_item: &[ItemCharge],
    ) |  |  |  |  |

| `consumed_in` | function | consumed_in(&self, dimension: ResourceDimension) |  |  |  |  |

| `cost` | function | cost(self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `has_expired` | function | has_expired(&self) |  |  |  |  |

| `is_cancelled` | function | is_cancelled(&self) |  |  |  |  |

| `is_engaged` | function | is_engaged(&self) |  |  |  |  |

| `is_engaged_in` | function | is_engaged_in(&self, dimension: ResourceDimension) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `limits` | function | limits(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `observe_peak` | function | observe_peak(
        &self,
        dimension: ResourceDimension,
        observed: u64,
    ) |  |  |  |  |

| `observe_peak_if_engaged` | function | observe_peak_if_engaged(
        &self,
        dimension: ResourceDimension,
        observed: u64,
    ) |  |  |  |  |

| `poll_stop` | function | poll_stop(&self) |  |  |  |  |

| `record_trip` | function | record_trip(&self, candidate: TrippedGovernor) |  |  |  |  |

| `schedule_index` | function | schedule_index(self) |  |  |  |  |

| `should_abandon` | function | should_abandon(&self) |  |  |  |  |

| `stop_signal` | function | stop_signal(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `with_fuel` | function | with_fuel(mut self, fuel: u64) |  |  |  |  |

| `with_max_answers` | function | with_max_answers(mut self, rows: u64) |  |  |  |  |

| `with_max_intermediate_cells` | function | with_max_intermediate_cells(mut self, cells: u64) |  |  |  |  |

| `with_max_remote_requests` | function | with_max_remote_requests(mut self, requests: u64) |  |  |  |  |

| `with_max_scratch_bytes` | function | with_max_scratch_bytes(mut self, bytes: u64) |  |  |  |  |

| `with_stop_signal` | function | with_stop_signal(mut self, signal: Arc<dyn StopSignal>) |  |  |  |  |

| `CancellationFlag` | struct |  |  |  |  |  |

| `GovernorState` | struct |  |  |  |  |  |

| `ItemCharge` | struct |  |  |  |  |  |

| `QueryGovernors` | struct |  |  |  |  |  |

| `WallDeadline` | struct |  |  |  |  |  |

| `StopSignal` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/knn/metric.rs

| `Kernel` | enum |  |  |  |  |  |

| `best` | function | best(k: usize, candidates: impl IntoIterator<Item = Ranked>) |  |  |  |  |

| `distance` | function | distance(
        self,
        query: &[f64],
        query_norm: f64,
        candidate: &[f64],
        candidate_norm: f64,
    ) |  |  |  |  |

| `needs_norms` | function | needs_norms(self) |  |  |  |  |

| `norm` | function | norm(vector: &[f64]) |  |  |  |  |

| `of` | function | of(metric: &DistanceMetric) |  |  |  |  |

| `Ranked` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/knn/metric.rs

| `Kernel` | enum |  |  |  |  |  |

| `best` | function | best(k: usize, candidates: impl IntoIterator<Item = Ranked>) |  |  |  |  |

| `distance` | function | distance(
        self,
        query: &[f64],
        query_norm: f64,
        candidate: &[f64],
        candidate_norm: f64,
    ) |  |  |  |  |

| `needs_norms` | function | needs_norms(self) |  |  |  |  |

| `norm` | function | norm(vector: &[f64]) |  |  |  |  |

| `of` | function | of(metric: &DistanceMetric) |  |  |  |  |

| `Ranked` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/knn/mod.rs

| `dimension` | function | dimension(&self) |  |  |  |  |

| `from_artifact` | function | from_artifact(
        artifact: &[u8],
        target_set: TargetSetId,
        vector_space: VectorSpaceId,
        bindings: Vec<(TargetId, TermValue) |  |  |  |  |

| `guard` | function | guard(&self) |  |  |  |  |

| `max_candidates` | function | max_candidates(self) |  |  |  |  |

| `max_neighbours` | function | max_neighbours(self) |  |  |  |  |

| `metric` | function | metric(&self) |  |  |  |  |

| `new` | function | new(max_candidates: u64, max_neighbours: u64) |  |  |  |  |

| `row_count` | function | row_count(&self) |  |  |  |  |

| `row_of` | function | row_of(&self, term: &TermValue) |  |  |  |  |

| `space` | function | space(&self) |  |  |  |  |

| `term` | function | term(&self, row: usize) |  |  |  |  |

| `EmbeddingKnnRelation` | struct |  |  |  |  |  |

| `EmbeddingSpace` | struct |  |  |  |  |  |

| `KnnGuard` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/knn/mod.rs

| `dimension` | function | dimension(&self) |  |  |  |  |

| `from_artifact` | function | from_artifact(
        artifact: &[u8],
        target_set: TargetSetId,
        vector_space: VectorSpaceId,
        bindings: Vec<(TargetId, TermValue) |  |  |  |  |

| `guard` | function | guard(&self) |  |  |  |  |

| `max_candidates` | function | max_candidates(self) |  |  |  |  |

| `max_neighbours` | function | max_neighbours(self) |  |  |  |  |

| `metric` | function | metric(&self) |  |  |  |  |

| `new` | function | new(max_candidates: u64, max_neighbours: u64) |  |  |  |  |

| `row_count` | function | row_count(&self) |  |  |  |  |

| `row_of` | function | row_of(&self, term: &TermValue) |  |  |  |  |

| `space` | function | space(&self) |  |  |  |  |

| `term` | function | term(&self, row: usize) |  |  |  |  |

| `EmbeddingKnnRelation` | struct |  |  |  |  |  |

| `EmbeddingSpace` | struct |  |  |  |  |  |

| `KnnGuard` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/modifier.rs

| `ValueAggregate` | enum |  |  |  |  |  |

| `compare_values` | function | compare_values(a: &TermValue, b: &TermValue) |  |  |  |  |

| `fold_values` | function | fold_values(
    aggregate: ValueAggregate,
    values: &[TermValue],
) |  |  |  |  |

| `order_values` | function | order_values(values: Vec<TermValue>, descending: bool) |  |  |  |  |


### vendor/purrdf-sparql-eval/src/modifier.rs

| `ValueAggregate` | enum |  |  |  |  |  |

| `compare_values` | function | compare_values(a: &TermValue, b: &TermValue) |  |  |  |  |

| `fold_values` | function | fold_values(
    aggregate: ValueAggregate,
    values: &[TermValue],
) |  |  |  |  |

| `order_values` | function | order_values(values: Vec<TermValue>, descending: bool) |  |  |  |  |


### vendor/purrdf-sparql-eval/src/path_relation.rs

| `PathDirection` | enum |  |  |  |  |  |

| `edge_count` | function | edge_count(&self) |  |  |  |  |

| `max_expansions_per_invocation` | function | max_expansions_per_invocation(&self) |  |  |  |  |

| `max_hops` | function | max_hops(&self) |  |  |  |  |

| `max_paths_per_seed` | function | max_paths_per_seed(&self) |  |  |  |  |

| `min_hops` | function | min_hops(&self) |  |  |  |  |

| `new` | function | new(alternatives: Vec<(TermValue, PathDirection) |  |  |  |  |

| `node_count` | function | node_count(&self) |  |  |  |  |

| `snapshot_fingerprint` | function | snapshot_fingerprint(&self) |  |  |  |  |

| `PathGraph` | struct |  |  |  |  |  |

| `PathLimits` | struct |  |  |  |  |  |

| `PathSnapshotFingerprint` | struct |  |  |  |  |  |

| `PathStep` | struct |  |  |  |  |  |

| `PathWitnessRelation` | struct |  |  |  |  |  |

| `ShortestPathWitnessRelation` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/path_relation.rs

| `PathDirection` | enum |  |  |  |  |  |

| `edge_count` | function | edge_count(&self) |  |  |  |  |

| `max_expansions_per_invocation` | function | max_expansions_per_invocation(&self) |  |  |  |  |

| `max_hops` | function | max_hops(&self) |  |  |  |  |

| `max_paths_per_seed` | function | max_paths_per_seed(&self) |  |  |  |  |

| `min_hops` | function | min_hops(&self) |  |  |  |  |

| `new` | function | new(alternatives: Vec<(TermValue, PathDirection) |  |  |  |  |

| `node_count` | function | node_count(&self) |  |  |  |  |

| `snapshot_fingerprint` | function | snapshot_fingerprint(&self) |  |  |  |  |

| `PathGraph` | struct |  |  |  |  |  |

| `PathLimits` | struct |  |  |  |  |  |

| `PathSnapshotFingerprint` | struct |  |  |  |  |  |

| `PathStep` | struct |  |  |  |  |  |

| `PathWitnessRelation` | struct |  |  |  |  |  |

| `ShortestPathWitnessRelation` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/plan_cache.rs

| `CacheLimits` | struct |  |  |  |  |  |

| `CacheStats` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/plan_cache.rs

| `CacheLimits` | struct |  |  |  |  |  |

| `CacheStats` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/plan_memory.rs

| `stats` | function | stats(&self) |  |  |  |  |

| `PlanMemoryObserver` | struct |  |  |  |  |  |

| `PlanMemoryStats` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/plan_memory.rs

| `stats` | function | stats(&self) |  |  |  |  |

| `PlanMemoryObserver` | struct |  |  |  |  |  |

| `PlanMemoryStats` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/property_fn.rs

| `all_free_mode` | function | all_free_mode(self) |  |  |  |  |

| `arity` | function | arity(&self) |  |  |  |  |

| `describe` | function | describe(&self) |  |  |  |  |

| `flattened` | function | flattened(&self) |  |  |  |  |

| `get` | function | get(&self, pos: usize) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `mode` | function | mode(&self) |  |  |  |  |

| `new` | function | new(subject: usize, object: usize) |  |  |  |  |

| `next_contained` | function | next_contained(cursor: &mut dyn PfCursor, iri: &str) |  |  |  |  |

| `object` | function | object(&self) |  |  |  |  |

| `open_contained` | function | open_contained(
    relation: &dyn PropertyFunction,
    iri: &str,
    args: &PfArgs<'_>,
    ceiling: Option<u64>,
) |  |  |  |  |

| `register` | function | register(&mut self, iri: impl Into<String>, relation: Arc<dyn PropertyFunction>) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `rows` | function | rows(&self) |  |  |  |  |

| `subject` | function | subject(&self) |  |  |  |  |

| `take_work_contained` | function | take_work_contained(cursor: &mut dyn PfCursor, iri: &str) |  |  |  |  |

| `total` | function | total(self) |  |  |  |  |

| `MemoryRelation` | struct |  |  |  |  |  |

| `PfArgs` | struct |  |  |  |  |  |

| `PfArity` | struct |  |  |  |  |  |

| `PfDescriptor` | struct |  |  |  |  |  |

| `PfMode` | struct |  |  |  |  |  |

| `PropertyFunctionRegistry` | struct |  |  |  |  |  |

| `PfCursor` | trait |  |  |  |  |  |

| `PropertyFunction` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/property_fn.rs

| `all_free_mode` | function | all_free_mode(self) |  |  |  |  |

| `arity` | function | arity(&self) |  |  |  |  |

| `describe` | function | describe(&self) |  |  |  |  |

| `flattened` | function | flattened(&self) |  |  |  |  |

| `get` | function | get(&self, pos: usize) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `mode` | function | mode(&self) |  |  |  |  |

| `new` | function | new(subject: usize, object: usize) |  |  |  |  |

| `next_contained` | function | next_contained(cursor: &mut dyn PfCursor, iri: &str) |  |  |  |  |

| `object` | function | object(&self) |  |  |  |  |

| `open_contained` | function | open_contained(
    relation: &dyn PropertyFunction,
    iri: &str,
    args: &PfArgs<'_>,
    ceiling: Option<u64>,
) |  |  |  |  |

| `register` | function | register(&mut self, iri: impl Into<String>, relation: Arc<dyn PropertyFunction>) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `rows` | function | rows(&self) |  |  |  |  |

| `subject` | function | subject(&self) |  |  |  |  |

| `take_work_contained` | function | take_work_contained(cursor: &mut dyn PfCursor, iri: &str) |  |  |  |  |

| `total` | function | total(self) |  |  |  |  |

| `MemoryRelation` | struct |  |  |  |  |  |

| `PfArgs` | struct |  |  |  |  |  |

| `PfArity` | struct |  |  |  |  |  |

| `PfDescriptor` | struct |  |  |  |  |  |

| `PfMode` | struct |  |  |  |  |  |

| `PropertyFunctionRegistry` | struct |  |  |  |  |  |

| `PfCursor` | trait |  |  |  |  |  |

| `PropertyFunction` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/remote.rs

| `RemoteError` | enum |  |  |  |  |  |

| `new` | function | new(endpoint: &'a str, query_text: &'a str) |  |  |  |  |

| `silent` | function | silent(mut self, silent: bool) |  |  |  |  |

| `stop_trip` | function | stop_trip(&self) |  |  |  |  |

| `with_max_intermediate_cells` | function | with_max_intermediate_cells(mut self, cells: Option<u64>) |  |  |  |  |

| `with_stop` | function | with_stop(mut self, stop: Option<&'a Arc<dyn StopSignal>>) |  |  |  |  |

| `ResolvedBindings` | struct |  |  |  |  |  |

| `ServiceRequest` | struct |  |  |  |  |  |

| `ServiceResolver` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/remote.rs

| `RemoteError` | enum |  |  |  |  |  |

| `new` | function | new(endpoint: &'a str, query_text: &'a str) |  |  |  |  |

| `silent` | function | silent(mut self, silent: bool) |  |  |  |  |

| `stop_trip` | function | stop_trip(&self) |  |  |  |  |

| `with_max_intermediate_cells` | function | with_max_intermediate_cells(mut self, cells: Option<u64>) |  |  |  |  |

| `with_stop` | function | with_stop(mut self, stop: Option<&'a Arc<dyn StopSignal>>) |  |  |  |  |

| `ResolvedBindings` | struct |  |  |  |  |  |

| `ServiceRequest` | struct |  |  |  |  |  |

| `ServiceResolver` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/remote_http.rs

| `catalog` | function | catalog(&self) |  |  |  |  |

| `new` | function | new(transport: T) |  |  |  |  |

| `with_catalog` | function | with_catalog(mut self, catalog: ServiceCatalog) |  |  |  |  |

| `with_timeout` | function | with_timeout(mut self, timeout: Duration) |  |  |  |  |

| `HttpRemoteQuerySource` | struct |  |  |  |  |  |

| `HttpRequest` | struct |  |  |  |  |  |

| `HttpTransport` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/remote_http.rs

| `catalog` | function | catalog(&self) |  |  |  |  |

| `new` | function | new(transport: T) |  |  |  |  |

| `with_catalog` | function | with_catalog(mut self, catalog: ServiceCatalog) |  |  |  |  |

| `with_timeout` | function | with_timeout(mut self, timeout: Duration) |  |  |  |  |

| `HttpRemoteQuerySource` | struct |  |  |  |  |  |

| `HttpRequest` | struct |  |  |  |  |  |

| `HttpTransport` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/scratch.rs

| `SolutionTerm` | enum |  |  |  |  |  |

| `computed_count` | function | computed_count(&self) |  |  |  |  |

| `computed_value` | function | computed_value(&self, sid: ScratchId) |  |  |  |  |

| `minted_bytes` | function | minted_bytes(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `ScratchId` | struct |  |  |  |  |  |

| `ScratchInterner` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/scratch.rs

| `SolutionTerm` | enum |  |  |  |  |  |

| `computed_count` | function | computed_count(&self) |  |  |  |  |

| `computed_value` | function | computed_value(&self, sid: ScratchId) |  |  |  |  |

| `minted_bytes` | function | minted_bytes(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `ScratchId` | struct |  |  |  |  |  |

| `ScratchInterner` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/service.rs

| `ServiceCapability` | enum |  |  |  |  |  |

| `ServiceCredential` | enum |  |  |  |  |  |

| `allows` | function | allows(self, capability: ServiceCapability) |  |  |  |  |

| `authorize` | function | authorize(
        &self,
        endpoint: &str,
        needs: ServiceCapabilities,
    ) |  |  |  |  |

| `capabilities` | function | capabilities(&self) |  |  |  |  |

| `catalog` | function | catalog(&self) |  |  |  |  |

| `credential` | function | credential(&self) |  |  |  |  |

| `dataset` | function | dataset(&self, endpoint: &str) |  |  |  |  |

| `detail` | function | detail(&self) |  |  |  |  |

| `endpoint` | function | endpoint(&self) |  |  |  |  |

| `grant` | function | grant(self, capability: ServiceCapability) |  |  |  |  |

| `granting` | function | granting(capabilities: impl IntoIterator<Item = ServiceCapability>) |  |  |  |  |

| `header` | function | header(&self) |  |  |  |  |

| `headers` | function | headers(&self) |  |  |  |  |

| `iter` | function | iter(self) |  |  |  |  |

| `new` | function | new(
        endpoint: impl Into<String>,
        withheld: ServiceCapability,
        detail: impl Into<String>,
    ) |  |  |  |  |

| `profile_for` | function | profile_for(&self, endpoint: &str) |  |  |  |  |

| `request_headers` | function | request_headers(&self) |  |  |  |  |

| `timeout` | function | timeout(&self) |  |  |  |  |

| `user_agent` | function | user_agent(&self) |  |  |  |  |

| `with_catalog` | function | with_catalog(mut self, catalog: ServiceCatalog) |  |  |  |  |

| `with_credential` | function | with_credential(mut self, credential: ServiceCredential) |  |  |  |  |

| `with_endpoint` | function | with_endpoint(mut self, endpoint: impl Into<String>, dataset: Arc<RdfDataset>) |  |  |  |  |

| `with_fallback` | function | with_fallback(mut self, profile: ServiceProfile) |  |  |  |  |

| `with_header` | function | with_header(mut self, name: impl Into<String>, value: impl Into<String>) |  |  |  |  |

| `with_route` | function | with_route(
        mut self,
        endpoint: impl Into<String>,
        resolver: &'a (dyn ServiceResolver + Sync) |  |  |  |  |

| `with_service` | function | with_service(mut self, endpoint: impl Into<String>, profile: ServiceProfile) |  |  |  |  |

| `with_timeout` | function | with_timeout(mut self, timeout: Duration) |  |  |  |  |

| `with_user_agent` | function | with_user_agent(mut self, user_agent: impl Into<String>) |  |  |  |  |

| `withheld` | function | withheld(&self) |  |  |  |  |

| `InProcessServiceResolver` | struct |  |  |  |  |  |

| `ServiceCapabilities` | struct |  |  |  |  |  |

| `ServiceCatalog` | struct |  |  |  |  |  |

| `ServiceDenial` | struct |  |  |  |  |  |

| `ServiceProfile` | struct |  |  |  |  |  |

| `ServiceRouter` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/service.rs

| `ServiceCapability` | enum |  |  |  |  |  |

| `ServiceCredential` | enum |  |  |  |  |  |

| `allows` | function | allows(self, capability: ServiceCapability) |  |  |  |  |

| `authorize` | function | authorize(
        &self,
        endpoint: &str,
        needs: ServiceCapabilities,
    ) |  |  |  |  |

| `capabilities` | function | capabilities(&self) |  |  |  |  |

| `catalog` | function | catalog(&self) |  |  |  |  |

| `credential` | function | credential(&self) |  |  |  |  |

| `dataset` | function | dataset(&self, endpoint: &str) |  |  |  |  |

| `detail` | function | detail(&self) |  |  |  |  |

| `endpoint` | function | endpoint(&self) |  |  |  |  |

| `grant` | function | grant(self, capability: ServiceCapability) |  |  |  |  |

| `granting` | function | granting(capabilities: impl IntoIterator<Item = ServiceCapability>) |  |  |  |  |

| `header` | function | header(&self) |  |  |  |  |

| `headers` | function | headers(&self) |  |  |  |  |

| `iter` | function | iter(self) |  |  |  |  |

| `new` | function | new(
        endpoint: impl Into<String>,
        withheld: ServiceCapability,
        detail: impl Into<String>,
    ) |  |  |  |  |

| `profile_for` | function | profile_for(&self, endpoint: &str) |  |  |  |  |

| `request_headers` | function | request_headers(&self) |  |  |  |  |

| `timeout` | function | timeout(&self) |  |  |  |  |

| `user_agent` | function | user_agent(&self) |  |  |  |  |

| `with_catalog` | function | with_catalog(mut self, catalog: ServiceCatalog) |  |  |  |  |

| `with_credential` | function | with_credential(mut self, credential: ServiceCredential) |  |  |  |  |

| `with_endpoint` | function | with_endpoint(mut self, endpoint: impl Into<String>, dataset: Arc<RdfDataset>) |  |  |  |  |

| `with_fallback` | function | with_fallback(mut self, profile: ServiceProfile) |  |  |  |  |

| `with_header` | function | with_header(mut self, name: impl Into<String>, value: impl Into<String>) |  |  |  |  |

| `with_route` | function | with_route(
        mut self,
        endpoint: impl Into<String>,
        resolver: &'a (dyn ServiceResolver + Sync) |  |  |  |  |

| `with_service` | function | with_service(mut self, endpoint: impl Into<String>, profile: ServiceProfile) |  |  |  |  |

| `with_timeout` | function | with_timeout(mut self, timeout: Duration) |  |  |  |  |

| `with_user_agent` | function | with_user_agent(mut self, user_agent: impl Into<String>) |  |  |  |  |

| `withheld` | function | withheld(&self) |  |  |  |  |

| `InProcessServiceResolver` | struct |  |  |  |  |  |

| `ServiceCapabilities` | struct |  |  |  |  |  |

| `ServiceCatalog` | struct |  |  |  |  |  |

| `ServiceDenial` | struct |  |  |  |  |  |

| `ServiceProfile` | struct |  |  |  |  |  |

| `ServiceRouter` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/solution.rs

| `contains` | function | contains(&self, var: &Variable) |  |  |  |  |

| `empty` | function | empty(schema: Arc<VarSchema>) |  |  |  |  |

| `from_vars` | function | from_vars(vars: impl IntoIterator<Item = Variable>) |  |  |  |  |

| `index_of` | function | index_of(&self, var: &Variable) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `push` | function | push(&mut self, var: Variable) |  |  |  |  |

| `shared_columns` | function | shared_columns(&self, other: &Self) |  |  |  |  |

| `union` | function | union(&self, other: &Self) |  |  |  |  |

| `unit` | function | unit() |  |  |  |  |

| `vars` | function | vars(&self) |  |  |  |  |

| `SolutionSeq` | struct |  |  |  |  |  |

| `VarSchema` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/solution.rs

| `contains` | function | contains(&self, var: &Variable) |  |  |  |  |

| `empty` | function | empty(schema: Arc<VarSchema>) |  |  |  |  |

| `from_vars` | function | from_vars(vars: impl IntoIterator<Item = Variable>) |  |  |  |  |

| `index_of` | function | index_of(&self, var: &Variable) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `push` | function | push(&mut self, var: Variable) |  |  |  |  |

| `shared_columns` | function | shared_columns(&self, other: &Self) |  |  |  |  |

| `union` | function | union(&self, other: &Self) |  |  |  |  |

| `unit` | function | unit() |  |  |  |  |

| `vars` | function | vars(&self) |  |  |  |  |

| `SolutionSeq` | struct |  |  |  |  |  |

| `VarSchema` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/stat_agg.rs

| `register_statistical_aggregates` | function | register_statistical_aggregates(&mut self, namespace: &str) |  |  |  |  |


### vendor/purrdf-sparql-eval/src/stat_agg.rs

| `register_statistical_aggregates` | function | register_statistical_aggregates(&mut self, namespace: &str) |  |  |  |  |


### vendor/purrdf-sparql-eval/src/update.rs

| `GraphResolveRequest` | struct |  |  |  |  |  |

| `GraphResolver` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/update.rs

| `GraphResolveRequest` | struct |  |  |  |  |  |

| `GraphResolver` | trait |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/user_fn.rs

| `Arity` | enum |  |  |  |  |  |

| `NodeKind` | enum |  |  |  |  |  |

| `UserFnBody` | enum |  |  |  |  |  |

| `Volatility` | enum |  |  |  |  |  |

| `insert` | function | insert(&mut self, iri: impl Into<String>, func: UserFunction) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `register_expr` | function | register_expr(&mut self, iri: impl Into<String>, arity: Arity, body: ExprFnBody) |  |  |  |  |

| `register_native` | function | register_native(
        &mut self,
        iri: impl Into<String>,
        arity: Arity,
        volatility: Volatility,
        body: NativeFnBody,
    ) |  |  |  |  |

| `requires_focus_graph` | function | requires_focus_graph(&self) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `resolve_expr` | function | resolve_expr(&self, iri: &str) |  |  |  |  |

| `resolve_native` | function | resolve_native(&self, iri: &str) |  |  |  |  |

| `ExprFnCall` | struct |  |  |  |  |  |

| `ExprFunction` | struct |  |  |  |  |  |

| `NativeFunction` | struct |  |  |  |  |  |

| `TypeConstraint` | struct |  |  |  |  |  |

| `UserFnParam` | struct |  |  |  |  |  |

| `UserFunction` | struct |  |  |  |  |  |

| `UserFunctionRegistry` | struct |  |  |  |  |  |


### vendor/purrdf-sparql-eval/src/user_fn.rs

| `Arity` | enum |  |  |  |  |  |

| `NodeKind` | enum |  |  |  |  |  |

| `UserFnBody` | enum |  |  |  |  |  |

| `Volatility` | enum |  |  |  |  |  |

| `insert` | function | insert(&mut self, iri: impl Into<String>, func: UserFunction) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `register_expr` | function | register_expr(&mut self, iri: impl Into<String>, arity: Arity, body: ExprFnBody) |  |  |  |  |

| `register_native` | function | register_native(
        &mut self,
        iri: impl Into<String>,
        arity: Arity,
        volatility: Volatility,
        body: NativeFnBody,
    ) |  |  |  |  |

| `requires_focus_graph` | function | requires_focus_graph(&self) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `resolve_expr` | function | resolve_expr(&self, iri: &str) |  |  |  |  |

| `resolve_native` | function | resolve_native(&self, iri: &str) |  |  |  |  |

| `ExprFnCall` | struct |  |  |  |  |  |

| `ExprFunction` | struct |  |  |  |  |  |

| `NativeFunction` | struct |  |  |  |  |  |

| `TypeConstraint` | struct |  |  |  |  |  |

| `UserFnParam` | struct |  |  |  |  |  |

| `UserFunction` | struct |  |  |  |  |  |

| `UserFunctionRegistry` | struct |  |  |  |  |  |


### wasm/src/ffi.rs

| `alloc_buf` | function | alloc_buf(len: u32) |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |


### wasm/src/ffi.rs

| `alloc_buf` | function | alloc_buf(len: u32) |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |



<!-- AGENT-FORBIDDEN-END -->

## Signature/type/default/errors table

<!-- RIGID table: header order is fixed; rows come only from the query. -->

| Item | Type | Signature | Params | Defaults | Errors | Invariants |
|------|------|-----------|--------|----------|--------|------------|

| `Term` | enum |  |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `blank` | function | blank(value: impl Into<Name>) |  |  |  |  |

| `default_prefixes` | function | default_prefixes() |  |  |  |  |

| `formula` | function | formula(triples: Vec<Triple>) |  |  |  |  |

| `fuse` | function | fuse(premise: Vec<Triple>) |  |  |  |  |

| `iri` | function | iri(value: impl Into<Name>) |  |  |  |  |

| `is_ground` | function | is_ground(&self) |  |  |  |  |

| `is_variable` | function | is_variable(&self) |  |  |  |  |

| `list` | function | list(items: Vec<Term>) |  |  |  |  |

| `literal` | function | literal(value: impl Into<Name>) |  |  |  |  |

| `merge` | function | merge(&mut self, other: Document) |  |  |  |  |

| `new` | function | new(s: Term, p: Term, o: Term) |  |  |  |  |

| `plain` | function | plain(value: impl Into<Name>) |  |  |  |  |

| `var` | function | var(value: impl Into<Name>) |  |  |  |  |

| `with_fuse` | function | with_fuse(mut self, is_fuse: bool) |  |  |  |  |

| `with_query` | function | with_query(mut self, is_query: bool) |  |  |  |  |

| `with_source` | function | with_source(mut self, source: Option<SourceRef>) |  |  |  |  |

| `Document` | struct |  |  |  |  |  |

| `Literal` | struct |  |  |  |  |  |

| `Name` | struct |  |  |  |  |  |

| `Rule` | struct |  |  |  |  |  |

| `SourceRef` | struct |  |  |  |  |  |

| `Triple` | struct |  |  |  |  |  |

| `Term` | enum |  |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `blank` | function | blank(value: impl Into<Name>) |  |  |  |  |

| `default_prefixes` | function | default_prefixes() |  |  |  |  |

| `formula` | function | formula(triples: Vec<Triple>) |  |  |  |  |

| `fuse` | function | fuse(premise: Vec<Triple>) |  |  |  |  |

| `iri` | function | iri(value: impl Into<Name>) |  |  |  |  |

| `is_ground` | function | is_ground(&self) |  |  |  |  |

| `is_variable` | function | is_variable(&self) |  |  |  |  |

| `list` | function | list(items: Vec<Term>) |  |  |  |  |

| `literal` | function | literal(value: impl Into<Name>) |  |  |  |  |

| `merge` | function | merge(&mut self, other: Document) |  |  |  |  |

| `new` | function | new(s: Term, p: Term, o: Term) |  |  |  |  |

| `plain` | function | plain(value: impl Into<Name>) |  |  |  |  |

| `var` | function | var(value: impl Into<Name>) |  |  |  |  |

| `with_fuse` | function | with_fuse(mut self, is_fuse: bool) |  |  |  |  |

| `with_query` | function | with_query(mut self, is_query: bool) |  |  |  |  |

| `with_source` | function | with_source(mut self, source: Option<SourceRef>) |  |  |  |  |

| `Document` | struct |  |  |  |  |  |

| `Literal` | struct |  |  |  |  |  |

| `Name` | struct |  |  |  |  |  |

| `Rule` | struct |  |  |  |  |  |

| `SourceRef` | struct |  |  |  |  |  |

| `Triple` | struct |  |  |  |  |  |

| `at` | function | at(message: impl Into<String>, offset: usize) |  |  |  |  |

| `new` | function | new(message: impl Into<String>) |  |  |  |  |

| `with_source_location` | function | with_source_location(&self, source: &str, label: &str) |  |  |  |  |

| `EyeronError` | struct |  |  |  |  |  |

| `at` | function | at(message: impl Into<String>, offset: usize) |  |  |  |  |

| `new` | function | new(message: impl Into<String>) |  |  |  |  |

| `with_source_location` | function | with_source_location(&self, source: &str, label: &str) |  |  |  |  |

| `EyeronError` | struct |  |  |  |  |  |

| `TokenKind` | enum |  |  |  |  |  |

| `lex` | function | lex(input: &str) |  |  |  |  |

| `Token` | struct |  |  |  |  |  |

| `TokenKind` | enum |  |  |  |  |  |

| `lex` | function | lex(input: &str) |  |  |  |  |

| `Token` | struct |  |  |  |  |  |

| `reason` | function | reason(input: &str) |  |  |  |  |

| `reason` | function | reason(input: &str) |  |  |  |  |

| `is_rdf_message_log` | function | is_rdf_message_log(input: &str) |  |  |  |  |

| `parse_n3` | function | parse_n3(input: &str, base_iri: Option<&str>) |  |  |  |  |

| `parse_n3_with_source` | function | parse_n3_with_source(
    input: &str,
    base_iri: Option<&str>,
    source_label: Option<&str>,
) |  |  |  |  |

| `parse_rdf_message_log` | function | parse_rdf_message_log(input: &str, base_iri: Option<&str>) |  |  |  |  |

| `is_rdf_message_log` | function | is_rdf_message_log(input: &str) |  |  |  |  |

| `parse_n3` | function | parse_n3(input: &str, base_iri: Option<&str>) |  |  |  |  |

| `parse_n3_with_source` | function | parse_n3_with_source(
    input: &str,
    base_iri: Option<&str>,
    source_label: Option<&str>,
) |  |  |  |  |

| `parse_rdf_message_log` | function | parse_rdf_message_log(input: &str, base_iri: Option<&str>) |  |  |  |  |

| `document_debug` | function | document_debug(doc: &Document) |  |  |  |  |

| `fuse_report` | function | fuse_report(prefixes: &BTreeMap<String, String>, fuse: &FiredFuse) |  |  |  |  |

| `rdf12_json` | function | rdf12_json(doc: &Document) |  |  |  |  |

| `rdf_result_to_string` | function | rdf_result_to_string(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `result_to_string` | function | result_to_string(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `term_to_n3_object` | function | term_to_n3_object(term: &Term, prefixes: &BTreeMap<String, String>) |  |  |  |  |

| `term_to_n3_predicate` | function | term_to_n3_predicate(term: &Term, prefixes: &BTreeMap<String, String>) |  |  |  |  |

| `triple_to_n3` | function | triple_to_n3(prefixes: &BTreeMap<String, String>, t: &Triple) |  |  |  |  |

| `triples_to_n3` | function | triples_to_n3(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `triples_to_trig` | function | triples_to_trig(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `document_debug` | function | document_debug(doc: &Document) |  |  |  |  |

| `fuse_report` | function | fuse_report(prefixes: &BTreeMap<String, String>, fuse: &FiredFuse) |  |  |  |  |

| `rdf12_json` | function | rdf12_json(doc: &Document) |  |  |  |  |

| `rdf_result_to_string` | function | rdf_result_to_string(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `result_to_string` | function | result_to_string(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `term_to_n3_object` | function | term_to_n3_object(term: &Term, prefixes: &BTreeMap<String, String>) |  |  |  |  |

| `term_to_n3_predicate` | function | term_to_n3_predicate(term: &Term, prefixes: &BTreeMap<String, String>) |  |  |  |  |

| `triple_to_n3` | function | triple_to_n3(prefixes: &BTreeMap<String, String>, t: &Triple) |  |  |  |  |

| `triples_to_n3` | function | triples_to_n3(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `triples_to_trig` | function | triples_to_trig(prefixes: &BTreeMap<String, String>, triples: &[Triple]) |  |  |  |  |

| `Checked` | enum |  |  |  |  |  |

| `Kind` | enum |  |  |  |  |  |

| `Resolution` | enum |  |  |  |  |  |

| `check` | function | check(document: &dyn Document) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `valid` | function | valid(&self) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `Failure` | struct |  |  |  |  |  |

| `Obligation` | struct |  |  |  |  |  |

| `Report` | struct |  |  |  |  |  |

| `Document` | trait |  |  |  |  |  |

| `Checked` | enum |  |  |  |  |  |

| `Kind` | enum |  |  |  |  |  |

| `Resolution` | enum |  |  |  |  |  |

| `check` | function | check(document: &dyn Document) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `valid` | function | valid(&self) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `Failure` | struct |  |  |  |  |  |

| `Obligation` | struct |  |  |  |  |  |

| `Report` | struct |  |  |  |  |  |

| `Document` | trait |  |  |  |  |  |

| `check_proof` | function | check_proof(source: &str, proof: &str, label: &str) |  |  |  |  |

| `check_proof_document` | function | check_proof_document(source: &crate::ast::Document, proof: &str) |  |  |  |  |

| `read_document` | function | read_document(document: &crate::ast::Document, proof: &str) |  |  |  |  |

| `N3Proof` | struct |  |  |  |  |  |

| `check_proof` | function | check_proof(source: &str, proof: &str, label: &str) |  |  |  |  |

| `check_proof_document` | function | check_proof_document(source: &crate::ast::Document, proof: &str) |  |  |  |  |

| `read_document` | function | read_document(document: &crate::ast::Document, proof: &str) |  |  |  |  |

| `N3Proof` | struct |  |  |  |  |  |

| `proof_to_n3` | function | proof_to_n3(prefixes: &BTreeMap<String, String>, result: &ReasonerResult) |  |  |  |  |

| `rule_statement` | function | rule_statement(rule: &Rule) |  |  |  |  |

| `proof_to_n3` | function | proof_to_n3(prefixes: &BTreeMap<String, String>, result: &ReasonerResult) |  |  |  |  |

| `rule_statement` | function | rule_statement(rule: &Rule) |  |  |  |  |

| `RdfFormat` | enum |  |  |  |  |  |

| `parse` | function | parse(value: &str) |  |  |  |  |

| `parse_rdf12` | function | parse_rdf12(input: &str, base_iri: Option<&str>, format: RdfFormat) |  |  |  |  |

| `RdfFormat` | enum |  |  |  |  |  |

| `parse` | function | parse(value: &str) |  |  |  |  |

| `parse_rdf12` | function | parse_rdf12(input: &str, base_iri: Option<&str>, format: RdfFormat) |  |  |  |  |

| `BackwardStep` | enum |  |  |  |  |  |

| `CompletionStatus` | enum |  |  |  |  |  |

| `ProofNode` | enum |  |  |  |  |  |

| `ReasonerError` | enum |  |  |  |  |  |

| `ReasonerLimit` | enum |  |  |  |  |  |

| `builtin_reads_outside_its_triple` | function | builtin_reads_outside_its_triple(predicate: &Term) |  |  |  |  |

| `explain_backward` | function | explain_backward(
    goal: &Triple,
    facts: &[Triple],
    given: &BTreeSet<Triple>,
    rules: &[Rule],
    max_depth: usize,
    emit: &mut dyn FnMut(BackwardStep) |  |  |  |  |

| `find_backward_proof_for_goal` | function | find_backward_proof_for_goal(
    goal: &Triple,
    facts: &[Triple],
    rules: &[Rule],
    max_depth: usize,
) |  |  |  |  |

| `incomplete_summary` | function | incomplete_summary(&self) |  |  |  |  |

| `is_complete` | function | is_complete(&self) |  |  |  |  |

| `new` | function | new(program: Document) |  |  |  |  |

| `program` | function | program(&self) |  |  |  |  |

| `reason` | function | reason(&self, data: &Document, options: &ReasonerOptions) |  |  |  |  |

| `verify_builtin_triple` | function | verify_builtin_triple(goal: &Triple) |  |  |  |  |

| `DerivedFact` | struct |  |  |  |  |  |

| `FiredFuse` | struct |  |  |  |  |  |

| `PreparedReasoner` | struct |  |  |  |  |  |

| `ReasonerOptions` | struct |  |  |  |  |  |

| `ReasonerResult` | struct |  |  |  |  |  |

| `ReasonerStatistics` | struct |  |  |  |  |  |

| `BackwardStep` | enum |  |  |  |  |  |

| `CompletionStatus` | enum |  |  |  |  |  |

| `ProofNode` | enum |  |  |  |  |  |

| `ReasonerError` | enum |  |  |  |  |  |

| `ReasonerLimit` | enum |  |  |  |  |  |

| `builtin_reads_outside_its_triple` | function | builtin_reads_outside_its_triple(predicate: &Term) |  |  |  |  |

| `explain_backward` | function | explain_backward(
    goal: &Triple,
    facts: &[Triple],
    given: &BTreeSet<Triple>,
    rules: &[Rule],
    max_depth: usize,
    emit: &mut dyn FnMut(BackwardStep) |  |  |  |  |

| `find_backward_proof_for_goal` | function | find_backward_proof_for_goal(
    goal: &Triple,
    facts: &[Triple],
    rules: &[Rule],
    max_depth: usize,
) |  |  |  |  |

| `incomplete_summary` | function | incomplete_summary(&self) |  |  |  |  |

| `is_complete` | function | is_complete(&self) |  |  |  |  |

| `new` | function | new(program: Document) |  |  |  |  |

| `program` | function | program(&self) |  |  |  |  |

| `reason` | function | reason(&self, data: &Document, options: &ReasonerOptions) |  |  |  |  |

| `verify_builtin_triple` | function | verify_builtin_triple(goal: &Triple) |  |  |  |  |

| `DerivedFact` | struct |  |  |  |  |  |

| `FiredFuse` | struct |  |  |  |  |  |

| `PreparedReasoner` | struct |  |  |  |  |  |

| `ReasonerOptions` | struct |  |  |  |  |  |

| `ReasonerResult` | struct |  |  |  |  |  |

| `ReasonerStatistics` | struct |  |  |  |  |  |

| `solve_sudoku_string` | function | solve_sudoku_string(puzzle: &str) |  |  |  |  |

| `solve_sudoku_string` | function | solve_sudoku_string(puzzle: &str) |  |  |  |  |

| `new` | function | new(program: &str, proof: bool) |  |  |  |  |

| `program_facts` | function | program_facts(&self) |  |  |  |  |

| `program_rules` | function | program_rules(&self) |  |  |  |  |

| `reason` | function | reason(input: &str) |  |  |  |  |

| `reason_report` | function | reason_report(&self, data: &str, rdf: bool, rdf_format: &str) |  |  |  |  |

| `reason_with_data` | function | reason_with_data(
    program: &str,
    data: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `reason_with_data_report` | function | reason_with_data_report(
    program: &str,
    data: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `reason_with_options` | function | reason_with_options(
    input: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `version` | function | version() |  |  |  |  |

| `EyeronSession` | struct |  |  |  |  |  |

| `new` | function | new(program: &str, proof: bool) |  |  |  |  |

| `program_facts` | function | program_facts(&self) |  |  |  |  |

| `program_rules` | function | program_rules(&self) |  |  |  |  |

| `reason` | function | reason(input: &str) |  |  |  |  |

| `reason_report` | function | reason_report(&self, data: &str, rdf: bool, rdf_format: &str) |  |  |  |  |

| `reason_with_data` | function | reason_with_data(
    program: &str,
    data: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `reason_with_data_report` | function | reason_with_data_report(
    program: &str,
    data: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `reason_with_options` | function | reason_with_options(
    input: &str,
    proof: bool,
    rdf: bool,
    rdf_format: &str,
) |  |  |  |  |

| `version` | function | version() |  |  |  |  |

| `EyeronSession` | struct |  |  |  |  |  |

| `call` | function | call(request: &[u8]) |  |  |  |  |

| `call_json` | function | call_json(request: &Value) |  |  |  |  |

| `dialect_by_name` | function | dialect_by_name(name: &str) |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() |  |  |  |  |

| `plan_from_json` | function | plan_from_json(plan: &Value) |  |  |  |  |

| `Fail` | struct |  |  |  |  |  |

| `AttestError` | enum |  |  |  |  |  |

| `from_bytes` | function | from_bytes(bytes: &[u8; 32]) |  |  |  |  |

| `from_hex` | function | from_hex(hex: &str) |  |  |  |  |

| `from_json` | function | from_json(text: &str) |  |  |  |  |

| `from_seed` | function | from_seed(seed: [u8; 32]) |  |  |  |  |

| `from_seed_hex` | function | from_seed_hex(hex: &str) |  |  |  |  |

| `get` | function | get(&self, key_id: &str) |  |  |  |  |

| `hex_decode` | function | hex_decode(s: &str) |  |  |  |  |

| `hex_encode` | function | hex_encode(bytes: &[u8]) |  |  |  |  |

| `insert` | function | insert(&mut self, key: VerifyingKey) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `key_id` | function | key_id(&self) |  |  |  |  |

| `lease_payload` | function | lease_payload(l: &Lease) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `receipt_payload` | function | receipt_payload(f: &ReceiptFields<'_>) |  |  |  |  |

| `sign_bytes` | function | sign_bytes(key: &SigningKey, payload: &[u8]) |  |  |  |  |

| `sign_lease` | function | sign_lease(key: &SigningKey, lease: Lease) |  |  |  |  |

| `sign_receipt` | function | sign_receipt(key: &SigningKey, receipt: &Receipt) |  |  |  |  |

| `to_hex` | function | to_hex(&self) |  |  |  |  |

| `to_json` | function | to_json(&self) |  |  |  |  |

| `verify_bytes` | function | verify_bytes(
    payload: &[u8],
    att: &Attestation,
    trusted: &TrustedKeys,
) |  |  |  |  |

| `verify_lease` | function | verify_lease(signed: &SignedLease, trusted: &TrustedKeys) |  |  |  |  |

| `verify_receipt` | function | verify_receipt(
    receipt: &Receipt,
    att: &Attestation,
    trusted: &TrustedKeys,
) |  |  |  |  |

| `verifying_key` | function | verifying_key(&self) |  |  |  |  |

| `Attestation` | struct |  |  |  |  |  |

| `ReceiptFields` | struct |  |  |  |  |  |

| `SigningKey` | struct |  |  |  |  |  |

| `TrustedKeys` | struct |  |  |  |  |  |

| `VerifyingKey` | struct |  |  |  |  |  |

| `consequence_authority` | function | consequence_authority(_repository: &str) |  |  |  |  |

| `donor` | function | donor(repository: &str) |  |  |  |  |

| `CapabilityDonor` | struct |  |  |  |  |  |

| `Dialect` | enum |  |  |  |  |  |

| `Engine` | enum |  |  |  |  |  |

| `RefusalKind` | enum |  |  |  |  |  |

| `admit_bytes` | function | admit_bytes(
    bytes: &[u8],
    hint: Option<&str>,
    base: Option<&str>,
) |  |  |  |  |

| `check` | function | check(bytes: &[u8], dialect: Dialect, base: Option<&str>) |  |  |  |  |

| `engine` | function | engine(self) |  |  |  |  |

| `media_type` | function | media_type(self) |  |  |  |  |

| `parse_rdf` | function | parse_rdf(
    bytes: &[u8],
    dialect: Dialect,
    base: Option<&str>,
) |  |  |  |  |

| `sniff` | function | sniff(bytes: &[u8], hint: Option<&str>) |  |  |  |  |

| `Refusal` | struct |  |  |  |  |  |

| `hooks` | function | hooks(&self) |  |  |  |  |

| `load` | function | load(pack: &LawState) |  |  |  |  |

| `materialize` | function | materialize(&self, state: &LawState) |  |  |  |  |

| `Firing` | struct |  |  |  |  |  |

| `Hook` | struct |  |  |  |  |  |

| `HookPack` | struct |  |  |  |  |  |

| `Materialized` | struct |  |  |  |  |  |

| `Ceiling` | enum |  |  |  |  |  |

| `LawError` | enum |  |  |  |  |  |

| `LeaseReason` | enum |  |  |  |  |  |

| `N3Error` | enum |  |  |  |  |  |

| `ReceiptReason` | enum |  |  |  |  |  |

| `Step` | enum |  |  |  |  |  |

| `as_str` | function | as_str(self) |  |  |  |  |

| `authorize` | function | authorize(
        &self,
        step: &str,
        required: Ceiling,
        trusted: &crate::attest::TrustedKeys,
        clock: &dyn Clock,
        max_skew_secs: u64,
    ) |  |  |  |  |

| `dataset` | function | dataset(&self) |  |  |  |  |

| `from_dataset` | function | from_dataset(dataset: Arc<purrdf::RdfDataset>) |  |  |  |  |

| `id` | function | id(&self) |  |  |  |  |

| `name` | function | name(&self) |  |  |  |  |

| `parse` | function | parse(s: &str) |  |  |  |  |

| `quad_count` | function | quad_count(&self) |  |  |  |  |

| `reason_n3_bounded` | function | reason_n3_bounded(input: &str) |  |  |  |  |

| `required_ceiling` | function | required_ceiling(&self) |  |  |  |  |

| `transition` | function | transition(&self, step: &Step<'_>) |  |  |  |  |

| `transition_authorized` | function | transition_authorized(
        &self,
        signed: &SignedLease,
        trusted: &crate::attest::TrustedKeys,
        clock: &dyn Clock,
        max_skew_secs: u64,
        step: &Step<'_>,
    ) |  |  |  |  |

| `transition_leased_unverified` | function | transition_leased_unverified(
        &self,
        lease: &Lease,
        step: &Step<'_>,
        now_unix: u64,
    ) |  |  |  |  |

| `with_subject` | function | with_subject(mut self, subject_sha256: impl Into<String>) |  |  |  |  |

| `FixedClock` | struct |  |  |  |  |  |

| `LawState` | struct |  |  |  |  |  |

| `Lease` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `SignedLease` | struct |  |  |  |  |  |

| `SystemClock` | struct |  |  |  |  |  |

| `Violation` | struct |  |  |  |  |  |

| `Clock` | trait |  |  |  |  |  |

| `BackendAuthority` | struct |  |  |  |  |  |

| `TripleError` | enum |  |  |  |  |  |

| `action` | function | action(mut self, name: impl Into<String>) |  |  |  |  |

| `adds` | function | adds(mut self, t: Triple) |  |  |  |  |

| `admit` | function | admit(&self, start: &LawState) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `build` | function | build(self) |  |  |  |  |

| `builder` | function | builder(name: impl Into<String>) |  |  |  |  |

| `canonical_json` | function | canonical_json(&self) |  |  |  |  |

| `deletes` | function | deletes(mut self, t: Triple) |  |  |  |  |

| `digest` | function | digest(&self) |  |  |  |  |

| `goal` | function | goal(mut self, t: Triple) |  |  |  |  |

| `goal_not` | function | goal_not(mut self, t: Triple) |  |  |  |  |

| `iri` | function | iri(s: &str, p: &str, o: &str) |  |  |  |  |

| `literal` | function | literal(s: &str, p: &str, text: &str) |  |  |  |  |

| `new` | function | new(name: impl Into<String>) |  |  |  |  |

| `requires` | function | requires(mut self, t: Triple) |  |  |  |  |

| `requires_not` | function | requires_not(mut self, t: Triple) |  |  |  |  |

| `Action` | struct |  |  |  |  |  |

| `ActionBuilder` | struct |  |  |  |  |  |

| `Admitted` | struct |  |  |  |  |  |

| `Plan` | struct |  |  |  |  |  |

| `PlanBuilder` | struct |  |  |  |  |  |

| `Triple` | struct |  |  |  |  |  |

| `PolicyRefusalKind` | enum |  |  |  |  |  |

| `admit` | function | admit(problem_json: &str, policy_json: &str) |  |  |  |  |

| `admit_parsed` | function | admit_parsed(problem: &Problem, policy: &[Entry]) |  |  |  |  |

| `as_str` | function | as_str(self) |  |  |  |  |

| `to_ntriples` | function | to_ntriples(&self) |  |  |  |  |

| `Entry` | struct |  |  |  |  |  |

| `Outcome` | struct |  |  |  |  |  |

| `PolicyAdmitted` | struct |  |  |  |  |  |

| `PolicyRefused` | struct |  |  |  |  |  |

| `Problem` | struct |  |  |  |  |  |

| `Standing` | enum |  |  |  |  |  |

| `compose_courts` | function | compose_courts(parent: &Value, child: &Value) |  |  |  |  |

| `evaluate` | function | evaluate(observation: Observation) |  |  |  |  |

| `generate_falsifier` | function | generate_falsifier(law: &Value, seed: &Value) |  |  |  |  |

| `temporal_rule_holds` | function | temporal_rule_holds(events: &[&str], rule: &str) |  |  |  |  |

| `Observation` | struct |  |  |  |  |  |

| `record` | function | record(state: &LawState, receipt: &Receipt) |  |  |  |  |

| `record_signed` | function | record_signed(
    state: &LawState,
    receipt: &Receipt,
    attestation: &Attestation,
) |  |  |  |  |

| `require` | function | require(state: &LawState, step: &str) |  |  |  |  |

| `require_signed` | function | require_signed(state: &LawState, step: &str, trusted: &TrustedKeys) |  |  |  |  |

| `StoreError` | enum |  |  |  |  |  |

| `list` | function | list(&self, subject_sha: &str) |  |  |  |  |

| `open` | function | open(dir: impl AsRef<Path>) |  |  |  |  |

| `put` | function | put(&self, receipt: &Receipt, subject_sha: &str) |  |  |  |  |

| `put_signed` | function | put_signed(
        &self,
        receipt: &Receipt,
        subject_sha: &str,
        attestation: &Attestation,
    ) |  |  |  |  |

| `receipt_digest` | function | receipt_digest(r: &Receipt) |  |  |  |  |

| `verify` | function | verify(&self, subject_sha: &str) |  |  |  |  |

| `verify_attested` | function | verify_attested(
        &self,
        subject_sha: &str,
        trusted: &TrustedKeys,
    ) |  |  |  |  |

| `ReceiptStore` | struct |  |  |  |  |  |

| `FieldDefault` | enum |  |  |  |  |  |

| `canonical` | function | canonical(v: &Value) |  |  |  |  |

| `op_names` | function | op_names() |  |  |  |  |

| `other_dialect_names` | function | other_dialect_names() |  |  |  |  |

| `rdf_dialect_names` | function | rdf_dialect_names() |  |  |  |  |

| `registry_json_pretty` | function | registry_json_pretty() |  |  |  |  |

| `registry_sha256` | function | registry_sha256() |  |  |  |  |

| `registry_turtle` | function | registry_turtle() |  |  |  |  |

| `registry_value` | function | registry_value() |  |  |  |  |

| `surface_sha256` | function | surface_sha256() |  |  |  |  |

| `DialectRow` | struct |  |  |  |  |  |

| `Field` | struct |  |  |  |  |  |

| `LawStepRow` | struct |  |  |  |  |  |

| `Op` | struct |  |  |  |  |  |

| `RefusalCodeRow` | struct |  |  |  |  |  |

| `Variant` | struct |  |  |  |  |  |

| `broaden` | function | broaden(
    ds: &RdfDataset,
    canonical_topic: &str,
) |  |  |  |  |

| `build_dataset` | function | build_dataset(
    turns: &[Turn],
    session_id: &str,
    base: &str,
) |  |  |  |  |

| `classify_turn` | function | classify_turn(text: &str) |  |  |  |  |

| `extract_topic` | function | extract_topic(text: &str) |  |  |  |  |

| `from_env` | function | from_env() |  |  |  |  |

| `read_turtle` | function | read_turtle(path: &Path) |  |  |  |  |

| `required` | function | required(&self, flag: &str) |  |  |  |  |

| `switch` | function | switch(&self, flag: &str) |  |  |  |  |

| `to_turtle` | function | to_turtle(ds: &RdfDataset) |  |  |  |  |

| `turns_from_jsonl` | function | turns_from_jsonl(raw: &str, assistant_only: bool) |  |  |  |  |

| `value` | function | value(&self, flag: &str) |  |  |  |  |

| `write_turtle` | function | write_turtle(ds: &RdfDataset, out: &Path) |  |  |  |  |

| `Args` | struct |  |  |  |  |  |

| `Broadened` | struct |  |  |  |  |  |

| `Turn` | struct |  |  |  |  |  |

| `native` | function | native(req: &Value) |  |  |  |  |

| `native_bytes` | function | native_bytes(req: &Value) |  |  |  |  |

| `ok` | function | ok(req: &Value) |  |  |  |  |

| `read` | function | read(path: &str) |  |  |  |  |

| `refused` | function | refused(req: &Value) |  |  |  |  |

| `wasm` | function | wasm(req: &Value) |  |  |  |  |

| `wasm_bytes` | function | wasm_bytes(req: &Value) |  |  |  |  |

| `wasm_path` | function | wasm_path() |  |  |  |  |

| `req` | function | req(start: &str) |  |  |  |  |

| `AlgebraicClass` | enum |  |  |  |  |  |

| `ScalarvalKind` | enum |  |  |  |  |  |

| `describe` | function | describe(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(name: &'static str, kind: ScalarvalKind) |  |  |  |  |

| `register` | function | register(&mut self, iri: impl Into<String>, aggregate: Arc<dyn CustomAggregate>) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `AggDescriptor` | struct |  |  |  |  |  |

| `AggregateRegistry` | struct |  |  |  |  |  |

| `ScalarvalSpec` | struct |  |  |  |  |  |

| `AggregateAccumulator` | trait |  |  |  |  |  |

| `CustomAggregate` | trait |  |  |  |  |  |

| `AlgebraicClass` | enum |  |  |  |  |  |

| `ScalarvalKind` | enum |  |  |  |  |  |

| `describe` | function | describe(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(name: &'static str, kind: ScalarvalKind) |  |  |  |  |

| `register` | function | register(&mut self, iri: impl Into<String>, aggregate: Arc<dyn CustomAggregate>) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `AggDescriptor` | struct |  |  |  |  |  |

| `AggregateRegistry` | struct |  |  |  |  |  |

| `ScalarvalSpec` | struct |  |  |  |  |  |

| `AggregateAccumulator` | trait |  |  |  |  |  |

| `CustomAggregate` | trait |  |  |  |  |  |

| `ShaclPrebinding` | enum |  |  |  |  |  |

| `cached_plan_count` | function | cached_plan_count(&self) |  |  |  |  |

| `explain_query` | function | explain_query(
        &self,
        dataset: &Arc<RdfDataset>,
        query_text: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `explain_query_with_options` | function | explain_query_with_options(
        &self,
        dataset: &Arc<RdfDataset>,
        query_text: &str,
        base_iri: Option<&str>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `memory_observer` | function | memory_observer(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `order_cache_stats` | function | order_cache_stats(&self) |  |  |  |  |

| `plan_cache_stats` | function | plan_cache_stats(&self) |  |  |  |  |

| `plan_memory_observer` | function | plan_memory_observer(&self) |  |  |  |  |

| `prepare` | function | prepare(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `prepare_algebra` | function | prepare_algebra(
        &self,
        query: Query,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `prepare_query` | function | prepare_query(
        &self,
        query: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `prepare_query_with_options` | function | prepare_query_with_options(
        &self,
        query: &str,
        base_iri: Option<&str>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `prepare_with` | function | prepare_with(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
        options: &ParserOptions,
    ) |  |  |  |  |

| `prepare_with_relations` | function | prepare_with_relations(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
        options: &ParserOptions,
        relations: &crate::property_fn::PropertyFunctionRegistry,
        aggregates: &crate::agg_fn::AggregateRegistry,
    ) |  |  |  |  |

| `query_governed` | function | query_governed(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
        governors: &QueryGovernors,
    ) |  |  |  |  |

| `query_governed_with_source` | function | query_governed_with_source(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        source: &(dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `query_prepared` | function | query_prepared(
        &self,
        dataset: &Arc<RdfDataset>,
        prepared: &PreparedQuery,
        substitutions: &[(String, TermValue) |  |  |  |  |

| `query_with_source` | function | query_with_source(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        source: &(dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `retained_size_bytes` | function | retained_size_bytes(&self) |  |  |  |  |

| `rewritten` | function | rewritten(query: Query, options: QueryOptions<'_>) |  |  |  |  |

| `stats` | function | stats(&self) |  |  |  |  |

| `update_governed` | function | update_governed(
        &self,
        dataset: &mut Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
        governors: &QueryGovernors,
    ) |  |  |  |  |

| `update_with_options` | function | update_with_options(
        &self,
        dataset: &mut Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `with_eval_options` | function | with_eval_options(mut self, options: EvalOptions) |  |  |  |  |

| `with_limits` | function | with_limits(limits: CacheLimits) |  |  |  |  |

| `with_loss_vocabulary` | function | with_loss_vocabulary(mut self, vocab: LossVocabulary) |  |  |  |  |

| `with_order_cache_limits` | function | with_order_cache_limits(mut self, limits: CacheLimits) |  |  |  |  |

| `with_parser_options` | function | with_parser_options(mut self, options: ParserOptions) |  |  |  |  |

| `with_plan_cache_limits` | function | with_plan_cache_limits(mut self, limits: CacheLimits) |  |  |  |  |

| `with_resolver` | function | with_resolver(mut self, resolver: Arc<dyn GraphResolver>) |  |  |  |  |

| `with_standpoint_predicates` | function | with_standpoint_predicates(mut self, predicates: StandpointPredicates) |  |  |  |  |

| `NativeSparqlEngine` | struct |  |  |  |  |  |

| `PlanCache` | struct |  |  |  |  |  |

| `PreparedQuery` | struct |  |  |  |  |  |

| `QueryOptions` | struct |  |  |  |  |  |

| `ShaclPrebinding` | enum |  |  |  |  |  |

| `cached_plan_count` | function | cached_plan_count(&self) |  |  |  |  |

| `explain_query` | function | explain_query(
        &self,
        dataset: &Arc<RdfDataset>,
        query_text: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `explain_query_with_options` | function | explain_query_with_options(
        &self,
        dataset: &Arc<RdfDataset>,
        query_text: &str,
        base_iri: Option<&str>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `memory_observer` | function | memory_observer(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `order_cache_stats` | function | order_cache_stats(&self) |  |  |  |  |

| `plan_cache_stats` | function | plan_cache_stats(&self) |  |  |  |  |

| `plan_memory_observer` | function | plan_memory_observer(&self) |  |  |  |  |

| `prepare` | function | prepare(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `prepare_algebra` | function | prepare_algebra(
        &self,
        query: Query,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `prepare_query` | function | prepare_query(
        &self,
        query: &str,
        base_iri: Option<&str>,
    ) |  |  |  |  |

| `prepare_query_with_options` | function | prepare_query_with_options(
        &self,
        query: &str,
        base_iri: Option<&str>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `prepare_with` | function | prepare_with(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
        options: &ParserOptions,
    ) |  |  |  |  |

| `prepare_with_relations` | function | prepare_with_relations(
        &mut self,
        query: &str,
        base_iri: Option<&str>,
        options: &ParserOptions,
        relations: &crate::property_fn::PropertyFunctionRegistry,
        aggregates: &crate::agg_fn::AggregateRegistry,
    ) |  |  |  |  |

| `query_governed` | function | query_governed(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
        governors: &QueryGovernors,
    ) |  |  |  |  |

| `query_governed_with_source` | function | query_governed_with_source(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        source: &(dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `query_prepared` | function | query_prepared(
        &self,
        dataset: &Arc<RdfDataset>,
        prepared: &PreparedQuery,
        substitutions: &[(String, TermValue) |  |  |  |  |

| `query_with_source` | function | query_with_source(
        &self,
        dataset: &Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        source: &(dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `retained_size_bytes` | function | retained_size_bytes(&self) |  |  |  |  |

| `rewritten` | function | rewritten(query: Query, options: QueryOptions<'_>) |  |  |  |  |

| `stats` | function | stats(&self) |  |  |  |  |

| `update_governed` | function | update_governed(
        &self,
        dataset: &mut Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
        governors: &QueryGovernors,
    ) |  |  |  |  |

| `update_with_options` | function | update_with_options(
        &self,
        dataset: &mut Arc<RdfDataset>,
        request: SparqlRequest<'_>,
        options: QueryOptions<'_>,
    ) |  |  |  |  |

| `with_eval_options` | function | with_eval_options(mut self, options: EvalOptions) |  |  |  |  |

| `with_limits` | function | with_limits(limits: CacheLimits) |  |  |  |  |

| `with_loss_vocabulary` | function | with_loss_vocabulary(mut self, vocab: LossVocabulary) |  |  |  |  |

| `with_order_cache_limits` | function | with_order_cache_limits(mut self, limits: CacheLimits) |  |  |  |  |

| `with_parser_options` | function | with_parser_options(mut self, options: ParserOptions) |  |  |  |  |

| `with_plan_cache_limits` | function | with_plan_cache_limits(mut self, limits: CacheLimits) |  |  |  |  |

| `with_resolver` | function | with_resolver(mut self, resolver: Arc<dyn GraphResolver>) |  |  |  |  |

| `with_standpoint_predicates` | function | with_standpoint_predicates(mut self, predicates: StandpointPredicates) |  |  |  |  |

| `NativeSparqlEngine` | struct |  |  |  |  |  |

| `PlanCache` | struct |  |  |  |  |  |

| `PreparedQuery` | struct |  |  |  |  |  |

| `QueryOptions` | struct |  |  |  |  |  |

| `GraphBuildError` | enum |  |  |  |  |  |

| `GraphBuildStats` | struct |  |  |  |  |  |

| `GraphBuildError` | enum |  |  |  |  |  |

| `GraphBuildStats` | struct |  |  |  |  |  |

| `EvalError` | enum |  |  |  |  |  |

| `UnsupportedKind` | enum |  |  |  |  |  |

| `code` | function | code(self) |  |  |  |  |

| `config` | function | config(what: impl Into<String>) |  |  |  |  |

| `data` | function | data(what: impl Into<String>) |  |  |  |  |

| `diagnostic_code` | function | diagnostic_code(&self) |  |  |  |  |

| `function` | function | function(what: impl Into<String>) |  |  |  |  |

| `internal` | function | internal(what: impl Into<String>) |  |  |  |  |

| `remote` | function | remote(what: impl Into<String>) |  |  |  |  |

| `unsupported` | function | unsupported(what: impl Into<String>) |  |  |  |  |

| `EvalError` | enum |  |  |  |  |  |

| `UnsupportedKind` | enum |  |  |  |  |  |

| `code` | function | code(self) |  |  |  |  |

| `config` | function | config(what: impl Into<String>) |  |  |  |  |

| `data` | function | data(what: impl Into<String>) |  |  |  |  |

| `diagnostic_code` | function | diagnostic_code(&self) |  |  |  |  |

| `function` | function | function(what: impl Into<String>) |  |  |  |  |

| `internal` | function | internal(what: impl Into<String>) |  |  |  |  |

| `remote` | function | remote(what: impl Into<String>) |  |  |  |  |

| `unsupported` | function | unsupported(what: impl Into<String>) |  |  |  |  |

| `Outcome` | enum |  |  |  |  |  |

| `new` | function | new(according_to: impl Into<String>, sharpens: impl Into<String>) |  |  |  |  |

| `with_aggregates` | function | with_aggregates(mut self, registry: &'d crate::agg_fn::AggregateRegistry) |  |  |  |  |

| `with_bnode_mint_prefix` | function | with_bnode_mint_prefix(mut self, prefix: &str) |  |  |  |  |

| `with_call_depth` | function | with_call_depth(mut self, depth: u32) |  |  |  |  |

| `with_eval_options` | function | with_eval_options(mut self, options: EvalOptions) |  |  |  |  |

| `with_focus_graph` | function | with_focus_graph(mut self, graph: &'d Arc<RdfDataset>) |  |  |  |  |

| `with_governors` | function | with_governors(mut self, governors: Arc<GovernorState>) |  |  |  |  |

| `with_loss_vocabulary` | function | with_loss_vocabulary(mut self, vocab: LossVocabulary) |  |  |  |  |

| `with_order_cache` | function | with_order_cache(mut self, cache: &'d BgpOrderCache) |  |  |  |  |

| `with_property_functions` | function | with_property_functions(
        mut self,
        registry: &'d crate::property_fn::PropertyFunctionRegistry,
    ) |  |  |  |  |

| `with_remote` | function | with_remote(mut self, source: &'d (dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `with_standpoint_predicates` | function | with_standpoint_predicates(mut self, predicates: StandpointPredicates) |  |  |  |  |

| `with_user_functions` | function | with_user_functions(
        mut self,
        registry: &'d crate::user_fn::UserFunctionRegistry,
    ) |  |  |  |  |

| `EvalCtx` | struct |  |  |  |  |  |

| `EvalOptions` | struct |  |  |  |  |  |

| `LossVocabulary` | struct |  |  |  |  |  |

| `StandpointPredicates` | struct |  |  |  |  |  |

| `Outcome` | enum |  |  |  |  |  |

| `new` | function | new(according_to: impl Into<String>, sharpens: impl Into<String>) |  |  |  |  |

| `with_aggregates` | function | with_aggregates(mut self, registry: &'d crate::agg_fn::AggregateRegistry) |  |  |  |  |

| `with_bnode_mint_prefix` | function | with_bnode_mint_prefix(mut self, prefix: &str) |  |  |  |  |

| `with_call_depth` | function | with_call_depth(mut self, depth: u32) |  |  |  |  |

| `with_eval_options` | function | with_eval_options(mut self, options: EvalOptions) |  |  |  |  |

| `with_focus_graph` | function | with_focus_graph(mut self, graph: &'d Arc<RdfDataset>) |  |  |  |  |

| `with_governors` | function | with_governors(mut self, governors: Arc<GovernorState>) |  |  |  |  |

| `with_loss_vocabulary` | function | with_loss_vocabulary(mut self, vocab: LossVocabulary) |  |  |  |  |

| `with_order_cache` | function | with_order_cache(mut self, cache: &'d BgpOrderCache) |  |  |  |  |

| `with_property_functions` | function | with_property_functions(
        mut self,
        registry: &'d crate::property_fn::PropertyFunctionRegistry,
    ) |  |  |  |  |

| `with_remote` | function | with_remote(mut self, source: &'d (dyn crate::remote::ServiceResolver + Sync) |  |  |  |  |

| `with_standpoint_predicates` | function | with_standpoint_predicates(mut self, predicates: StandpointPredicates) |  |  |  |  |

| `with_user_functions` | function | with_user_functions(
        mut self,
        registry: &'d crate::user_fn::UserFunctionRegistry,
    ) |  |  |  |  |

| `EvalCtx` | struct |  |  |  |  |  |

| `EvalOptions` | struct |  |  |  |  |  |

| `LossVocabulary` | struct |  |  |  |  |  |

| `StandpointPredicates` | struct |  |  |  |  |  |

| `FallibleSparqlError` | enum |  |  |  |  |  |

| `diagnostic` | function | diagnostic(&self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `into_parts` | function | into_parts(self) |  |  |  |  |

| `operational_error` | function | operational_error(&self) |  |  |  |  |

| `partial_answers` | function | partial_answers(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `CompleteSparqlResult` | struct |  |  |  |  |  |

| `FallibleSparqlError` | enum |  |  |  |  |  |

| `diagnostic` | function | diagnostic(&self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `into_parts` | function | into_parts(self) |  |  |  |  |

| `operational_error` | function | operational_error(&self) |  |  |  |  |

| `partial_answers` | function | partial_answers(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `CompleteSparqlResult` | struct |  |  |  |  |  |

| `GovernedOutcome` | enum |  |  |  |  |  |

| `GovernedUpdateOutcome` | enum |  |  |  |  |  |

| `PartialAnswers` | enum |  |  |  |  |  |

| `barrier` | function | barrier(&self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `exhausted` | function | exhausted(&self) |  |  |  |  |

| `into_complete` | function | into_complete(self) |  |  |  |  |

| `into_result` | function | into_result(self) |  |  |  |  |

| `is_applied` | function | is_applied(&self) |  |  |  |  |

| `is_certain` | function | is_certain(&self) |  |  |  |  |

| `is_complete` | function | is_complete(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_positional_prefix` | function | is_positional_prefix(&self) |  |  |  |  |

| `relations` | function | relations(&self) |  |  |  |  |

| `result` | function | result(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `withholding_blank_nodes` | function | withholding_blank_nodes(self, mut withhold: impl FnMut(&str) |  |  |  |  |

| `BudgetExhausted` | struct |  |  |  |  |  |

| `GovernedEvidence` | struct |  |  |  |  |  |

| `PartialSparqlResult` | struct |  |  |  |  |  |

| `RelationIdentity` | struct |  |  |  |  |  |

| `GovernedOutcome` | enum |  |  |  |  |  |

| `GovernedUpdateOutcome` | enum |  |  |  |  |  |

| `PartialAnswers` | enum |  |  |  |  |  |

| `barrier` | function | barrier(&self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `exhausted` | function | exhausted(&self) |  |  |  |  |

| `into_complete` | function | into_complete(self) |  |  |  |  |

| `into_result` | function | into_result(self) |  |  |  |  |

| `is_applied` | function | is_applied(&self) |  |  |  |  |

| `is_certain` | function | is_certain(&self) |  |  |  |  |

| `is_complete` | function | is_complete(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_positional_prefix` | function | is_positional_prefix(&self) |  |  |  |  |

| `relations` | function | relations(&self) |  |  |  |  |

| `result` | function | result(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `withholding_blank_nodes` | function | withholding_blank_nodes(self, mut withhold: impl FnMut(&str) |  |  |  |  |

| `BudgetExhausted` | struct |  |  |  |  |  |

| `GovernedEvidence` | struct |  |  |  |  |  |

| `PartialSparqlResult` | struct |  |  |  |  |  |

| `RelationIdentity` | struct |  |  |  |  |  |

| `aggregates` | function | aggregates(&self) |  |  |  |  |

| `current` | function | current() |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `fuel_at` | function | fuel_at(&self, point: ChargePoint) |  |  |  |  |

| `fuel_total` | function | fuel_total(&self) |  |  |  |  |

| `join_orders` | function | join_orders(&self) |  |  |  |  |

| `ledger` | function | ledger(&self) |  |  |  |  |

| `peak_cells` | function | peak_cells(&self) |  |  |  |  |

| `profile` | function | profile(&self) |  |  |  |  |

| `relations` | function | relations(&self) |  |  |  |  |

| `render` | function | render(&self) |  |  |  |  |

| `NodeCharges` | struct |  |  |  |  |  |

| `PlanEstimate` | struct |  |  |  |  |  |

| `ProfileIdentity` | struct |  |  |  |  |  |

| `QueryExplanation` | struct |  |  |  |  |  |

| `aggregates` | function | aggregates(&self) |  |  |  |  |

| `current` | function | current() |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `fuel_at` | function | fuel_at(&self, point: ChargePoint) |  |  |  |  |

| `fuel_total` | function | fuel_total(&self) |  |  |  |  |

| `join_orders` | function | join_orders(&self) |  |  |  |  |

| `ledger` | function | ledger(&self) |  |  |  |  |

| `peak_cells` | function | peak_cells(&self) |  |  |  |  |

| `profile` | function | profile(&self) |  |  |  |  |

| `relations` | function | relations(&self) |  |  |  |  |

| `render` | function | render(&self) |  |  |  |  |

| `NodeCharges` | struct |  |  |  |  |  |

| `PlanEstimate` | struct |  |  |  |  |  |

| `ProfileIdentity` | struct |  |  |  |  |  |

| `QueryExplanation` | struct |  |  |  |  |  |

| `operator` | function | operator(self) |  |  |  |  |

| `NonMonotoneBarrier` | struct |  |  |  |  |  |

| `operator` | function | operator(self) |  |  |  |  |

| `NonMonotoneBarrier` | struct |  |  |  |  |  |

| `ChargePoint` | enum |  |  |  |  |  |

| `after` | function | after(budget: Duration) |  |  |  |  |

| `cancel` | function | cancel(&self) |  |  |  |  |

| `charge` | function | charge(&self, dimension: ResourceDimension, amount: u64) |  |  |  |  |

| `charge_if_engaged` | function | charge_if_engaged(
        &self,
        dimension: ResourceDimension,
        amount: u64,
    ) |  |  |  |  |

| `charge_point` | function | charge_point(&self, point: ChargePoint) |  |  |  |  |

| `charge_point_if_engaged` | function | charge_point_if_engaged(&self, point: ChargePoint) |  |  |  |  |

| `commit_ordered_items` | function | commit_ordered_items(
        &self,
        per_item: &[ItemCharge],
    ) |  |  |  |  |

| `consumed_in` | function | consumed_in(&self, dimension: ResourceDimension) |  |  |  |  |

| `cost` | function | cost(self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `has_expired` | function | has_expired(&self) |  |  |  |  |

| `is_cancelled` | function | is_cancelled(&self) |  |  |  |  |

| `is_engaged` | function | is_engaged(&self) |  |  |  |  |

| `is_engaged_in` | function | is_engaged_in(&self, dimension: ResourceDimension) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `limits` | function | limits(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `observe_peak` | function | observe_peak(
        &self,
        dimension: ResourceDimension,
        observed: u64,
    ) |  |  |  |  |

| `observe_peak_if_engaged` | function | observe_peak_if_engaged(
        &self,
        dimension: ResourceDimension,
        observed: u64,
    ) |  |  |  |  |

| `poll_stop` | function | poll_stop(&self) |  |  |  |  |

| `record_trip` | function | record_trip(&self, candidate: TrippedGovernor) |  |  |  |  |

| `schedule_index` | function | schedule_index(self) |  |  |  |  |

| `should_abandon` | function | should_abandon(&self) |  |  |  |  |

| `stop_signal` | function | stop_signal(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `with_fuel` | function | with_fuel(mut self, fuel: u64) |  |  |  |  |

| `with_max_answers` | function | with_max_answers(mut self, rows: u64) |  |  |  |  |

| `with_max_intermediate_cells` | function | with_max_intermediate_cells(mut self, cells: u64) |  |  |  |  |

| `with_max_remote_requests` | function | with_max_remote_requests(mut self, requests: u64) |  |  |  |  |

| `with_max_scratch_bytes` | function | with_max_scratch_bytes(mut self, bytes: u64) |  |  |  |  |

| `with_stop_signal` | function | with_stop_signal(mut self, signal: Arc<dyn StopSignal>) |  |  |  |  |

| `CancellationFlag` | struct |  |  |  |  |  |

| `GovernorState` | struct |  |  |  |  |  |

| `ItemCharge` | struct |  |  |  |  |  |

| `QueryGovernors` | struct |  |  |  |  |  |

| `WallDeadline` | struct |  |  |  |  |  |

| `StopSignal` | trait |  |  |  |  |  |

| `ChargePoint` | enum |  |  |  |  |  |

| `after` | function | after(budget: Duration) |  |  |  |  |

| `cancel` | function | cancel(&self) |  |  |  |  |

| `charge` | function | charge(&self, dimension: ResourceDimension, amount: u64) |  |  |  |  |

| `charge_if_engaged` | function | charge_if_engaged(
        &self,
        dimension: ResourceDimension,
        amount: u64,
    ) |  |  |  |  |

| `charge_point` | function | charge_point(&self, point: ChargePoint) |  |  |  |  |

| `charge_point_if_engaged` | function | charge_point_if_engaged(&self, point: ChargePoint) |  |  |  |  |

| `commit_ordered_items` | function | commit_ordered_items(
        &self,
        per_item: &[ItemCharge],
    ) |  |  |  |  |

| `consumed_in` | function | consumed_in(&self, dimension: ResourceDimension) |  |  |  |  |

| `cost` | function | cost(self) |  |  |  |  |

| `evidence` | function | evidence(&self) |  |  |  |  |

| `has_expired` | function | has_expired(&self) |  |  |  |  |

| `is_cancelled` | function | is_cancelled(&self) |  |  |  |  |

| `is_engaged` | function | is_engaged(&self) |  |  |  |  |

| `is_engaged_in` | function | is_engaged_in(&self, dimension: ResourceDimension) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `limits` | function | limits(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `observe_peak` | function | observe_peak(
        &self,
        dimension: ResourceDimension,
        observed: u64,
    ) |  |  |  |  |

| `observe_peak_if_engaged` | function | observe_peak_if_engaged(
        &self,
        dimension: ResourceDimension,
        observed: u64,
    ) |  |  |  |  |

| `poll_stop` | function | poll_stop(&self) |  |  |  |  |

| `record_trip` | function | record_trip(&self, candidate: TrippedGovernor) |  |  |  |  |

| `schedule_index` | function | schedule_index(self) |  |  |  |  |

| `should_abandon` | function | should_abandon(&self) |  |  |  |  |

| `stop_signal` | function | stop_signal(&self) |  |  |  |  |

| `tripped` | function | tripped(&self) |  |  |  |  |

| `with_fuel` | function | with_fuel(mut self, fuel: u64) |  |  |  |  |

| `with_max_answers` | function | with_max_answers(mut self, rows: u64) |  |  |  |  |

| `with_max_intermediate_cells` | function | with_max_intermediate_cells(mut self, cells: u64) |  |  |  |  |

| `with_max_remote_requests` | function | with_max_remote_requests(mut self, requests: u64) |  |  |  |  |

| `with_max_scratch_bytes` | function | with_max_scratch_bytes(mut self, bytes: u64) |  |  |  |  |

| `with_stop_signal` | function | with_stop_signal(mut self, signal: Arc<dyn StopSignal>) |  |  |  |  |

| `CancellationFlag` | struct |  |  |  |  |  |

| `GovernorState` | struct |  |  |  |  |  |

| `ItemCharge` | struct |  |  |  |  |  |

| `QueryGovernors` | struct |  |  |  |  |  |

| `WallDeadline` | struct |  |  |  |  |  |

| `StopSignal` | trait |  |  |  |  |  |

| `Kernel` | enum |  |  |  |  |  |

| `best` | function | best(k: usize, candidates: impl IntoIterator<Item = Ranked>) |  |  |  |  |

| `distance` | function | distance(
        self,
        query: &[f64],
        query_norm: f64,
        candidate: &[f64],
        candidate_norm: f64,
    ) |  |  |  |  |

| `needs_norms` | function | needs_norms(self) |  |  |  |  |

| `norm` | function | norm(vector: &[f64]) |  |  |  |  |

| `of` | function | of(metric: &DistanceMetric) |  |  |  |  |

| `Ranked` | struct |  |  |  |  |  |

| `Kernel` | enum |  |  |  |  |  |

| `best` | function | best(k: usize, candidates: impl IntoIterator<Item = Ranked>) |  |  |  |  |

| `distance` | function | distance(
        self,
        query: &[f64],
        query_norm: f64,
        candidate: &[f64],
        candidate_norm: f64,
    ) |  |  |  |  |

| `needs_norms` | function | needs_norms(self) |  |  |  |  |

| `norm` | function | norm(vector: &[f64]) |  |  |  |  |

| `of` | function | of(metric: &DistanceMetric) |  |  |  |  |

| `Ranked` | struct |  |  |  |  |  |

| `dimension` | function | dimension(&self) |  |  |  |  |

| `from_artifact` | function | from_artifact(
        artifact: &[u8],
        target_set: TargetSetId,
        vector_space: VectorSpaceId,
        bindings: Vec<(TargetId, TermValue) |  |  |  |  |

| `guard` | function | guard(&self) |  |  |  |  |

| `max_candidates` | function | max_candidates(self) |  |  |  |  |

| `max_neighbours` | function | max_neighbours(self) |  |  |  |  |

| `metric` | function | metric(&self) |  |  |  |  |

| `new` | function | new(max_candidates: u64, max_neighbours: u64) |  |  |  |  |

| `row_count` | function | row_count(&self) |  |  |  |  |

| `row_of` | function | row_of(&self, term: &TermValue) |  |  |  |  |

| `space` | function | space(&self) |  |  |  |  |

| `term` | function | term(&self, row: usize) |  |  |  |  |

| `EmbeddingKnnRelation` | struct |  |  |  |  |  |

| `EmbeddingSpace` | struct |  |  |  |  |  |

| `KnnGuard` | struct |  |  |  |  |  |

| `dimension` | function | dimension(&self) |  |  |  |  |

| `from_artifact` | function | from_artifact(
        artifact: &[u8],
        target_set: TargetSetId,
        vector_space: VectorSpaceId,
        bindings: Vec<(TargetId, TermValue) |  |  |  |  |

| `guard` | function | guard(&self) |  |  |  |  |

| `max_candidates` | function | max_candidates(self) |  |  |  |  |

| `max_neighbours` | function | max_neighbours(self) |  |  |  |  |

| `metric` | function | metric(&self) |  |  |  |  |

| `new` | function | new(max_candidates: u64, max_neighbours: u64) |  |  |  |  |

| `row_count` | function | row_count(&self) |  |  |  |  |

| `row_of` | function | row_of(&self, term: &TermValue) |  |  |  |  |

| `space` | function | space(&self) |  |  |  |  |

| `term` | function | term(&self, row: usize) |  |  |  |  |

| `EmbeddingKnnRelation` | struct |  |  |  |  |  |

| `EmbeddingSpace` | struct |  |  |  |  |  |

| `KnnGuard` | struct |  |  |  |  |  |

| `ValueAggregate` | enum |  |  |  |  |  |

| `compare_values` | function | compare_values(a: &TermValue, b: &TermValue) |  |  |  |  |

| `fold_values` | function | fold_values(
    aggregate: ValueAggregate,
    values: &[TermValue],
) |  |  |  |  |

| `order_values` | function | order_values(values: Vec<TermValue>, descending: bool) |  |  |  |  |

| `ValueAggregate` | enum |  |  |  |  |  |

| `compare_values` | function | compare_values(a: &TermValue, b: &TermValue) |  |  |  |  |

| `fold_values` | function | fold_values(
    aggregate: ValueAggregate,
    values: &[TermValue],
) |  |  |  |  |

| `order_values` | function | order_values(values: Vec<TermValue>, descending: bool) |  |  |  |  |

| `PathDirection` | enum |  |  |  |  |  |

| `edge_count` | function | edge_count(&self) |  |  |  |  |

| `max_expansions_per_invocation` | function | max_expansions_per_invocation(&self) |  |  |  |  |

| `max_hops` | function | max_hops(&self) |  |  |  |  |

| `max_paths_per_seed` | function | max_paths_per_seed(&self) |  |  |  |  |

| `min_hops` | function | min_hops(&self) |  |  |  |  |

| `new` | function | new(alternatives: Vec<(TermValue, PathDirection) |  |  |  |  |

| `node_count` | function | node_count(&self) |  |  |  |  |

| `snapshot_fingerprint` | function | snapshot_fingerprint(&self) |  |  |  |  |

| `PathGraph` | struct |  |  |  |  |  |

| `PathLimits` | struct |  |  |  |  |  |

| `PathSnapshotFingerprint` | struct |  |  |  |  |  |

| `PathStep` | struct |  |  |  |  |  |

| `PathWitnessRelation` | struct |  |  |  |  |  |

| `ShortestPathWitnessRelation` | struct |  |  |  |  |  |

| `PathDirection` | enum |  |  |  |  |  |

| `edge_count` | function | edge_count(&self) |  |  |  |  |

| `max_expansions_per_invocation` | function | max_expansions_per_invocation(&self) |  |  |  |  |

| `max_hops` | function | max_hops(&self) |  |  |  |  |

| `max_paths_per_seed` | function | max_paths_per_seed(&self) |  |  |  |  |

| `min_hops` | function | min_hops(&self) |  |  |  |  |

| `new` | function | new(alternatives: Vec<(TermValue, PathDirection) |  |  |  |  |

| `node_count` | function | node_count(&self) |  |  |  |  |

| `snapshot_fingerprint` | function | snapshot_fingerprint(&self) |  |  |  |  |

| `PathGraph` | struct |  |  |  |  |  |

| `PathLimits` | struct |  |  |  |  |  |

| `PathSnapshotFingerprint` | struct |  |  |  |  |  |

| `PathStep` | struct |  |  |  |  |  |

| `PathWitnessRelation` | struct |  |  |  |  |  |

| `ShortestPathWitnessRelation` | struct |  |  |  |  |  |

| `CacheLimits` | struct |  |  |  |  |  |

| `CacheStats` | struct |  |  |  |  |  |

| `CacheLimits` | struct |  |  |  |  |  |

| `CacheStats` | struct |  |  |  |  |  |

| `stats` | function | stats(&self) |  |  |  |  |

| `PlanMemoryObserver` | struct |  |  |  |  |  |

| `PlanMemoryStats` | struct |  |  |  |  |  |

| `stats` | function | stats(&self) |  |  |  |  |

| `PlanMemoryObserver` | struct |  |  |  |  |  |

| `PlanMemoryStats` | struct |  |  |  |  |  |

| `all_free_mode` | function | all_free_mode(self) |  |  |  |  |

| `arity` | function | arity(&self) |  |  |  |  |

| `describe` | function | describe(&self) |  |  |  |  |

| `flattened` | function | flattened(&self) |  |  |  |  |

| `get` | function | get(&self, pos: usize) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `mode` | function | mode(&self) |  |  |  |  |

| `new` | function | new(subject: usize, object: usize) |  |  |  |  |

| `next_contained` | function | next_contained(cursor: &mut dyn PfCursor, iri: &str) |  |  |  |  |

| `object` | function | object(&self) |  |  |  |  |

| `open_contained` | function | open_contained(
    relation: &dyn PropertyFunction,
    iri: &str,
    args: &PfArgs<'_>,
    ceiling: Option<u64>,
) |  |  |  |  |

| `register` | function | register(&mut self, iri: impl Into<String>, relation: Arc<dyn PropertyFunction>) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `rows` | function | rows(&self) |  |  |  |  |

| `subject` | function | subject(&self) |  |  |  |  |

| `take_work_contained` | function | take_work_contained(cursor: &mut dyn PfCursor, iri: &str) |  |  |  |  |

| `total` | function | total(self) |  |  |  |  |

| `MemoryRelation` | struct |  |  |  |  |  |

| `PfArgs` | struct |  |  |  |  |  |

| `PfArity` | struct |  |  |  |  |  |

| `PfDescriptor` | struct |  |  |  |  |  |

| `PfMode` | struct |  |  |  |  |  |

| `PropertyFunctionRegistry` | struct |  |  |  |  |  |

| `PfCursor` | trait |  |  |  |  |  |

| `PropertyFunction` | trait |  |  |  |  |  |

| `all_free_mode` | function | all_free_mode(self) |  |  |  |  |

| `arity` | function | arity(&self) |  |  |  |  |

| `describe` | function | describe(&self) |  |  |  |  |

| `flattened` | function | flattened(&self) |  |  |  |  |

| `get` | function | get(&self, pos: usize) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `mode` | function | mode(&self) |  |  |  |  |

| `new` | function | new(subject: usize, object: usize) |  |  |  |  |

| `next_contained` | function | next_contained(cursor: &mut dyn PfCursor, iri: &str) |  |  |  |  |

| `object` | function | object(&self) |  |  |  |  |

| `open_contained` | function | open_contained(
    relation: &dyn PropertyFunction,
    iri: &str,
    args: &PfArgs<'_>,
    ceiling: Option<u64>,
) |  |  |  |  |

| `register` | function | register(&mut self, iri: impl Into<String>, relation: Arc<dyn PropertyFunction>) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `rows` | function | rows(&self) |  |  |  |  |

| `subject` | function | subject(&self) |  |  |  |  |

| `take_work_contained` | function | take_work_contained(cursor: &mut dyn PfCursor, iri: &str) |  |  |  |  |

| `total` | function | total(self) |  |  |  |  |

| `MemoryRelation` | struct |  |  |  |  |  |

| `PfArgs` | struct |  |  |  |  |  |

| `PfArity` | struct |  |  |  |  |  |

| `PfDescriptor` | struct |  |  |  |  |  |

| `PfMode` | struct |  |  |  |  |  |

| `PropertyFunctionRegistry` | struct |  |  |  |  |  |

| `PfCursor` | trait |  |  |  |  |  |

| `PropertyFunction` | trait |  |  |  |  |  |

| `RemoteError` | enum |  |  |  |  |  |

| `new` | function | new(endpoint: &'a str, query_text: &'a str) |  |  |  |  |

| `silent` | function | silent(mut self, silent: bool) |  |  |  |  |

| `stop_trip` | function | stop_trip(&self) |  |  |  |  |

| `with_max_intermediate_cells` | function | with_max_intermediate_cells(mut self, cells: Option<u64>) |  |  |  |  |

| `with_stop` | function | with_stop(mut self, stop: Option<&'a Arc<dyn StopSignal>>) |  |  |  |  |

| `ResolvedBindings` | struct |  |  |  |  |  |

| `ServiceRequest` | struct |  |  |  |  |  |

| `ServiceResolver` | trait |  |  |  |  |  |

| `RemoteError` | enum |  |  |  |  |  |

| `new` | function | new(endpoint: &'a str, query_text: &'a str) |  |  |  |  |

| `silent` | function | silent(mut self, silent: bool) |  |  |  |  |

| `stop_trip` | function | stop_trip(&self) |  |  |  |  |

| `with_max_intermediate_cells` | function | with_max_intermediate_cells(mut self, cells: Option<u64>) |  |  |  |  |

| `with_stop` | function | with_stop(mut self, stop: Option<&'a Arc<dyn StopSignal>>) |  |  |  |  |

| `ResolvedBindings` | struct |  |  |  |  |  |

| `ServiceRequest` | struct |  |  |  |  |  |

| `ServiceResolver` | trait |  |  |  |  |  |

| `catalog` | function | catalog(&self) |  |  |  |  |

| `new` | function | new(transport: T) |  |  |  |  |

| `with_catalog` | function | with_catalog(mut self, catalog: ServiceCatalog) |  |  |  |  |

| `with_timeout` | function | with_timeout(mut self, timeout: Duration) |  |  |  |  |

| `HttpRemoteQuerySource` | struct |  |  |  |  |  |

| `HttpRequest` | struct |  |  |  |  |  |

| `HttpTransport` | trait |  |  |  |  |  |

| `catalog` | function | catalog(&self) |  |  |  |  |

| `new` | function | new(transport: T) |  |  |  |  |

| `with_catalog` | function | with_catalog(mut self, catalog: ServiceCatalog) |  |  |  |  |

| `with_timeout` | function | with_timeout(mut self, timeout: Duration) |  |  |  |  |

| `HttpRemoteQuerySource` | struct |  |  |  |  |  |

| `HttpRequest` | struct |  |  |  |  |  |

| `HttpTransport` | trait |  |  |  |  |  |

| `SolutionTerm` | enum |  |  |  |  |  |

| `computed_count` | function | computed_count(&self) |  |  |  |  |

| `computed_value` | function | computed_value(&self, sid: ScratchId) |  |  |  |  |

| `minted_bytes` | function | minted_bytes(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `ScratchId` | struct |  |  |  |  |  |

| `ScratchInterner` | struct |  |  |  |  |  |

| `SolutionTerm` | enum |  |  |  |  |  |

| `computed_count` | function | computed_count(&self) |  |  |  |  |

| `computed_value` | function | computed_value(&self, sid: ScratchId) |  |  |  |  |

| `minted_bytes` | function | minted_bytes(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `ScratchId` | struct |  |  |  |  |  |

| `ScratchInterner` | struct |  |  |  |  |  |

| `ServiceCapability` | enum |  |  |  |  |  |

| `ServiceCredential` | enum |  |  |  |  |  |

| `allows` | function | allows(self, capability: ServiceCapability) |  |  |  |  |

| `authorize` | function | authorize(
        &self,
        endpoint: &str,
        needs: ServiceCapabilities,
    ) |  |  |  |  |

| `capabilities` | function | capabilities(&self) |  |  |  |  |

| `catalog` | function | catalog(&self) |  |  |  |  |

| `credential` | function | credential(&self) |  |  |  |  |

| `dataset` | function | dataset(&self, endpoint: &str) |  |  |  |  |

| `detail` | function | detail(&self) |  |  |  |  |

| `endpoint` | function | endpoint(&self) |  |  |  |  |

| `grant` | function | grant(self, capability: ServiceCapability) |  |  |  |  |

| `granting` | function | granting(capabilities: impl IntoIterator<Item = ServiceCapability>) |  |  |  |  |

| `header` | function | header(&self) |  |  |  |  |

| `headers` | function | headers(&self) |  |  |  |  |

| `iter` | function | iter(self) |  |  |  |  |

| `new` | function | new(
        endpoint: impl Into<String>,
        withheld: ServiceCapability,
        detail: impl Into<String>,
    ) |  |  |  |  |

| `profile_for` | function | profile_for(&self, endpoint: &str) |  |  |  |  |

| `request_headers` | function | request_headers(&self) |  |  |  |  |

| `timeout` | function | timeout(&self) |  |  |  |  |

| `user_agent` | function | user_agent(&self) |  |  |  |  |

| `with_catalog` | function | with_catalog(mut self, catalog: ServiceCatalog) |  |  |  |  |

| `with_credential` | function | with_credential(mut self, credential: ServiceCredential) |  |  |  |  |

| `with_endpoint` | function | with_endpoint(mut self, endpoint: impl Into<String>, dataset: Arc<RdfDataset>) |  |  |  |  |

| `with_fallback` | function | with_fallback(mut self, profile: ServiceProfile) |  |  |  |  |

| `with_header` | function | with_header(mut self, name: impl Into<String>, value: impl Into<String>) |  |  |  |  |

| `with_route` | function | with_route(
        mut self,
        endpoint: impl Into<String>,
        resolver: &'a (dyn ServiceResolver + Sync) |  |  |  |  |

| `with_service` | function | with_service(mut self, endpoint: impl Into<String>, profile: ServiceProfile) |  |  |  |  |

| `with_timeout` | function | with_timeout(mut self, timeout: Duration) |  |  |  |  |

| `with_user_agent` | function | with_user_agent(mut self, user_agent: impl Into<String>) |  |  |  |  |

| `withheld` | function | withheld(&self) |  |  |  |  |

| `InProcessServiceResolver` | struct |  |  |  |  |  |

| `ServiceCapabilities` | struct |  |  |  |  |  |

| `ServiceCatalog` | struct |  |  |  |  |  |

| `ServiceDenial` | struct |  |  |  |  |  |

| `ServiceProfile` | struct |  |  |  |  |  |

| `ServiceRouter` | struct |  |  |  |  |  |

| `ServiceCapability` | enum |  |  |  |  |  |

| `ServiceCredential` | enum |  |  |  |  |  |

| `allows` | function | allows(self, capability: ServiceCapability) |  |  |  |  |

| `authorize` | function | authorize(
        &self,
        endpoint: &str,
        needs: ServiceCapabilities,
    ) |  |  |  |  |

| `capabilities` | function | capabilities(&self) |  |  |  |  |

| `catalog` | function | catalog(&self) |  |  |  |  |

| `credential` | function | credential(&self) |  |  |  |  |

| `dataset` | function | dataset(&self, endpoint: &str) |  |  |  |  |

| `detail` | function | detail(&self) |  |  |  |  |

| `endpoint` | function | endpoint(&self) |  |  |  |  |

| `grant` | function | grant(self, capability: ServiceCapability) |  |  |  |  |

| `granting` | function | granting(capabilities: impl IntoIterator<Item = ServiceCapability>) |  |  |  |  |

| `header` | function | header(&self) |  |  |  |  |

| `headers` | function | headers(&self) |  |  |  |  |

| `iter` | function | iter(self) |  |  |  |  |

| `new` | function | new(
        endpoint: impl Into<String>,
        withheld: ServiceCapability,
        detail: impl Into<String>,
    ) |  |  |  |  |

| `profile_for` | function | profile_for(&self, endpoint: &str) |  |  |  |  |

| `request_headers` | function | request_headers(&self) |  |  |  |  |

| `timeout` | function | timeout(&self) |  |  |  |  |

| `user_agent` | function | user_agent(&self) |  |  |  |  |

| `with_catalog` | function | with_catalog(mut self, catalog: ServiceCatalog) |  |  |  |  |

| `with_credential` | function | with_credential(mut self, credential: ServiceCredential) |  |  |  |  |

| `with_endpoint` | function | with_endpoint(mut self, endpoint: impl Into<String>, dataset: Arc<RdfDataset>) |  |  |  |  |

| `with_fallback` | function | with_fallback(mut self, profile: ServiceProfile) |  |  |  |  |

| `with_header` | function | with_header(mut self, name: impl Into<String>, value: impl Into<String>) |  |  |  |  |

| `with_route` | function | with_route(
        mut self,
        endpoint: impl Into<String>,
        resolver: &'a (dyn ServiceResolver + Sync) |  |  |  |  |

| `with_service` | function | with_service(mut self, endpoint: impl Into<String>, profile: ServiceProfile) |  |  |  |  |

| `with_timeout` | function | with_timeout(mut self, timeout: Duration) |  |  |  |  |

| `with_user_agent` | function | with_user_agent(mut self, user_agent: impl Into<String>) |  |  |  |  |

| `withheld` | function | withheld(&self) |  |  |  |  |

| `InProcessServiceResolver` | struct |  |  |  |  |  |

| `ServiceCapabilities` | struct |  |  |  |  |  |

| `ServiceCatalog` | struct |  |  |  |  |  |

| `ServiceDenial` | struct |  |  |  |  |  |

| `ServiceProfile` | struct |  |  |  |  |  |

| `ServiceRouter` | struct |  |  |  |  |  |

| `contains` | function | contains(&self, var: &Variable) |  |  |  |  |

| `empty` | function | empty(schema: Arc<VarSchema>) |  |  |  |  |

| `from_vars` | function | from_vars(vars: impl IntoIterator<Item = Variable>) |  |  |  |  |

| `index_of` | function | index_of(&self, var: &Variable) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `push` | function | push(&mut self, var: Variable) |  |  |  |  |

| `shared_columns` | function | shared_columns(&self, other: &Self) |  |  |  |  |

| `union` | function | union(&self, other: &Self) |  |  |  |  |

| `unit` | function | unit() |  |  |  |  |

| `vars` | function | vars(&self) |  |  |  |  |

| `SolutionSeq` | struct |  |  |  |  |  |

| `VarSchema` | struct |  |  |  |  |  |

| `contains` | function | contains(&self, var: &Variable) |  |  |  |  |

| `empty` | function | empty(schema: Arc<VarSchema>) |  |  |  |  |

| `from_vars` | function | from_vars(vars: impl IntoIterator<Item = Variable>) |  |  |  |  |

| `index_of` | function | index_of(&self, var: &Variable) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `push` | function | push(&mut self, var: Variable) |  |  |  |  |

| `shared_columns` | function | shared_columns(&self, other: &Self) |  |  |  |  |

| `union` | function | union(&self, other: &Self) |  |  |  |  |

| `unit` | function | unit() |  |  |  |  |

| `vars` | function | vars(&self) |  |  |  |  |

| `SolutionSeq` | struct |  |  |  |  |  |

| `VarSchema` | struct |  |  |  |  |  |

| `register_statistical_aggregates` | function | register_statistical_aggregates(&mut self, namespace: &str) |  |  |  |  |

| `register_statistical_aggregates` | function | register_statistical_aggregates(&mut self, namespace: &str) |  |  |  |  |

| `GraphResolveRequest` | struct |  |  |  |  |  |

| `GraphResolver` | trait |  |  |  |  |  |

| `GraphResolveRequest` | struct |  |  |  |  |  |

| `GraphResolver` | trait |  |  |  |  |  |

| `Arity` | enum |  |  |  |  |  |

| `NodeKind` | enum |  |  |  |  |  |

| `UserFnBody` | enum |  |  |  |  |  |

| `Volatility` | enum |  |  |  |  |  |

| `insert` | function | insert(&mut self, iri: impl Into<String>, func: UserFunction) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `register_expr` | function | register_expr(&mut self, iri: impl Into<String>, arity: Arity, body: ExprFnBody) |  |  |  |  |

| `register_native` | function | register_native(
        &mut self,
        iri: impl Into<String>,
        arity: Arity,
        volatility: Volatility,
        body: NativeFnBody,
    ) |  |  |  |  |

| `requires_focus_graph` | function | requires_focus_graph(&self) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `resolve_expr` | function | resolve_expr(&self, iri: &str) |  |  |  |  |

| `resolve_native` | function | resolve_native(&self, iri: &str) |  |  |  |  |

| `ExprFnCall` | struct |  |  |  |  |  |

| `ExprFunction` | struct |  |  |  |  |  |

| `NativeFunction` | struct |  |  |  |  |  |

| `TypeConstraint` | struct |  |  |  |  |  |

| `UserFnParam` | struct |  |  |  |  |  |

| `UserFunction` | struct |  |  |  |  |  |

| `UserFunctionRegistry` | struct |  |  |  |  |  |

| `Arity` | enum |  |  |  |  |  |

| `NodeKind` | enum |  |  |  |  |  |

| `UserFnBody` | enum |  |  |  |  |  |

| `Volatility` | enum |  |  |  |  |  |

| `insert` | function | insert(&mut self, iri: impl Into<String>, func: UserFunction) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `register_expr` | function | register_expr(&mut self, iri: impl Into<String>, arity: Arity, body: ExprFnBody) |  |  |  |  |

| `register_native` | function | register_native(
        &mut self,
        iri: impl Into<String>,
        arity: Arity,
        volatility: Volatility,
        body: NativeFnBody,
    ) |  |  |  |  |

| `requires_focus_graph` | function | requires_focus_graph(&self) |  |  |  |  |

| `resolve` | function | resolve(&self, iri: &str) |  |  |  |  |

| `resolve_expr` | function | resolve_expr(&self, iri: &str) |  |  |  |  |

| `resolve_native` | function | resolve_native(&self, iri: &str) |  |  |  |  |

| `ExprFnCall` | struct |  |  |  |  |  |

| `ExprFunction` | struct |  |  |  |  |  |

| `NativeFunction` | struct |  |  |  |  |  |

| `TypeConstraint` | struct |  |  |  |  |  |

| `UserFnParam` | struct |  |  |  |  |  |

| `UserFunction` | struct |  |  |  |  |  |

| `UserFunctionRegistry` | struct |  |  |  |  |  |

| `alloc_buf` | function | alloc_buf(len: u32) |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `alloc_buf` | function | alloc_buf(len: u32) |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |


<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-END: nothing below this line may describe     -->
<!-- code behavior.                                                -->
<!-- ============================================================= -->
