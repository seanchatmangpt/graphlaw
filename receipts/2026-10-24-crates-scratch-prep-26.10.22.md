# crates.io publish scratch — verification receipt (v26.10.22)

Subject: the crates.io publish scratch at
**`/tmp/gl-pub-26.10.22.1dfLUa`**, staged 2026-10-24 03:05 (file mtimes) with
the publish-prep deltas applied 11:14–11:15 same day. This lane re-verified
the scratch live on 2026-10-24 and re-ran the eyeron dry-run per the P16
Stage-A/B machine-prep directive. Verdict: **scratch is tag-faithful to
`v26.10.22` with an EXACT 4-file delta (plus 1 added untracked file), all
in-scratch gates green, eyeron `cargo publish --locked --dry-run` exit 0.**

- **Identity**: read+build lane over `/tmp/gl-pub-26.10.22.1dfLUa` vs the
  `git show v26.10.22:` blobs and a fresh `git archive v26.10.22` extraction
  (`/tmp/lane-tagtree.FsbLoB`), plus `/Users/sac/graphlaw` (main) for the
  tag-era-delta proof.
- **Authority**: write lease = this file only; the scratch itself was only
  read + dry-run-built into its existing `target-scratch/` lease. No
  publish, no git transitions, no scratch mutation beyond cargo's own
  `--dry-run` packaging scratch.
- **Consequence**: one file created (this one); zero code edits.
- **Replay**: every block below re-runs read-only; the archive extraction
  uses a fresh `mktemp -d` each time (the `/tmp/lane-tagtree.*` path below
  is this run's instance).
- **Standing**: valid while the scratch mtime set is unchanged; any edit to
  the scratch after 2026-10-24 invalidates this receipt — re-derive the
  whole tree diff at use time.

## 1. Whole-tree delta vs tag (the EXACT-delta proof)

Fresh `git archive v26.10.22` extraction diffed against the scratch
(excluding cargo's build lease `target-scratch/`, the intentional `vendor/`,
and the untracked dot-dirs verified separately below):

```text
$ TAGTREE=$(mktemp -d /tmp/lane-tagtree.XXXXXX) && git archive v26.10.22 | tar -x -C "$TAGTREE" && diff -rq "$TAGTREE" /tmp/gl-pub-26.10.22.1dfLUa -x target-scratch -x vendor -x .git -x .claude -x .qme -x .cargo
Files /tmp/lane-tagtree.FsbLoB/Cargo.toml and /tmp/gl-pub-26.10.22.1dfLUa/Cargo.toml differ
Only in /tmp/gl-pub-26.10.22.1dfLUa: LICENSE-P28-SIL.md
Files /tmp/lane-tagtree.FsbLoB/LICENSE.md and /tmp/gl-pub-26.10.22.1dfLUa/LICENSE.md differ
Files /tmp/lane-tagtree.FsbLoB/crates/graphlaw-eyeron/Cargo.toml and /tmp/gl-pub-26.10.22.1dfLUa/crates/graphlaw-eyeron/Cargo.toml differ
Files /tmp/lane-tagtree.FsbLoB/crates/graphlaw-eyeron/UPSTREAM.md and /tmp/gl-pub-26.10.22.1dfLUa/crates/graphlaw-eyeron/UPSTREAM.md differ
```

**Exactly 4 differing files + 1 added file** — and `Cargo.lock` is NOT in
the list, i.e. the lock is byte-identical to the tag (the `--locked`
publish precondition holds). The added `LICENSE-P28-SIL.md` at scratch root
is the referent of the restored MIT `LICENSE.md` pointer line (below); it
is byte-identical to the main tree's copy:

```text
$ shasum -a 256 /tmp/gl-pub-26.10.22.1dfLUa/LICENSE-P28-SIL.md /Users/sac/graphlaw/LICENSE-P28-SIL.md
994336a8bd096bf69f1e710a532e501d19d688969e2949b9dfbb513d90e86993  /tmp/gl-pub-26.10.22.1dfLUa/LICENSE-P28-SIL.md
994336a8bd096bf69f1e710a532e501d19d688969e2949b9dfbb513d90e86993  /Users/sac/graphlaw/LICENSE-P28-SIL.md
$ diff /Users/sac/graphlaw/LICENSE-P28-SIL.md /tmp/gl-pub-26.10.22.1dfLUa/LICENSE-P28-SIL.md && echo identical
identical
```

Hidden dirs verified identical to the tag archive:

```text
$ for d in .cargo .claude .qme .github; do diff -rq "$TAGTREE/$d" "/tmp/gl-pub-26.10.22.1dfLUa/$d"; echo "[$d exit=$?]"; done
[.cargo exit=0]
[.claude exit=0]
[.qme exit=0]
[.github exit=0]
```

## 2. The 4-file delta, each file vs `git show v26.10.22:<path>`

### 2a. `Cargo.toml` — license MIT + docs.rs curated feature pin

```text
$ git show v26.10.22:Cargo.toml > /tmp/lane-tag-blob.txt && diff /tmp/lane-tag-blob.txt /tmp/gl-pub-26.10.22.1dfLUa/Cargo.toml
7c7
< license = "MIT AND LicenseRef-P28-SIL-2.1.0"
---
> license = "MIT"
165c165
< all-features = true
---
> features = ["abi", "evidentiary-ring", "pack-tools"]
```

### 2b. `crates/graphlaw-eyeron/Cargo.toml` — license MIT + `LICENSE-P28-SIL.md` dropped from `include`

```text
$ git show v26.10.22:crates/graphlaw-eyeron/Cargo.toml > /tmp/lane-tag-blob.txt && diff /tmp/lane-tag-blob.txt /tmp/gl-pub-26.10.22.1dfLUa/crates/graphlaw-eyeron/Cargo.toml
7c7
< license = "MIT AND LicenseRef-P28-SIL-2.1.0"
---
> license = "MIT"
14c14
< include = ["/src/**", "/Cargo.toml", "/README.md", "/LICENSE.md", "/LICENSE-P28-SIL.md", "/UPSTREAM.md"]
---
> include = ["/src/**", "/Cargo.toml", "/README.md", "/LICENSE.md", "/UPSTREAM.md"]
```

### 2c. `LICENSE.md` — proven-MIT restore (first line and tail shown; the
tag-era body was the P28-SIL instrument, 274 lines, replaced by the MIT text
with Pieter Bonte / Ghent University - imec copyright plus a pointer line)

