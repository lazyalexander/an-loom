# Native development to a loaded spool

`develop-native` composes existing candidate and host contracts for a trusted
operator. It does not implement an agent loop or a new execution backend.

```sh
an-loom task ./project ./task
# A person or coding tool submits the assigned replacement.
an-loom receive-task ./task kit/transform.sh ./replacement.sh
an-loom develop-native ./task HOST_CLI ./check-job ./delivery
# Review the printed profile, acceptance commands, steps and selected host hash.
an-loom develop-native ./task HOST_CLI ./check-job ./delivery --approve DIGEST
```

Preview freezes the candidate and prepares its profile, but does not execute
project commands, create the job/delivery or load a spool. Approval covers one
bounded check, conditional formal acceptance of its declared output, evidence
reception, byte-identical packaging, explicit v2 export and nonexecuting host
load. Each host invocation still uses the existing preview/approval contract.
The selected host executable is hashed and rechecked by the existing transport.

The successful result names the checked archive, portable `spool/` directory,
local `instance/`, release identity and host check attempt. The instance is
loaded without execution permission. Inspect and approve `spool-start` separately.
The new entry therefore validates that this machine can admit the delivered v2
profile; ordinary export remains available for packaging without local admission.

The delivery layout is:

- `spool/`: selected immutable v2 payload suitable for delivery.
- `project/`: separate candidate project and the exact checked release archive.
- `instance/`: local host-owned loaded instance; do not distribute it as source.
- `intent.json`, `host-*.json`, `result.json`: local operational evidence, which
  can contain absolute paths. Keep these out of public release materials.

## Single submission and uncertain outcomes

Job and delivery must be fresh sibling/nonoverlapping paths outside source/task.
The adapter closes `authority/develop-native.json` before submission. A submitted
task cannot use the entry again, even with different destinations. Failed checks,
transport errors, post-check packaging/admission failures and interrupted clients
retain available evidence and do not automatically retry or clean up.

Inspect the retained host job using its existing custody/status tools as applicable
(the check job is a workspace job, not a spool instance). Explicit manual
reconciliation can use the existing individual commands when evidence permits;
this entry is not an automatic resume engine. If the client disappears, do not
infer that the host or external work stopped. The existing transport is a trusted
local CLI transport, not a new cancellation supervisor; host execution retains
its own deadline and cancellation behavior. Do not expose this operator entry
as an unrestricted command tool to an untrusted coding agent.

The task admission fence applies to this convenience entry. It is not a filesystem
sandbox against the operator deliberately invoking lower-level commands. Unknown
results require inspection before a new task is authorized.

## Qualification

The supported route remains the reviewed shell profile
([platform matrix](platforms.md)), no network or GPU,
15 project input files plus one runner and a 30-second check deadline. Dependencies
are not installed. Unsupported declarations may be packaged but fail local host
admission, with intermediate results retained. A syntax-only project acceptance
is still only syntax checking; domain tests remain the author's responsibility.

The real native-spool test creates a deterministic replacement, checks a
candidate-specific domain condition, delivers/loads it, separately executes the
instance and checks out its formal output. It also verifies preview and rejected
approval do not submit, known failure prevents release, and prior submission
fences prevent reruns after success/failure or a simulated interrupted intent.
The interrupted-intent fixture does not claim a real process-kill experiment.
No model invocation is claimed. Existing v1/v2 lifecycle tests remain enabled.
