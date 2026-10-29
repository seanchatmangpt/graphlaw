# TAG-PREP

Status: PREPARED (2026-10-24). Lane: docs/jira/v26.10.23/TAG-PREP.md only.
Scope: prepare the `v26.10.22` annotated tag — no tag commands run, no cargo.
Every claim below was re-read from disk or a live registry query on 2026-10-24;
receipts embed the real output.

> **Pin update (2026-10-24, closeout)**: the artifact pin moved after this
> document was written. The digest cited below as current (`472d5557…bf80ea`,
> 58,121,500 B) is the superseded settle pin; the shipped artifact is
> `58a44095…d2280a` (58,124,612 B), verified one-digest across all four pin
> surfaces (`docs/jira/v26.10.23/final-tree-audit.md` check 2). Sections 3/4/5
> read as-of their writing; the live pin governs.

## 1. Version triple — verified on disk 2026-10-24

```
$ grep -n '^version' Cargo.toml wasm/Cargo.toml crates/graphlaw-eyeron/Cargo.toml
Cargo.toml:3:version = "26.10.22"
wasm/Cargo.toml:8:version = "26.10.22"
crates/graphlaw-eyeron/Cargo.toml:3:version = "0.7.9"
$ grep -n '"version"' js/npm/package.json
3:  "version": "26.10.22",
$ grep -n '## \[26.10.22\]' CHANGELOG.md
10:## [26.10.22] - 2026-10-22
```

Triple agrees: workspace `26.10.22` (`Cargo.toml:3`), wasm unit `26.10.22`
(`wasm/Cargo.toml:8`), npm `26.10.22` (`js/npm/package.json:3`, name
`@seanchatmangpt/graphlaw`). CHANGELOG carries the matching dated entry
(`CHANGELOG.md:10`).

## 2. eyeron 0.7.9 is the version crates.io will accept (0.7.8 collision)

- Local manifest: `crates/graphlaw-eyeron/Cargo.toml:3` = `0.7.9`.
- Root dependency pins the same: `Cargo.toml:68`
  `eyeron = { package = "graphlaw-eyeron", version = "0.7.9", path = "crates/graphlaw-eyeron" }`.
- Documented preflight: `docs/jira/v26.10.23/compliance-gap-closure-plan.md:233`
  — "preflight: `graphlaw-eyeron 0.7.8` is already on crates.io — bump to 0.7.9".
- Live crates.io receipt (2026-10-24, API query with UA header):

```
$ curl -s -A "graphlaw-tag-prep (sean@local)" https://crates.io/api/v1/crates/graphlaw-eyeron
max_version: 0.7.8
versions: ['0.7.8', '0.7.7']
```

`0.7.9` is unpublished on crates.io → publish will be accepted. (The 0.7.8
collision is real and already dodged by the on-disk bump.)

## 3. Registry state this tag publishes into (live, 2026-10-24)

```
crates.io graphlaw          max_version: 26.10.21   → 26.10.22 unpublished
crates.io graphlaw-eyeron   max_version: 0.7.8      → 0.7.9  unpublished
npm @seanchatmangpt/graphlaw dist-tags.latest: 26.10.14 → 26.10.22 unpublished
```

Final artifact identity (verified live this lane):

```
$ b3sum priv/graphlaw.wasm
472d5557ba8311c0fab68bed044be9c44254e65fd523a0ff00684c0869bf80ea  priv/graphlaw.wasm # historical (superseded by final pin 58a44095…, GO-NO-GO §A G1)
$ cat priv/graphlaw.wasm.blake3
472d5557ba8311c0fab68bed044be9c44254e65fd523a0ff00684c0869bf80ea # historical (superseded by final pin 58a44095…, GO-NO-GO §A G1)
```

Pin file byte-identical, size 58,121,500 B, pinned at `d9cab2f`
(`receipts/2026-10-23-v26.10.22-terminal.md` §1); npm generated embed matches
(`js/npm/src/generated/artifact-meta.ts` contains the same digest, verified by
grep). Terminal receipt exists on disk:
`receipts/2026-10-23-v26.10.22-terminal.md`; EH-closure wave receipt:
`receipts/2026-10-23-eh-closure-wave.md`; banking witness:
`receipts/2026-10-23-banking-4of4-witness.md`.

## 4. Proposed annotated-tag message (v26.10.22)

Apply with `git tag -a v26.10.22 -F <msgfile>` (operator; this lane ran no git
tag command). Note: `release.yml` auto-tags on the next push to main if the tag
does not exist (`--generate-notes`); the message below is for the explicit
annotated form.

