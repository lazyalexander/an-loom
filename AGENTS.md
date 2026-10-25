# an-loom contributor contract

This is the single repository instruction entrypoint. It provides development
instructions, never runtime grants. Read [the architecture](docs/architecture.md)
and the relevant implementation before proposing a new contract.

## Product and ownership

Loom helps developers turn domain knowledge and existing software into reusable
spool releases. It scaffolds, prepares bounded coding tasks, requests checks,
validates evidence and packages selected bytes. Reuse mature coding tools and
workflow engines; do not implement competing editing or model loops.

- A **silk** is a callable computation inside an owning spool, normally a CLI,
  with explicit inputs and outputs. It does not receive a separate VM or grant.
  Independent CLI use outside the host carries no managed-execution guarantee.
- A **workflow** describes a domain process combining silks, agents and human
  decisions. This is terminology, not a new runtime or schema.
- A **spool release** carries domain implementation, silks, instructions,
  configuration, state schemas, migrations and acceptance rules. A spool execution
  attempt is the isolation and lifecycle unit, distinct from committed state.
- **an-agent** owns runtime authority, model/context services, execution supervision,
  committed state, accepted artifacts, retention and boundary evidence.
- A **backend** enforces isolation and confirms execution termination. A
  **shuttle** mediates authorized interaction with external applications.

Keep domain decisions in spool projects, host implementation outside Loom and
runtime dependencies one-way: an-agent must not depend on Loom. Use Rust 2024 and
FP style: deterministic planning/validation at the center, explicit effect
adapters at the edge. Payload languages are open.

See [spool boundaries and naming compatibility](docs/spool-boundary.md). Existing
`kit` CLI/profile/API tokens and historical Silk communication records remain
compatibility identities; never globally replace them or rewrite old releases.
OpenShell belongs behind the host backend adapter, not in Loom or semantic logic.

## Changes and review

- English only for repository documentation, comments and examples.
- Main is stable. Work on feature branches and open PRs to main. The maintainer
  reviews and merges. Never merge on their behalf, force push or weaken CI gates.
- Distinguish implemented guarantees, tool claims and proposed capabilities.
  Preserve existing behavior unless the task explicitly changes it.
- Generic packaging is broader than controlled execution support. The packaged
  deployment adapter remains one dependency-free Rust source and `file-kit-v1`.
  An operator example using a host workspace profile does not add a spool ABI.
- Preserve include allowlists, path containment, secret checks, deterministic
  identities, exclusive destinations and archive verification.
- Preserve upstream licenses and pinned provenance. Keep historical an-waver
  material available privately until its remaining usage references are checked.

## Existing agents and workflow tools

Prepare bounded tasks with relevant input references, editable paths, reusable
kit entrypoints, acceptance commands and expected outputs. Prefer calling an
existing silk over asking the agent to recreate it. Keep vendor-specific behavior
in qualified adapters; do not infer support from another CLI's flags.

Repository hooks, plugins, MCP servers, agent overrides and build scripts are
executable inputs to review. Instructions and hooks cannot expand host authority.
Never execute project commands implicitly, disable isolation or enable blanket
approval to make acceptance appear successful. Show exact commands, selected
inputs, output boundaries and approval requirements; disclose unsupported controls.

External agents own their context and native tape. Do not scrape private session
stores or require transcript ingestion. The host keeps attempt identity, effective
authority, termination evidence and accepted output references. Direct model API
tracing is a separate, optional integration choice, with missing usage and unknown
outcomes disclosed. See [the coding-tool contract](docs/code-agents.md).

## Evidence and public hygiene

Keep credentials, machine profiles, absolute user paths, native transcripts and
private records out of public files. Publish portable summaries separately from
local operational data. Never claim that hashes authenticate a publisher, that
filesystem backup reverses external effects, or that a process exit proves remote
or detached work stopped. Unknown outcomes require reconciliation, not auto-retry.

Before committing:

- `cargo fmt --all -- --check`
- `cargo test --locked`
- `cargo clippy --all-targets --locked -- -D warnings`
- `cargo quality --base origin/main`
- `cargo test --manifest-path tools/quality/Cargo.toml --locked`

The TOML quality policy and empty baseline are merge gates, not sandbox grants.
Do not raise limits or add exceptions to make a feature pass. See
[quality rules](docs/quality.md).

Real host acceptance is required before claiming a new execution guarantee.
Fixtures alone are insufficient. Follow [the roadmap](docs/roadmap.md): research
release lineage before changing identity, observe real workflows before growing
file management, and qualify one coding-tool adapter before adding a catalog.
Report changes, actual verification and remaining limitations in the PR.

## Test placement

Follow [the test map](tests/README.md). Private unit tests live in adjacent
`tests.rs`/`tests/` modules behind `cfg(test)`; public contract suites live under
`tests/` with focused local support. Preserve Cargo target names, assertions and
ignored real-host coverage when moving files. Domain tests stay with examples;
quality-tool tests stay in their independent workspace. Generated output and
private acceptance data never become source fixtures.
