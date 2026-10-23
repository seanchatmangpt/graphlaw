//! Cold-path query-receipt ledger (PRD §3.2.2).
//!
//! Companion to the hot-path evidence ring: the ring drainer packages drained
//! descriptors into batched canonical-JSON batch files per
//! `docs/jira/v26.10.22/designs/cold-path-ledger.md` (§Format). Conventions are
//! mirrored from [`crate::receipt_store`]: fixed key order, hand-rolled escaper
//! (no serde), content-addressed `<blake3-of-batch-bytes>.json`, subject
//! `query-ledger-v1`, atomic temp+rename.
//!
//! Where C21 receipt chains chain *states*, this ledger chains *batches*:
//! each batch names its parent (the previous batch's content digest); the
//! genesis parent is 64 ASCII zeros. A monotonic `first_seq` makes
//! drain-completeness an arithmetic check, and startup discontinuities are
//! persisted as gap markers — never backfilled. Aggregation over a period
//! reads per-batch `.agg` sidecars (saturating, with a `saturated` flag — no
//! overflow laundering) and spot-re-folds at least one batch per query as the
//! tamper check.
//!
//! Durability bound (disclosed, no WAL): flush at count 1024 or age 2s,
//! whichever first, plus drain-and-flush on [`Drop`]. A crash yields a batch
//! fully valid or absent; loss is bounded by the un-flushed partial batch and
//! disclosed through gap markers, so periods containing them aggregate with
//! `complete: false`.

mod batch;
mod storage;

pub use batch::*;
pub use storage::*;
