# Session and retention development handoff

Status: native input-reference handoff qualified; the remaining retention/session
work is a coordinated plan. A host-owned finite cohort/dispatch
adapter is now paired with the packaged event case. No manifest, runtime grant or package format changes
are required for the initial handoff. GUI and TUI are deferred. Local headless
execution remains the qualification target.

The host's [session and retention plan](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/session-and-retention.md)
is authoritative for runtime scope. Formal artifact reference release/collection
remains separate from cohort coordination; closing a cohort releases no
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

A spool activation uses instance identity and round generation for addressing.
Use `cohort create|run|show|control` for a group of finite dispatches, and
`spool-supervisor dispatch WORKSPACE COHORT DISPATCH show` for a member.
The legacy commands and JSON field names remain compatible. Neither cohort nor
dispatch selects a model conversation. A session carries private interaction
history, a thread identifies execution or conversation in its own system,
context is selected material, and the workspace holds versions and facts.
Existing coding agents retain their own sessions, compaction and
tape. Weaver/harness adapters exchange explicitly selected material and candidate
changes; they must not scrape private histories or silently share conversations.
Native harness IDs stay private and cannot grant access to a workspace.

For a bounded peer request, author a small declared UTF-8 output containing
`an-context-v1` with the intended receiving spool's instance UUID. Select that
output using `an-native-peer-context-request-v1`. The host checks the 4096-byte
wire limit and both endpoint generations and requires separate execution approval.
A cohort and its one-shot dispatch control
work. A core agent's private tape, a harness conversation and an OS worker each
have their own identity. Authors must not derive an audience from those names.
The packaged log fixture delivers only the selected first line after sender
scratch collection. Its full output and the workspace fact log remain separate.

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
scheduling, harness/model-context delivery and
successful peer input-pin release remain unqualified. The host owns the failure
matrix; Loom checks one real exported release round trip and unchanged workspace
facts. A peer result is formal only after the ordinary host acceptance step.

## Caller-owned and independently supervised execution

The host's legacy foreground `spool-start` qualification shows that SIGKILL of the launching
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
The native CLI now submits both peer phases and event deliveries to the same
attempt supervisor. A finite host controller owns each approved batch or cohort dispatch. The three-round packaged log case kills the event client, cancels its
original attempt from another CLI, and verifies whole-group settlement. It
separately kills the actual attempt owner and verifies retention with zero
replays. Loom packages ordinary entrypoints and checks host evidence. The
host owns cancellation, deadlines and the decision to admit the next entry.

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
   The host also qualifies successful combined event release under an accepted
   spool policy and approved exhaustive input coverage. Other stateful owners,
   opaque dependencies and legacy/unmanaged readers remain conservatively retained.
   Host tests own interruption, races and corruption; Loom does not infer deletion
   authority from a reference observation.
   For retired outputs the host can cross-check source markers against explicitly
   selected consumer receipts with `spool-artifacts ... --review-retirement`.
   Missing or foreign completions remain uncertain, and checkpoint pins stay held.
   Selected-output inspection (`spool-artifacts ... --output OUTPUT`) verifies the
   original inventory and one payload. A missing sibling payload does not invalidate
   that inspection; missing sibling metadata still does. Whole-run custody remains
   strict. Existing catalogs have unmanaged readers, so zero claims grant no deletion.
   Loom uses the host contract and owns no reader inventory or collection policy.
2. At first acceptance the operator may select the host's registered-copy profile.
   It registers readers from publication. A native event can deliver that exact
   version into a private attempt snapshot alongside an explicitly selected
   checkpoint. The host owns admission, confirmed releases and deletion receipts.
   In the paired three-round log example a checkpoint keeps the first batch,
   confirmed cancellation allows collection of a different output, and losing
   the actual attempt owner leaves its version held. The sample then explicitly retires
   the first enrolled checkpoint and releases its pin by receipt. A stale event
   approval fails and the current checkpoint remains selectable. Legacy catalogs
   stay retained.
3. The packaged event case now uses `cohort create`, digest-approved `cohort run`,
   `cohort show` and `cohort control ... closed`, then accepts and hands off its
   products normally. Its legacy thread record binds one finite dispatch; fixed state
   selections remain in the existing dispatch request. Host CI owns overlapping
   cohort cancellation, stale generations and missing evidence. Simultaneous
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
