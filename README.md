# GraphLaw

GraphLaw is a **library-backed semantic law-state boundary** for Rust.

## v26.9.28 semantic reset

GraphLaw no longer ships a second implementation of RDF/N3/Datalog/SPARQL/SHACL/ShEx/OWL semantics. The RoXi/Praxis-derived parser, triplestore, query engine, reasoner, built-ins, Datalog fixpoint, SHACL/ShEx validators, OWL-RL rules, DRed/RSP engine code, local WASM wrapper, and local POWL helper were removed from the production crate.

Executable standards semantics are delegated to two pinned authorities:

| Capability | Authority |
| --- | --- |
| RDF 1.2, codecs, event model | PurRDF 2.0.2 |
| SPARQL 1.1/1.2 | PurRDF 2.0.2 |
| SHACL | PurRDF 2.0.2 |
| ShEx 2.1 | PurRDF 2.0.2 |
| Datalog / stratification / semi-naive fixpoint / chase | PurRDF 2.0.2 |
| RDF / RDFS / OWL-RL entailment | PurRDF 2.0.2 |
| Notation3 parsing/reasoning/proofs | Eyeron 0.7.7 (MIT, vendored as `crates/graphlaw-eyeron` from eyereasoner/eyeron@d6568f65; see its `UPSTREAM.md`) |

GraphLaw itself owns the **composition boundary and semantic assets**: ontologies, packs, queries, and the decision about which upstream implementation has standing. It does not keep a fallback implementation.

## Law-state layer

GraphLaw's own code is the composition layer above the engines:

- `dialect` routes a document to its owning engine by content (never by trusting an extension, never by trying engines in turn) and returns a typed `Refusal` naming the engine and dialect when it cannot.
- `law::LawState` is an immutable dataset identified by `sha256:` of its RDFC-1.0 canonical form. `transition(&Step)` runs an upstream-owned step (SHACL admission, N3 derivation, RDFS/OWL-RL entailment) and returns the child state with a `Receipt` naming the authority and revision. A refused step yields no state.
- `tests/differential.rs` is a cross-engine oracle: N3 (Eyeron), Datalog and OWL-RL (PurRDF) must agree with each other and with an independent Warshall closure.
- `ASSETS.sha256` pins every shipped asset by hash and sniffed dialect; `tests/corpus_conformance.rs` routes and parses all of them (`cargo test -- --ignored` adds the large vendored vocabularies).

## WebAssembly module (for Elixir/Wasmex and other WASI hosts)

```sh
cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm   # -> target/wasm32-wasip1/wasm/graphlaw_wasm.wasm
```

One self-contained WASI module (imports are `wasi_snapshot_preview1` only: clock, random, stdio; no JavaScript). Exports `gl_alloc`, `gl_free`, `gl_call` and memory; call `_initialize` once if the host does not. A request is UTF-8 JSON written into `gl_alloc`ed memory; `gl_call(ptr, len)` returns `(out_ptr << 32) | out_len` for a UTF-8 JSON response, which the host frees with `gl_free`. Ops: `capabilities`, `sniff`, `parse`, `convert`, `canonical`, `sparql`, `shacl`, `shex`, `n3`, `entail`, `datalog`, `hooks`, `law` (see `src/abi.rs`). All RDF dialects (Turtle, TriG, N-Triples, N-Quads, RDF/XML, JSON-LD, YAML-LD, TriX, HexTuples), N3, SPARQL, SHACL, ShEx (ShExC/ShExJ), RDF/RDFS/OWL-RL/D entailment, Datalog and knowledge hooks execute inside the module; `tests/wasm_abi.rs` drives all of them in a real wasm runtime. `wasm32-unknown-unknown` is not a supported module target: it needs a JavaScript host.

`vendor/` carries two cfg-only upstream patches that give WASI the standard clock/RNG path; see `vendor/README.md`.

### Releasing (automatic)

