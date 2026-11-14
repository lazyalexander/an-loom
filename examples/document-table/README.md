# PDF → Markdown → Excel

A text-table reference workflow using pypdf and openpyxl. The domain supports a
three-column pipe table embedded in a text PDF. It does not implement OCR or a
general table-layout recognizer. The two programs run as ordinary Python spools.
Loom checks and packages them; an-agent owns execution and artifact versions.

Install `requirements.txt` in an operator-selected virtual environment. The real
qualification also needs reportlab to generate its synthetic input. Run
`qualify.py --help` for binary and environment selection. Qualification uses the
host's retained test driver from the exact paired checkout, never a runtime
Python dependency on the host test suite.

The main Codex app-server sees only its explicit task and authorized tool results.
It must call the existing host tools to run and accept both stages. The second
stage consumes the exact accepted Markdown revision. An operator-issued child
grant lets a separate app-server thread review only that Markdown; it cannot read
the source PDF, inherit the main conversation, escalate or access sibling results.
Private prompts, credentials and model events stay in the local evidence folder.
Only the portable acceptance summary is suitable for publication.

```sh
python3 -m venv .local/document-env
.local/document-env/bin/python -m pip install -r examples/document-table/requirements.txt reportlab==4.4.5
.local/document-env/bin/python -B examples/document-table/qualify.py \
  --loom /operator/an-loom --host /operator/an-agent-host \
  --host-tests /operator/paired-an-agent/crates/an-agent-host/tests/cli \
  --codex /operator/codex --python .local/document-env/bin/python \
  --output .local/document-acceptance
```

Use [the paired host revision](https://github.com/lazyalexander/an-agent/commit/4a7ecd82f63ec1b048926155ad3126e250bc4c79) and Codex CLI 0.166.0 with native file login.
This command explicitly authorizes disclosure of newly generated synthetic data.
It creates a new evidence directory, performs no automatic replay, and keeps
prompts/native events private. Workbook inspection independently checks typed
rows and total 38.5. The Markdown revision is selected after its successful
acceptance, so the controller explicitly issues a second main-task grant for
that exact revision. It never grants access to a mutable latest selector.

The main task may poll an independently supervised child. Cancelling the main
harness does not automatically cancel a separately admitted spool. Applications
must explicitly cancel those tasks using their retained identities. The first
adapter starts fresh threads; persistent-session resumption remains unqualified.

[The portable local acceptance](acceptance/app-server-local.json) records a real
Codex app-server run, exact binary digests, selected model, turn durations and
fixed document identities. Its main/child permissions and cancellation checks
passed. It is one synthetic text-table case, not an OCR or arbitrary-PDF claim.
CI uses credential-free protocol fixtures and domain tests; vendor inference
remains an explicitly invoked local qualification.
