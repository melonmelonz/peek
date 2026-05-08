#!/usr/bin/env bash
# Vocab lint: PEEK ships to a restricted-environment course audience.
# Reject exploitation vocabulary in source, content, and docs.
#
# This is intentionally narrow. The list is the words this codebase
# should never carry, not a general-purpose security taxonomy.
set -uo pipefail

ROOT="${1:-.}"

# Words to reject (case-insensitive whole-word match).
WORDS=(
  exploit
  exploitation
  payload
  shellcode
  rootkit
  keylogger
  malware
  ransomware
  backdoor
  pwn
  pwned
)

# Paths to scan. Skip target/, .git/, and the vocab-lint script itself.
mapfile -t FILES < <(
  find "$ROOT" \
    -type f \
    \( -name '*.rs' -o -name '*.md' -o -name '*.ron' -o -name '*.txt' -o -name '*.toml' -o -name '*.yml' -o -name '*.yaml' \) \
    -not -path '*/target/*' \
    -not -path '*/.git/*' \
    -not -path '*/scripts/vocab-lint.sh' \
    -not -path '*/docs/superpowers/*'
)

fail=0
for w in "${WORDS[@]}"; do
  # \b for word boundaries; -i for case-insensitive.
  if grep -InE -i "\\b${w}\\b" "${FILES[@]}" 2>/dev/null; then
    fail=1
  fi
done

if [[ $fail -ne 0 ]]; then
  echo
  echo "vocab-lint: forbidden terms found. PEEK is course material; please rephrase." >&2
  exit 1
fi

echo "vocab-lint: clean."
