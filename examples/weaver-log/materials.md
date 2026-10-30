# Public log summary requirements

Source S1: the an-loom log-summary example (project-owned public requirements).
Records contain a severity token followed by an optional message. Recognize
INFO, WARN and ERROR. Blank lines are ignored. Unknown severity must fail.
The kit receives a filename as $1 and emits exactly total, INFO, WARN, ERROR counts
in that order as key=value lines. The existing report silk adds Report as header.
Use existing awk rather than rebuilding a parser framework. No network, model
calls, dependency installation, GUI or GPU is needed inside the spool.

Source S2: operator decisions for this qualification.
Matching is case sensitive. Whitespace-only records are blank. A severity-only
line is valid. Invalid input may produce diagnostic/partial bytes but must return
nonzero and cannot count as a successful product. Empty input has four zero counts.

Keep stable silks and acceptance scripts protected. Only kit/transform.sh is
editable in the coding task. Resolve any conflicting requirements with the
operator; do not invent new severity levels or repair unknown records silently.
