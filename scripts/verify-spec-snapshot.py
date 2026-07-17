#!/usr/bin/env python3
import hashlib, json, pathlib, sys
root = pathlib.Path('tests/spec/v0.1')
manifest_path = root / 'SHA256SUMS.json'
manifest = json.loads(manifest_path.read_text())
ok = True
for rel, expected in sorted(manifest.items()):
    if rel == 'SHA256SUMS.json':
        continue
    path = root / rel
    if not path.exists():
        print(f'missing: {rel}', file=sys.stderr); ok = False; continue
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    if actual != expected:
        print(f'mismatch: {rel}: {actual} != {expected}', file=sys.stderr); ok = False
actual_files = {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file() and p.name != 'SHA256SUMS.json'}
expected_files = {rel for rel in manifest if rel != 'SHA256SUMS.json'}
for rel in sorted(actual_files - expected_files):
    print(f'untracked snapshot file: {rel}', file=sys.stderr); ok = False
sys.exit(0 if ok else 1)
