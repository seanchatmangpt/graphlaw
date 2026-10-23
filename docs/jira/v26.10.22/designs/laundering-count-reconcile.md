# Laundering corpus count reconcile

Lane L154, 2026-10-22. Reconciles GO/NO-GO board row 1 ("laundering 11/11",
from designs/RECEIPTS.md §2) against the M2/L147 witness
(`sql_rdf_laundering_corpus` at 23/0).

## Live run (this lane, 2026-10-22)

```
$ CARGO_TARGET_DIR=/tmp/gl-target cargo test --test sql_rdf_laundering_corpus
...
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
```

Exit 0. 23 tests, matching 23 `#[test]` attributes in
`tests/sql_rdf_laundering_corpus.rs` (583 lines, byte-identical to the blob at
commit 897838a — no working-tree drift).

## Trace

- **11/11** — designs/RECEIPTS.md §2 (lines ~58-72), real run by its lane at
  **12:58 PDT**, against a mid-wave working tree. The court file was first
  committed at **16:09 PDT** in 897838a (583 lines, 23 `#[test]`). The 11-test
  state never existed in any commit; it was an uncommitted intermediate.
- **23/0** — current tree; identical to the only commit touching the file
  (`git log --follow`: 897838a is the sole entry). Secondary discrepancy: the
  897838a commit message cites "15 passed / 0 failed" for this court on its own
  final tree, which contradicts its own blob (23 `#[test]`). The 15 appears to
  be another mid-wave snapshot quoted in prose.
- Growth 11→23 came from the E1/E3/E4/E5 never-both + typed-parse + cardinality
  closures named in the 897838a message, added to the working tree after the
  12:58 RECEIPTS run.

## Verdict

- **Current number: 23/0** (live exit-0 run today; exit code witnessed above).
- **11/11 is historical-as-of 12:58 PDT 2026-10-22**, pre-final working tree;
  the file was not yet committed.

## Corrected board row text

Replace row 1 with:

> Laundering corpus **23/23** (23 passed / 0 failed, exit 0; live run 2026-10-22,
> `/Users/sac/graphlaw` HEAD 615dbfa,
> `CARGO_TARGET_DIR=/tmp/gl-target cargo test --test sql_rdf_laundering_corpus`,
> 0.23s; supersedes designs/RECEIPTS.md §2's 11-test run of 12:58 PDT, which
> predates the court's final state committed at 897838a).
