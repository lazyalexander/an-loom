# Nextflow execution, interruption and file custody

Status: native macOS acceptance, 2026-10-24. This report distinguishes an unchanged
upstream workflow, a local file-producing fixture and a separately reviewed host
profile. It is not qualification of arbitrary Nextflow pipelines or code agents.

## Reproduced route

The prior Loom documentation PR #2 merged with all three required checks passing.
Java was installed and Nextflow 26.10.3's official distribution was verified
against its release SHA-256. The upstream hello source was verified against both
its pinned commit and Git blob identity. The exact sources, command and required
host operations are in [the example](../examples/nextflow/README.md).

The acceptance runs without an LLM. Python's standard library only drives the
fixed operator fixture and reads diagnostic inventories. Execution policy,
process ownership, exact approval, formal artifact publication and scratch
collection are Rust host responsibilities. Generic Loom packaging does not need
Java or a running host.

1. Run upstream hello without changing its source. A small labelled wrapper
   redirects stdout into a declared deliverable.
2. Run the local two-prepare/two-finish workflow to completion. Receive its two
   declared result files as immutable revisions using existing host storage.
3. Run the local workflow again; stop after one finish result is published while
   another finish task is active. Active collection is refused. Send SIGINT to
   the real host CLI, then verify a cancelled, termination-confirmed receipt.
4. Refuse partial-output acceptance. Preserve the native work/cache and explicitly
   continue from the selected predecessor with Nextflow `-resume`.
5. Observe three cached tasks and one completed/recomputed task in the actual
   resume trace. Receive both results, check out pinned revisions, explicitly
   abandon recovery and collect scratch. Check out the same revisions again and
   compare bytes; reject subsequent resume.

The resulting formal bytes are `ALPHA\n` and `BETA\n`, with identical content
hashes between the ordinary and interrupted/resumed cases. Revision identities
can differ because they also bind producer evidence. Native task IDs, transient
filenames and inventory sizes vary; acceptance asserts relationships, not their
incidental random values. See [portable evidence](acceptance/nextflow-local.json).

## Observed files and ownership

| Observed path/class | Meaning established by this experiment | Host treatment |
| --- | --- | --- |
| `.nextflow/cache/<session>/db/` and session indices | Native task cache, including database log/table/lock files | Retain with work files while choosing to resume; do not reinterpret as host state |
| `.nextflow/history` | Native execution/session lookup material | Private engine recovery/diagnostic data |
| `work/<task>/.command.sh`, `.command.run`, `.command.begin`, `.exitcode` | Generated task script, wrapper and task-local progress/exit material | Internal evidence; a task exit file cannot prove whole-workspace termination |
| `work/<task>/.command.out`, `.command.err`, `.command.log` | Native task diagnostics | Private scratch or explicitly retained diagnostics |
| Staged input symlinks between task directories | Native input binding, not new output ownership | Inventory without following; declared linked deliverables are refused |
| `results/*.result.txt` | Explicit published candidates | Validate/copy into existing immutable artifact revisions only after host approval |
| `trace-*.tsv`, `.nextflow.log`, engine-created metadata directories | Diagnostic observations, including paths and timing | Keep raw files private; publish only selected portable facts |
| `engine-home/` and `tmp/` | Runtime-local directories, including an empty native secrets directory in this fixture | Private scratch, not package content or inherited provider credentials |
| `JOB/attempts/` and `JOB/artifacts/` | Host plans, intents, receipts and accepted revisions | Outside the workload's writable and reclaimed tree |

Concurrent inventory reads are diagnostic, not an atomic snapshot. They mark
changed/disappearing entries. The host only takes acceptance/reclamation decisions
under its lock after its own termination gate and a fresh bounded inventory.
Filesystem events and engine exit markers are insufficient authority.

[Nextflow's cache contract](https://docs.seqera.io/nextflow/cache-and-resume)
requires both cache metadata and native work files for resume. The observation
confirms that preserving only exported results would lose useful engine recovery
material. This is not an argument to retain every internal file permanently:
keep recovery material until explicit abandonment, then collect it together.

## Adaptation failures and fixes

Two preparation failures were retained rather than silently retried:

- The downloaded distribution was initially not executable. Its launcher resolved
  an absolute script invocation incorrectly and tried a download. The sandbox
  refused it. Make the already digest-verified distribution executable during
  dependency preparation; do not grant network access during the workload.
- The qualified Homebrew JVM crashed even for `java -version` under the initial
  policy. Its crash stack ended in system-property initialization. Isolated
  policy probes showed that allowing only the macOS user-directory-service lookup
  `com.apple.system.opendirectoryd.libinfo` fixed startup. Neither a blanket Mach
  allow rule nor broader filesystem/network access was needed. The same narrow
  service allowance is present in the reviewed local Codex sandbox implementation.

The host also preserves process receipts when a scratch inventory fails, instead
of losing termination evidence behind an inventory error. Linked-output regressions exercise this receipt path; invalid inventories still
refuse collection until the exact allowed file tree can be validated.

## Minimum reusable host boundary

The companion host change adds operator-owned `workspace-run`,
`workspace-resume`, `workspace-accept` and `workspace-reclaim` operations. It uses
an existing supervisor, immutable revision publisher and consumer API. It does
not add another artifact database, event scheduler, workflow language or VM
controller. A native test feeds the new revision into the existing kernel-admitted
file kit both before and after collection, checking pinned input and applied
execution evidence.

Continuation binds an explicit predecessor and current scratch inventory. Missing
or uncertain receipts, active locks and stale predecessors refuse continuation or
collection. Acceptance checks selected inputs and exact candidate bytes, rejects
failed/cancelled and linked outputs, and fences duplicate/partial publication.
Collection is separately approved and states that resume will be lost. An output
change or late file invalidates the corresponding reviewed plan.

The existing file-kit ABI and its stricter process restrictions remain intact.
Workflow-specific task order, cache meaning, result validation and future agent
instructions stay in the example/spool. Tool-native tape stays with its owner;
no normalized model transcript or provider billing system was introduced.

## Limits and next work

The new profile is for **reviewed local tools without daemonization**. It owns a
process group, reaps its leader and checks group emptiness; it cannot certify
escaped workers, remote jobs or adversarial process-tree behavior. Source/profile
approval is not a substitute for a stronger backend for untrusted coding agents.
No general JVM workload, Linux workspace backend, state migration, remote resume,
periodic retention or formal-artifact GC is qualified here.

Scratch retention is manual and explicit in this first profile. A bounded resume
lease/scheduler is future work, not an implemented guarantee. Formal outputs are
bounded UTF-8 files; multi-output publication is not an atomic transaction, and
no independent backup or power-loss recovery is claimed. Native runtime paths
must stay available; retained files do not make external dependencies recoverable.

The next useful validation is one real domain workflow using this boundary, with
its own output checks. A live coding-tool adapter must separately prove authority,
cancellation and credential behavior. Neither requires replacing Nextflow's cache
or making Loom a runtime authority.
