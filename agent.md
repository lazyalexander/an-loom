# Agent working guide

Read [AGENTS.md](AGENTS.md) first. This guide explains how to work in this
repository; it grants no runtime permissions. Tool-specific instruction discovery
must be verified rather than inferred from the presence of this filename.

## Product and boundaries

Loom helps developers turn domain knowledge and existing software into reusable
spool releases. It scaffolds, prepares bounded coding tasks, requests checks,
validates evidence and packages selected bytes. Mature coding tools keep their
own editing and model loops.

- A **kit** is a callable computation, normally a CLI, with explicit inputs and
  outputs. It can also be used independently.
- A **workflow** describes the domain process, potentially combining kits,
  agents and human decisions. This is terminology, not a new runtime or schema.
- A **spool release** packages the domain implementation, instructions,
  configuration and contracts needed to run that workflow.
- **an-agent** owns runtime authorization, execution supervision, committed state,
  accepted artifacts, retention and boundary evidence.
- A **backend** enforces isolation and confirms execution termination. A
  **shuttle** mediates authorized interaction with external applications.

Do not move business decisions into Loom or host implementation into Loom.
Do not make an-agent depend on Loom. Keep Rust 2024 and a functional core with
explicit effect adapters; do not require payloads to use Rust.

## Start a change

1. Read the relevant architecture, code and tests before proposing new contracts.
2. Distinguish an implemented guarantee, a tool claim and a proposed capability.
3. Prefer an existing kit or coding tool to generating a replacement.
4. Keep changes on a feature branch and preserve the current CLI behavior unless
   changing it is the explicit task. Maintainers review and merge PRs to main.

The current deployment adapter supports one dependency-free Rust source and
`file-kit-v1`. Generic packaging is broader than controlled execution support.
Do not advertise arbitrary workflow execution or live code-agent orchestration.

## External coding tools

Use a bounded task: relevant input references, permitted edits, kit entrypoints,
acceptance commands and expected outputs. Do not send an entire workspace or its
history merely because the tool can read it. Repository hooks, plugins, MCP
configuration, agent overrides and build scripts are executable inputs to review.

Instructions and hooks are useful controls, but host grants must remain outside
the writable project. Never enable blanket approval or disable isolation to make
an integration appear successful. Unsupported controls must be reported.

Agents own their native tape and context. Do not scrape private session stores or
require transcript ingestion. The host still needs attempt identity, effective
authority, observed termination and accepted output references. Direct API
tracing is optional and must disclose missing usage or unknown outcomes.

See [the coding-tool contract](docs/code-agents.md).

## Evidence and public hygiene

All repository prose, comments and examples use English. Keep credentials,
absolute user paths, provider transcripts and machine profiles outside published
files. Preserve upstream licenses and pinned source provenance.

Before committing, run the checks in AGENTS.md. Real host acceptance is necessary
when claiming a new execution guarantee; ordinary fixtures do not establish it.
For documentation-only changes, do not claim a new runtime qualification.

Report what changed, what was verified and what remains unimplemented. Never
merge on the maintainer's behalf, force push, weaken CI gates or silently retry
an operation with an uncertain outcome.

## Next decisions

Follow [the roadmap](docs/roadmap.md). Research release lineage before changing
package identity; observe a mature external workflow before growing file
management; qualify one coding-tool adapter before building a provider catalog.