There is nothing to run. When a change that bumps the version in `Cargo.toml` reaches `main`, `.github/workflows/release.yml` builds and tests, tags `vX.Y.Z`, creates the GitHub release with `graphlaw.wasm` (plus checksum), and publishes `graphlaw-eyeron` then `graphlaw` to crates.io (each only if that version is not already there; every step is idempotent). The only one-time setup is the `CARGO_REGISTRY_TOKEN` repository secret; without it the release and tag still happen and the publish step fails loudly until it is added. Building `graphlaw` for WASI outside this workspace is a compile error by design (the clock fix lives in `vendor/`); use the release asset.

## Plan admission

A planner only proposes. `plan::Plan::admit(&LawState)` (also `Step::Plan`, and `{"step":"plan","plan":{"actions":[{"name","pre","pre_not"?,"add","del"}],"goal","goal_not"?}}` in the `law` op) replays a candidate plan: every `pre` triple (N-Triples, ground) must be present and every `pre_not` triple (PDDL `(not p)`, closed-world) must be absent in the current state (checked in that order), `del` then `add` produce the child state, and `goal` must hold and `goal_not` must be absent at the end. The first violated precondition, or an unmet goal, refuses the whole plan with `LawError::PlanRefused { index, action, missing, violated_absent }` and yields no state. Each applied action returns a `Receipt` (`step: "plan-action"`, authority `purrdf`), so an admitted plan is a chain of content-addressed states; replay is byte-identical. Blank nodes are refused. See `tests/plan_admission.rs`.

Receipt feedback: `receipt::record(&state, &receipt)` writes a receipt into the state as RDF (`urn:graphlaw:receipt:<child>`), so it changes the state id and the next admission; `Step::RequireReceipt { step }` (ABI `{"step":"record-receipts"}` / `{"step":"require-receipt","step_name":"..."}`) refuses with `LawError::ReceiptRequired` unless that step's receipt is recorded. Plan-action receipts carry `plan_sha256` and `index`; `Admitted.plan_digest` is the same digest. See `tests/receipt_feedback.rs`.

## Knowledge hooks

`hooks::HookPack` loads `kh:Hook` / `kh:Action` resources from any RDF pack (see `packs/self-monitoring-pack/hook.ttl`) and runs them to a fixpoint over a `LawState` (`Step::Hooks`). Triggers and actions are SPARQL executed by PurRDF; each firing is recorded in the state so re-running a saturated state changes nothing.

## Pack tooling (Rust, feature `pack-tools`)

The self-monitoring pack's former Python scripts are Rust binaries that write RDF through PurRDF only:

```sh
cargo run --features pack-tools --bin smon-transcript-to-turtle -- --transcript S.jsonl --out out.ttl
cargo run --features pack-tools --bin smon-broaden-topic -- --in-ttl in.ttl --out-ttl out.ttl
```

`tests/smon_tools.rs` checks the broadening output against the committed fixture the Python script produced.

## Rust surface

```rust
use graphlaw::{n3, rdf, sparql, shacl, shex, datalog, entailment};

let dataset = rdf::parse_dataset(
    b"<https://example.org/s> <https://example.org/p> <https://example.org/o> .",
    "application/n-triples",
    None,
)?;
assert_eq!(dataset.quad_count(), 1);

let derived = n3::reason(r#"
    @prefix : <https://example.org/> .
    :s :kind :Human .
    { ?x :kind :Human . } => { ?x :kind :Mortal . } .
"#)?;
assert!(derived.contains("Mortal"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

The complete upstream surfaces are intentionally re-exported. Consumers can use the authoritative library API directly rather than a lossy GraphLaw copy.

## API stability and supply chain

Growable enums are `#[non_exhaustive]` and every public item is documented (`#![deny(missing_docs)]`);
see [`docs/api-stability.md`](docs/api-stability.md) for the stability, deprecation, MSRV and
`ABI_VERSION` contracts. Build plans with `Plan::builder()` and `Triple` instead of hand-written
N-Triples. CI runs `cargo-semver-checks`, `cargo-deny` (`deny.toml`) and `cargo-audit`.

## Production rule

**No standards algorithm is reimplemented in GraphLaw.** If an admitted upstream library lacks a required standard feature, GraphLaw either refuses that feature or contributes the capability upstream. It does not create another local parser, reasoner, validator, query engine, or fixpoint implementation.

The pre-v26.9.28 engine remains available through Git history only. There is no `legacy` feature and no extraction workflow capable of restoring it onto `main`.