```text
$ git show v26.10.22:LICENSE.md > /tmp/lane-tag-blob.txt && diff /tmp/lane-tag-blob.txt /tmp/gl-pub-26.10.22.1dfLUa/LICENSE.md | head -27
1c1
< # P28 Sovereign Invariant License (P28-SIL)
---
> The MIT License (MIT)
3c3,5
< **Canonical Specification Identifier:** `urn:graphlaw:license:p28-sil:v2.1.0-sovereign`
---
> graphlaw is a sovereign reasoning engine. Modifications made
> for the graphlaw workspace are Copyright (c) 2026 Sean Chatman and are offered
> under the same MIT terms.
5c7,8
< **Mathematical Category:** Sheaf-Theoretic State Admission & Monotonic Conservation Topology
---
> Copyright (c) 2022-now Pieter Bonte
> Ghent University - imec, Belgium
7c10,15
< **Evidentiary Jurisprudence:** Self-Attesting Algorithmic Estoppel / Statutory Accrual Mandate
---
> Permission is hereby granted, free of charge, to any person obtaining a copy
> of this software and associated documentation files (the "Software"), to deal
> in the Software without restriction, including without limitation the rights
> to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
> copies of the Software, and to permit persons to whom the Software is
> furnished to do so, subject to the following conditions:
9c17,18
< **Operational Framework:** Adversarial Hyper-Dimensional Cognitive Load Theory (AHDCLT)
---
> The above copyright notice and this permission notice shall be included in
> all copies or substantial portions of the Software.
11c20,26
< ---
---
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
> IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
> FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
> AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
> LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
> OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
> THE SOFTWARE.
13,274c28
< ### Preamble: *Lex Machina Non Excusat et Pacta Sunt Servanda*
< [...11861 characters truncated by the capture...]
< `urn:graphlaw:license:p28-sil:v2.1.0-sovereign`
---
> Use/execution of the graphlaw kernel is additionally governed by LICENSE-P28-SIL.md (P28-SIL v2.1.0).
```

