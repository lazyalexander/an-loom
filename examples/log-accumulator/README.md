# Explicit-state log accumulator

This dependency-free headless CLI combines a selected log batch with a selected
checkpoint. It reuses the log-summary transform verbatim; `kit/accumulate.sh`
adds four validated counts. Silk only connects the inputs to that domain code.
Neither Loom nor an-agent interprets the count format.

```sh
# From a new work directory; paths must identify the selected source files.
bash /selected/project/kit/accumulate.sh /selected/batch.txt /selected/initial-state.txt
# state.txt and report.txt are new outputs; tmp/ is private work.
```

The state has exactly total/INFO/WARN/ERROR integer counters, each at most one
million on input, with total equal to the other three combined. Unknown levels,
duplicate/missing keys or inconsistent state refuse without publishing outputs.
The bounded business rules and tests belong here, not in the host or Loom.

```sh
python3 -B -m unittest discover -s examples/log-accumulator/tests -v
an-loom check examples/log-accumulator
an-loom pack examples/log-accumulator
an-loom verify RETURNED_ARCHIVE
an-loom export-native-spool-v3 RETURNED_ARCHIVE NEW_DELIVERY
```

The v3 authoring contract declares `inputs = ["batch.log"]` and a separate
`state_input = "checkpoint.txt"`. At runtime the domain entry receives read-only
`AN_SPOOL_INPUT` and `AN_SPOOL_STATE` snapshot paths. No HEAD, state catalog or
upstream workspace is exposed. Existing single-input releases remain unchanged.
An operator explicitly initializes the instance using initial-state.txt and the
declared text output state.txt. A plain start cannot omit this release's state.

Direct host selection uses `spool-start-data-state INSTANCE RUN SELECTION SECONDS`,
where SELECTION is strict operator JSON:

```json
{"schema":"an-native-data-state-v1","data":{"catalog":"/selected/catalog","revision":"EXACT_DATA_REVISION"},"state_revision":"EXACT_COMMITTED_STATE_REVISION"}
```

Alternatively deliver an accepted-artifact event using `spool-event-deliver
WORKSPACE EVENT SUBSCRIPTION --state EXACT_COMMITTED_STATE_REVISION`. Each
mutation needs its own host preview/approval. There is no implicit latest input,
state injection, commit or execution retry. Formal acceptance, expected-version
state commit and event consumption completion are separate decisions. The normal
qualified example commits state before completing consumption; a conflict does
not silently rebase a candidate or discard its accepted report.

From the repository root, run the paired qualification with built binaries:

```sh
python3 -B tests/cli/native_data_state.py --loom /selected/an-loom \
  --host /selected/an-agent-host --host-revision FULL_HOST_COMMIT \
  --output /private/new-log-state-acceptance
```

It packages this project, runs direct and event-bound data/state selections,
checks a stale commit conflict, and verifies fixed products/checkpoints after
collection. The host's own suite owns real cancellation, uncertain delivery and
retention fault cases. No model, GUI, background workflow engine or new backend
is used. Keep generated machine paths and command logs private.
