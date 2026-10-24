# final-tree-audit

Adversarial audit of the RELEASED tree at HEAD
`4df19b5b1d2480d50851e7c0e0bf855802a2c46a`
(`4df19b5 docs(v26.10.23): terminal state — npm/crates publish typed-blocked on
operator credentials`), 2026-10-24. Read-only audit: no cargo, no git
transitions. All counts and digests below are re-read from disk at use time.

## Verdict table

| # | Check | Verdict | Evidence |
|---|---|---|---|
| 1 | Version coherence triple + eyeron 0.7.9 | CLEAN | `Cargo.toml:3` = 26.10.22; `wasm/Cargo.toml:8` = 26.10.22; `js/npm/package.json` version = 26.10.22; `Cargo.lock`: graphlaw 26.10.22 (L1707), graphlaw-eyeron 0.7.9 (L1729), graphlaw-wasm 26.10.22 (L1741); path dep `Cargo.toml:68` pins eyeron `0.7.9`; root `CHANGELOG.md` top release section `## [26.10.22] - 2026-10-22` (L10); npm CHANGELOG mirrors root. No triple member diverges. |
| 2 | Pin chain one-digest (priv/blake3, registry, npm meta) | CLEAN | `priv/graphlaw.wasm.blake3:1`, `registry/ARTIFACTS.blake3:13`, `js/npm/src/generated/artifact-meta.ts:3` all = `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a`. Live recomputation: `b3sum priv/graphlaw.wasm` → `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a`, file size 58,124,612 B matches the registry size field. Stale-digest sweep of pin surfaces: only `registry/ARTIFACTS.blake3:2` carries `332ea218…`, in a line explicitly labeled "historical pin … (26.10.21)" — labeled history, not a live pin. |
| 3 | No leaked mutants | CLEAN | `grep -rn 'changed by cargo-mutants' src/` → 0; `grep -rn 'cargo-mutants' src/` → 0. Only repo hit outside src is `tests/no_mutant_markers.rs:9`, which is the anti-leak court itself (builds the marker via `concat!("changed by ", "cargo-mutants")` precisely so it never self-matches). `wasm/src/` and `tests/` sweep otherwise 0. |
| 4 | No reconstituted-crate refs in workspace manifest | CLEAN | `grep -i reconstitut Cargo.toml` → no match. `[workspace] members = [".", "wasm", "crates/graphlaw-eyeron"]` (`Cargo.toml:16`); `crates/` holds only `graphlaw-eyeron`; the sole `crates/` path dependency is eyeron (`Cargo.toml:68`). Excluded paths (`vendor/purrdf-sparql-eval`, `courts/castle-diff`) exist on disk. |
| 5 | Publish paths clean (uncommitted = 0) | CLEAN at HEAD / 1 residue file in working tree | `git status --porcelain` filtered to `src/ tests/ priv/ registry/` modified/deleted: 0. `priv/`, `registry/`, `js/npm/src/` fully clean and tracked. One untracked file in a publish path: `tests/zz_setop_probe.rs` (59 lines, self-declared "TEMPORARY engine-reality probe (deleted before turn end)"). Not in HEAD — the released tree is clean; residue is post-release lane work. |
| 6 | COPY-egress disclosure in RELEASE-NOTES | CLEAN | `docs/jira/v26.10.22/RELEASE-NOTES-26.10.22.md:87-95`, under "Known limitations (disclosed, typed, ticketed)": COPY TO egress file-locking fails under wasi preview1 (`Could not set lock … Invalid argument`), witnessed deterministically (`tests/wasm_fibo_egress_eh.rs`), ticketed SQLRDF-27 (reopened), with the packet-4 host-side egress workaround stated. This is a real disclosure (limitation + witness + ticket + workaround), not a name-check. |
| 7 | Compliance verdict + matrices registered | CLEAN (registration on disk; git-track pending) | `docs/jira/v26.10.23/INDEX.md` is landed on disk (untracked) and registers the consolidated verdict (`compliance-final-verdict.md`) plus all 6 matrices on disk (EU AI Act, logging, robustness; NIST GOVERN/MAP, MEASURE/MANAGE; transparency) + closure plan, with the dispatch-vs-disk delta noted in the index itself ("dispatch projected 7; disk holds 6"). Disk census re-verified: 24 `.md` files in the window = 23 registered + INDEX itself. Counts reconcile. |
| 8 | Untracked accumulation | CLASSIFIED | 158 untracked total (prior pass: 149 → +9 receipts minted during this audit; latest receipt `2026-10-24T102207Z` — lanes still active). Classification in the next section. |

