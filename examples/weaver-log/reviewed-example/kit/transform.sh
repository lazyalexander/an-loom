set -eu
awk '
  BEGIN { info = warn = error = bad = 0 }
  NF == 0 { next }
  $1 == "INFO" { info++; next }
  $1 == "WARN" { warn++; next }
  $1 == "ERROR" { error++; next }
  { bad = 1 }
  END {
    if (bad) exit 1
    print "total=" (info + warn + error)
    print "INFO=" (info + 0)
    print "WARN=" (warn + 0)
    print "ERROR=" (error + 0)
  }
' "$1"
