# n3-audit-100-report

Audit order N3-AUDIT-100, wave 3 closeout of the N3 -> `purrdf::n3` migration
audit for /Users/sac/graphlaw, 2026-10-26. Supersedes the wave-2 closeout
previously at this path; wave-2 F-numbers (F1-F12) are preserved below as
cross-references so standing dispositions remain addressable. Wave-3 adds
execution witnesses through the prebuilt `target/debug/n3_derive` binary and
re-adjudicates every wave-2 surface an independent skeptic could re-derive.

## Context

graphlaw retired the in-tree `graphlaw-eyeron` crate and re-homed the
Notation3 pipeline behind the vendored `purrdf-n3` crate (pinned 3.0.1,
crates.io-normalized), reached through the `vendor/purrdf` umbrella path
dependency (`[patch.crates-io]`, root `Cargo.toml`), re-exported as
`graphlaw::n3` (`src/lib.rs:190-193`), named `purrdf::n3` 3.0.1 in
`BACKEND_AUTHORITIES` (`src/lib.rs:246-250`), and consumed by the logic tier
(`src/tier.rs:146-153`), the dialect router (`src/dialect.rs:408-414`), the
ABI `n3` op (`src/abi/ops/query_engines.rs:221-228`), the wasm N3 step
(`src/law.rs:1049`), and `reason_n3_bounded` (`src/law.rs:555-615`). The
vendored copy is workspace-excluded (root `Cargo.toml:17`); upstream tests,
examples, and benches were dropped at vendoring (`vendor/README.md`).

This wave closes the same order as wave 2: full-pipeline soundness of the N3
parse -> derive -> prove -> print path against that authority. Lane count was
raised to 84 concern lanes; verification was adversarial with default-refute.

## Method

- 84 concern lanes dispatched read-only over the canonical checkout
  /Users/sac/graphlaw; no lane created worktrees or clones.
- Builds were prohibited by the audit constraints. Lanes that needed a real
  collaborator executed witnesses through the prebuilt
  `target/debug/n3_derive` binary, with mtime ordering checks (binary newer
  than `vendor/purrdf-n3/src/reasoner.rs`) recorded in each re-derivation, so
  executed witnesses reflect current source. Probe documents were written
  only under /tmp.
- Adversarial verify with default-refute: a filing enters the confirmed
  register only on an independent skeptic re-derivation from the tree
  (grep/sed receipts, or execution output). A filing whose verifier died is
  recorded refuted — loudly, as unadjudicated, never as a silent pass.
- Refutation grounds observed: (a) verifier death (30 filings); (b) facts
  reproduce but classification fails — deliberate-and-receipted behavior,
  misread artifact, or count error (8 filings).
- Court confirmation: 6 courts re-run post-adjudication (all exit 0, 40 cases
  green). A court pass does not clear a finding; each court was checked for
  whether it could ever fire on a confirmed surface (see Courts section).
- Severity adjudication: two filed-majors corrected to minor on
  re-derivation (C11 lexer.rs:316, C12 reasoner.rs:3431). No filed-minor was
  raised.
- Dedup accounting: 58 raw filings delivered; 6 were cross-lane duplicates of
  a surface filed elsewhere (A04 parser.rs:1238 = A03; refuted law.rs:590
  x2; refuted law.rs:577 = emit_json.rs:169; refuted parser.rs:1143 =
  lexer.rs:334; refuted lexer.rs:338 no-op-guard facet). Deduped register:
  52 adjudicated filings, duplicates retained as cross-linked rows rather
  than silently merged.

Counts: confirmed 14 (12 major / 2 minor / 0 block; 13 unique defect surfaces
— parser.rs:1238 is independently double-confirmed by lanes A03 and A04),
refuted 38 (30 verifier-death, 8 substantively refuted).

## Confirmed Findings

Ranked block -> major -> minor. No block-severity finding survived
verification. `W2-Fn` = wave-2 cross-reference.

