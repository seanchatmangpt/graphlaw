# reference-generated (GENERATED — do not edit by hand)

Doc-hdit scaffolded reference skeletons for GraphLaw, rendered from the code
surface (`src/` + `crates/`) by `rust-doc-hdit-pack` templates.

Everything in this directory is a generated projection of the code surface.
Hand edits are refused by the doc_quality court and are overwritten on the
next regeneration.

## Regenerate

```sh
DOC_HDIT_BIN=/Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/target/release/doc-hdit
$DOC_HDIT_BIN scaffold \
  --code /tmp/hdit/graphlaw.code.v3.json \
  --templates /Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/templates \
  --out docs/reference-generated
```

Where the code surface JSON is produced by:

```sh
python3 /Users/sac/ggen-marketplace/scripts/gen_doc_surface.py code /Users/sac/graphlaw \
  > /tmp/hdit/graphlaw.code.v3.json
```

## Files

- `reference.md` — per-module symbol tables (rigid body; AGENT-FORBIDDEN).
- `how_to.md` — task skeletons.
- `explanation.md` — concept skeletons.

## See Also

`docs/api-stability.md` · rust-doc-hdit-pack (`packs/rust-doc-hdit-pack` in
ggen-marketplace)
