# Candidate check custody

This is a qualified local shell-project route, not a general build executor or
an authenticated remote attestation. The operator owns the task authority and
host ledger. A coding agent receives only its assigned working copy, not either
store and not an unrestricted host CLI.

## Ownership and identities

1. Loom admits one editable kit replacement against the complete project baseline.
2. `check-task` freezes all selected files, modes and the manifest into a candidate
   archive. It creates a fresh UUID check request, a full release digest, an
   archive digest, a runner and a profile. These are different identities.
3. The operator reviews the profile and acceptance commands before granting host
   execution. The host snapshots the full candidate tree read-only, supervises
   the process and records its own attempt UUID, plan and execution digests,
   exit status and process-group termination observation.
4. The operator formally accepts `checks.txt` through the host. The host binds
   that immutable output to the attempt receipt and profile.
5. `receive-check TASK HOST_JOB` validates that chain against the frozen plan.
   `pack-task` repeats the checks and copies the frozen archive byte-for-byte.
   It also writes a separate editable project copy for subsequent development.
   The source project and historical releases remain unchanged.

No second authority or process supervisor is added inside a kit. Loom owns
candidate selection and publication; the host owns authorization, execution,
termination and artifact custody. Domain commands and their correctness belong
to the project author. A test that lies or omits a requirement is still a bad
project test: execution evidence cannot establish domain completeness.

## Operator sequence

Start with a task that has received a candidate and has not been revoked:

```sh
an-loom check-task ./task
# Review authority/check-plan.json, check.sh and host-profile.json.
# The profile contains local paths and must not be published.
an-agent-host workspace-run ./task/authority/host-profile.json ./check-job
an-agent-host workspace-run ./task/authority/host-profile.json ./check-job --approve REVIEWED_DIGEST
# Use the host attempt ID printed by execution, not the Loom check request UUID.
an-agent-host workspace-accept ./check-job HOST_ATTEMPT
an-agent-host workspace-accept ./check-job HOST_ATTEMPT --approve REVIEWED_ACCEPT_DIGEST
an-loom receive-check ./task ./check-job
an-loom pack-task ./task ./checked-project
```

The final command returns `checked-project/checked-release.tar.gz`. This is the
same archive that defined the checked candidate. Normal `pack PROJECT` remains
an ungated packaging operation; it does not claim that project checks ran.

Commands execute with the complete candidate snapshot as the current directory.
`AN_LOOM_WORK` points to writable host scratch. Tests that need generated files
must write there; they cannot overwrite the selected candidate. The fixed runner
runs each acceptance command through `/bin/bash -eu -c`, stops on failure, and
records completion only after every command exits successfully. Domain tests
must themselves propagate failures (for example, enable `pipefail` when needed).
`checks.txt` captures their combined output and completion identity. This is a
check artifact, not a claim that the spool's production outputs were generated.

## Admission and publication gates

- Full baseline, candidate, frozen archive, runner and profile must still match.
- Every selected file is bound to the host snapshot, not just the edited kit.
- Exactly one retained host attempt is allowed. A resumed/superseded or reclaimed
  job cannot be used by this adapter. There is no automatic retry.
- A failed/cancelled exit, unknown termination, required group cleanup, missing
  host approval, mismatched execution mapping or missing formal acceptance fails.
- An older check's successful job cannot satisfy a new check request, even when
  both requests have identical release bytes.
- Accepted check output is verified from its fixed host artifact revision,
  including receipt/profile provenance. A handwritten v1 evidence directory is
  insufficient. Failed preparation leaves evidence for diagnosis; create a new
  task rather than modifying a committed check request.

Keep the source baseline, task authority and retained host job until publication
completes. These are local trusted stores, not a cryptographic defense against a
malicious operator or hostile same-user mutation. Hashes are not signatures.

## Compatibility and qualified scope

Legacy `an-native-spool-v1` contracts containing only schema, entry, silk, kit and
outputs remain checkable, packable, verifiable and exportable. Their bytes are
not rewritten. They need explicit authoring fields before creating a coding task;
Loom does not invent editing scope or acceptance commands. Partial authoring
metadata is rejected. Existing v1 deployment JSON and CLI names are unchanged.
The provisional v1 candidate-check plan/evidence must be regenerated as v2:
accepting it as equivalent would preserve the original defect.

The execution adapter uses the qualified host workspace v2
([platform matrix](platforms.md)) with explicit
`input_layout`, at most 15 selected project files plus one runner, a 30-second
wall limit, no extra runtime read roots and no network. Broader package formats
remain supported independently. This route is for reviewed local tools without
daemonization, not an untrusted code-agent sandbox. Dependencies are not installed,
models are not invoked, and project hooks are not run implicitly.

## Acceptance evidence

`tests/candidate_snapshot.rs` covers full-content versus request identity, stale
silk changes, frozen-byte mutation, revocation and legacy compatibility.
`tests/task_handoff.rs` rejects bare operator success records.
`tests/native_spool.rs::candidate_check_uses_real_host_execution_before_packaging`
uses the actual macOS host and verifies:

- a domain check that succeeds only for the replacement kit;
- an attempted write to the candidate snapshot is denied;
- a syntactically broken candidate fails although the original source is valid;
- exit success without formal output acceptance is insufficient;
- missing termination, cancelled/stale/mismatched receipts and an older job fail;
- actual cancellation settles the group and cannot authorize publication;
- the final archive exactly equals the frozen archive, and later snapshot
  mutation is rejected.

Run with `AN_AGENT_HOST_BIN` selecting the coordinated host build:

```sh
cargo test --test native_spool --locked -- --ignored --nocapture
```

`LOOM_CANDIDATE_EVIDENCE` optionally retains private observation JSON. Candidates
are deterministic test replacements; this acceptance does not claim a real
coding-model integration.