Full paths appear in the per-finding sections below; in the table,
unprefixed file names are under vendor/purrdf-n3/ and "vendor README" is
vendor/purrdf-n3/README.md.

| # | Sev | Class | Location | One-line claim |
|---|-----|-------|----------|----------------|
| C1 | major | behavior | reasoner.rs:5691 | blank in a builtin premise vacates its constraint |
| C2 | major | behavior | reasoner.rs:5476 | quotient mints a wrong xsd:integer on big ints |
| C3 | major | behavior | reasoner.rs:5613 | INF/NaN minted as invalid xsd:decimal literals |
| C4 | major | parity+gap | parser.rs:1238 | bare words fabricate example.org IRIs, silently |
| C5 | major | parity | src/law.rs:590 | three limit identities misread as n3_iterations |
| C6 | major | drift | emit_json.rs:169 | registry pins a cap no code path enforces (250x) |
| C7 | major | behavior | parser.rs:993 | path verbs drop their chain triples silently |
| C8 | major | behavior | lexer.rs:334 | @-words -> Lang with no N3 langtag validation |
| C9 | major | drift | vendor README:35 | scope-escape refusal promised, none exists |
| C10 | major | gap | docs/DIALECTS.md:285 | quantifier surface claimed, never executed |
| C11 | minor | behavior | lexer.rs:316 | eager @prefix split mis-lexes @prefixfr words |
| C12 | minor | parity | reasoner.rs:3431 | explain/checker trust boundaries diverge |
| C13 | major | (unresolved) | payload-truncated | 14th confirmed entry arrived truncated |

### C1 — blank vacates the arithmetic constraint (W2-F12, stands)

`eval_math_sum` (reasoner.rs:5691), `eval_numeric_list` (5540),
`eval_math_difference` (5450) and `eval_exponentiation` (5657) all return
`vec![bindings.clone()]` on a `Term::Blank` right-hand side — the builtin
succeeds WITHOUT binding. A blank shared between a fact premise and a builtin
premise is the same N3 existential, so cwm semantics require the builtin to
bind it; here the fact premise is free to bind any value. Skeptic controls
ruled out ground mismatches and ordinary existential witnessing; the vacate
is specific to a builtin-RHS blank shared with another premise. Mechanism is
structural: `resolve` (7151-7188) never consults `blank_binding_name` and
`term_has_var(Term::Blank) = false` (255-261).

Skeptic re-derivation (compressed; execution receipt, exit=0):

```text
:w :bad :fire .
:w :good :y2 .
rule { :x2 :p _:z . (1 2) math:sum _:z } => { :w :bad :fire }
fires against fact :x2 :p 5 although 1+2=3 — unsound;
the variable version fires only for the object-3 fact.
```

### C2 — math:quotient wrong on big integers (W2-F11, stands)

`MATH_QUOTIENT` (reasoner.rs:5474-5477) has no exact BigInt path — unlike
`MATH_PRODUCT`, `MATH_INTEGER_QUOTIENT`, `MATH_REMAINDER`, which route
through `all_exact()`/`integer_literal(BigInt)`. It divides in f64 and
`numeric_literal` (6514-6520) promotes any zero-fract f64 to xsd:integer via
`format!("{:.0}")`. The engine contradicts itself on identical operands.
Skeptic re-derivation (execution receipt, exit=0):

```text
999999999999999999999999 / 3:
:w :quotient        333333333333333327740928 .
:w :integerQuotient 333333333333333333333333 .
:w :remainder       0 .
```

### C3 — INF/NaN minted as xsd:decimal (prior wave: unadjudicated, now confirmed)

The inverse gate at 5613 `if !computed.is_finite() && !computed.is_infinite()
{ return Vec::new(); }` rejects only NaN and passes +/-infinity by
construction; the forward arm (5620-5623) has no finiteness check at all;
`trim_float` (7054-7056) renders "INF"/"-INF"/"NaN"; `numeric_literal`
(6524-6528) types anything without a lowercase 'e' as xsd:decimal — a
lexical form XSD 1.1 forbids for decimal (INF/NaN are float/double-only).
Six reachable paths witnessed via n3_derive, all exit 0: cosh overflow,
tanh inverse at range boundary, exponentiation overflow, negative-base
fractional exponent, base-1 inverse (ln(b)=0 division), f64 sum overflow.

