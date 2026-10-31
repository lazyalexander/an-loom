# CLI handoff acceptance

`native_state.py` qualifies unchanged v1/v2 exports from the existing native text
reference against an explicitly selected real host. It requires Python 3 standard
library, macOS native enforcement and two already built binaries. It never builds
inside a scenario or invokes a model.

```sh
python3 tests/cli/native_state.py --loom target/debug/an-loom \
  --host /path/to/an-agent-host --host-revision FULL_HOST_COMMIT \
  --output /private/tmp/loom-native-state-NEW
```

The output directory must not exist. The runner checks/archives/exports source,
loads each format, explicitly seeds state, consumes a fixed revision twice through
fresh workspaces, accepts outputs, reclaims scratch before state commit and reads
state after unload. It also pauses admission after the first round, explicitly
resumes before the second, and suspends at a fixed checkpoint after scratch
collection. Resume never launches the next round.
Private input snapshots are explicitly collected before state commit; formal
products and checkpoint continuation remain available. Host tests own the large
input and failure matrix. The reference text transformation is a protocol fixture;
domain state schemas/validation belong to real spool authors.

The host owns conflict, cancellation, interruption and retention enforcement; its
state suite covers that matrix. This narrow Loom suite verifies packaging and
handoff only. All receipts and local observations remain for diagnosis. Unknown
termination fails without retry or recursive cleanup. `commands.local.json`
contains local paths; publish only a reviewed portable summary if needed.

`native_release.py` uses the same prerequisites and arguments, with a fresh output
directory. It checks/archives two actual source revisions, exports both native
formats, asks the host to compare their verified bytes, and explicitly seeds a
fresh independent instance from a fixed checkpoint. The old release/checkpoint
remain unchanged. It neither authorizes a successor nor tests generic migration.
Host tests own refusal/failure cases. CI pins the host implementing this command.

`native_events.py` uses the same prerequisites and arguments. It packages the
existing native reference with explicit v3 event interfaces, loads two instances,
and exercises host-owned publication, static subscription, approved delivery,
formal acceptance and causal consumption confirmation. Fixed products and
duplicate confirmation remain valid after work/private-input collection. This is
an explicit foreground handoff; the host owns revocation/cancellation/fault cases.

`native_documents.py` additionally needs `--python /selected/venv/bin/python`,
with examples/log-report/requirements.txt installed explicitly. It prepares two
projects from the existing document kit, checks/packages/exports v3, selects a
host runtime, generates accepted XLSX versions and delivers the older exact event
to create a PPTX. Native editable objects and retained products are verified.
No GUI, model, automatic dependency installation or unknown-result retry occurs.
The native-spool CI job records its portable acceptance summary; local runtime
profiles and command logs remain private.

`native_data_state.py` takes --loom, --host, --host-revision and a new --output
directory. It packages examples/log-accumulator, binds fixed business and state
versions through direct/event execution, rejects a stale commit and verifies
native products/provenance after collection and unload. No extra dependency,
model or GUI is needed. The host's corresponding suite owns real cancellation
and missing-evidence tests; this paired suite owns package/host compatibility.

The native event CLI suite also checks host-owned delivery inventory for an
actual packaged v3 release. It does not implement monitoring or dispatch in Loom.

The data/state CLI smoke also submits a finite explicitly state-bound host
batch for the actual packaged log accumulator. Host suites own its multi-entry
lifecycle, cancellation and ledger-fault guarantees.

The data/state smoke also checks host subscription pause/resume and stale batch
approvals for the actual packaged log spool; Loom owns no runtime control ledger.

The data/state smoke also cuts over to a second instance of the same packaged
release. It checks explicit fixed-checkpoint seeding, historical-event refusal,
new-event dispatch and independent old/new state. Runtime cutover remains host-owned.

The data/state smoke uses finite host private-material maintenance for two runs,
retaining state-commit, stale conflict and downstream-product assertions. Larger
capacity/concurrency/collection-fault acceptance remains host-owned.
