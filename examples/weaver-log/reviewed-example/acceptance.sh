#!/bin/bash
set -eu
project="$1"
work="$2"
check() {
  printf '%s' "$1" > "$work/input"
  /bin/bash "$project/kit/transform.sh" "$work/input" > "$work/actual"
  printf '%s' "$2" > "$work/expected"
  cmp "$work/actual" "$work/expected"
}
check $'INFO a\nERROR b\nINFO c\n' $'total=3\nINFO=2\nWARN=0\nERROR=1\n'
check '' $'total=0\nINFO=0\nWARN=0\nERROR=0\n'
check $'  \nWARN\nINFO x\nWARN y\n' $'total=3\nINFO=1\nWARN=2\nERROR=0\n'
for invalid in 'UNKNOWN x' 'info x' 'INFO:x' 'INFORMATION'; do
  printf '%s\n' "$invalid" > "$work/input"
  if /bin/bash "$project/kit/transform.sh" "$work/input" > "$work/partial"; then
    printf 'invalid severity accepted\n' >&2
    exit 1
  fi
done

if /bin/bash "$project/kit/transform.sh" "$work/does-not-exist" > "$work/partial" 2> "$work/missing-error"; then
  printf 'missing input accepted\n' >&2
  exit 1
fi