(The `13,274c28` hunk body is elided at the marked points exactly as the
capture truncated it; every hunk header and shown line is verbatim — the
real diff's hunk sequence is `1c1`, `3c3,5`, `5c7,8`, `7c10,15`, `9c17,18`,
`11c20,26`, `13,274c28`. The full body re-derives with the shown command.)

### 2d. `crates/graphlaw-eyeron/UPSTREAM.md` — the 0.7.9 version claim

```text
$ git show v26.10.22:crates/graphlaw-eyeron/UPSTREAM.md > /tmp/lane-tag-blob.txt && diff /tmp/lane-tag-blob.txt /tmp/gl-pub-26.10.22.1dfLUa/crates/graphlaw-eyeron/UPSTREAM.md
9c9
< - This crate's own version is 0.7.8: upstream 0.7.7 plus the GraphLaw changes below. The
---
> - This crate's own version is 0.7.9 (bumped from 0.7.8 — crates.io already carries an unrelated 0.7.8; the local 0.7.8 was never published, commit 6b57840): upstream 0.7.7 plus the GraphLaw changes below. The
```

## 3. Root `Cargo.toml` vs MAIN — the tag-faithful delta (disclosed precisely)

The scratch `Cargo.toml` vs the main tree's `Cargo.toml` differs ONLY by the
tag-era runner/wasmi material (feature + bin + optional deps + dev-deps +
dev profiles, all `> ` = scratch-only because main retired wasmi) plus
comment-only lines. The block below is a LANE-RENDERED HUNK-LEVEL SUMMARY
(annotations in parentheses are the lane's, not diff output); the command
line shown re-derives the raw diff:

```text
$ diff Cargo.toml /tmp/gl-pub-26.10.22.1dfLUa/Cargo.toml     # cwd = /Users/sac/graphlaw
94a95,99        > runner = ["dep:wasmi", "dep:wasmi_wasi"]   (with GWS-18 comment, 4 lines)
112a118,124     > [[bin]] name = "graphlaw-run" required-features = ["runner"]   (with GWS-18 comment)
114,115c126     < "# artifact — the sole packet-4 stdio runner after the wasmi retirement (ADR-wasmi-retirement.md)"
                > "# artifact — successor to graphlaw-run for the pinned module."
153,156d163     < (4-line docs.rs rationale comment — main-only: "# docs.rs cannot take all-features: …")
160,161c167,168 (2-line optional-deps intro comment: "runner-eh" vs "runner" wording)
163a171,174     > wasmi = { version = "2", optional = true } / wasmi_wasi = { … }   (with wasmi-2.0.0 comment)
173a185 / 177a190 > wasmi = "2" / wasmi_wasi = "2"   (dev-deps, scratch-only)
192a206,233     > [profile.dev.package.wasmi{,_core,_ir,_collections,_wasi}] blocks (scratch-only)
```

**Semantic content** (features/bins/deps/profiles): exactly the tag-era
runner/wasmi blocks — this is the tag-faithful proof: the scratch carries
`graphlaw-run` + wasmi because the TAG carried them, not because the scratch
diverged from the release. **Comment-only deltas**: (a) main's 4-line docs.rs
rationale comment is absent from the scratch (the scratch keeps the tag's
bare `features = [...]` line — the FEATURE LIST ITSELF is identical:
`["abi", "evidentiary-ring", "pack-tools"]`), and (b) two comment lines word
the eh-runner transition in tag-era rather than retired wording. No
version, license, dependency, or profile semantics differ beyond the
tag-era blocks.

## 4. In-scratch gates (verbatim)

```text
$ ls -d /tmp/gl-pub-26.10.22.1dfLUa/.git
ls: /tmp/gl-pub-26.10.22.1dfLUa/.git: No such file or directory

$ sed -n '1,8p' /tmp/gl-pub-26.10.22.1dfLUa/Cargo.toml
[package]
name = "graphlaw"
version = "26.10.22"
edition = "2024"
rust-version = "1.96"
authors = ["Sean Chatman"]
license = "MIT"
description = "GraphLaw: library-backed RDF law-state composition over PurRDF and Eyeron"

$ grep -n '^version' /tmp/gl-pub-26.10.22.1dfLUa/crates/graphlaw-eyeron/Cargo.toml
3:version = "0.7.9"

$ ls /tmp/gl-pub-26.10.22.1dfLUa/vendor
purrdf-sparql-eval
README.md
upstream
```

Gates: versions `26.10.22` / `0.7.9` present; **no `.git`** (publishes from
the scratch cannot carry git metadata); **`vendor/` present** (the
`[patch]`ed workspace deps resolve; note crates.io publishes ignore
`[patch]` — the crates.io-side resolution is what the dry-run's index
update exercises); eyeron's own deps are registry-only (`num-bigint`,
`num-traits`, `regex` — no path deps), so the dependency-order act
(eyeron before graphlaw) is a version-availability matter, not a path
matter.

