# ADR: WASM Not in the Hex Tarball

- **Status**: Accepted
- **Date**: 2026-11-25

## Context

The wasm artifact is about 60 MiB (`registry/ARTIFACTS.blake3`; see
`docs/explanation/eh-closure-story.md`), above Hex's 8 MB limit
(`graphlaw_ex/mix.exs:96`).

## Decision

The package files list excludes `priv/graphlaw/graphlaw.wasm` (`graphlaw_ex/mix.exs:96-101`);
only MANIFEST.json and the blake3/sha256 sidecars ship. `mix graphlaw_ex.fetch` obtains the
Release asset and verifies sha256 and BLAKE3; `GRAPHLAW_WASM_PATH` supplies a local file for
air-gapped installs, verified the same way (`graphlaw_ex/lib/mix/tasks/graphlaw_ex.fetch.ex:13`).

## Consequences

- A fresh dependency has no runnable engine until `fetch` runs.
- The digests in the tarball pin which bytes are accepted.
- Network or Release availability is a build-time dependency.

## See Also

- `docs/explanation/eh-closure-story.md`