```text
:w :negBase "NaN"^^xsd:decimal .
:w :huge    "INF"^^xsd:decimal .
:w :sum     "INF"^^xsd:decimal .
(exit=0, no typed refusal)
```

### C4 — bare words fabricate example.org IRIs (W2-F5, stands; double-confirmed)

Lexer arms 337-338 map any word to `PName` (the `contains(':')` guard is
inert). `parse_pname_or_path`'s colon check is rdf12-gated (1193-1196), so on
the N3/Turtle profile a colon-less word reaches `expand_pname`, which falls
through base-iri (1232-1234) and default-prefix (1235-1237) to
`Ok(format!("http://example.org/{}", pname))` (1238) — the only occurrence of
that fallback string in the crate. The colon-ful sibling `ex:nope` is loudly
refused as "unknown prefix" (1227) and `tests/parse_refusal_court.rs:42-53`
pins only that refusal. Case variants: 'A' -> <http://example.org/A>, 'True'
-> xsd:boolean true via `to_ascii_lowercase`, subject-position 'is' ->
<http://example.org/is>. Same token is a syntax error under the RDF12
profile (1196) — internal proof it is not a grammar production. Lanes A03
and A04 re-derived this independently from different facets (parity and
silent-recovery-gap); both survive.

Skeptic re-derivation (A04, compressed):

```text
parser.rs 1196: if self.profile.is_rdf12() { ... if !pname.contains(':')
    { return Err(... "expected IRI or prefixed name, got {}" ...) } }
parser.rs 1227: return Err(... "unknown prefix '{}:'" ...);
parser.rs 1238: Ok(format!("http://example.org/{}", pname))
input 'nope nope nope .' parses Ok into triples of example.org IRIs.
```

### C5 — limit identities misreported (W2-F8, stands)

The `limits_reached` match at src/law.rs:573-595 covers exactly four named
arms (MatchSteps/TermSize/ClosureSize/ClosureFacts) with a wildcard
(590-594) returning `("n3_iterations", statistics.iterations,
N3_MAX_ITERATIONS)`. `ReasonerLimit` is `#[non_exhaustive]` with 8 variants;
`BackwardDepth` (cap 32), `BackwardSolutionsPerGoal` (cap 1024) and
`TermDepth` (cap 64, raised from the derived-fact admission path) all
surface as n3_iterations with observed = the forward-fixpoint counter and
max = 4000 — a self-inconsistent Limit error. Backward solving is on the
main fixpoint path ('<=' rules parse as backward rules at parser.rs:806;
`solve_backward_goal` is called from premise matching with no toggle), so
the wildcard is reachable through the ABI `n3` op and the law-engine N3
step. `docs/refusals.md:49` and `src/abi_manifest.rs:171-175` define no row
for the three identities, so consumers cannot type these refusals — against
the repo's stable-coding typed-refusal contract.

Skeptic re-derivation (compressed):

```text
law.rs 590-594:  _ => ("n3_iterations", result.statistics.iterations,
                     N3_MAX_ITERATIONS),
reasoner.rs 919-921: BackwardDepth, BackwardSolutionsPerGoal, (no arm)
```

### C6 — registry pins an unenforced cap (prior wave: unadjudicated, now confirmed)

`src/registry/emit_json.rs:169` pins `"n3_max_match_steps"` to
`crate::law::N3_MAX_TOTAL_STEPS` (50,000,000; `src/law.rs:466`), and
`tests/registry_limits.rs:22` re-pins the same value. But
`ReasonerLimit::MatchSteps` is raised by two different caps: the per-search
budget (`DEFAULT_MAX_MATCH_STEPS = 200_000`, reasoner.rs:43, wired as
`SearchBudget.max_steps`; law.rs never overrides it — grep of law.rs
`max_match_steps` returns no hits) and the run-total cap (50M). The
registered 50M per-search value is enforced by no code path; when the real
200k cap fires, the refusal reports `n3_match_steps` with max=50,000,000
(law.rs:574-578) — 250x the actual bound. `src/abi_manifest.rs:175`
advertises the same constant under a third name (`n3_max_total_steps`).
docs/refusals.md:49 and docs/capability-registry.md:74 propagate the claim.

