#!/bin/bash
# Redirect only the upstream workflow's stdout into a declared candidate file.
set -eu
/bin/bash "$1" run "$2" -ansi-log false > greetings.txt
