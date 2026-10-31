#!/bin/sh
# Domain rules stay here; host state custody does not interpret these counts.
set -eu
test "$#" -eq 2
test ! -e state.txt
test ! -e report.txt
mkdir -p tmp
/bin/bash "$(dirname "$0")/transform.sh" "$1" > tmp/batch-summary.txt
LC_ALL=C awk -F= '
BEGIN { names[1]="total"; names[2]="INFO"; names[3]="WARN"; names[4]="ERROR" }
NF != 2 || $1 !~ /^(total|INFO|WARN|ERROR)$/ || $2 !~ /^[0-9]+$/ || $2 > 1000000 {
  bad=1; exit 2
}
{
  if (seen[FILENAME,$1]++) { bad=1; exit 2 }
  rows[FILENAME]++; values[FILENAME,$1]=$2; sum[$1]+=$2
}
END {
  for (i=1;i<=2;i++) {
    file=ARGV[i]
    if (rows[file] != 4 || values[file,"total"] != values[file,"INFO"] + values[file,"WARN"] + values[file,"ERROR"]) bad=1
  }
  if (bad || sum["total"] > 1000000) exit 2
  for (i=1;i<=4;i++) printf "%s=%d\n", names[i], sum[names[i]]
}' "$2" tmp/batch-summary.txt > tmp/state-candidate.txt
cat tmp/state-candidate.txt > state.txt
cat state.txt > report.txt
