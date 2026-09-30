#!/usr/bin/env bash
# Enforce that every `unsafe {` block in src/ has a `// SAFETY:` comment within
# the 3 lines preceding it.
set -euo pipefail

if [[ ! -d src ]]; then
  exit 0
fi

unsafe_lines=()
while IFS= read -r line; do
  unsafe_lines+=("$line")
done < <(grep -rEn 'unsafe[[:space:]]*\{' src 2>/dev/null || true)

if [[ ${#unsafe_lines[@]} -eq 0 ]]; then
  exit 0
fi

missing=0
for entry in "${unsafe_lines[@]}"; do
  file=${entry%%:*}
  rest=${entry#*:}
  line=${rest%%:*}

  start=$(( line > 3 ? line - 3 : 1 ))
  if ! sed -n "${start},${line}p" "$file" | grep -q '// SAFETY:'; then
    echo "missing SAFETY comment near ${file}:${line}" >&2
    missing=1
  fi
done

exit "$missing"
