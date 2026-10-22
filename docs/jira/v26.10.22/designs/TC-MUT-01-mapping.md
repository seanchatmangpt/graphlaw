# TC-MUT-01 / TC-MOCK-00 mapping (design record)

Condensed from the TC-MUT/MOCK audit lane.

## TC-MOCK-00 — LANDED as tests/no_mock_markers.rs
Banned markers: mockall, automock!, jest.mock/vi.mock/sinon.stub, struct Mock*.
Allowlist (adjudicated, load-bearing): MockEdgeAstReader + MockAstReader are
hand-written impls of the real RdfAstGraphReader trait over real parsed data —
real collaborators per Chicago doctrine, not doubles. Recommended criterion
wording: "zero interaction-recording test doubles... hand-written
implementations of production traits over real data are lawful collaborators
regardless of identifier; identifiers carry an allowlist entry with
justification, never a silent exemption." Suggested renames
(AstReaderFixture) would empty the allowlist entirely. Residual: name-based —
also grep Cargo.lock for mockall-via-rename.

## TC-MUT-01 — both arms witnessed separately; two gaps
Arm A (operator enum): document-side refusal witnessed
(disallowed_operator_token_is_refusal_01); SHAPE-side mutation never
exercised — mutation_court.sh GUARDED set is registry/abi only, excludes
ontologies/sql-ast.ttl.
Arm B (aggregate into ProcessScope): halves witnessed separately (aggregate
arms + ProcessScope-leak court); the CONJUNCTION (aggregate node carrying
sql:scope sql:ProcessScope) is not a corpus row.

## Court spec: scripts/shape_mutation_court.sh (MISSING, designed)
Snapshot → mutate in place → run named courts → restore via EXIT trap →
exit 3 on non-flip. Mutations: M1 remove "LIKE" "ILIKE" from sh:in →
disallowed-token docs must newly refuse; M2 inject "SOUNDS-LIKE" into sh:in →
disallowed_operator_token test must FLIP to failure; M3 aggregate+ProcessScope
in WHERE operand → typed REFUSAL-03 + real frozen-duckdb witness (count
unchanged, zero execution). Plus new corpus row aggregate_scope_process_in_where
with benign-twin flip. Court's own anti-vacuity: delete an enum token from its
expected-refusal table → court fails.
