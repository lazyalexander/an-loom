# Native spool packaging and host handoff

This qualified adapter exports reviewed shell spools and explicit v3 Python
requests with binary products. See the [platform matrix](platforms.md) for current
export and real-host coverage. It does not implement
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
`runtime` must be `macos-native-process-v1`, or `macos-native-python-v1` with the
explicit v3 interface described below. Other runtime labels are refused.

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

The host's optional [native state contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-state.md)
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
[native control contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-spool.md#native-control-at-round-boundaries).

### Private snapshot retention

The host's `spool-storage INSTANCE` observes logical bytes and available headroom.
After explicit work collection, `spool-reclaim-snapshots INSTANCE RUN` can preview
and collect private input bodies under the existing evidence/approval boundary.
Formal artifacts, checkpoint objects and execution/control records stay retained.
The paired v1/v2 qualification collects input snapshots before state commit and
then continues from the independent checkpoint. No Loom retention engine is added.

Consumers use fixed artifact checkout or explicit state selection, rather than
private snapshot paths. This does not enable general artifact GC or implicit
discovery of references inside opaque business state. Native preparation adds
filesystem headroom checks; these are admission pressure, not runtime quotas.
See [the host storage contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-storage.md).

## Acceptance and next ownership

### Release review and independent continuation

Export old and candidate archives separately, then use the host's read-only
`spool-release-diff OLD_DELIVERY CANDIDATE_DELIVERY`. It independently verifies
both payloads and shows exact code/contract/request differences, including
unsupported requests. Same names or version labels establish no update authority.
Neither export nor comparison grants execution or authenticates a successor.

For the qualified path, load the candidate into a fresh instance. If the domain
contract permits it, explicitly checkout a fixed old checkpoint and initialize
the new history from that file, then separately approve its execution. Old state
and products remain in the original instance. This is independent continuation,
not in-place upgrade, automatic migration or state-schema compatibility proof.
Edited working kits need separate review; comparison uses release baselines.

The real `tests/cli/native_release.py` packages two revisions through ordinary
check/pack/verify and both v1/v2 exporters, compares them and executes independent
state histories while retaining the old pins. Host tests own unsupported-request,
stale-approval and failed-candidate refusal. No new Loom release format, updater,
dependency resolver or runtime authority is introduced. See
[the host release contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-state.md#native-release-review-and-independent-continuation).

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

The real native acceptance now exercises this sequence using the actual release
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

## Explicit v3 event interfaces

Add the following table to the complete native authoring file and use
`export-native-spool-v3 ARCHIVE NEW_DIRECTORY`:

```toml
[events]
consumes = ["artifact.accepted"]
produces = ["artifact.accepted"]
```

Each array may be empty or contain that one supported name. Unknown names,
duplicates and unknown fields refuse. V3 retains v2 requirements and binds the
interface into the release digest. V1/v2 without event authoring remain unchanged;
exporting an event-aware project as v1/v2 refuses rather than dropping its contract.
Older hosts reject v3. The existing `develop-native` convenience route still
exports v2 and does not qualify event-aware projects; use explicit packaging and
v3 export. This does not widen that route's build authority or dependency support.

These declarations are interfaces, not subscriptions or execution grants. The
host independently authorizes static workspace routes, verifies accepted artifact
facts and admits delivery against exact versions. Receipt and output confirmation
remain host-owned. No routing service, runtime dependency, business predicate,
loop or global state is added to Loom. Domain workflows stay in silk/kit.

The paired `tests/cli/native_events.py` packages the existing two-silk reference,
loads two actual v3 instances, publishes one accepted artifact event, approves one
delivery, accepts and confirms its result, and verifies fixed products plus
duplicate confirmation after scratch/snapshot collection. This is explicit
foreground dispatch, not a background event bus or model integration. Host tests
own cancellation, revocation and missing-evidence cases. See
[the host event contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-events.md).

## Native Python and byte products

V3 authoring may declare `binary_outputs`, a unique subset of `outputs`.
Omission preserves text behavior. V1/v2 exports reject these declarations instead
of dropping them; existing source and export bytes remain unchanged. A binary
file is an opaque product to the host, not a request to parse an Office document.
The existing byte-artifact limits apply; the consumer owns format/domain checks.

`runtime = "macos-native-python-v1"` requests the qualified CPU/headless native
Python profile with a complete v3 event interface. Dependency labels remain
portable data. Loom emits no local interpreter path, installs nothing and grants
no runtime authority. A host operator separately selects an installed environment
with matching labels, interpreter digest and complete site-package inventory:

```text
an-agent-host spool-load DELIVERY NEW_INSTANCE --runtime LOCAL_RUNTIME_JSON
```

The host verifies that binding before load/start and artifact acceptance. Python
runs with site startup and bytecode writes disabled, selected read-only dependency
roots and attempt-private code/data snapshots. The declared input slot supplies
its snapshot filename; `source.xlsx` need not inherit the catalog's `content.bin`
name. Input/code basename conflicts refuse. Runtime selection is not publisher
authentication, permission to install packages or a hostile-harness sandbox.
The system standard library remains an operator-owned dependency. See
[the host contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-spool.md#operator-selected-native-python-and-binary-products)
for the strict JSON, inventory algorithm and limits.

The [log-report example](../examples/log-report/README.md#native-spool-event-handoff)
prepares two projects by copying its existing independent document adapters.
Loom checks/archives/exports them without domain branches; the host executes each
as a separate spool and delivers the exact accepted XLSX revision by event. The
consumer creates a three-page PPTX with native editable text/table objects. The
paired `tests/cli/native_documents.py` confirms old-version consumption after a
newer upstream result and retained products after collection. It performs no GUI
edit or model call. Host tests own cancellation/runtime-mutation refusal cases.

Use explicit `check`, `pack`, `verify` and `export-native-spool-v3` for this path.
The existing `develop-native` convenience route still targets v2 shell checking;
it is not qualified to provision Python dependencies or deliver v3 projects.

## Separate business data and checkpoint inputs

V3 authoring may add `state_input = "checkpoint.txt"`, separate from the one
business `inputs` slot and all code basenames. Loom preserves the name in host
requirements and coding-task briefs; it protects the selected authoring file
under the existing task contract. V1/v2 export cannot drop this requirement.
Existing declarations without the field retain their bytes and semantics.

The host independently binds a fixed data artifact plus an explicitly selected
committed state through `spool-start-data-state`, or adds `--state REVISION` to
an artifact-event delivery. `AN_SPOOL_INPUT` and `AN_SPOOL_STATE` name separate
read-only snapshots. No HEAD lookup, state merge, domain interpretation or retry
is added to Loom. Starts, acceptance, checkpoint commit and event completion each
remain independently approved host operations. Stale state cannot overwrite HEAD;
accepted reports survive even when their proposed state conflicts.

The [log accumulator](../examples/log-accumulator/README.md) reuses the existing
summary transform and adds domain-owned cumulative counts. Its standalone tests
need only Bash, awk and Python's standard library. The paired CLI qualification
checks actual v3 packaging, direct/event input roles, a stale commit, and fixed
outputs/state after private collection and unload. Host acceptance owns actual
cancellation, changed selection and missing-evidence cases. See the
[host state contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-state.md#independent-business-data-and-state)
for strict selectors, frozen previews and delivery identity rules.

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

## Host-owned event inventory

The host provides `spool-events-pending WORKSPACE` and foreground
`spool-events-watch WORKSPACE SECONDS INTERVAL_MS`. Inventory associates matching
fixed events and subscriptions with admission refusals and evidence-derived
delivery phases. Watch emits initial/changed NDJSON snapshots and a final stop
summary; it does not promise every intermediate transition. Neither command
prepares a run, selects state, acknowledges consumption or grants execution.
Stopping observation does not cancel an independently admitted delivery.

Loom continues to declare/package interfaces only. Runtime observation, polling,
subscription admission and execution remain in an-agent. Do not add a second
scheduler in Loom or treat a coding-task instruction as runtime authority.
The paired native event acceptance observes pending, acceptance, confirmation
and consumed phases for an actual Loom-built release. Host acceptance owns
monitor signals, deadline and evidence-loss cases. This P1 slice does not add
automatic dispatch, latest-state selection or broad batch authorization.

## Finite host dispatch with fixed state

The host's `spool-events-dispatch WORKSPACE REQUEST_JSON [--approve HASH]` accepts
1..16 explicitly selected existing event/subscription pairs, each with an exact
`state_revision` where required. Preview and hash approval bind every delivery's
start plan; no latest state, wildcard or future-event authorization is inferred.
SIGTERM drains the selected entry and stops further selection; SIGINT requests
its supervised cancellation. A durable intent fences the entire batch against
replay/resume, including after missing results or ledger failure. Observation is
available through `spool-events-dispatch-show`; repeating a recorded request never
runs the remainder. Products/state acceptance and completion remain explicit.

Loom adds no runtime scheduler, signal handler or state policy. The existing
log-accumulator paired acceptance packages the same v3 contract, submits a fixed
stateful event through the host dispatcher, independently accepts/commits/completes
it and checks pinned earlier products plus no repeated attempt after collection.
Host acceptance owns multi-entry stop/cancel, real ledger faults and unknown-result
fences. Existing standalone and direct-delivery paths remain supported.

## Host subscription admission control

`spool-event-pause` and `spool-event-resume` are explicitly approved host
operations on a fixed subscription. Pause blocks new admission, without cancelling
running work or blocking already-admitted acceptance/commit/completion. Resume
requires fresh delivery/batch approval and the same target instance/generation;
it does not replay a recorded batch or undo permanent revocation.

The host preserves route/delivery identity and records predecessor-bound control
generations. First use upgrades workspace metadata to v2 so earlier hosts refuse
new opens; existing v1 workspaces without control remain supported. Mixed old/new
host processes on one upgraded workspace are not qualified. This does not change
Loom release formats or allow a project declaration to authorize runtime actions.
The paired log-spool acceptance checks pause/resume, stale batch approval refusal,
unchanged delivery identity, custody while paused and retained products/checkpoint.
Host suites own concurrent admission, history faults and old-host qualification.

## Host dispatch summary reconstruction

The host's explicitly approved `spool-events-dispatch-reconcile WORKSPACE NAME`
can reconstruct a missing completed-batch summary only from all original entry
records, exact delivery/start bindings and confirmed successful native receipts.
It never runs a process, resumes a partial batch, accepts output or commits state.
Missing or contradictory evidence refuses. Loom's paired log-spool smoke loses
only the known completed batch projection after custody/collection, reconstructs
it and verifies that the attempt count remains one. This is not arbitrary crash
recovery. Runtime reconciliation remains host-owned.

## Host receiver succession

`spool-event-replace WORKSPACE OLD NEW ROUTE_JSON [--approve HASH]` reuses the
existing strict route request format and replaces future receiver admission only.
The producer/event slot stays fixed, the successor uses a fresh name, and the
old route and all admitted delivery identities remain retained. Replacement does
not run a process, cancel old work or migrate state. An immutable catalog commit
retires old admission and binds a fixed log-prefix cutover; earlier events cannot
be replayed through the successor. New events still need explicit execution
approval and a selected checkpoint. Old pending/batch approvals cannot retarget
work to the new instance, and resuming old admission is refused.

First use upgrades host workspace metadata to v3; old hosts refuse new opens.
Prior v1/v2 workspaces remain supported and v3 preserves existing control history.
Keep old instance directories for historical custody; do not mix already-running
old/new hosts on an upgraded workspace. This is not automatic release upgrade,
state migration, route-history GC or arbitrary topology editing. Existing bounds
still count retired declarations. An unrelated separately approved route remains
a distinct authority, not a promise of global exactly-once behavior.

The packaged log-spool smoke loads a second instance of the same exact release,
explicitly seeds it from a pinned checkpoint, refuses historical delivery through
the successor, then processes a new event through finite dispatch and verifies
independent state/products. Loom declares/checks/packages the existing interface;
cutover, authority, supervision and custody stay in an-agent. The host suite owns
in-flight replacement, stale approval, uncertain-result and old-host refusal cases.

## Finite private-material maintenance

The host's `spool-storage-collect INSTANCE REQUEST_JSON [--approve HASH]` groups
1..16 selected runs for either work or private-input-snapshot collection. Every
existing per-run preview is validated before deletion and bound by the aggregate
approval. Original operations retain their locks, exact inventories and durable
one-shot deletion fences. Work and snapshots are separate explicitly approved
phases. Cancel/unknown results stop further items; this is not an atomic batch,
auto-resume or rollback service. Formal artifacts, state, release/kit and evidence
stay retained. Loom adds no runtime collector or retention authority.

The packaged log-spool acceptance collects two accepted runs through each phase,
then verifies explicit state commit, stale-state refusal and fixed downstream
products as before. The host suite owns larger multi-round inputs, concurrent
consumer progress, cancellation and lost collection receipt fences. Native formal
payload deletion still requires a host-owned reference/release contract; Loom
declarations or absence of observed readers cannot authorize it.

## Host artifact input admission retirement

`spool-artifact-retire INSTANCE RUN OUTPUT [--approve HASH]` closes one verified
accepted native output to new revision-based inputs without deleting payloads or
changing their identities. New checkout/edit-parent selection, native business
input/event delivery and workspace artifact preparation honor the host fence.
Old direct/event/batch previews cannot authorize a new acquisition after closure.
Other outputs and new revisions are independent. Already-acquired inputs,
output acceptance, state commit, causal confirmation and private collection can
still verify retained evidence. A committed checkpoint is an independent version.

Closure writes use the catalog's existing lock and exclusive creation. Immutable
input reads check the monotonic fence before and after acquisition, preserving
concurrent readers. Already-acquired bytes can finish; a concurrent closure may
refuse preparation without authorizing a retry.
A torn/invalid record fails closed and is not overwritten as a retry. It is not
access revocation for raw files, independent copies or an older host executable;
use the qualified host consistently. There is no complete reference census,
automatic GC or deletion grant. Native release schemas and old package bytes do
not change. See [the host storage contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-storage.md#retiring-new-artifact-input-admission).

The packaged log-spool smoke closes one old producer output, refuses a new
stateful input selection, and preserves existing consumption proof, independent
state and saved reports. Host tests own concurrent retirement, cancellation,
concurrent-reader behavior and marker faults. Loom adds no runtime retirement policy.

### Native input reference handoff

The host now registers exact managed artifact/checkpoint inputs when admitting a
new native run. `spool-input-references INSTANCE RUN` reports the run-bound claims;
preview and raw-file selection do not acquire them. The packaged log-spool smoke
checks that its data/checkpoint revisions and release match the claims, which
remain after private collection, state commit and unload. No release-format or
manifest change is needed, and Loom owns no runtime reference store.

The host's `spool-input-release INSTANCE RUN [--approve HASH]` preserves the
confirmed cancelled-owner contract after work and snapshot collection. The added
`spool-input-release INSTANCE RUN --delivery WORKSPACE SUBSCRIPTION [--approve HASH]`
supports successful stateless event consumers after accepted outputs, original
consumption acknowledgement and private collection. The paired event case checks
explicit release, unchanged formal products and duplicate confirmation without
execution. Successful stateful owners and incomplete registrations stay retained;
opaque state dependencies are not inferred from content. Host CI owns cancellation,
partial registration and lost-confirmation qualification. Source claims are not a
complete census of old/unmanaged readers or opaque checkpoint dependencies. The
operation deletes no payload and cannot authorize collection by itself.

The remaining [retention/session handoff](session-retention-handoff.md) is a plan.
Stateful-success release and formal payload collection remain unsupported. The
packaged event smoke now runs in an explicit host session/finite thread and closes
the session before ordinary product acceptance; it introduces no release fields. Unknown outcomes are not retried. Loom
adds no release policy, package fields or runtime store for this host capability.

### Observation and control concurrency

The qualified host pending/show/watch path reads a bounded fixed log without taking
its workspace mutation lock or repairing workspace projections. Its additive
`observation` metadata identifies the log and reports whether it changed by the
end; external evidence remains non-atomic. Loom must not turn an observed phase
into execution, cancellation confirmation or collection authority. Mutating host
commands still revalidate under their original locks. Malformed captured evidence
fails closed; unknown work is never retried. Host CI owns the concurrent observer/
revocation matrix; Loom retains its narrow packaged delivery/state smoke tests.
