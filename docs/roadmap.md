# Research and next acceptance slices

Status: planned, 2026-10-24. This is an executable work plan, not a list of
implemented runtime capabilities. Preserve the existing package and host checks.

## Terminology

Use **workflow** for the domain process, including branches, agents and human
decisions. Keep **kit** for a callable component and **spool** for its deployable
domain carrier. No additional machine abstraction or execution engine is needed
to adopt an existing workflow. A workflow need not be a static DAG.

## 1. Establish spool update identity

Research exact release identity, authorized lineage, compatibility and instance
identity separately. The starting findings and refusal cases are in
[spool-identity.md](spool-identity.md). Reuse package coordinates, SemVer where
applicable, and supply-chain evidence concepts before adding manifest fields.

Deliver a small local update acceptance: two releases in one authorized lineage,
an unauthorized successor refusal, an explicit capability increase decision and
a failed state migration preserving the old checkpoint. Loom packages evidence;
the host owns admission and instance transition. Do not infer equivalence from
matching names, UUIDs, version labels or a model's opinion.

## 2. Observe an existing workflow and its files

Initial candidate: [Nextflow](https://github.com/nextflow-io/nextflow), using the
upstream [hello workflow](https://github.com/nextflow-io/hello) as a small engine
probe. Its [source](https://github.com/nextflow-io/hello/blob/master/main.nf)
contains four greeting tasks with stdout output. It is not a realistic domain
pipeline or a file-output/agent acceptance by itself. Keep that limitation visible.

The reason to start here is its existing work/cache/resume mechanism. Nextflow
[resumption](https://docs.seqera.io/nextflow/cache-and-resume) needs both task
metadata and task working files. Its [reports](https://docs.seqera.io/nextflow/reports)
offer separate execution diagnostics. We should observe these, not replace its
scheduler, task cache or native logs with an-agent implementations.

This research did not run Nextflow. A local prerequisite check found no usable
Java runtime. Before running, pin/review an upstream workflow commit and engine
version, license, configuration, plugins and dependency provenance. Inspect the
engine's local execution needs and authorize a qualified host route; the current
Loom Rust build profile does not support arbitrary JVM workflows. Preparation
downloads and execution should have separate permissions.

The first real observation must inventory files before, during, after and after
an interrupted attempt. Record relative paths, sizes, hashes, file kinds and
producer/attempt identity, without publishing private contents. Treat symlinks,
hardlinks and files still being written as hazards to verify at acceptance.

| File class | Intended handling | Acceptance evidence |
| --- | --- | --- |
| Pinned workflow code/configuration and immutable inputs | Read-only release/input material | Selected source and input digests unchanged |
| Task work files and engine cache | Scratch by default; retain together under an explicit resume lease | Resume works while retained; no false recovery claim after collection |
| Native execution logs/reports | Bounded, private diagnostics unless explicitly exported | Collected status, retention deadline and redaction decision |
| Declared deliverables | Candidate until host acceptance; then immutable artifact revisions | Closed output manifest, validated bytes and producer identity |
| Agent session/tape, if present | Tool-owned private data; optional explicit reference | No required transcript import or completeness claim |
| Credentials and machine configuration | Trusted service configuration outside payloads | Not copied into releases or artifacts |

For the hello probe, select the four greeting results as a small explicit
deliverable through a reviewed wrapper; do not promote every work file. Then use
a pinned, existing file-producing workflow, or a clearly labelled local extension,
to qualify actual output-file admission. Do not call the extension upstream code.

Derive the **an-agent file-management slice** from the observed boundary:

1. Give the attempt a disposable work area and immutable selected inputs.
2. Keep internal paths opaque to downstream spools; receive only declared outputs.
3. After observed termination, validate/copy/hash outputs into host custody and
   commit an identified acceptance record. A file event is only a candidate signal.
4. Retain checkpoint/input references, pending delivery references and explicit
   resume/debug leases. Unknown termination or possible remote writers forbid
   scratch collection. Leases must be bounded and visible, not permanent defaults.
5. Collect eligible scratch without affecting accepted artifacts; reject late
   outputs and test partial writes, failed output checks and repeated acceptance.

Inventory must be reconciled at acceptance; a filesystem watcher cannot establish
file completeness or a consistent checkpoint. Reuse current host artifact,
checkpoint and retention mechanisms first. Add only the gap the observation shows.
Workflow-specific file semantics stay in its adapter/spool.

## 3. Allow agents without centralizing their tape

For the first agent-assisted workflow, an operator supplies the reasoning/editing
step. Make its bounded input, output and authority visible; do not invent an
autonomous orchestrator just to remove this manual step. Distinguish this from a
live code-agent adapter acceptance.

External agents manage their own tape. Keep host boundary receipts and optional
session references. Direct API-call metadata is a separate, optional choice;
filesystem backup and provider billing each retain their own purpose. Details and
unknown-outcome handling are in [code-agents.md](code-agents.md).

## 4. Qualify existing coding tools

Use one task contract and provider-specific instruction/policy adapters.
[The survey and acceptance](code-agents.md) cover Codex, Claude Code, Kimi Code,
Grok Build and Gemini CLI. Start with one installed tool and a lightweight kit.
Require actual refusal and cancellation evidence before claiming controlled
integration. Do not silently select a model or upload a workspace during research.

## Sequence and scope

Complete the identity research and choose the first authority policy; then observe
one workflow and qualify the minimum host file boundary. Use the operator-filled
agent step before automating it. Qualify one mature code-agent adapter against the
same boundary. These can share a fixture, but need separate evidence.

No new core engine, VM framework, universal workflow DSL, transcript service or
provider billing system is part of these slices. Update this plan from observed
failures instead of pre-building a general framework. Runtime changes belong in
separate an-agent PRs; Loom remains independently usable for authoring/packaging.