## Check 8 classification (158 untracked + 2 modified)

| Class | Count | Items | Action |
|---|---|---|---|
| receipts (commit-later) | 155 | `receipts/transition-2026-10-24T*.json` | Commit-later, together with the modified `receipts/CHAIN.blake3`. |
| real docs (commit) | 2 + 1 modified | `docs/jira/v26.10.23/INDEX.md`, `docs/jira/v26.10.23/PUBLISH-STATUS.md`; modified `docs/reference/wasm-gap-register.md` | Commit. INDEX.md is the window register that check 7 depends on — until it is committed, registration exists only on disk. |
| lane residue (delete, never commit) | 1 | `tests/zz_setop_probe.rs` | Delete at integration. It is in a publish path: an untracked file under `tests/` would be swept into any future `cargo package`/publish. Must be gone before any crates.io publish. |
| lane build roots (ignored; delete at integration) | 32 dirs | `target-lane*` (gitignored — 32 `!!` entries) | Coordinator deletes per the lane cleanup law; listed for disk weight only. |

## Informational findings (not check failures)

1. `docs/jira/v26.10.23/ROADMAP.md:17-18` still cites artifact identity
   `472d5557…` as what v26.10.22 was "witnessed end-to-end on". The shipped
   artifact is `58a44095…` (GO-NO-GO itself records `472d5557…` as the
   superseded proposed-tag pin that "None mention 58a44095"). Stale cite in a
   registered doc; the index registers ROADMAP's self-declared status, so this
   is doc hygiene, not an index defect.
2. `js/npm/package.json` local version is 26.10.22 while the published npm
   version is 26.10.14 and crates.io sits at graphlaw 26.10.21 / eyeron 0.7.8
   (per `PUBLISH-STATUS.md`). This is publish-state lag, not tree incoherence —
   the publish is typed-blocked on operator credentials (HEAD commit subject).
3. `registry/ARTIFACTS.blake3:2` historical-pin comment mentions superseded
   digest `332ea218…`; the line is explicitly labeled historical (26.10.21) and
   the live pin line (13) is `58a44095…`.

## Receipt commands (real output embedded above)

- `git rev-parse HEAD` → `4df19b5b1d2480d50851e7c0e0bf855802a2c46a`
- `b3sum priv/graphlaw.wasm` →
  `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a`
- `git status --porcelain` → 158 `??`, 2 `M`;
  `receipts-untracked=155`, `untracked-total=158`,
  `modified-src-count=0`
- `grep -rn 'changed by cargo-mutants' src/ | wc -l` → `0`

## Post-audit resolution addendum (2026-10-24, closeout)

`tests/zz_setop_probe.rs` (check 5) was deleted at the set-operations
integration (`6da1d33`, 03:45:14-0700), per this audit's own
"Delete at integration" action. It was never tracked, so `git log --all`
shows nothing — "was on disk at 03:26" and "absent now" are both true.

Correction to the check-5 evidence note: an untracked file under
`tests/` cannot enter the `graphlaw` .crate while the manifest include
whitelist stands — `Cargo.toml [package] include` omits `/tests/**`
(verified), so `tests/` residue is packaging-inert. Untracked files are
packaging-relevant only under include-covered paths (`src/`, `docs/`,
`examples/`, `ontologies/`, `ggen/`) or as whitelisted root filenames.
The check-8 lane-root count has also moved since (coordinator lane-root
cleanup at integration).

Verdict cells above are untouched; this addendum is appended, not
rewritten.
