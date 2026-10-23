# wasm artifact pin re-verification receipt (lane V-L17, staleness-gate R4 remediation)

- Date: 2026-10-22 (local)
- Subject: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty.
- Backs: `docs/jira/v26.10.22/RECEIPTS.md:122-123` (§2 wasm artifact re-pin block).
- Command: `blake3sum priv/graphlaw.wasm`

```
332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c  priv/graphlaw.wasm
```

On-disk artifact matches the pinned digest `332ea218…2c30c` recorded in
`docs/jira/v26.10.22/RECEIPTS.md` §2 and registered in
`registry/ARTIFACTS.blake3`. Standing: PASS (pin verified, digest registered).