### C7 — parse_verb drops generated triples (prior wave surface, now confirmed)

`parse_verb` (parser.rs:993-997) binds `(term, _generated)` and returns only
the term; the adjacent comment justifies dropping only "list-support"
triples, but the drop is unconditional for every N3 verb. The lexer does not
break words on '!'/'^', so `:a!:b` lexes as one PName and
`parse_path_pname` (via `split_unescaped_path`, 1701) returns
`(_:pathN, [:a :b _:pathN])`. `:s :a!:b :o .` therefore asserts
`:s _:pathN :o` with the defining chain discarded — a dangling fresh
existential as predicate, Ok, no error. Subject position keeps generated
(782), object position keeps it (843/862/880), and the rdf12 verb arm
errors on generated (975-977) — the N3 arm is the lone silent drop. No
in-repo consumer exercises verb-position paths (hence major, not block).

### C8 — @-word catch-all, no N3 langtag validation (W2-F4, stands)

The read_word catch-all (lexer.rs:334) turns any '@'-word that is not one of
five exact-case directives into `Lang(everything after '@')`: '@PREFIX'
(five arms compare exact case; the eager `starts_with("@prefix")` split is
case-sensitive), '@keywords' (the TokenKind enum has no Keywords variant at
all — the corpus fixture `turtle-syntax-bad-n3-extras-13.ttl` knows the
directive; this lexer does not), and '@en-!!!'. The parser's RDF12 branch
validates and lowercases (parser.rs:1132-1143 via
`validate_lang_or_lang_dir`); the N3 branch is bare
`lit.language = Some(lang.into())` — zero validation, no case folding.
`Literal` derives field-wise Eq/Hash over the verbatim `Name` (ast.rs:
193-197), so `"v" @EN` and `"v" @en` are distinct literals on the N3 surface
while identical after the RDF12 route's fold. This is graphlaw's actual
Tier::N3Logic admission surface (tier.rs:146-153 -> n3::parse_n3 ->
ParserProfile::n3(), is_rdf12()==false). Wave-3 confirms wave-2 F4's own
skeptic re-derivation; the open fix item is scheduled as v26.10.26 DoD F8.

### C9 — README safety guarantee contradicted by disk (W2-F6, stands)

`vendor/purrdf-n3/README.md:34-35`: "`@forAll`/`@forSome` bind within their
enclosing formula scope; a universal that escapes its scope is a parse-time
refusal, not a silent capture." No such refusal exists: `TokenKind::Var` is
inserted into the document-root scope unconditionally (parser.rs:1024-1026,
no depth check, no Err); `parse_quantifier` (1378-1407) errs only on
ordering/duplicate/name conditions; grep for scope-escape semantics outside
IRI/string/JSON escaping returns zero hits; `parse_formula` (744-775) pushes
and pops per-formula scopes while '?var' writes the ROOT scope (hoisted
silently) and `quantified_iri` (1342-1351) walks scopes innermost-first with
no boundary, falling back to `Term::iri`. The reasoner ADMITS universal
facts as axioms (reasoner.rs:1130-1131). No test pins the claimed refusal
(the vendored crate has no tests/ directory). The fix is doc correction or
adding the refusal; either way the register classifies this as open drift.

### C10 — quantifier surface claimed but never executed (W2-F7, stands)