See `MIGRATION.md` for the old-to-new surface map.

## Authority leases

A `Lease { id, holder, ceiling, scope, expires_unix, issued_unix }` grants a holder authority to run
named steps until it expires. The unsigned path `LawState::transition_leased_unverified(&lease, &step, now_unix)`
(unverified: anyone can build a lease and pick the time; see "Signed receipts and leases") refuses
with `LawError::LeaseRefused { reason: Expired | OutOfScope | Ceiling }`. Required ceilings:
`Observe` for gates (`admit:shacl`, `admit:require-receipt`), `Select` for `admit:plan`,
`Construct` for `derive:*`. The receipt carries `lease_id`, and `receipt::record` writes it as
a triple. The JSON ABI `law` op accepts `"lease"` plus `"now_unix"`; receipts then carry
`lease_id`. Unleased behavior is unchanged.

## Out-of-subject receipt store

`receipt_store::ReceiptStore` keeps receipts outside the subject's tree, at
`<dir>/<subject>/<sha256>.json` (canonical JSON, atomic write, no git dependency).
`verify(subject)` re-hashes every file, requires canonical form, and requires one linear
`parent -> child` chain; a tampered or missing file is refused with a typed `StoreError`.

## Refusal details

Every refusal from the `law` and `policy` ops keeps `error.{kind,engine,dialect,message}`
unchanged and adds `error.details`, a machine-readable object keyed by `code`. Clients
should read `details`, never scrape `message`.

| `details.code` | Fields | Source |
|---|---|---|
| `NotAdmitted` | `violations: [{focus, path, component, message, severity}]` (one entry per SHACL result) | `LawError::NotAdmitted { violations, results }` |
| `PlanRefused` | `index`, `action`, `unmet: [N-Quads lines]`, `violated_absent: [N-Quads lines]` (forbidden `pre_not`/`goal_not` triples that were present) | `LawError::PlanRefused` |
| `PolicyRefused` | `policy_kind`, `state`, `action` | `policy::PolicyRefused` |
| `LeaseRefused` | `reason` (`expired`/`out_of_scope`/`ceiling`), `lease_id`, `step` | `LawError::LeaseRefused` |
| `ReceiptRequired` | `step` | `LawError::ReceiptRequired` |
| `Refused` | `kind` (upstream engine or routing refusal inside a step) | `LawError::Refused` |
| `ResourceLimit` | `limit`, `observed`, `max` (see below) | `abi` caps |

In Rust, `NotAdmitted.violations` is the count and `results: Vec<law::Violation>` the report;
`violations == results.len()`.

## Resource limits

Caps are pub consts in `graphlaw::abi` (and `law::N3_MAX_ITERATIONS`), checked before
parsing or heavy work. Over-limit input returns `kind: "ResourceLimit"` with
`details: {code: "ResourceLimit", limit, observed, max}`; it never panics or allocates the
oversized input.

| `limit` | Const | Value | Rationale |
|---|---|---|---|
| `request_bytes` | `MAX_REQUEST_BYTES` | 16 MiB | Largest document a host should ship in one call |
| `json_depth` | `MAX_JSON_DEPTH` | 64 | String-aware bracket scan; serde's own limit is 128 |
| `plan_actions` | `MAX_PLAN_ACTIONS` | 1,000 | Each action is replayed and re-canonicalized |
| `atoms_per_field` | `MAX_ATOMS_PER_FIELD` | 10,000 | Non-empty N-Triples lines in one `pre`/`pre_not`/`add`/`del`/`goal`/`goal_not` |
| `policy_entries` | `MAX_POLICY_ENTRIES` | 100,000 | FOND policy entries |
| `n3_iterations` | `law::N3_MAX_ITERATIONS` | 4,000 | Eyeron `ReasonerOptions::max_iterations` (its default is 1,000,000; a runaway rule set costs superlinear time) |

WebAssembly: `gl_alloc(len)` returns null (0) when `len > MAX_REQUEST_BYTES`. `gl_call` on a
null buffer returns a typed JSON error (no trap), and `gl_call` with `len > MAX_REQUEST_BYTES`
returns a `request_bytes` `ResourceLimit` refusal without reading the buffer.

