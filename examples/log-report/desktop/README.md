# Native editor acceptance (optional operator tools)

These tools stage and inspect a real desktop save. They never edit workbook cells.
The independent kit still needs only its own `requirements.txt`; neither Loom nor
an-agent depends on LibreOffice, psutil or matplotlib.

## Boundaries

The operator's replaceable desktop adapter opens the checked-out XLSX, edits cells
and invokes Save. Excel or LibreOffice may be used. The example validates selected
business fields read-only; the host accepts exact approved bytes into its existing
artifact catalog. The editor producer property is a hint, not proof of authorship.
Loom packages files; the host owns revision custody, fixed input bindings and
settled workspace reclamation. This does not introduce a shuttle ABI, execution
backend or untrusted-agent desktop sandbox.

Use a dedicated editor session with no other documents. Install observer dependencies
from this directory's requirements into a separate environment. Invoke `session.py`
with that environment. Preserve the business environment's Python launcher path;
do not resolve its virtual-environment symlink.

## Manual native roundtrip

1. Run `session.py prepare --root NEW_DIRECTORY --host HOST_BINARY
   --host-revision FULL_COMMIT --python KIT_VENV/bin/python
   --observer-python OBSERVER_VENV/bin/python --editor-executable EXACT_EXECUTABLE`.
   Optionally use `--working-directory VISIBLE_DIRECTORY` for native file dialogs
   that do not admit hidden project paths. `report.xlsx` must not already exist.
   Use canonical paths (on macOS, `/private/tmp` rather than `/tmp`).
2. Open the returned working copy through the real desktop UI. Mark
   `editor.window.ready`, `document.open.requested` and `document.opened` only when
   their corresponding actions have actually been observed.
3. Edit `Report!E8` (Owner) and `F8` (Review note). Mark
   `document.save.v2.requested`, save XLSX in the application and verify completion,
   then mark `document.saved.v2`.
4. Hash the saved file, review the selected bytes and run `session.py commit
   --root DIRECTORY --label v2 --approve SHA256 --expected-note EXACT_CELL_TEXT`.
5. Change F8 again in the real UI, save and commit v3 the same way, using v3 events.
6. Mark `document.close.requested`, close the document, mark `document.closed`.
   Mark `editor.quit.requested`, quit the dedicated application and verify exit.
7. Run `session.py finish --root DIRECTORY`. It requires both saved revisions and
   an observed editor exit. It selects v2 after v3 exists, runs the presentation
   kit, verifies v2 provenance and note text, reclaims settled scratch and checks
   all three formal revisions remain readable.
8. Run `report.py --root DIRECTORY --output NEW_REPORT_DIRECTORY` with the observer
   environment. An incomplete or expired capture cannot become a success report.

Use `session.py mark --root DIRECTORY --event EVENT` to record observed UI steps.
These marks are operator attestations, not automatically verified GUI receipts.
Never mark an attempted operation as completed. Do not auto-retry an uncertain
host operation. Reconcile it first.

## Observations and limitations

The 250 ms sampler observes only the selected editor executable, explicitly watched
host processes and their descendants. Process identity includes creation time.
Private watch records contain PIDs; portable samples do not contain command lines,
environments, PIDs or machine paths. Inspect any acceptance summary before publishing.
Actual documents may contain editor-added author metadata and should remain local
unless separately reviewed.

Charts distinguish process presence, GUI document state, scratch retention and
formal artifact retention. They do not demonstrate VM/spool hot-loading. RSS sums
can count shared pages more than once. Short workers and final CPU bursts can be
missed. CPU is sampled process time, not whole-system energy. UI windows include
operator and automation delays, so timings are not application benchmarks. Warm
restart and first-run setup must be reported separately. An observer expiry leaves
private diagnostic evidence and must not be reported as a completed acceptance.
