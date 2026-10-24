# Nextflow lifecycle and file custody acceptance

This operator-owned example connects an existing workflow engine to the host's
reviewed local workspace profile. It uses no model API, installs no workflow
scheduler inside Loom and leaves Nextflow's native task cache/resume semantics
intact. The example is a generic Loom content package, not a `file-kit-v1` release.

## Sources and requirements

- Nextflow `26.10.3`, [official distribution](https://github.com/nextflow-io/nextflow/releases/tag/v26.10.3),
  asset `nextflow-26.10.3-dist`, SHA-256
  `e0d46f3274882e73abfb0672cf84b04c6f408b1882d11b51e50dcd533b707da6`.
- A compatible ARM JDK; native acceptance used Homebrew OpenJDK 21.0.13.0.2.
  See [Nextflow installation](https://docs.seqera.io/nextflow/install).
- Python 3 standard library for the observation fixture. It avoids an additional
  orchestration dependency; product planning and host custody remain Rust.
- Native macOS permitting Seatbelt activation, with the independently built host
  exposing `workspace-run`, `workspace-resume`, `workspace-accept` and
  `workspace-reclaim`. See [the host contract](https://github.com/lazyalexander/an-agent/blob/fa8cc05e981e743bcc3cfc94fd04efd732827a6b/crates/an-agent-host/WORKSPACE-JOBS.md)
  from [companion host PR #100](https://github.com/lazyalexander/an-agent/pull/100). There is no unsandboxed fallback.

`upstream-hello.nf` is byte-for-byte upstream `nextflow-io/hello/main.nf` at
`3c2cdc9823c2b4636e5e3e73e223878099ff5dc9`; its Apache-2.0 license is retained
as `LICENSE.nextflow-hello`. `upstream-driver.sh` only redirects its stdout to
an explicit candidate file. `files.nf` is a separate local file-producing
fixture, not an upstream pipeline. It has two prepare and two finish tasks, with
one finish at a time so interruption can occur after useful work completed.
This establishes engine/file boundaries, not production domain-workflow quality.

## Run explicitly

Install Java and download the pinned distribution separately from execution.
Check the SHA-256 before making the distribution executable. Do not commit the
runtime, machine-specific profile, generated work/cache or raw execution logs.

```sh
python3 examples/nextflow/observe.py \
  --host /operator/selected/an-agent-host \
  --nextflow /operator/dependencies/nextflow-26.10.3-dist \
  --java-home /operator/dependencies/jdk/Contents/Home \
  --output .local/nextflow-acceptance
```

Use a fresh output directory. The harness is a trusted operator acceptance driver:
its explicit invocation authorizes its fixed fixture plans. It generates private
profiles, obtains host previews and approves their exact digests. Do not use it as
an unattended launcher for arbitrary project-provided profiles or source code.
Runtime roots are read-only, the environment is cleared and execution is offline.

The harness runs the unchanged upstream hello workflow and the local file fixture,
then observes a second file-fixture run while one result is complete and another
finish task is active. It verifies active reclamation refusal and sends SIGINT to
the real host CLI. A confirmed cancellation preserves the native work/cache;
partial output is refused. An explicit successor uses Nextflow `-resume` and
checks three `CACHED` tasks and one `COMPLETED` task from the real trace.

Declared outputs are received through the existing artifact revision catalog.
The harness checks out each pinned version, explicitly abandons recovery and
collects scratch, then checks out the same versions again and compares bytes.
A subsequent resume request must fail. No transcript ingestion or model call is
needed. This is not an automatic deletion/retention service.

## Evidence and packaging

The private output includes per-case profiles, plans, intents, process receipts,
full relative-path inventories, native trace observations, accepted revision
bindings and before/after checkouts. Concurrent inventories are best-effort
observations, not consistent checkpoints; changed/disappearing files are marked.
Native cache symlinks are recorded without traversing them. Raw logs and generated
scripts can contain absolute paths and stay private.

[The observation report](https://github.com/lazyalexander/an-loom/blob/main/docs/nextflow-observation.md) describes the measured
file classes, adaptation failures, runtime limits and remaining gaps. The public
acceptance JSON includes only portable counts, assertions, source/tool identities
and explicitly selected relative paths and output digests.

The fixture can be inventoried and packaged without a JVM or an-agent:

```sh
an-loom check examples/nextflow
an-loom build examples/nextflow
an-loom pack examples/nextflow
```

Those commands snapshot selected source bytes; they do not run Nextflow or grant
runtime access. The `observe.py` result is host acceptance evidence, not a Loom
runtime service or proof that other JVM workflows are safe to execute.
