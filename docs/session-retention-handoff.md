# Session and retention development handoff

Status: native input-reference handoff qualified; the remaining retention/session
work is a coordinated plan. A first host-owned foreground session/finite-thread
adapter is now paired with the packaged event case. No manifest, runtime grant or package format changes
are required for the initial handoff. GUI and TUI are deferred. Local headless
execution remains the qualification target.

The host's [session and retention plan](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/session-and-retention.md)
is authoritative for runtime scope. Formal artifact reference release/collection
remains separate from session/thread coordination; closing a session releases no
artifact references.
Existing [native spool](native-spool.md) contracts remain the implemented surface.

## Loom's part

Loom helps authors describe inputs, outputs, explicit checkpoint requirements,
useful bounded context and domain acceptance. It checks and packages selected
bytes and hands execution requests to the host. Retention intent is not deletion
authority; declarations cannot create grants, reference releases or model budgets.
Do not add a runtime session manager, pin store, collector or model loop to Loom.

Domain projects should distinguish output provenance from dependencies that still
require original bytes. State remains opaque: authors must explicitly declare
required artifact references in the eventual qualified checkpoint contract rather
than expecting the host to infer them from text. Do not add speculative manifest
fields before the host contract exists. A human-edited product is a new revision;
a summary is a derived context view, not authoritative business state.

A target runtime session represents one spool-instance activation generation and
its resource ownership. OS threads are execution machinery, not model-context
identities. The existing host `session-*` CLI instead groups finite dispatches;
its legacy `THREAD` parameter must not be reinterpreted as an OS thread or a
continuable conversation. No package schema changes follow from this terminology.
Existing coding agents retain their own sessions, compaction and
tape. Weaver/harness adapters exchange explicitly selected material and candidate
changes; they must not scrape private histories or silently share conversations.
Native harness IDs stay private and cannot grant access to a workspace.

Workspace stores versioned materials and facts, not peer conversations. Projects
must use explicit authorized protocol delivery, even when payloads travel as
files. Private notes must not become implicit commands or system instructions.
Context selection remains domain/harness behavior; the host authorizes access and
model disclosure separately. Sharing one spool environment does not provide
hard isolation between its internal agents. The host's current native profile
protects unselected contents but permits file metadata; it does not promise
existence secrecy. Loom must not claim stronger isolation through a declaration.

The host qualifies inert private notes, cross-instance content denials and a
split-phase artifact request/reply using two real CLI spools. The paired packaged
example submits an accepted output, runs the target, accepts its declared reply,
then runs the requester against that pinned version. Waiting is a host record,
not an occupied process. Replies bind both exact instance/release identities and
their existing round-control generations. Pausing/resuming an instance fences the
old endpoint. No package schema or new runtime authority is implied.

Loom supplies entrypoints and declared outputs; the trusted host authorizes
request, reply and execution separately. Do not generate direct host CLI access,
shared-directory polling or a private workspace mailbox. Live IPC, global waiter
scheduling, harness/model-context delivery, caller-independent supervision and
successful peer input-pin release remain unqualified. The host owns the failure
matrix; Loom checks one real exported release round trip and unchanged workspace
facts. A peer result is formal only after the ordinary host acceptance step.

## Foreground owner loss

The host's real two-worker qualification shows that SIGKILL of the launching
CLI leaves workers running beyond its deadline. Handled SIGTERM retains the
original supervisor, which confirms whole-group termination. After SIGKILL,
late files and externally observed process exit remain insufficient evidence
for acceptance or collection. Removing stale locks from the disposable test
fixture still leaves those operations refused.

Development tools must preserve the original attempt identity and report an
unknown result when that supervisor evidence is missing. They must not rerun a
command, accept a late file, delete scratch materials or remove operation locks
to turn an interrupted interaction into success. Inspection can remain available.

The host now provides an explicit attempt-scoped supervisor for native file
inputs, exact checkpoints and combined data/state through `spool-supervisor`
(`start`, `start-state`, `start-data-state`, `show`, `cancel`), reusing execution
and receipt contracts. Loom supplies package and
request identity and receives the verified outcome. Supervisor startup, process
ownership, cancellation and recovery belong to the host. A new package field,
Loom daemon or model integration is unnecessary for this qualification.

The packaged event acceptance also runs the unmodified release through that
supervised start, checks original termination evidence, accepts its exact output,
and verifies preservation after private collection. Host CI owns client SIGKILL,
continuing deadline/cancellation, supervisor death and uncertain-result refusal.
Existing foreground, event/session dispatch and peer execution retains its qualified
scope. A package declaration cannot opt itself into supervisor authority or
rewrite an unknown receipt.

Explicit checkpoint runs can use `spool-supervisor start-state` with an exact
revision. Loom's native-state acceptance runs exported v1/v2 releases through
this entry and preserves the existing state commit and custody checks. The host
owns client-loss, cancellation, deadline and old-revision-versus-new-HEAD tests.

The packaged log-accumulator acceptance uses `spool-supervisor start-data-state`
for its direct combined runs. The host retains exact business/checkpoint bindings
at approval and revalidates admission before execution. Loom keeps selection JSON
as request material, not a shared mutable runtime control. Existing state-conflict,
accepted-product and reference-retention assertions remain unchanged. Host tests
own client/supervisor loss and the reservation-to-worker mutation window.

## Phased compatibility work

1. The packaged log case now checks host-owned data/state claims against exact
   revisions and the exported release, retaining them through collection and unload.
   The packaged event case also checks explicit stateless-success reference release
   after accepted outputs, original consumption acknowledgement and private collection.
   Cancelled-owner release retains its existing host contract. Successful stateful
   owners, opaque state dependencies and legacy/unmanaged readers remain retained.
   Host tests own interruption, races and corruption; Loom does not infer deletion
   authority from a reference observation.
2. After the host qualifies formal collection, verify that a declared retained
   dependency blocks it and an explicitly released version becomes unavailable
   without changing unrelated products or package bytes. Missing content must
   not silently select another version or rerun a command.
3. The packaged event case now uses `session-create`, digest-approved `session-run`,
   `session-show` and `session-control ... closed`, then accepts and hands off its
   products normally. Its legacy thread record binds one finite dispatch; fixed state
   selections remain in the existing dispatch request. Host CI owns overlapping
   session cancellation, stale generations and missing evidence. Simultaneous
   same-workspace admission remains fail-fast; there is no automatic retry,
   background scheduler or Loom-side lifecycle policy. No package fields change.
4. Reuse explicit context packets and bounded receipt-linked export. Verify the
   intended audience and source identity, record bytes and model-call count, and
   distinguish fixture evidence from live provider usage. Unknown usage is not
   zero; documentation must not promise measured token savings without evidence.

Keep these follow-ups tied to qualified host revisions, not speculative command
names. Reuse existing Python paired CLI acceptance and build once per selected
binary. The host remains independently runnable without a Loom runtime dependency.
Do not replace domain acceptance with a framework confidence score.
