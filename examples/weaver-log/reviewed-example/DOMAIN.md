# Log summary domain contract — pending developer approval

This revision resolves questions in the retained Kimi model-draft.md. S1/S2
are the original public materials; S3 contains proposed developer decisions.
No coding or execution is authorized until review-plan.json is approved.

## Rules and acceptance mapping

| Rule | Source | Acceptance |
| --- | --- | --- |
| R1: first whitespace-delimited field is the exact INFO, WARN or ERROR token; messages are optional | S1/S2/S3 | normal, severity-only, unknown token |
| R2: empty and whitespace-only lines are ignored | S1/S2 | empty input and mixed blank lines |
| R3: success emits literal total=, INFO=, WARN=, ERROR= in that order, one newline-terminated count per line; total is their sum | S1/S3 | exact output comparison for varied counts |
| R4: unknown/lowercase/attached tokens and missing input fail with any nonzero status; no failure timing is prescribed | S1/S2/S3 | UNKNOWN, info, INFO:x, INFORMATION and missing file rejected |
| R5: partial failure output is not a successful product | S2/S3 | local failure status; managed acceptance remains host-owned |
| R6: existing report silk adds Report and newline; kit emits only counts | S1/S3 | standalone full-pipeline comparison |

## Implementation and boundaries

Reuse awk; input filename is $1, output is stdout. Only kit/transform.sh may
change. Silks, this contract, selected materials, checks and manifests are
protected. No network, model, installation, GUI or GPU capability is requested
inside the spool. Model use occurs only in the separately approved development
harness. The fixed runtime remains the existing reviewed macOS shell route.

Standalone iteration uses operator-native authority and gives no host isolation
or accepted artifact claim. Managed execution needs an-agent admission and
separate approval. Any changed rule/test/source requires another review/task.

## Review decision

No unresolved question remains after accepting S3. S3 is a proposal until the
developer explicitly approves the exact review digest. Hashes do not establish
truth, identity or permission.
