#!/usr/bin/env bash
set -euo pipefail
if [[ $# -ne 1 ]]; then
  echo "usage: $0 /path/to/refetch-project/concept-at-locked-commit" >&2
  exit 2
fi
src="$1"
commit="a49e51bbfd04462398bbb7ea613f003b2c417544"
actual="$(git -C "$src" rev-parse HEAD)"
if [[ "$actual" != "$commit" ]]; then
  echo "concept checkout must be at $commit (got $actual)" >&2
  exit 1
fi
rm -rf tests/spec/v0.1
mkdir -p tests/spec/v0.1
cp -R "$src/schemas" "$src/fixtures" "$src/rfcs" "$src/docs" tests/spec/v0.1/
python3 - <<'PY'
import hashlib, json, pathlib
root=pathlib.Path('tests/spec/v0.1')
manifest={str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(root.rglob('*')) if p.is_file()}
(root/'SHA256SUMS.json').write_text(json.dumps(manifest, indent=2, sort_keys=True)+'\n')
PY
