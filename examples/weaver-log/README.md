# Domain preparation with your existing harness

English · [简体中文](../../docs/zh-CN/examples/weaver-log/README.md)

The spool example owns its public materials, domain instructions and acceptance
rules. `weaver.py` prepares and reviews those bytes, then receives a candidate
through Loom. Your existing harness owns model selection, authentication,
conversation history, generation and cancellation. No harness is imported,
launched or supervised by this adapter. No model service is required by CI.

## Prepare, generate and review

Build Loom once. Use an explicit, new absolute review directory and select the
native harness/model and disclosure policy before invoking it. These commands
illustrate Codex's [non-interactive interface](https://learn.chatgpt.com/docs/non-interactive-mode);
they are operator commands, not a new runtime adapter or a containment claim.
`CODEX_MODEL` is an operator selection; the example does not choose a model.

```sh
python3 -B examples/weaver-log/weaver.py prepare --loom /path/to/an-loom \
  --output /new/review-directory
export REVIEW=/new/review-directory
# Set CODEX_MODEL and review the native client's effective configuration first.
codex exec --ignore-user-config --sandbox read-only --skip-git-repo-check \
  --ephemeral --model "$CODEX_MODEL" --cd "$REVIEW/client" \
  --output-last-message "$REVIEW/DOMAIN.md" - < "$REVIEW/draft-prompt.txt"
# Inspect and, if needed, edit DOMAIN.md, materials.md and acceptance.sh.
python3 -B examples/weaver-log/weaver.py review --output "$REVIEW"
python3 -B examples/weaver-log/weaver.py approve --output "$REVIEW" --approve REVIEW_SHA256
codex exec --ignore-user-config --sandbox read-only --skip-git-repo-check \
  --ephemeral --model "$CODEX_MODEL" --cd "$REVIEW/client" \
  --output-last-message "$REVIEW/candidate.sh" - < "$REVIEW/prompt.txt"
# Inspect the complete script and its digest; no code runs during reception.
python3 -B examples/weaver-log/weaver.py receive --output "$REVIEW" --source-sha256 CANDIDATE_SHA256
python3 -B examples/weaver-log/weaver.py standalone --output "$REVIEW"
```

An editor, another native harness or a human can supply the same DOMAIN.md and
candidate.sh. The prepared prompt injects the selected domain knowledge; the
harness does not become the owner of the domain rules. Native configuration,
plugins and external-service disclosure remain operator responsibilities.
Read-only shell policy does not establish tool-free execution or prevent every
external service effect. The prompt requests no tools but is not enforcement.

Review binds the selected materials, specification, checks and snapshotted Loom
binary. Approval also binds the prepared coding prompt and exact Loom task.
Changing those bytes requires a new review/task. Candidate reception binds the
operator-reviewed digest and permits only kit/transform.sh; Loom separately
checks the current project baseline, revocation, regular-file limits and paths.
Candidate bytes remain untrusted regardless of who produced them. Reception does
not attest model completion, authorship or provider policy. A native client's
success message cannot replace candidate checks or host acceptance.

Keep native request/results private and inspect uncertain runs with that client
before any explicit new submission. This adapter never submits or retries a
model request, reads native session stores, or creates a second conversation.
The native harness owns its session; the host owns managed execution and custody.

## Independent and managed checks

The standalone stage freezes the received candidate, runs the project-owned
checks with operator-native permissions, verifies the snapshot and exports it.
It then runs the exported spool through its existing CLI/environment contract.
Timeouts retain evidence and never trigger automatic retry. Native process exit
is not proof of remote or detached process termination.

To activate under an-agent, use the existing separate handoff:

```sh
an-loom develop-native REVIEW/task /path/to/an-agent-host NEW_CHECK_JOB NEW_DELIVERY
# Inspect and approve that exact plan with --approve.
an-agent-host spool-start NEW_DELIVERY/instance demo INPUT 15
# Inspect and approve the start, then spool-accept for formal artifact custody.
```

The host rechecks the candidate under its own boundary. Standalone success cannot
satisfy receive-check/pack-task. Raw prompts, responses and machine paths stay
local; only separately reviewed portable summaries belong in the repository.

## Migration and evidence

The former `draft`, `generate` and `--kimi` commands are removed. Use `prepare`
and invoke your chosen native harness directly. New review records use v2;
old directories are refused rather than reinterpreted. Finish an old directory
with its original pinned adapter, or prepare a fresh one. The separate old
[kimi-native launcher](../kimi-native/README.md) is also retired; its historical
evidence and fixed reproduction source remain available.

The [historical Kimi summary](acceptance/macos.json) and [reviewed source](reviewed-example/DOMAIN.md)
remain unchanged. They describe the former adapter, not a Codex or v2 acceptance.
The [Codex v2 summary](acceptance/codex.json) records two actual native CLI turns
and the same candidate passing independent checks, managed checks and formal
runtime artifact acceptance. The assistant selected the existing reviewed domain
specification after inspecting the generated draft; no new human approval or
new domain decision is claimed. Both turns emitted no tool calls. This is not
qualification of arbitrary client tools, cancellation or remote termination.
CI exercises the current review gates and qualify_fixture.py with externally
supplied fixture bytes. It rejects protected paths and proves standalone success
cannot satisfy host custody. No installed model client or credentials are needed.
Linux standalone checks do not qualify the managed macOS runtime.
