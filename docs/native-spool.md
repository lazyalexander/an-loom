# Native spool packaging and host handoff

This qualified adapter exports a reviewed macOS shell spool. It does not implement
runtime authorization, supervision, cleanup or a domain pipeline engine. The
legacy file-kit deployment adapter and its wire names are unchanged.

## Authoring

Use a generic `loom.toml` package with an explicit include allowlist. Include
`native-spool.toml` and every selected code file. The authoring file has
`schema = "an-native-spool-v1"`, `runtime`, `entry`, `silk`, `kit`, `editable`,
`inputs`, `outputs`, `capabilities`, `dependencies` and `acceptance`.
`loom.toml` `description` carries the domain summary. See
`examples/native-spool/` for two domain silk steps and one transform kit, and
for `an-loom init-native`, `check`, `pack` and `verify`.

An entry is one declared silk script. Paths are one role directory plus a unique
ASCII basename; there are at most 12 files, 1 MiB each and 8 MiB total. Input
and output slots are bounded identifiers. `editable` is a subset of `kit`.
Capability names and acceptance commands are records, not grants and not
processes. Dependency values are unresolved labels: no URL, path or version
range is fetched. `check`, `pack` and `verify` refuse a missing entry, a path
listed in both roles, an escaping path, and a `silk/` or `kit/` file that is
not listed. They do not spawn acceptance commands or download dependencies.
`runtime` must be `macos-native-process-v1`; other runtime labels are refused.

```text
an-loom pack PROJECT
an-loom export-native-spool ARCHIVE NEW_DELIVERY
```

The exporter uses existing archive inventory, allowlist, secret and digest checks.
It emits only selected silk/kit bytes and `native-spool.json`; README, source build
files and authoring metadata are not copied into the execution package. The host
JSON keeps `schema`, `name`, `version`, `entry`, `files` and `outputs`. Runtime,
inputs, editable paths, capability requests, dependency labels and acceptance
records stay in the source package. Name and version come from the verified
package identity. The exact emitted manifest bytes and selected file hashes
determine the release digest. Existing output directories are refused. The host
parses and verifies this contract independently.

The export result records the source bundle digest. It grants no runtime rights,
proves no publisher identity and does not resolve version ranges. This is an
explicit new profile, not a reinterpretation of the legacy `[kit]`/`[silk]` adapter.

## Legacy compatibility

Historical native v1 files with only `schema`, `entry`, `silk`, `kit` and
`outputs` remain valid for packaging, verification and export. No authoring
metadata is fabricated and existing release identities are preserved. Coding
tasks require explicit complete authoring metadata. Partial additions are
rejected rather than silently treated as a legacy contract. Candidate check
plans and evidence have a separate versioned contract described in
[candidate check custody](candidate-checks.md).

## Host usage

Use an an-agent host with the `an-native-spool-v1` contract:

```text
an-agent-host spool-load DELIVERY NEW_INSTANCE
an-agent-host spool-show INSTANCE
an-agent-host spool-kit-update INSTANCE transform.sh EDITED_FILE
an-agent-host spool-kit-update INSTANCE transform.sh EDITED_FILE --approve DIGEST
an-agent-host spool-start INSTANCE run-1 INPUT_FILE 30
an-agent-host spool-start INSTANCE run-1 INPUT_FILE 30 --approve DIGEST
an-agent-host spool-accept INSTANCE run-1
an-agent-host spool-accept INSTANCE run-1 --approve DIGEST
an-agent-host spool-reclaim INSTANCE run-1
an-agent-host spool-reclaim INSTANCE run-1 --approve DIGEST
an-agent-host spool-unload INSTANCE
an-agent-host spool-unload INSTANCE --approve DIGEST
```

Each digest belongs to its immediately preceding preview, not another command or
instance. SIGINT/SIGTERM to the running host CLI request cancellation. The host
owns effective policy, selected-code snapshots, termination and custody. A fresh
load to another directory restores the kit baseline while retaining prior products.
Reclaim affects work scratch; release, input evidence and formal products are not
implicitly deleted. Outputs remain in the per-run host artifact catalog.

Kit edits are allowed between runs. Runtime code snapshots are read-only. Domain
scripts use `AN_SPOOL_SILK_DIR`, `AN_SPOOL_KIT_DIR` and `AN_SPOOL_INPUT`; their current
directory is writable scratch. This is not live agent self-modification or strong
containment for malicious native code. Live process pause/suspension, weaver execution, dependency
resolution and automatic retention are not implemented by this adapter.