## Signed receipts and leases

`attest` adds Ed25519 (`ed25519-dalek =2.2.0`, BSD-3-Clause, deterministic signing, no RNG)
attestations. An `Attestation { payload_sha256, key_id, signature }` (hex) signs a canonical
payload (sorted keys, no whitespace). `key_id` is the SHA-256 of the public key. Verification
needs only the payload, the attestation and a caller-provided `TrustedKeys` set: offline.

- Receipts: `attest::sign_receipt(&key, &receipt)` signs `{added, authority, child, lease_id,
  parent, plan_sha256, revision, step, subject_sha256}`. `receipt.with_subject(digest)` binds a
  caller-provided digest of an external subject (a git commit, an artifact) before signing.
  `receipt::record_signed` records the receipt plus attestation triples in a state;
  `Step::RequireSignedReceipt` / `receipt::require_signed(state, step, &trusted)` (ABI step
  `require-signed-receipt` with `trusted_keys`) refuse with `LawError::ReceiptRefused
  { reason: Unattested | BadSignature | UntrustedKey }`. `receipt::require` /
  `Step::RequireReceipt` prove *presence only*: hand-written triples satisfy them.
- Leases: `attest::sign_lease(&issuer_key, lease)` gives a `SignedLease`.
  `LawState::transition_authorized(&signed, &trusted, &clock, max_skew_secs, &step)` refuses
  with `LeaseReason::BadSignature | UntrustedKey | ClockSkew | Expired | OutOfScope | Ceiling`.
- ABI: `law` with a lease requires `"signed_lease"` + `"trusted_keys"` (hex public keys).
  An unsigned `"lease"` is refused (`UnverifiedLeaseRefused`) unless the request sets
  `"unverified_lease": true`, which also needs the caller's `now_unix`.
- Store: `ReceiptStore::put_signed` writes `<digest>.sig` beside `<digest>.json`;
  `verify_attested(subject, &trusted)` requires every receipt to carry a valid attestation.

Key custody, key distribution and revocation are outside the library: a verifier is only as
good as the `TrustedKeys` it is given.

## Trusted clock

Lease expiry is judged by a `law::Clock` (`fn now_unix(&self) -> u64`) that the *verifier*
supplies: `SystemClock` (host wall clock; a clock before the epoch reads `u64::MAX`, so every
lease is expired) or `FixedClock(t)` for tests and audits pinned to an instant. A signed-lease
request has no time field: in the ABI `now_unix` is ignored and the module uses its own clock.
Expiry is strict (`now >= expires_unix` refuses). `max_skew_secs` (ABI default 60) bounds how
far the issuer's `issued_unix` may lead the verifier's clock (`ClockSkew`); it never extends
an expiry.

## Verifying without trusting the operator

```text
graphlaw-verify --start state.nt --plan plan.json --receipts ./receipts \
    [--subject <key>] --trusted-key <hex-public-key> [--trusted-key ...]
```

Build with `--features abi`. The command replays `Plan::admit` from the start state, checks
the receipt store holds exactly the replayed chain (parent/child ids, step, authority, added,
plan digest), and verifies each receipt's attestation against the trusted keys. It prints
`ADMITTED` (exit 0) or `REFUSED: <code>: <detail>` (exit 1; codes `PlanRefused`,
`ReceiptTampered`, `ChainBroken`, `Unattested`, `BadSignature`, `UntrustedKey`,
`MalformedAttestation`, `ReceiptMismatch`). Usage and I/O errors, including no
`--trusted-key`, exit 2. It opens no sockets. `--subject` may be omitted when the store root
holds one subject directory.

### Receipt fields and the portfolio R

Informal mapping only; GraphLaw does not claim conformance to any external receipt schema.

| portfolio R field | GraphLaw field |
|---|---|
| identity | `parent`, `child` (state ids), `subject_sha256` (external subject digest) |
| authority | `authority`, `revision` (upstream engine), `lease_id`, attestation `key_id` |
| consequence | `step`, `added` |
| replay | `plan_sha256`, `graphlaw-verify` replay from the start state |
| standing | not stored: derived by the verifier (ADMITTED/REFUSED) from the receipts |
