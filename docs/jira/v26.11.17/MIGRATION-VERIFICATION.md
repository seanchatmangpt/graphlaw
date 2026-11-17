# Migration verification (re-check of v26.11.16 migration-plan.md)

Date 2026-11-17. Standing: PARTIALLY VERIFIED. Evidence: read-only git in a fresh SHALLOW clone (/home/user/graphlaw, branch claude/exciting-turing-ku4eo6) plus `git ls-remote origin`. The original machine was not reachable.

Summary: the commit-and-push step happened; origin/main is dd6a80e. Local-machine claims (dirty, ahead 8, local branches, 12G fuzz dir) cannot be verified here.

| # | Claim | Status | Evidence |
|---|---|---|---|
| 1 | remote github.com/seanchatmangpt/graphlaw | VERIFIED | git remote -v |
| 2 | main@ccaa171 | STALE | ls-remote main = dd6a80e; ccaa171 is its parent |
| 3 | 455 commits | UNVERIFIED | 50 reachable; shallow |
| 4 | ahead 8 / behind 0 | STALE here, UNVERIFIED on Mac | origin/main..HEAD = 0 |
| 5 | dirty 1 / untracked 29 | STALE here | git status clean; CHAIN.blake3 and 3671 transition-* tracked |
| 6 | python/, docs/python untracked | STALE | 28 tracked files; HEAD changed 31 files (+2379) |
| 7 | stash 0 | UNVERIFIED | fresh clone |
| 8 | 11 unpushed branches | UNVERIFIED locally; see below | |
| 9 | target-arr-P19-fuzz 12G, mutants.out*, 13405 MB | UNVERIFIED | untracked; absent in clone (du = 760M) |
| 10 | sizes graphlaw_ex 165M, docs 142M, ontologies 87M, js 61M | PARTIAL | here: graphlaw_ex 61M, docs 142M, ontologies 88M, js 344K |
| 11 | Rust 1.98.0 | VERIFIED | rust-toolchain.toml |
| 12 | no nested repos/submodules | VERIFIED | no .gitmodules |
| 13 | workspace layout | VERIFIED | |
| 14 | var/ | UNVERIFIED | absent in clone |

## "Unpushed" branches vs origin (38 heads)
On origin: errc-2, errc-26929, final-errc, plan-admission, reconstituted-specimen, release/crates-io-26.9.29, fix/runaway-loop-and-resource-caps-v26.9.29, wave4-negation. NOT on origin: wave1-negation, wave2-negation, wave3-negation (origin has wave1, wave2, wave3). 8 of 11 names are on origin; whether Mac tips equal origin tips is UNVERIFIED. The wave1-3 naming is the open question.

## Plan steps
1-2 (commit, push main): DONE per origin. 3 (push branches): mostly done. 4-5 (delete fuzz dirs, var/): not verifiable here. 6 (fresh clone): this container is it; `cargo check` not run.

## Mac-side re-check (at /Users/sac/graphlaw)
```bash
git fetch origin && git status -sb
git rev-list --count origin/main..main
git for-each-ref --format='%(refname:short) %(upstream:track)' refs/heads
du -sh target-arr-P19-fuzz mutants.out* var
```
Delete the 12G dirs only after these confirm no unique local commits.