docs/DIALECTS.md:285 claims N3 support "yes (`=>`, `@forAll`, `@forSome`)".
grep of tests/ and src/ excluding W3C fixtures returns only the sniff
substrings (`src/dialect/sniff.rs:310-311`); zero occurrences of "@for" or
"quantifier" in any tests/*.rs; vendored parser.rs has 0 cfg(test) blocks
and no vendored tests/ dir; all four quantifier refusal arms (parser.rs
1380/1386/1397/1404) are reachable from no committed input; the only
fixtures containing the tokens are negative W3C Turtle/TriG tests asserting
quantifiers are INVALID there; all six committed .n3 files use zero
quantifier tokens. Receipts/2026-10-24-dialect-n3.md discloses "no dedicated
court" only as a bound on its own greens; v26.10.26 DoD carries an UNCHECKED
"Quantifier court" item — the repo itself tracks this as open. Corroborates
sibling gaps (W2 F1/F9).

### C11 — eager @prefix split (W2-F3, severity corrected major -> minor)

lexer.rs:316-321: any word starting with "@prefix" and longer than it is
split into `AtPrefix` + `PName(suffix)`. "@prefixfr: <http://x/> ." (not
valid N3) tokenizes exactly into a complete prefix declaration and
`parse_prefix` consumes it — invalid input silently admitted. Langtag case:
"lit"@prefixfr gets `AtPrefix` instead of `Lang`, loses its language, and
errors at a shifted offset, while '@prefixfr' without the split would hit
the Lang arm and pass the parser's own validator. '@PREFIX' escapes the
split (case-sensitive) into the Lang catch-all (C8). Not
deliberate-and-receipted: no comment, no test in the vendored crate or any
repo corpus (grep for '@prefix[A-Za-z0-9]' over vendor/, tests/,
ontologies/ = zero files). Corrected to minor: triggering requires a
langtag beginning with the literal string "@prefix" (contrived; zero corpus
occurrences) and the accept-side produces a coherent declaration, not data
corruption.

### C12 — explain/checker trust-boundary mismatch (W2-F10, severity corrected major -> minor)

`explain_backward_inner` emits a held `BackwardStep::Builtin` on
`verified || builtin_reads_outside_its_triple(&goal.p)` (reasoner.rs:3431)
— i.e. emits AS HELD even when its own re-evaluation just failed — for
LOG_INCLUDES/NOT_INCLUDES/COLLECT_ALL_IN/FOR_ALL_IN (no quoted-vs-open
scope distinction) plus LOG_CONJUNCTION. The checker trusts only the 6
`IMPURE_BUILTINS` plus open-scope current-graph tests (proof_check_n3.rs:
26-39, 54-67) and re-decides the quoted-scope/conjunction class over
deliberately empty evidence (`Evidence::new(Vec::new()).evaluate`, line
642), rejecting with "re-evaluating {name} did not hold" (649). The two
files' contract comments contradict each other (reasoner.rs:3417-3420 vs
proof_check_n3.rs:22-25/47-53). Corrected to minor: the proof-check side
allocates a fresh budget per call (3365), so the budget-exhaustion window
is narrow; store-dependent argument shapes are the surviving window; and
`proof_to_n3`/`check_proof` have zero callers in graphlaw src/ — the proof
pipeline is latent in this tree.

### C13 — fourteenth confirmed entry (identity truncated)

The synthesis payload's 14th confirmed entry arrives truncated
mid-re-derivation ("Independently re-derivation in /Users/sac/graphlaw
(read-only; reused prebuilt n3_derive ... wrote probes only under
/tmp/skeptic_p12/). STATIC (my own reads of /Users/sac/graphlaw/vendor/
purrdf-"), with lane/file/line/claim/evidence fields never delivered. Its
corrected severity is major and its skeptic re-ran witnesses with exit=0,
so it counts toward the confirmed totals, but its locus is NOT
reconstructible from this payload and it is excluded from the ranked
consequence order until recovered from the verify-lane transcript. Given
the /tmp/skeptic_p12/ probe family and the wave-2 queue, the probable
subject is the math:equalTo decimals/NaN surface (v26.10.26 DoD F5) —
recorded as a hypothesis, not a finding.

## Refuted Findings

38 filings refuted under default-refute. "verifier died" = the skeptic
could not complete re-derivation; the filing is unadjudicated, not
disproven. 4 filings are verifier-death duplicates of surfaces that stand
confirmed via a sibling lane (marked [dup]) — the surface stands, only that
filing's verification died.

- vendor/purrdf-n3/src/ast.rs:225 — Debug impl drops g.existentials for
  Formula when universals empty — why: verifier died.
- vendor/purrdf-n3/src/lexer.rs:196 — lexer comment cites 'notation3tests'
  corpus dropped at vendoring — why: verifier died.
- vendor/purrdf-n3/src/lexer.rs:273 — read_number accepts "1e" with zero
  exponent digits — why: verifier died.
- vendor/purrdf-n3/src/lexer.rs:338 — both match arms produce identical
  PName; contains(':') guard is a no-op — why: verifier died; [dup] facet of
  confirmed C4.
- vendor/purrdf-n3/src/lexer.rs:298 — whitespace-separated path operators
  refused not parsed — why: verifier died.
- src/lib.rs:247 — BACKEND_AUTHORITIES pins purrdf::n3 3.0.1 + glob
  re-export — why: verifier died.
- vendor/purrdf-n3/Cargo.toml:67 — declared test/bench targets missing —
  why: facts mostly reproduce but classification fails on all three refute
  grounds (deliberate-and-receipted; claim misreads two artifacts).
- vendor/purrdf-n3/src/reasoner.rs:4711 — log:content/semantics as
  fixture-keyed fakes in production dispatch — why: facts reproduce
  (hardcoded "Hello, world!" for /HELLO suffixes) but behavior is
  fail-closed outside the fixture suffixes; classification fails.
- vendor/purrdf-n3/Cargo.toml:59 — zero tests anywhere — why: verifier died.
- src/lib.rs:248 — pin to a crate that does not exist on crates.io — why:
  evidence reproduces but framing refuted: pin is deliberate-and-receipted
  and the "recorded nowhere" claim is false.
- vendor/purrdf-n3/src/wasm.rs:16 — entire wasm-bindgen surface dead — why:
  verifier died.
- src/law.rs:590 — limits funnel wildcard — why: verifier died; [dup] of
  confirmed C5.
- src/registry/definitions.rs:522 — hashed surface omits the n3 Fuse refusal
  — why: verifier died.
- tests/purrdf_capabilities.rs:19 — court claims "pinned purrdf 2.0.2" in 5
  places — why: verifier died.
- src/lib.rs:177 — purrdf::retrieval re-export never called — why: verifier
  died.
- tests/chicago_n3_datalog.rs:1 — builtin execution coverage 7 of ~90 — why:
  verifier died.
- src/law.rs:590 — 3 of 8 ReasonerLimit variants lack an arm — why: verifier
  died; [dup] of confirmed C5.
- src/law.rs:577 — n3_match_steps arm attributes run-wide 50M budget — why:
  verifier died; [dup] of confirmed C6.
- vendor/purrdf-n3/src/lib.rs:35 — ported wasm surface dead — why: facts
  reproduce; defect interpretation fails (claim misreads the vendored
  surface on two explicit refute grounds).
- /Users/sac/graphlaw/Cargo.toml:17 — proof half and vendored tests execute
  under zero rungs — why: verifier died.
- tests/registry_artifacts.rs:640 — no default gate verifies the
  ARTIFACTS.blake3 wasm row — why: line citations reproduce but two
  load-bearing universal claims fail independent re-derivation.
- tests/corpus_conformance.rs:78 — .nt asset routed against the dialect law
  — why: verifier died.
- courts/castle-diff/Cargo.lock:1511 — "zero graphlaw-eyeron packages"
  invariant false — why: verifier died.
- vendor/purrdf/Cargo.lock:267 — committed lock is a pre-N3 fossil — why:
  verifier died.
- vendor/purrdf/Cargo.toml:172 — normalized-manifest invariant — why:
  truncated in payload; visible gist: evidence reproduces (header is the
  standard generated-file header) and the finding was still refuted.
- docs/abi-reference.md:166 — wire-step table lists 9 steps, code maps 10 —
  why: verifier died.
- docs/api-stability.md:43 — MSRV stated 1.96, manifest differs — why:
  verifier died.
- src/bin/n3_derive.rs:40 — pinned authority's proof surface under zero
  rungs — why: verifier died.
- tests/registry_artifacts.rs:639 — wasm digest court env-gated — why:
  evidence reproduces (grep GRAPHLAW_REQUIRE_WASM_PIN = 0; blake3-vs-row
  check is env-gated, not absent); remaining why truncated in payload.
- target/wasm32-wasip1/wasm/graphlaw_wasm.wasm:305 — staged wasm lacks the
  typed N3 Fuse arm — why: verifier died.
- docs/PHD_THESIS_GRAPHLAW_WASM_SQLRDF.md:1371 — determinism claim vs
  runtime timestamps — why: verifier died.
- vendor/purrdf-n3/src/printing.rs:481 — sanitize_blank maps distinct legal
  labels to the same name — why: verifier died.
- vendor/purrdf-n3/src/printing.rs:179 — top-level log:implies fact
  rendered as a rule — why: verifier died.
- vendor/purrdf-n3/src/printing.rs:53 — no executed parse->print->parse
  identity court — why: verifier died.
- vendor/purrdf-n3/src/parser.rs:1143 — N3 profile zero language-tag
  handling — why: verifier died; [dup] of confirmed C8.
- tests/parse_refusal_court.rs:42 — byte-offset diagnostic surface
  untested — why: verifier died.
- src/bin/dl_dualroute.rs:492 — ground truth for 2 of 17 emitted goals —
  why: miscounts (15 goals, not 17; more cases cross-checked than claimed).
- wasm4pm fixture situation_calculus-frame-heavy.json:9 — 44 of 48 fixtures
  carry stale generator provenance — why: verifier died.

## Court Confirmation Runs

All six courts re-run at the audited head; all exit 0, 40 cases green, no
disruption reproduced. The load-bearing observation: none of them can fire
on a confirmed defect surface, so their green status neither clears nor
disrupts the findings above.

| Court | Exit | Cases | Relevance to findings |
|---|---|---|---|
| parse_refusal_court | 0 | 12 | pins only colon-ful unknown-prefix; bare-word arm (C4) untested |
| registry_limits | 0 | 2 | witness for C6: asserts the 50M mispin at registry_limits.rs:22 |
| capabilities_digest | 0 | 6 | digests match live; faithfully digest the C5/C6 mispins |
| authority_manifest | 0 | 6 | authority/revision pins hold; no N3 limit-name coverage (C5) |
| backend_authorities | 0 | 8 | purrdf-n3 owns N3 execution; vendored defects are live surface |
| chicago_n3_datalog | 0 | 6 | happy-path only; C1/C2/C10/C12 surfaces never executed |

## Completeness-Critic Open Questions

Surfaces no lane covered (A) and sibling-defect hypotheses raised by the
confirmed classes (B). These bound the verdict's coverage claim.

A1. Mutation-court scope excludes the audited subject: all five
    .cargo/mutants*.toml configs examine src/registry.rs, src/abi.rs,
    src/registry/* only — no config mutates src/law.rs, src/dialect.rs, or
    vendor/purrdf-n3. The anti-vacuity story for the N3 pipeline is empty by
    construction.
A2. Quantifier execution: no lane drove a quantifier document end-to-end
    through reason_n3_bounded derive->prove->print (C10 is a gap finding,
    not an execution).
A3. The packet-4 stdio runner (src/bin/graphlaw_run_eh.rs): frame
    truncation/zero-length/oversized-prefix handling, and a packet-4 vs
    n3_derive differential on the same N3 document — unowned.
A4. Receipt-chain flow: whether receipt_chain_sync.sh verifies the per-file
    blake3 digests it claims, and whether src/receipt_chain.rs is wired into
    the N3 flow at all (CLAUDE.md flags chaining as not-yet-serialized).
A5. Blackbox Python courts (tests_py/test_blackbox_ops.py,
    test_ubo_sanctions.py): exit-code consistency with the n3_derive
    contract; whether UBO drives N3 admission through ABI ops.
A6. Bench and columnar/parquet family (9 bench targets): cargo test never
    builds benches — do they compile under the current feature set, and does
    columnar lowering preserve term identity (C2's arithmetic class)?
A7. Standalone pin/contract echoes (cicd.toml, rust-toolchain.toml,
    contracts/abi-manifest.json, qualification/, conformance/): does
    contracts/abi-manifest.json agree with src/abi_manifest.rs?
A8. No fuzz harness exists; the lexer is ~900 lines of adversarial input
    handling with point-sample probes only.
A9. Hygiene receipt gap: ~40 target-lane-* / target-battery-* build roots
    present at audit time — unadjudicated under the cleanup law.

B1. Offset arithmetic (C2-adjacent class): same (offset, len) addressing in
    src/abi.rs buffer ops and src/egress/arrow.rs — unprobed siblings.
B2. Limit-identity misreporting echoes (C5/C6): js/ TS limit fields, the
    doc<->TS<->ABI echo chain, and wasm ABI limit opcodes — unverified.
B3. Lenient-acceptance x roundtrip (C4 + printing): does printing.rs re-emit
    fabricated example.org IRIs, making fabrication durable through
    serialization?
B4. Panic-as-refusal bypass: remaining unwrap/expect/index paths in the N3
    route across the wasm boundary (trap instead of typed refusal);
    release-mode overflow wrap in limit arithmetic as a silent limit bypass.
B5. Proof writer/checker under budget (C12's sibling): proof_writer.rs
    output produced under a budget then re-verified under the same budget —
    truncated-proof-accepted is the failure mode.
B6. Unicode -> printing: roundtrip the confirmed unicode input classes
    through the printing surface; lexer-side findings do not imply printing
    symmetry.
B7. Blank scope across consumers: N3-derived graphs entering
    purrdf-sparql-eval (blank_scope.rs) — blank-label collision between the
    two consumers.
B8. Fixture/vector regen drift: regenerate committed conformance vectors and
    diff (generated-artifacts-never-patched law).
B9. Publish-vs-vendored drift: release.yml publishes crates while
    [patch.crates-io] pins the vendored path — diff published 3.0.1 against
    vendor/purrdf-n3.

## Verdict

**Conditional — no, the full N3 purrdf pipeline is not sound as a whole; it
is sound only on the court-pinned happy-path subset.** What is sound: the
well-formed corpus the courts actually execute — ground triples, forward
derivation on courted inputs, typed refusals for well-formed error classes
(unknown prefix), and the registry/authority digest machinery (six courts,
40 cases green at this head). What is not: three witnessed
soundness/correctness breaks in the arithmetic core (C1 a blank shared
between fact and builtin premise vacates the constraint and derives 5=1+2
at exit 0; C2 quotient mints a self-contradicting wrong xsd:integer; C3 six
paths mint datatype-invalid "INF"/"NaN"^^xsd:decimal), a parse route that
silently admits out-of-grammar input and fabricates IRIs in a namespace the
pipeline does not own (C4, double-confirmed; C7 silently drops path-verb
chains; C8 admits unvalidated, case-sensitive langtags making literal
identity case-dependent), and a typed-refusal contract that misreports
three reachable limit identities with fabricated observed/max while the
registry pins a per-search cap 250x larger than the one that fires (C5, C6)
— with the quantifier surface claimed-but-never-executed behind a README
safety guarantee the parser does not implement (C9, C10), and zero
mutation-court coverage of any of it (critic A1). Until the C1/C2/C3
arithmetic classes are fixed or fenced, the C4/C7/C8 lenient-acceptance
classes are refused instead of fabricated, and the C5/C6 refusal
identities are registered and told apart, treat full-N3 admission as
conditional on the court-pinned subset and route any document outside that
subset through an explicit human or court gate.
