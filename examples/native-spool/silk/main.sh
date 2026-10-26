set -eu
/bin/bash "$AN_SPOOL_KIT_DIR/transform.sh" "$AN_SPOOL_INPUT" > transformed.txt
/bin/bash "$AN_SPOOL_SILK_DIR/report.sh" transformed.txt > report.txt
