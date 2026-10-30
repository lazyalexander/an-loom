#!/bin/sh
set -eu
LC_ALL=C awk '
NF == 0 { next }
$1 !~ /^(INFO|WARN|ERROR)$/ { exit 2 }
{ count[$1]++; total++ }
END {
  printf "total=%d\nINFO=%d\nWARN=%d\nERROR=%d\n", total, count["INFO"], count["WARN"], count["ERROR"]
}' "$1"
