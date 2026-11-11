#!/usr/bin/env bash
# Fails if any workspace crate's package listing contains environment, key or
# certificate files, so such local files are never published.
set -euo pipefail

cd "$(dirname "$0")/.."

pattern='(^|/)\.env|\.op$|\.pem$|\.key$'
bad=0
count=0

for dir in crates/*/; do
  [ -f "${dir}Cargo.toml" ] || continue
  crate=$(basename "$dir")
  count=$((count + 1))
  if ! listing=$(cd "$dir" && cargo package --list --allow-dirty 2>&1); then
    echo "$crate: cargo package --list failed" >&2
    echo "$listing" >&2
    exit 1
  fi
  while IFS= read -r path; do
    if [[ "$path" =~ $pattern ]]; then
      echo "$crate: $path"
      bad=1
    fi
  done <<< "$listing"
done

if [ "$bad" -ne 0 ]; then
  exit 1
fi
echo "package contents ok ($count crates)"
