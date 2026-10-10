# GraphLaw extraction receipt

GraphLaw was originally extracted from `seanchatmangpt/praxis` at source commit
`15504cd791cc13e117ff7cd02fd662be7a1f8631`.

That extraction is now **historical provenance, not an executable source path**.

As of v26.9.28, the copied RoXi/Praxis standards engine has been retired from the
production tree and replaced by pinned upstream Rust libraries.

**Correction (as of 2026-10-10):** an earlier revision of this document claimed the
`extract-from-praxis` workflow had been deleted so automation could not repopulate
obsolete handwritten implementations. That claim was false: the workflow remained at
`.github/workflows/extract-from-praxis.yml` on `main`, and it was destructive — a push
touching the workflow file re-ran the bootstrap and replaced the canonical tree from
`seanchatmangpt/praxis@15504cd`.

**Update (2026-10-10, later):** the workflow has now been deleted (it was recoverable
from Git history, and nothing else referenced it). The retirement of the copied engine
is enforced by upstream dependency pins (`Cargo.toml`) — and there is no longer any
automation capable of repopulating it.

The original implementation remains recoverable from Git history. Current
standards semantics come from the authorities recorded in `README.md` and
`src/lib.rs`.