## Explicit state handoff

The host's optional [native state contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/crates/an-agent-host/NATIVE-STATE.md)
works with existing v1/v2 exports. An author defines the opaque state format and
its validation in spool code, selects an existing text output as next state, and
uses the one declared input for the selected state document. No extra Loom state
schema, state interpreter or execution supervisor is introduced.

Operators initialize state through `spool-state-init INSTANCE OUTPUT FILE`, select
an exact committed revision through `spool-start-state INSTANCE RUN REVISION SECONDS`,
formally accept that run, then explicitly `spool-state-commit INSTANCE EXPECTED RUN`.
Each mutation needs its own host preview/approval. Ordinary `spool-start` remains
unchanged and cannot promote its output through the state-commit route merely
because its input bytes happen to match a checkpoint.

The host refuses stale, unsuccessful, cancelled or unaccepted state candidates.
Scratch collection preserves committed state and accepted artifacts. New runs do
not inherit scratch; a fresh load may be explicitly seeded from a checked-out
fixed state, creating a new history without implied migration or lineage. State
meaning, business acceptance and cross-release compatibility remain author-owned.

The [narrow CLI qualification](../tests/cli/README.md) packages the existing text
reference and verifies both export formats through two state-bound host runs,
reclaim-before-commit and post-unload inspection. Host tests own the full failure
matrix. This is protocol evidence, not a new domain workflow or generic Loom
state-management capability.

### Round-boundary host control

The qualified host also exposes `spool-pause`, `spool-suspend INSTANCE REVISION`
and `spool-resume`, with separate preview/approval for every transition. Pause
fences new starts, state mutations and kit edits only after all started attempts
have confirmed settlement and successful outputs have formal acceptance. Suspend
additionally pins the current committed checkpoint and working-code identity.
Inspection and existing guarded artifact reclamation remain available.

Resume verifies retained bindings and reopens admission without executing or
injecting state. The next round explicitly selects its state input and obtains
a new approval. The host's control generation rejects pre-pause execution and
edit proposals. The paired v1/v2 CLI qualification exercises this path through
the existing source package; Loom adds no model loop or lifecycle supervisor.

Activation upgrades the host's private instance record so older hosts refuse it
instead of ignoring control. Package identity and v1/v2 export formats do not
change. The safe point is a completed round; in-flight signaling, process/VM
images and automatic restart remain unsupported. See the host's
[native control contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/crates/an-agent-host/NATIVE-CONTROL.md).

## Acceptance and next ownership

`cargo test --test native_spool --locked` checks deterministic exclusive export and
invalid role/output bindings. The ignored real host test requires a qualified
`AN_AGENT_HOST_BIN`; it exercises actual CLI load, kit edit, native execution,
acceptance, reclaim, unload, baseline reload and SIGTERM cancellation. No fake host substitutes for that
claim. CI pins the host implementation separately from legacy acceptance profiles.
Optional `LOOM_NATIVE_EVIDENCE` selects a private JSON evidence file; it can contain
local paths and must not be committed publicly.

Loom may later prepare bounded weaver edit tasks and resolve declared dependencies.
The host must still grant and enforce execution, checkpoint, artifact and shuttle
access. Supporting a code tool must not move its model loop into Loom or grant it
unrestricted host CLI access.


## Optional weaver and the two-instance acceptance path

The host provides a file proposal slot, independently of Loom. Request one kit with
`spool-weaver-request INSTANCE transform.sh`; forward only the returned data to an
external CLI adapter or an authorized shuttle. Import its proposal with
`spool-weaver-apply INSTANCE RESPONSE_JSON`, inspect the returned plan, then repeat
with `--approve HASH`. `spool-weaver-revoke INSTANCE REQUEST_UUID` prevents late
admission, but does not assert that an external tool has stopped. A proposal grants
no execution authority. Never give an untrusted agent the unrestricted host CLI.

The response schema is `an-weaver-proposal-v1` with `request` (the request UUID),
`request_sha256`, `kit` (the selected `kit/BASENAME`), and UTF-8 `content`. The host
binds it to the instance, release, original kit and run/edit generation. Changed,
revoked, duplicate, foreign or protected-path proposals are refused. Starting a run
requires a separate approval after application. The slot launches no model and
keeps model/context behavior outside Loom.

The real macOS acceptance now exercises this sequence using the actual release
exported from `examples/native-spool`, without a fake execution backend:

