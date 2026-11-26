#!/usr/bin/env bash
# Projection sync for graphlaw_ex: ggen sync run -> prune strays -> mix format -> sha256 manifest.
#
# Usage: scripts/ggen_sync.sh [--check | --list-projections | --help]
#   (none)              run the sync once, prune, format, print the sha256 manifest.
#   --check             determinism self-check: sync twice, compare manifests, then
#                       `git diff --exit-code -- graphlaw_ex/lib`, fail on untracked outputs under
#                       graphlaw_ex and on any lib/ash_graphlaw directory.
#   --list-projections  print every path in PROJECTIONS, one per line.
#
# Op and model lists are derived from ../registry/capability-registry.json (".ops[].name",
# ".models[].name"); --check also asserts ../contracts/abi-manifest.json ops equal the registry ops.
# The result projection set is the capability set minus `law` (no result decoder for law).
# Staging of ontology/.staged happens only inside a sync run, never for --list-projections/--help.
#
# Exit codes: 0 ok, 2 pack missing, 3 pack version mismatch / manifest unreadable, 4 sync failed,
# 5 format failed, 6 non-deterministic, 9 committed tree differs from projection,
# 10 untracked outputs, 11 forbidden lib/ash_graphlaw dir, 64 usage.
set -euo pipefail
cd "$(dirname "$0")/.."
here=$(pwd)
repo_root=$(cd .. && pwd)

pack="../../ggen-marketplace/packs/graphlaw-ash-capability-pack"
abi_manifest="../contracts/abi-manifest.json"

# Expected pack version, asserted against the pack's pack.toml before every sync.
expected_pack_version="26.11.25"

registry_json="../registry/capability-registry.json"

command -v python3 >/dev/null 2>&1 || { echo "REFUSED(tool:python3-required)" >&2; exit 3; }
[ -f "$registry_json" ] || { echo "REFUSED(registry:missing $registry_json)" >&2; exit 3; }

# Ops and models both come from the one registry json.
read_ops() {
  python3 -c 'import json,sys; [print(o["name"]) for o in json.load(open(sys.argv[1]))["ops"]]' "$registry_json"
}
read_manifest_ops() {
  python3 -c 'import json,sys; [print(o["name"]) for o in json.load(open(sys.argv[1]))["ops"]]' "$abi_manifest"
}
read_models() {
  python3 -c 'import json,re,sys; [print(re.sub(r"(?<!^)(?=[A-Z])","_",m["name"]).lower()) for m in json.load(open(sys.argv[1]))["models"]]' "$registry_json"
}

# ggen refuses `..` in ontology paths: stage the canonical registry graph inside the project.
# The staged copy is a byte copy of ../registry/capability-registry.ttl (gitignored), never an edit.
# Called only from run_sync so --list-projections/--help never touch the tree.
stage_ontology() {
  mkdir -p ontology/.staged
  cp ../registry/capability-registry.ttl ontology/.staged/capability-registry.ttl
}

CAPABILITY_OPS=()
while IFS= read -r op; do [ -n "$op" ] && CAPABILITY_OPS+=("$op"); done < <(read_ops)
[ "${#CAPABILITY_OPS[@]}" -gt 0 ] || { echo "REFUSED(registry:no-ops)" >&2; exit 3; }
RESULT_OPS=()
for op in "${CAPABILITY_OPS[@]}"; do [ "$op" = "law" ] || RESULT_OPS+=("$op"); done

PROJECTIONS=(
  lib/graphlaw_ex/capability.ex
  lib/graphlaw_ex/capability/registry.ex
  lib/graphlaw_ex/capability/api.ex
  lib/graphlaw_ex/capability/limits.ex
  lib/graphlaw_ex/result/term.ex
)
for op in "${CAPABILITY_OPS[@]}"; do PROJECTIONS+=("lib/graphlaw_ex/capability/$op.ex"); done
for op in "${RESULT_OPS[@]}"; do PROJECTIONS+=("lib/graphlaw_ex/result/$op.ex"); done
# Typed models (one file per registry model, snake_case) plus the shared enums module.
PROJECTIONS+=(lib/graphlaw_ex/model/enums.ex lib/graphlaw_ex/model.ex)
while IFS= read -r m; do [ -n "$m" ] && PROJECTIONS+=("lib/graphlaw_ex/model/$m.ex"); done < <(read_models)

