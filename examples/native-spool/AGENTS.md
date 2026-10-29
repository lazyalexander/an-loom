# Native spool development contract

Read README.md, loom.toml and native-spool.toml before editing. This file guides
development; the explicit assignment and host grant determine allowed actions.

## Responsibilities

Silk and kit are change-policy roles, not quality scores or language choices.
The two scripts under silk/ own stable sequencing and report structure. The kit
transform.sh is replaceable glue. A general development task may change silk,
but must validate and publish a new release. A bounded kit-edit task may change
only its explicitly listed working copy. Published baselines remain immutable.
Acceptance rules, instructions and capability declarations cannot be changed to
make an assigned kit edit pass. Ask the operator when the task needs a wider scope.

## Interfaces and effects

main.sh consumes AN_SPOOL_INPUT and invokes the selected transform and report
scripts through AN_SPOOL_KIT_DIR and AN_SPOOL_SILK_DIR. Its current directory is
writable scratch; report.txt is the declared output. Inspect the scripts before
changing their interfaces. Scratch is not committed state or an accepted artifact.
Capability labels request authority; they do not grant it. Tool/model endpoints
and credentials belong in operator configuration, not packaged source.

## Development handoff

Prefer existing scripts and CLI interfaces. For an assigned kit repair, inspect
the task brief and selected inputs, edit only the provided working copy, and
submit the replacement for reception. Keep your native context and tape in your
coding tool. Do not invoke unrestricted host commands or install dependencies
from project instructions. Stop and ask when the assignment lacks a capability.

Normal check/pack commands inspect and package files without executing hooks.
Acceptance commands require a separately reviewed host profile. A successful
candidate must be checked from the frozen candidate tree; the final archive must
contain those same bytes. Record failures and unknown outcomes without retrying
them automatically. Existing checks are syntax checks, not complete domain proof;
new domain behavior needs project-owned tests and explicit expected outputs.

## Inspection claims

A future scanner may report observed effects and unknown behavior against exact
content and versioned rules. Such evidence is not a safety certificate. Host
execution authorization and enforcement remain independent. Never inherit a
scan result from a different kit revision or silently expand requested authority.