1. Load two independent instances of one release (two silks and one kit).
2. Import a deterministic file proposal into A, rejecting foreign and silk edits.
3. Run A and accept a versioned report; checkout that exact artifact revision.
4. Publish a newer report from A, refuse an old receipt claiming the new candidate,
   and run B against the earlier checked-out version.
5. Reclaim A's accepted scratch and unload it; load a fresh instance and verify the
   original kit behavior.
6. Hold B in real execution while the fresh instance completes another run; refuse
   B's unload, reclamation and weaver request while it is active.
7. Signal B's CLI, require confirmed cancellation, refuse cancelled output acceptance,
   accept B's successful downstream report before reclaiming it, and unload B.
8. Verify the pinned earlier artifact remains unchanged.

The test's file proposal is explicitly deterministic test input, not an LLM or a
claim of coding-tool security. `LOOM_NATIVE_EVIDENCE` retains identities, receipts,
fixed handoff revision, cancellation and independent execution evidence. This is
an operator validation route: run the ignored integration test with a qualified
`AN_AGENT_HOST_BIN`. Domain code is still packaged independently, and no runtime
Rust dependency connects Loom to an-agent.

## Explicit v2 host delivery

Use `an-loom export-native-spool-v2 ARCHIVE NEW_DIRECTORY` to preserve authoring
requirements in the host release. Ordinary export remains v1 and byte-compatible.
V2 requires explicit authoring metadata; it does not fabricate declarations for
legacy six-field contracts. The emitted native manifest has schema
`an-native-spool-v2` and a required `requirements` object containing runtime,
inputs, editable paths, capabilities and dependencies from authoring.

This qualified exporter maps its existing macOS shell runtime to OS `macos`,
architecture `portable-source`, interaction `headless` and GPU `none`. These are
fixed profile facts, not detected device capabilities or new generic authoring
fields. No arbitrary executable, GUI or GPU profile is claimed. Declaration
labels are requests; the host checks its supported set before admission.
Additional dependency labels are preserved so the host can reject unresolved
requirements rather than silently dropping them. The host independently verifies
v2, rejects unsupported requirements, and binds accepted requirements into the
release identity. Empty editable scope makes packaged kits noneditable.

This is executable runtime admission metadata, separate from source-package
instructions and acceptance commands. It is not a scanner certificate or grant.
See `spool-runtime-handoff.md` for the planned wider contract and future adapters.

The same real host lifecycle acceptance runs against v1 and v2: two independent
instances, bounded kit proposal application, fixed artifact checkout, cancellation
with group settlement, refused collection while active, unload and fresh baseline
reload. V2 does not inherit lifecycle correctness solely from its admission test.
When LOOM_NATIVE_EVIDENCE is set, v2 records a separate `.v2.json` evidence file.

## Runtime artifact discovery

The host's `spool-artifacts INSTANCE RUN` returns verified fixed-version bindings
for formally accepted outputs. Loom declares output slots; runtime discovery,
retention and content/provenance verification remain host responsibilities. Query
results remain available after scratch collection and unload; use the existing
artifact-checkout command to create an explicit downstream input. Source working
copies and scratch filenames are never substitute acceptance evidence.

The host's `spool-status INSTANCE RUN` exposes evidence-derived run phases and
separate execution/termination/artifact observations. Loom's real v1/v2 CLI
acceptance observes running uncertainty, confirmed cancellation, accepted outputs
and reclaimed scratch, including after unload. No PID guess, lock presence or
status response grants permission to execute or reclaim. Runtime monitoring stays
in the host; a common observation vocabulary does not imply identical backend
support for GPU, GUI, pause or suspend. Headless remains the first qualified mode.

For an interrupted instance acceptance handoff, use the host's explicit
`spool-reconcile-accept INSTANCE RUN` preview/approval route. Loom's real CLI test
creates an actual facade-write failure after workspace output publication, checks
unconfirmed custody, reconciles it, and verifies unchanged attempt count, revision
count and workspace acceptance bytes before normal downstream checkout. This is
not a process retry, a second publication or a power-loss durability claim.
Incomplete workspace acceptance and uncertain execution remain unsupported for
recovery. The host owns reconciliation; Loom owns no runtime repair state machine.


The native qualification also exercises spool-kit-reset against its explicitly
pinned host revision: preview is inert, restoring the kit preserves instance
identity, a new run uses release-baseline code, and earlier pinned products remain
readable. This is a host lifecycle capability, not a Loom implementation of kit
mutation or an additional grant. Other historical host jobs retain their own pins.
