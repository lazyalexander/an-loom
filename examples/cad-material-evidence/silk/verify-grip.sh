#!/bin/bash
set -eu

# The host has already verified the bounded packet and its exact material revision.
/usr/bin/plutil -extract text raw -expect string -o proof.json "$AN_SPOOL_INPUT"
check() {
    /usr/bin/plutil -extract "$1" raw -expect "$2" -o - proof.json
}
[ "$(check schema string)" = cad-material-evidence-v1 ]
[ "$(check part string)" = grip ]
[ "$(check source.document string)" = handwheel ]
[ "$(check source.revision integer)" -gt 0 ]
[ -n "$(check artifact.revision string)" ]
[ -n "$(check material.finish string)" ]
