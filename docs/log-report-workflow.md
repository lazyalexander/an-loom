# Log report: support, coupling and domain ownership

The [example](../examples/log-report/README.md) produces an editable workbook and
presentation from synthetic logs. Its CLI kit also runs without Loom, an-agent,
Nextflow, a model API, a GPU or an Office installation. Reusable support is tested
through an optional host adapter rather than imported into the business kit.

## Boundaries

| Category | Acceptable responsibility | Boundary in this example |
| --- | --- | --- |
| Support | Generic bounded byte custody, lineage, pinned inputs, termination and retention | an-agent host, with no Office or workflow-engine dependency |
| Explicit coupling | A replaceable format/protocol adapter with a stated contract and qualification scope | openpyxl/python-pptx in the example; optional host CLI adapter outside `kit/` |
| Invasion | Domain assumptions becoming required behavior of a general component | Log fields, percentile rules, worksheet columns, slide layout and Office ZIP parsing must not enter host/core or Loom |

Loom checks and packages selected sources; it does not become a runtime scheduler,
Office service or model loop. The package uses the generic content path, leaving
the fixed Rust file-kit deployment adapter unchanged. Model narration can be a
future domain stage through an authorized service; no model is needed to count
requests or carry a saved review note into a report. This acceptance makes zero
model calls.

## What the acceptance proves

The real host route runs the reviewed Python kit under the existing macOS workspace
profile. Explicit binary outputs become immutable byte revisions. An editable copy
is saved and committed with a parent and approved digest. A later edit creates
another revision; the downstream profile still binds the selected earlier revision
and snapshot. Substituting that snapshot is refused before execution. Accepted
PPT provenance identifies the exact input revision, and native editable text/table
objects contain the selected review note.

The test simulates the saved edit with a document library. It does not claim that
a human operated a GUI, or authenticate who edited a document. A real spreadsheet
editor can replace that save step without changing the host contract. The supplied
workbook may be opened/recalculated in an Office-compatible application; downstream
validation does not depend on formula caches.

Cancellation happens only after a real PPTX exists while the process is still
active. Active, cancelled and late replacement candidates are refused. Native
termination is confirmed; no partial formal revision is created. Accepted outputs
survive explicit scratch collection. A text-only output declaration refuses the
same binary bytes. Domain parsing/editability checks are separate from host byte
identity and cannot certify every Office document safe to open.

## Execution environment scope

The host's first module move separates identity/settlement from concrete backend
mechanics while preserving paths and behavior. It adds no VM, container, pause,
snapshot or orchestration service. Location, operating system and isolation type
remain separate capability dimensions. A future backend must qualify actual
termination and its own recovery guarantees; a process sandbox cannot silently
stand in for a VM requirement.

The native workspace profile is for reviewed local tools without daemonization,
not a hostile coding-agent sandbox. Full host custody acceptance remains macOS-only;
standalone kit checks run on Linux and macOS. CPU Linux is the deployment direction,
not an expanded claim for this host profile.

## Observed adaptation evidence

The first native attempt failed importing openpyxl because Python's MIME registry
tried to read `/etc/apache2/mime.types`. The failed, terminated receipt was retained.
A fresh explicit attempt used the built-in MIME registry plus the libraries' OOXML
registrations, initialized in `kit/ooxml.py`. No sandbox rule or host permission was
expanded. This is a concrete library compatibility detail in a replaceable adapter,
not a new host policy for one document library.

Portable results are in [the acceptance record](acceptance/log-report.json).
Raw profiles, machine paths, process logs and development probes stay private.
The CI integration selects a pinned host source revision; it does not add a host
runtime dependency to Loom or a Loom dependency to an-agent.
