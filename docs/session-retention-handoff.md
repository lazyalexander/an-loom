# Session and retention development handoff

Status: native input-reference handoff qualified; the remaining retention/session
work is a coordinated plan. No manifest, runtime grant or package format changes
are required for the initial handoff. GUI and TUI are deferred. Local headless
execution remains the qualification target.

The host's [session and retention plan](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/session-and-retention.md)
is authoritative for the proposed runtime work. It separates formal artifact
reference release/collection from the subsequent minimal session/thread layer.
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

A future host session groups a bounded engagement; a logical thread scopes context
and cancellation. Neither replaces release/instance/attempt identity or runtime
authorization. Existing coding agents retain their own sessions, compaction and
tape. Weaver/harness adapters exchange explicitly selected material and candidate
changes; they must not scrape private histories or silently share conversations.
Native harness IDs stay private and cannot grant access to a workspace.

## Phased compatibility work

1. The packaged log case now checks host-owned data/state claims against exact
   revisions and the exported release, retaining them through collection and unload.
   Only cancelled-owner release is currently qualified by the host. Successful
   owners, opaque state dependencies and legacy/unmanaged readers remain retained.
   Host tests own interruption, races and corruption; Loom does not infer deletion
   authority from a reference observation.
2. After the host qualifies formal collection, verify that a declared retained
   dependency blocks it and an explicitly released version becomes unavailable
   without changing unrelated products or package bytes. Missing content must
   not silently select another version or rerun a command.
3. When host session/thread commands exist, associate packaged execution with
   explicit host identities. Verify one session's cancellation leaves another
   independent session alone. Do not add Loom-side retry or lifecycle policy.
4. Reuse explicit context packets and bounded receipt-linked export. Verify the
   intended audience and source identity, record bytes and model-call count, and
   distinguish fixture evidence from live provider usage. Unknown usage is not
   zero; documentation must not promise measured token savings without evidence.

Keep these follow-ups tied to qualified host revisions, not speculative command
names. Reuse existing Python paired CLI acceptance and build once per selected
binary. The host remains independently runnable without a Loom runtime dependency.
Do not replace domain acceptance with a framework confidence score.
