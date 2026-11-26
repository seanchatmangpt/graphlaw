# Hook Refuse Semantics

What `Effect::Refuse` does in the hook engine and why a caller cannot suppress it.

## Fuse, not advisory

`materialize` (`src/hooks.rs:295`) runs hooks in rounds. For non-refuse effects, a
`kh:Firing` marker already present in the input state marks the firing as seen and skips it.
Those seeded markers are advisory idempotency only: no receipt-attested prior-state notion
exists there (`:316-321`).

For `Effect::Refuse` the `seen` check is bypassed (`seen && hook.effect != Effect::Refuse`,
`:322`). A forged `kh:Firing` marker therefore cannot pre-suppress it.

## Halt and no partial commit

A refuse firing returns `RefusalKind::Unsupported` with message
`hook-refused: <hook IRI>: <reason>` (`:335-340`). The reason is the hook's `reason`, falling
back to its name. `state` is borrowed and the working copy is dropped, so the caller's prior
state is unchanged.

## Limits

A pack that exceeds `MAX_FIRINGS` refuses as `Unsupported` (`:326-331`). Deferred to
v26.11.26: streaming hooks.

## See Also

- `docs/refusals.md`
- `docs/explanation/lease-policy-and-fail-closed-defaults.md`
