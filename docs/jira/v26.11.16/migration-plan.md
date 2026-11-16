# graphlaw migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/graphlaw
- Remote: https://github.com/seanchatmangpt/graphlaw (origin/main)
- Branch: main@ccaa171, last commit 2026-11-09, 455 commits
- Size: 13405 MB — target-arr-P19-fuzz 12G, graphlaw_ex 165M, docs 142M, ontologies 87M, js 61M
- Languages/build: Rust 1.98.0 (rust-toolchain.toml, Cargo workspace, deny.toml), Elixir
  (graphlaw_ex/), wasm/, js/, python/, ggen
- Nested repos: none

## Observed state
- Dirty 1 / untracked 29: receipts/CHAIN.blake3 modified; untracked receipts/transition-*.json,
  python/, docs/python.
- Ahead 8 / behind 0 vs origin/main. Stash 0.
- Unpushed local branches: 11 (errc-2, errc-26929, final-errc, fix/runaway-loop-*,
  plan-admission, reconstituted-specimen, release/crates-io-26.9.29, wave1..4-negation), all
  v26.9.29-era unreleased work.
- Build dirs: target-arr-P19-fuzz 12G (fuzz target dir), mutants.out / .old / .prev-L3,
  mutation-court-L3.log. No submodules.

## What it is
GraphLaw: Rust law-admission kernel — immutable content-addressed RDF/SHACL states, admitted
transitions with Ed25519-signed receipts, typed refusals, SQL/DuckDB projection, same artifact
on host and wasm. Published on crates.io (graphlaw-eyeron 0.7.8).

## Classification
ACTIVE (census agrees).

## Migration risk
HIGH. 8 unpushed commits on main plus 11 local-only branches of v26.9.29 work; receipt chain
has an uncommitted mint (CHAIN.blake3 + transition json) that must be committed in order or the
chain breaks. python/ and docs/python entirely untracked.

## Recommended action
COMMIT-PUSH
1. `cd /Users/sac/graphlaw && git add receipts python docs/python && git commit -F <msg>` —
   repo convention chains receipt mints in their own commit.
2. `git push origin main` (8+1 commits).
3. `git push origin --all` for the 11 branches. DECISION: which of wave1..4 / errc-* are
   superseded by main and can be deleted after push.
4. `rm -rf target-arr-P19-fuzz mutants.out mutants.out.old mutants.out.prev-L3` — drops 12G;
   repo becomes ~1.4G.
5. `du -sh var` and DECISION on var/ (runtime state, untracked?) — rsync if needed.
6. Clone fresh on the new machine; verify rust-toolchain 1.98.0 installs.

## First commands on the new machine
`git clone https://github.com/seanchatmangpt/graphlaw && cd graphlaw && cargo check --workspace && cargo test --workspace` then `cd graphlaw_ex && mix deps.get && mix test`

## See Also
- ../v26.10.28/
- /Users/sac/docs/jira/v26.11.16/README.md