## 5. eyeron dry-run — re-run this lane, green (verbatim)

```text
$ cd /tmp/gl-pub-26.10.22.1dfLUa && CARGO_TARGET_DIR=target-scratch cargo publish --locked --dry-run -p graphlaw-eyeron
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /private/tmp/gl-pub-26.10.22.1dfLUa/wasm/Cargo.toml
workspace: /private/tmp/gl-pub-26.10.22.1dfLUa/Cargo.toml
    Updating crates.io index
   Packaging graphlaw-eyeron v0.7.9 (/private/tmp/gl-pub-26.10.22.1dfLUa/crates/graphlaw-eyeron)
    Updating crates.io index
    Packaged 29 files, 613.9KiB (137.2KiB compressed)
   Verifying graphlaw-eyeron v0.7.9 (/private/tmp/gl-pub-26.10.22.1dfLUa/crates/graphlaw-eyeron)
   Compiling graphlaw-eyeron v0.7.9 (/private/tmp/gl-pub-26.10.22.1dfLUa/target-scratch/package/graphlaw-eyeron-0.7.9)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.93s
   Uploading graphlaw-eyeron v0.7.9 (/private/tmp/gl-pub-26.10.22.1dfLUa/crates/graphlaw-eyeron)
warning: aborting upload due to dry run
DRYRUN-EXIT=0
```

(`DRYRUN-EXIT=0` was echoed by the wrapping shell from
`${pipestatus[1]}` — the cargo exit was 0. The "Packaging … verify leg"
label line is this receipt's annotation; all `cargo`-prefixed lines are
verbatim. The profile warning is pre-existing/benign and appears on every
workspace command.)

P16's "×2" precedent plus this re-run: the dry-run is green on the exact
bytes an operator publish would package — 29 files, 613.9KiB.

## 6. Remaining operator act (typed)

The scratch is machine-prep-complete; the publish itself is operator-gated
(crates.io token). Sequence, poll discipline, smokes, and closure:
`docs/jira/v26.10.23/PUBLISH-COMPLETION-RUNBOOK.md` §B.2–§B.4. Publishes
must run FROM this scratch (`cargo publish --locked -p graphlaw-eyeron`,
then `-p graphlaw`), never from the dirty main worktree.

## Falsifiers

- Any re-run of the §1 tree diff showing a 5th differing file invalidates
  the EXACT-delta claim and blocks publish until re-derived.
- `cargo publish --locked --dry-run -p graphlaw-eyeron` exiting non-zero in
  the scratch invalidates §5.
- A crates.io read showing `0.7.9`/`26.10.22` published supersedes the
  "pending operator" standing.
- Scratch mtimes advancing past 2026-10-24 11:15 outside a receipted
  transition invalidate the whole receipt (re-verify §4 first).

## See Also

- `/Users/sac/graphlaw/docs/jira/v26.10.23/PUBLISH-COMPLETION-RUNBOOK.md` (§B)
- `/Users/sac/graphlaw/docs/jira/v26.10.23/PUBLISH-STATUS.md` (rows 3/4)
- `/Users/sac/graphlaw/docs/jira/v26.10.23/OPERATOR-RUNBOOK-v26.10.22-PUBLISH.md` (§S2)
