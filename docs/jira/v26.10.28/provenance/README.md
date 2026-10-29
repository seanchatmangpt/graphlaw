# Provenance keepers (v26.10.28 purrdf fork swap)

Copied from /tmp (OS-reap risk) on 2026-10-28; bytes verified with cmp.
Both sha256 values re-verified against disk 2026-10-28.

- `Cargo.lock.pre-swap` — pre-swap Cargo.lock snapshot (workspace lock before the
  vendored purrdf-n3 fork swap), 130856 bytes,
  sha256 62ee4c83ab995fd20a6267df4148179702f3f598e1f62c8abd82fb61c5563321.
- `purrdf-fork-44f4979.tar.gz` — purrdf fork tarball at 44f4979f, the commit the
  graphlaw-carried branch was cut from, 58773100 bytes,
  sha256 dc6f5f183e440ad91cb590107ceef7a3894b8549fda6de7ec9df3866a5004a75.
  This blob is the GitHub **codeload** archive of that commit, not local
  `git archive` output — see the replay recipe below before assuming either
  command reproduces these bytes.

## Replay recipe (purrdf-fork-44f4979.tar.gz)

### Path A — codeload download (byte-exact; the sha256 gate)

```bash
curl -fsSL \
https://codeload.github.com/seanchatmangpt/purrdf/tar.gz/44f4979f73c570b6ea4938afffe4d3713e63fec1 \
  -o purrdf-fork-44f4979.tar.gz
shasum -a 256 purrdf-fork-44f4979.tar.gz
```

Expected:

```text
dc6f5f183e440ad91cb590107ceef7a3894b8549fda6de7ec9df3866a5004a75  purrdf-fork-44f4979.tar.gz
```

Receipt (live run 2026-10-28, downloaded to /tmp/purrdf-codeload.tar.gz):

```text
$ shasum -a 256 /tmp/purrdf-codeload.tar.gz
dc6f5f183e440ad91cb590107ceef7a3894b8549fda6de7ec9df3866a5004a75  /tmp/purrdf-codeload.tar.gz
$ stat -f "%z bytes" /tmp/purrdf-codeload.tar.gz
58773100 bytes
```

Byte- and size-identical to the kept blob — this is the gate.

### Path B — local git archive (content-equivalent, byte-DIFFERENT)

```bash
git -C /Users/sac/purrdf archive --format=tar.gz -o purrdf-fork-44f4979.tar.gz \
  44f4979f73c570b6ea4938afffe4d3713e63fec1
```

Do NOT apply the `dc6f5f18…` gate to Path B output. The codeload tarball carries
the `purrdf-<sha>/` entry prefix and GitHub's own gzip stream; a local
`git archive` produces a different byte stream. Observed on 2026-10-28:

```text
$ shasum -a 256 /tmp/purrdf-replay-check.tar.gz
0c76b886f34f22c77565652a67db89bed2b432842ecf726cf54e218c454e2650  /tmp/purrdf-replay-check.tar.gz
$ stat -f "%z bytes" /tmp/purrdf-replay-check.tar.gz
58656534 bytes
```

Content-equivalence witness: the tar entry lists of Path A and Path B are
identical (22665 shared entries, `diff` clean) after stripping the codeload
`purrdf-<sha>/` prefix and dropping the root directory entry plus
`pax_global_header`. Use Path B only for content recovery, verified by
extraction diff — never by the Path A sha256 gate.

## Deletion proposal

PROPOSED: replace this 58,773,100-byte blob with the replay recipe (coordinator-owned
git rm). The tarball stays in place until the coordinator executes the transition;
this file plus Path A above is the replacement provenance.
