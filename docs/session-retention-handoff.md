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

A host session groups a bounded engagement; a logical thread scopes context
and cancellation. Neither replaces release/instance/attempt identity or runtime
authorization. Existing coding agents retain their own sessions, compaction and
tape. Weaver/harness adapters exchange explicitly selected material and candidate
changes; they must not scrape private histories or silently share conversations.
Native harness IDs stay private and cannot grant access to a workspace.

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
   products normally. A logical thread binds one finite dispatch; fixed state
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
