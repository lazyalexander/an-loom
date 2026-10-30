# Reviewed domain preparation with an optional Weaver adapter

This example connects public log materials, a Kimi-drafted specification, explicit
operator review, a bounded coding task and independent spool execution. It reuses
the existing Kimi 2.3.1 tool-free profile and candidate adapter; its provider-native
context and model loop stay in Kimi. It does not add a Weaver loop to Loom or core.
Another adapter can implement the same selected-material/review/candidate file
contracts. No Kimi configuration or credential is packaged.

```sh
python3 -B examples/weaver-log/weaver.py draft --loom /path/to/an-loom \
  --kimi /path/to/kimi --output /new/review-directory
# Read DOMAIN.md, materials.md, acceptance.sh and review-plan.json.
# Resolve questions by revising the draft and selected materials, then refresh:
python3 -B examples/weaver-log/weaver.py review --output /new/review-directory
python3 -B examples/weaver-log/weaver.py approve --output /new/review-directory --approve REVIEW_SHA256
python3 -B examples/weaver-log/weaver.py generate --output /new/review-directory
# Inspect candidate.sh before receiving or executing its bytes.
python3 -B examples/weaver-log/weaver.py receive --output /new/review-directory --source-sha256 CANDIDATE_SHA256
python3 -B examples/weaver-log/weaver.py standalone --output /new/review-directory
```

Drafting sends only the selected public materials to Kimi K3. It generates no
implementation and runs no project command. Review binds exact source, DOMAIN.md,
acceptance script and selected tool/profile identities. Changing selected bytes
invalidates review. Approval is an operator-owned local record, not authenticated
human identity, publisher certification or an OS boundary. The example does not
read or monitor any private coding sessions.

The coding task protects specification, materials, checks, silks and manifests;
only kit/transform.sh can be submitted. Generation verifies the review and bound
prompt/plan, then delegates to the existing Kimi adapter. It has no exposed tools
or subagents. No submission is automatically retried after failure or timeout;
a remote outcome may remain unknown. Keep all private evidence and inspect it
before authorizing a separate new task. Do not treat this trusted driver or tool
allowlist as hostile-client isolation.

## Zero-mount iteration

The standalone stage requires no an-agent binary or connection. It freezes the
candidate using check-task, executes project-owned checks with native operator
permissions, verifies the snapshot again and exports the exact frozen archive.
It then runs the exported silks/kit directly through their existing CLI/environment
contract. A successful offline report is not a host artifact receipt. Local waits
are bounded, but timeout/client exit does not prove descendant termination; files
remain for inspection and no destructive cleanup or automatic retry occurs.

To activate under an-agent, use the existing separate contract:

```sh
an-loom develop-native REVIEW/task /path/to/an-agent-host NEW_CHECK_JOB NEW_DELIVERY
# Review and approve that plan using --approve; standalone success is not approval.
an-agent-host spool-start NEW_DELIVERY/instance demo INPUT 15
# Review and approve execution separately, then use spool-accept for formal custody.
```

The host rechecks the same candidate under its own boundary before delivery.
Independent development never bypasses receive-check/pack-task evidence checks.
Raw materials, model output, paths and request records stay in the local review
directory. Publish only separately reviewed portable summaries.

## Qualification evidence

The portable [macOS summary](acceptance/macos.json) records two actual Kimi K3
calls: a draft, then an implementation after explicit developer confirmation.
The [reviewed example](reviewed-example/DOMAIN.md) preserves the selected bytes;
its pending-review wording was not rewritten after approval. Approval is a separate
local record and does not authenticate a person or prove domain truth. The
standalone and managed routes used identical checked archive bytes and produced
identical output; host activation and acceptance were approved separately.

CI runs credential-free review-gate tests and qualify_fixture.py with a synthetic
specification/candidate. It rejects edits to DOMAIN.md, acceptance.sh and silk,
and proves standalone success cannot satisfy pack-task's host evidence gate.
That is not a live model or human-review claim. Raw prompts, responses, client
configuration and session output remain private. Provider usage is unavailable.

An initial tool-identity check rejected a rebuilt Loom binary before dispatch.
The exact original binary was restored and retained; specification approval was
not changed. New drafts snapshot the Loom binary to avoid build-directory drift.
A managed start preview also rejected a noncanonical /tmp path before allocating
a run; only the remaining start was issued with the canonical path. No unknown
execution or completed development task was resubmitted.

The standalone fixture runs on both CI operating systems using bash/awk. Its
success on Linux does not qualify an-agent's managed Linux runtime: the exported
native manifest still requests the existing macOS profile.

The reviewed example includes the selected Kimi-produced kit and the portable
review-plan.json (digests only). Recompute the latter's SHA-256 to check the
confirmed review identity. CI reuses those recorded code bytes with a synthetic
review/generator; it never claims the test itself contacted a provider.