```
graphlaw v26.10.22

One executable law on host and wasm. Sovereign in-wasm DuckDB: frozen-duckdb
2.0 linked unconditionally into graphlaw-wasm (wasm32-wasip1), SQL-in-RDF
admits and evaluates inside the module; compile_turtle is the sole admission
path, refusals are typed (REFUSAL-01..05, docs/refusals.md).

EH-closure wave landed: the legacy-EH artifact is superseded by the
new-EH (wasm-exceptions) build; lifecycle court 3/0 DURABLE on the final
artifact (SQLRDF-27 amendment); SQLRDF-24 trap root-caused to the
atexit writer/reader skew with a full exoneration matrix (EH flavor, LTO,
table encoding, gc-sections, compat archive).

SQLRDF-21..28: sovereign in-wasm DuckDB execution, EH wall closed (21);
FIBO-matrix compiler gaps — DECIMAL, COPY egress policy, multi-row INSERT
(22); statement sequences G5 (23); artifact trap root-cause (24); dialect
expansion — window functions, QUALIFY, CTEs, PIVOT/UNPIVOT (25, remaining
gaps ledgered as 25-RG); atomic document execution mode (26); tc08
COPY-egress EINVAL, musl-wasi-libc fcntl mismatch, Closed (27);
mutation-court coverage on abi.rs response builders + spec_of (28).

Batteries: banking DvP 4/4 GREEN
(receipts/2026-10-23-banking-4of4-witness.md; BCBS/Rehypo/UBO 3/3 as-of the
prior witness). Compliance battery — EU AI Act (core, logging, robustness,
transparency) + NIST AI RMF (govern/map, measure/manage) matrices with a gap
closure plan (docs/jira/v26.10.23/compliance-*.md).

Hardening: RUST-4 phase-2 concurrency audit recorded on src/evidence_ring.rs
(2 HIGH, 1 LOW, directive: record-then-fix post-wave,
docs/jira/v26.10.22/RUST-4-PHASE2-HARDENING.md); SQLRDF-28 names the abi.rs
mutation survivors; receipt chain 1550 receipts VERIFIED
(head 38422ef5…87129, --check exit 0, terminal receipt §3).

Final artifact: priv/graphlaw.wasm
b3sum 472d5557ba8311c0fab68bed044be9c44254e65fd523a0ff00684c0869bf80ea # historical (superseded by final pin 58a44095…, GO-NO-GO §A G1)
(58,121,500 B), pin file priv/graphlaw.wasm.blake3 byte-identical; npm
js/npm/src/generated/artifact-meta.ts embeds the same digest. Terminal
receipt: receipts/2026-10-23-v26.10.22-terminal.md.

Publish order: crates.io graphlaw-eyeron 0.7.9 first, then graphlaw 26.10.22
(dependency order; CI publish is refused by design — local-only);
npm @seanchatmangpt/graphlaw 26.10.22 follows the tag.
```

Honesty boundaries carried in the wording: SQLRDF-24's INDEX row still reads
Open (root-caused; re-pin held until its closure landed on the final artifact
identity), so the message says "root-caused", not "closed". SQLRDF-25-RG is a
ledger, not a closure. Terminal receipt §4 mutation sweep is PENDING (40/88
read at receipt time) — deliberately absent from the message.

## 5. Post-tag watch list

1. **publish-npm.yml (fires on the tag: `tags: ['v*']`, jobs `validate` →
   `publish`).** Watch `validate` (pack + digest output), then `publish`, whose
   cross-job blake3 equality gate recomputes and compares
   (`.github/workflows/publish-npm.yml:196-203`). Then confirm the registry:
   `@seanchatmangpt/graphlaw` `latest` must move `26.10.14 → 26.10.22`.
2. **crates.io eyeron → graphlaw, local-only publish.** release.yml refuses to
   publish in CI (`.github/workflows/release.yml` "Publish (refused in CI)"
   step): run `cargo publish -p graphlaw-eyeron && cargo publish -p graphlaw`
   from a logged-in machine, in that order (graphlaw 26.10.22 depends on
   eyeron 0.7.9, `Cargo.toml:68`). Verify afterwards on the API: eyeron
   `max_version: 0.7.9`, graphlaw `max_version: 26.10.22`.
3. **GitHub release assets + blake3 sidecars.** release.yml (push to main,
   `release if unreleased`) creates the tag if absent and attaches
   `graphlaw.wasm`, `capability-registry.json`, `capability-registry.ttl`,
   `op-examples.json`, each with a `.blake3` sidecar generated by b3sum on the
   runner. Watch item: those sidecars digest the CI rebuild, which is not
   guaranteed byte-identical to the committed pin — record the release-asset
   digest and compare against `472d5557…bf80ea`; the pin continues to govern
   the committed `priv/graphlaw.wasm`.
4. **RELEASE_HOLD.** The authoritative check is the repo Actions variable
   `vars.RELEASE_HOLD` (release.yml plan step: if `true`, it reports
   "RELEASE_HOLD is true: skipping tag and publish"). A local `RELEASE_HOLD`
   file check is necessary, not sufficient; no such file exists on disk
   (verified), and the variable must be confirmed absent/false by the operator
   before tagging.

## 6. Replay

```
grep -n '^version' Cargo.toml wasm/Cargo.toml crates/graphlaw-eyeron/Cargo.toml
grep -n '"version"' js/npm/package.json
b3sum priv/graphlaw.wasm && cat priv/graphlaw.wasm.blake3
curl -s -A "ua" https://crates.io/api/v1/crates/graphlaw-eyeron   # max_version 0.7.8 pre-publish
ls receipts/2026-10-23-v26.10.22-terminal.md receipts/2026-10-23-eh-closure-wave.md
```

Falsifier: any command above disagreeing with the quoted output (version drift,
pin mismatch, crates.io accepting 0.7.8 as next) invalidates this prep and the
tag message section 4 must be re-derived before tagging.

## 7. Not done by this lane

No `git tag` (explicitly out of scope), no cargo commands, no registry
mutations. The tag, the publishes, and the release are operator/coordinator
transitions.
