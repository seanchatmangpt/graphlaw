# cargo-deny Receipt (RUST-8, lane L63)

- Repo: /Users/sac/graphlaw
- HEAD at receipt: `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`
- Tool: cargo-deny 0.20.2 (`cargo deny --version` → exit 0)
- Date: 2026-10-22T23:58:25Z

## Commands and exits (real output)

| Command | Exit |
|---|---|
| `cargo deny --version` | 0 (`cargo-deny 0.20.2`) |
| `CARGO_TARGET_DIR=target-lane63 cargo deny check` (before) | 5 — config deserialize error: `expected a 'WITH' here` at deny.toml:31 (`"MIT AND LicenseRef-P28-SIL-2.1.0"`) |
| `CARGO_TARGET_DIR=y cargo deny check licenses` (after) | 4 — residual: 2 x workspace LicenseRef rejections |
| `cargo deny check advisories` | 1 — 9 RUSTSEC errors, see below |
| `cargo deny check bans sources` | 0 — ok/ok |
| `cargo deny check` (after, full) | 5 — `advisories FAILED, bans ok, licenses FAILED, sources ok` (11 errors) |

(All runs used `CARGO_TARGET_DIR=target-lane63`; the "after" full run has 11
errors, down from a config-parse failure plus 12 errors at first successful
run.)

## deny.toml changes (only file edited; via Bash scripted edit per hook)

- Removed `"MIT AND LicenseRef-P28-SIL-2.1.0"` from `[licenses] allow`.
  cargo-deny's SPDX parser rejects `LicenseRef-*` identifiers at config parse
  time, so the entry made the whole check fail to deserialize — the check
  could not run at all with it present.
- Added `"bzip2-1.0.6"` (real SPDX id carried by `libbz2-rs-sys 0.2.5`, a
  transitive dep of frozen-duckdb).
- Kept `"MulanPSL-2.0"` (purrdf 3.x, L7 migration unblock) — currently emits
  a `license-not-encountered` warning because the resolved graph still has
  purrdf 2.0.2; it becomes load-bearing when the 3.x migration lands.

## Findings delta

- Before: check could not even run (config deserialize error), exit 5.
- After: check runs; licenses residual is exactly 2 errors, both the
  workspace crates' own `license = "MIT AND LicenseRef-P28-SIL-2.1.0"` in
  `Cargo.toml:7` and `crates/graphlaw-eyeron/Cargo.toml:7`. **Not fixable from
  deny.toml** — cargo-deny has no per-crate license exception mechanism that
  parses `LicenseRef-*`. Typed residual; fix requires the coordinator to move
  the workspace license to a pure-SPDX expression (e.g. `MIT AND
  LicenseRef-...` → document P28-SIL as a patent/extra grant outside the SPDX
  field, or use `license-file`).
- Advisories: NOT force-greened, per task. 9 findings, exit 1:
  RUSTSEC-2026-0258 (h2 0.3.27, via frozen-duckdb→reqwest 0.11), 0.23.45-line
  rustls x4 (RUSTSEC-2026-0346/0347/0348/0349, via ureq 3.4.2→eyeron),
  RUSTSEC-2025-0134 (rustls-pemfile unmaintained), RUSTSEC-2026-0382
  (RUSTSEC-2026-0382: 0.23.45), RUSTSEC-2024-0449 (unmaintained
  unsafe-libyaml via serde_yaml_ng), RUSTSEC-2026-0381 (webpki-roots 1.0.9;
  fix is `cargo update -p webpki-roots` to >=1.0.10, a Cargo.lock transition).
  All fixes are Cargo.lock/dependency bumps — outside lane ownership;
  coordinator owns the `cargo update` transitions.
- bans: multiple-versions stays `warn` per operator law; check exits 0.
- sources: ok.

## CI-wiring truth (L7's ci.yml, cross-checked on disk)

`.github/workflows/ci.yml` lines 145-153 define a `deny` job invoking
`EmbarkStudios/cargo-deny-action@3c6349835b2b7b196a839186cb8b78e02f7b5f25 # v2`
with `command: check advisories licenses bans sources`. So YES — CI invokes
cargo-deny, which means the workspace LicenseRef residual and the 9 advisory
findings above will fail the `deny` CI job until the coordinator applies the
dependency/license transitions. A separate `audit` job (cargo-audit, RustSec)
also exists (line 156+).

## Verdict

- licenses: TYPED-RESIDUAL (2 errors, workspace-only, unfixable in deny.toml)
- advisories: FAIL reported, not force-greened (9 RUSTSEC)
- bans/sources: PASS
- One transient failure observed mid-run: another lane's in-flight Cargo.toml
  edit (duplicate `[patch.crates-io]`) briefly broke `cargo metadata`; it
  resolved on retry. Not a deny.toml defect.
