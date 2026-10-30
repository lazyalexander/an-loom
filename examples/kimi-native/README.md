# Kimi proposal to native spool delivery

This narrow operator adapter connects Kimi Code 2.3.1, model alias `kimi-code/k3`,
to Loom's existing native candidate route. It uses the client's original model
connection and authentication; no credential or private session store is copied.
The fixture asks for lowercase, sorted unique lines prefixed with `item:` using
existing CLI programs. Domain checks stay in the example project.

The agent profile exposes no tools or subagents. Only the selected public source
and fixed goal are sent in the prompt. It returns inert JSON; reception does not
execute code. The official client/global configuration remains trusted: this is
not a sandbox for a hostile client binary or a general arbitrary-code agent.
Inspect global plugins/hooks/MCP configuration before running. No `--auto` or
`--yolo` grant is used. The isolated client directory and empty skills directory
reduce incidental context; they are not an OS isolation boundary.

## Run once per new operator directory

Build Loom and a qualified native v2 host. Use an existing authenticated Kimi CLI;
the driver refuses an unqualified CLI version. Python 3.9+ needs no third-party
packages. All client stdout/stderr and operational plans stay in the new local
output directory, outside package include lists.

```sh
python3 examples/kimi-native/qualify.py prepare --output .local/kimi-native \
  --loom target/debug/an-loom --host /selected/an-agent-host --kimi /selected/kimi
# Review prompt.txt, the explicit tool-free profile and generation-plan.json.
python3 examples/kimi-native/qualify.py generate --output .local/kimi-native \
  --approve GENERATION_DIGEST
# Review candidate.sh, not merely the model's success text.
python3 examples/kimi-native/qualify.py receive --output .local/kimi-native \
  --source-sha256 REVIEWED_SOURCE_DIGEST
an-loom develop-native .local/kimi-native/task /selected/an-agent-host \
  .local/kimi-native/check-job .local/kimi-native/delivery
# Review the printed host profile and development plan.
an-loom develop-native .local/kimi-native/task /selected/an-agent-host \
  .local/kimi-native/check-job .local/kimi-native/delivery --approve DEVELOPMENT_DIGEST
```

The model's response has exactly schema `an-loom-kit-candidate-v1`, path
`kit/transform.sh` and a nonempty `content` string. The parser accepts a bare
object or one complete JSON code block, optionally rendered with the qualified
client's leading bullet. Arbitrary prose, multiple objects, extra fields and
protected/out-of-scope paths refuse. Candidate adoption still goes through Loom's
existing baseline, editable-scope and revocation checks.

If a completed response has an unsupported render wrapper, inspect it and improve
the parser deliberately. `extract --output DIRECTORY` only reparses that retained
completed response; it never calls the model. It writes candidate.sh exclusively.
Unknown, failed or timed-out generations cannot use extraction. Never restart a
model just to repair a known completed response's presentation.

## Run and consume

The delivered instance is loaded without runtime execution permission. Inspect
and approve its existing `spool-start INSTANCE demo INPUT 15`, then
`spool-accept INSTANCE demo`, separately. Query `spool-artifacts INSTANCE demo` and
use artifact-checkout with the exact returned revision. For input
`BETA\nAlpha\nbeta\n`, the final report includes the stable report silk's header:

```text
Report
item:alpha
item:beta
```

Explicitly reclaim settled scratch and unload after acceptance. Accepted versions
remain available for fixed-version downstream consumption. Generation and model
context stay with Kimi; runtime supervision and custody stay with an-agent; Loom
coordinates development and packaging. `develop-native` reporting model_invoked
false describes that command, not the authorship of its input candidate.

## Unknown outcomes and limits

An exclusive generation-intent file is written before launching the client. The
same directory cannot generate again after any outcome. A 240-second client
timeout is recorded as unknown and never automatically retried. Client exit or
local timeout does not prove remote model cancellation. The response is not
accepted automatically and never inherits execution authority from model text.
The selected tools/profile and prompt are checked by digest between stages.

The native check route retains its existing macOS/headless/CPU/input/deadline
limits. This path qualifies tool-free code proposals, not arbitrary tool-enabled
harness execution. Real model cancellation is not exercised here; the earlier
finite-tool Kimi qualification is separate. Token usage is unknown rather than
inferred from logs. Do not scrape native session/tape stores.

[Portable acceptance](../../docs/acceptance/kimi-native.json) records an actual K3
proposal, domain checks, immutable delivery, host admission, runtime execution,
formal output checkout and retained output after cleanup. A response-format issue
was corrected by extracting the original completed response without another call.
Raw diagnostics, absolute paths, credentials and client session identifiers are
excluded from the public summary. Credential-free parser/admission tests run in CI;
paid model calls remain explicit operator acceptance.
