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