usage() { echo "usage: scripts/ggen_sync.sh [--check | --list-projections | --help]" >&2; }

manifest() {
  local f
  for f in "${PROJECTIONS[@]}"; do
    if [ -f "$f" ]; then shasum -a 256 "$f"; else printf 'MISSING  %s\n' "$f"; fi
  done | LC_ALL=C sort -k2
}

is_projection() {
  local p
  for p in "${PROJECTIONS[@]}"; do [ "$p" = "$1" ] && return 0; done
  return 1
}

# Remove untracked files under model/ documentation/ test/generated/ that are not projections.
prune_strays() {
  local d f
  for d in model documentation test/generated; do
    [ -d "$d" ] || continue
    while IFS= read -r f; do
      is_projection "$f" && continue
      git ls-files --error-unmatch -- "$f" >/dev/null 2>&1 && continue
      rm -f -- "$f"
      echo "pruned stray output: $f"
    done < <(find "$d" -type f)
  done
}

assert_pack_version() {
  local actual
  actual=$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$pack/pack.toml" | head -n1)
  [ -n "$expected_pack_version" ] || { echo "REFUSED(pack:version-unpinned scripts/ggen_sync.sh expected_pack_version)" >&2; exit 3; }
  [ "$actual" = "$expected_pack_version" ] \
    || { echo "REFUSED(pack:version-mismatch expected=$expected_pack_version actual=$actual)" >&2; exit 3; }
}

run_sync() {
  [ -d "$pack" ] || { echo "REFUSED(pack:missing $pack)" >&2; exit 2; }
  assert_pack_version
  stage_ontology
  local log
  log=$(mktemp)
  ggen sync run --skip-ea-admission >"$log" 2>&1 || { cat "$log" >&2; rm -f "$log"; echo "REFUSED(sync:failed)" >&2; exit 4; }
  rm -f "$log"
  prune_strays
  mix format || { echo "REFUSED(format:failed)" >&2; exit 5; }
  echo "ggen sync ok"
}

# --check: contracts/abi-manifest.json ops must equal the registry ops (order-insensitive).
assert_manifest_ops_match() {
  [ -f "$abi_manifest" ] || { echo "REFUSED(manifest:missing $abi_manifest)" >&2; exit 3; }
  local want got
  want=$(printf '%s\n' "${CAPABILITY_OPS[@]}" | LC_ALL=C sort)
  got=$(read_manifest_ops | LC_ALL=C sort)
  if [ "$want" != "$got" ]; then
    echo "REFUSED(manifest:ops-differ abi-manifest vs registry json)" >&2
    diff <(printf '%s\n' "$want") <(printf '%s\n' "$got") >&2 || true
    exit 3
  fi
}

case "${1:-}" in
  "") run_sync; manifest ;;
  --check)
    assert_manifest_ops_match
    run_sync; first=$(manifest)
    run_sync; second=$(manifest)
    if [ "$first" != "$second" ]; then
      echo "REFUSED(determinism:differs)" >&2
      diff <(printf '%s\n' "$first") <(printf '%s\n' "$second") >&2 || true
      exit 6
    fi
    printf '%s\n' "$second"
    echo "determinism ok: two syncs byte-identical ($(printf '%s\n' "$second" | wc -l | tr -d ' ') files hashed)"
    (cd "$repo_root" && git diff --exit-code -- graphlaw_ex/lib) \
      || { echo "REFUSED(tree:differs projection vs committed graphlaw_ex/lib)" >&2; exit 9; }
    echo "tree ok: git diff --exit-code -- graphlaw_ex/lib clean"
    if [ -d lib/ash_graphlaw ]; then
      echo "REFUSED(tree:forbidden-dir lib/ash_graphlaw)" >&2; exit 11
    fi
    untracked=$(cd "$repo_root" && git status --porcelain graphlaw_ex | grep '^??' || true)
    if [ -n "$untracked" ]; then
      printf '%s\n' "$untracked" >&2
      echo "REFUSED(tree:untracked-outputs under graphlaw_ex)" >&2; exit 10
    fi
    echo "untracked ok: no untracked outputs under graphlaw_ex"
    ;;
  --list-projections) printf '%s\n' "${PROJECTIONS[@]}" ;;
  -h|--help) usage ;;
  *) echo "REFUSED(usage:unknown-flag $1)" >&2; usage; exit 64 ;;
esac
