#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
if [[ $# -ne 1 ]]; then
  echo "usage: $0 /path/to/refetch-project/concept-at-locked-commit" >&2
  exit 2
fi

if [[ ! -d "$1" ]]; then
  echo "concept checkout does not exist or is not a directory: $1" >&2
  exit 1
fi
src="$(cd -- "$1" && pwd -P)"
lock_file="$repo_root/SPEC_LOCK.json"
commit="$(python3 - "$lock_file" <<'PY'
import json, pathlib, sys
lock = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(lock['commit'])
PY
)"
actual="$(git -C "$src" rev-parse HEAD)"
if [[ "$actual" != "$commit" ]]; then
  echo "concept checkout must be at $commit (got $actual)" >&2
  exit 1
fi

dirty="$(git -C "$src" status --porcelain --untracked-files=all)"
if [[ -n "$dirty" ]]; then
  echo "concept checkout must be clean before snapshot sync" >&2
  echo "$dirty" >&2
  exit 1
fi
for directory in schemas fixtures rfcs docs; do
  if [[ ! -d "$src/$directory" ]]; then
    echo "concept checkout is missing required directory: $src/$directory" >&2
    exit 1
  fi
done

snapshot_parent="$repo_root/tests/spec"
target="$snapshot_parent/v0.1"
stage_root="$(mktemp -d "$snapshot_parent/.sync-v0.1.XXXXXX")"
staged="$stage_root/v0.1"
backup="$stage_root/previous-v0.1"
restore_backup=false
cleanup() {
  status=$?
  if [[ "$restore_backup" == true && ! -e "$target" && -e "$backup" ]]; then
    mv -- "$backup" "$target" || true
  fi
  rm -rf -- "$stage_root"
  return "$status"
}
trap cleanup EXIT

mkdir -p "$staged"
cp -R "$src/schemas" "$src/fixtures" "$src/rfcs" "$src/docs" "$staged/"
python3 - "$staged" <<'PY'
import hashlib, json, pathlib
import sys
root=pathlib.Path(sys.argv[1])
manifest={str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(root.rglob('*')) if p.is_file()}
(root/'SHA256SUMS.json').write_text(json.dumps(manifest, indent=2, sort_keys=True)+'\n')
PY
python3 "$repo_root/scripts/verify-spec-snapshot.py" "$staged"

if [[ -e "$target" ]]; then
  mv -- "$target" "$backup"
  restore_backup=true
fi
mv -- "$staged" "$target"
restore_backup=false
rm -rf -- "$backup"
trap - EXIT
rm -rf -- "$stage_root"
