# Log report workflow

A standalone CLI kit converts synthetic JSONL service logs into an editable XLSX
workbook and a native, editable PPTX report. It needs Python 3.9+ and the pinned
example dependencies. No GPU, model, Nextflow, Office installation, Loom or
an-agent is required for the business computation.

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt
.venv/bin/python kit/main.py workbook --input sample.jsonl --output report.xlsx
# Edit Owner and Review note in Excel or another compatible spreadsheet editor.
# Save a separate report-reviewed.xlsx.
.venv/bin/python kit/main.py slides --input report-reviewed.xlsx --output report.pptx
.venv/bin/python -m unittest discover -s tests -v
```

Outputs must be new files. The workbook summarizes request counts, HTTP 5xx errors
and nearest-rank p95 latency; its editable review columns feed native PowerPoint
text and a native table. It does not rasterize the presentation. Summary formulas
are for spreadsheet display; the deck reads validated service rows rather than
trusting formula caches. Recalculation belongs to the selected spreadsheet engine.
The small slide layout supports three services and refuses overflow.

Document libraries are replaceable adapters in `kit/workbook.py` and `kit/slides.py`.
The bounded row schema in `kit/data.py` owns domain meaning. Dependency versions
are pinned, not publisher attestations or a claim of byte-reproducible OOXML.
Consumer-local checks bound ZIP expansion, refuse unsupported external/macro
parts and formula-valued review cells. They are not a universal Office malware
scanner. Native dependency security remains the operator's responsibility.

## Optional host custody acceptance

The host is optional for generation. The fixed operator harness additionally
qualifies authorization, immutable binary versions, explicit saved edits, pinned
consumption, cancellation and scratch collection. Use native macOS and the
coordinated host revision with workspace binary outputs and artifact input
bindings. This reviewed-tool profile is not a hostile coding-agent sandbox.

```sh
python3 observe.py --host /selected/an-agent-host --host-revision FULL_COMMIT \
  --python /selected/venv/bin/python --output /private/new-acceptance \
  --loom /selected/an-loom
```

The output directory must be new and outside this package. The operator invocation
authorizes only these fixed fixture plans. Generated runtime profiles, process
logs and working files stay private. Only `acceptance.json` is designed to be
portable; inspect it before publication. `documents/` contains the accepted
workbook/deck versions for manual examination.

The harness simulates a saved operator edit using the document library. It does
not claim a human GUI interaction or authenticated authorship. In ordinary use,
replace that fixture step with a real saved workbook: `artifact-checkout` creates
a working copy, `artifact-edit` commits its approved digest with a parent revision,
and the downstream profile binds the chosen revision and a checked-out snapshot.
A later edit never changes that binding or triggers execution automatically.

The cancellation-only wrapper writes a real PPTX and then pauses. Acceptance while
active, after SIGINT-confirmed termination, and after a late candidate replacement
is refused. Existing formal versions remain readable after scratch collection.
The host treats each document as bounded opaque bytes; document layout and schema
checks stay here. No host, Loom, model or engine is imported by `kit/`.

## Loom packaging

```sh
an-loom check .
an-loom build .
an-loom pack .
```

This is a generic content package containing CLI sources, dependencies and domain
checks. It does not widen the existing fixed Rust `file-kit-v1` deployment adapter
or grant runtime permissions from package metadata. The optional `tools/host.py`
adapter is an explicit host contract dependency; replace it without changing the
business kit.
