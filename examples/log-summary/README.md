# Log summary spool

A second-domain qualification of the existing native spool route. The editable
kit counts INFO, WARN and ERROR records using the system awk CLI. Blank lines are
ignored; unknown levels fail. Stable silks coordinate the kit and add the report
header. This is a reviewed, headless macOS shell workflow, without model calls,
network access or dependency installation.

```sh
cargo build --locked
python3 -B -m unittest discover -s examples/log-summary/tests -v
python3 -B examples/log-summary/qualify.py \
  --loom "$PWD/target/debug/an-loom" \
  --host /path/to/an-agent-host \
  --output /path/to/new-qualification-directory --execute
```

`--execute` authorizes the trusted operator driver to inspect and approve the
existing bounded plans. Do not expose this driver as an unrestricted agent tool.
The output must not exist; failed or unknown attempts retain evidence and cannot
be automatically resubmitted. A client timeout does not prove host termination.
Inspect retained commands/receipts before authorizing a separate new task.

The driver scaffolds a project, supplies domain acceptance before task creation,
receives a deterministic kit candidate, runs `develop-native`, and separately
executes the loaded spool twice. Editing the input produces a different fixed
artifact version. Both reports remain readable by pinned revision after scratch
reclamation and unload. Domain acceptance stays in the example; no host or Loom
business branches are needed. This does not qualify hostile agent containment,
a new runtime, log streaming or a full production observability system.

`acceptance.json` is a portable summary; project, task, delivery, raw command
outputs and paths are local operational evidence. Do not publish those raw files.
The development task uses the existing text scaffold and retains its package
name; it is a distinct content-identified release, not a new runtime profile.

The driver also submits an unknown log level as a third run. Its real execution
must fail with confirmed termination; formal acceptance and artifact discovery
must refuse. Scratch is reclaimed only afterward, and both earlier successful
versions are checked again after unloading. A partial report is not a product.
