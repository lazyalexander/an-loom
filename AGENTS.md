# an-loom contributor contract

This is the single repository instruction entrypoint. It provides development
instructions, never runtime grants. Read [the architecture](docs/architecture.md)
and the relevant implementation before proposing a new contract.

## Product and ownership

Loom helps developers turn domain knowledge and existing software into reusable
spool releases. It scaffolds, prepares bounded coding tasks, requests checks,
validates evidence and packages selected bytes. Reuse mature coding tools and
workflow engines; do not implement competing editing or model loops.

- A **silk** is core domain code and pipeline skeleton inside an owning spool,
  read-only in the selected release/attempt. Development edits require a new release.
- A **kit** is internal glue: agents may adapt explicitly editable working copies,
  never immutable packaged baselines. Temporary copies are scratch unless retained.
  Neither role receives a separate VM or grant. Follow [the code contract](docs/spool-code.md).
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
- `main` is stable; its promotions remain maintainer-reviewed and merged.
  `dev-advance` is the development authority. Branch ordinary work from it and
  target feature PRs there. Agents may merge dev PRs after diff review, applicable
  local/real acceptance and all required CI checks pass on the selected head.
- Pause for user review if changes may expose private data, credentials,
  transcripts or materials with unclear publication scope. Never force push,
  bypass protection or weaken gates. Repository guidance cannot create runtime
  authority or override a privacy restriction.
- Notify the user after every dev merge with repository, PR, target branch,
  resulting commit and a short description. Dev authorization excludes main.
- Keep PR scope stable; only add fixes or review responses once handed off.
  Independent follow-ups start from dev. Dependent work may continue on local
  branches with its base PR/commit recorded, then use verified merged dev before
  publication. Do not create long hidden integration chains.
- Open main promotion as a separate PR for the maintainer; never merge it
  automatically. Verify remote dev ancestry after a merge before using it as
  another task's base.
- Clean local branches only when older than seven days, merged into development
  and not checked out in any worktree. Preserve main/dev-advance and unmerged
  work, verify ancestry and use safe deletion. Do not detach worktrees to make
  their branches eligible.
- Distinguish implemented guarantees, tool claims and proposed capabilities.
  Preserve existing behavior unless the task explicitly changes it.
- Generic packaging is broader than controlled execution support. The packaged
  legacy deployment adapter remains one dependency-free Rust source and `file-kit-v1`.
  The separate native-spool adapter exports bounded shell releases only; its host
  contract and real acceptance are documented in `docs/native-spool.md`.
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
- `cargo quality --base TARGET` — use `origin/dev-advance` for development PRs
  and `origin/main` for stable promotions, matching the actual PR target.
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

## Test language and host coordination

Follow [the test map and execution rules](tests/README.md). Retain Rust for
internal invariants and private fault seams; prefer Python for new black-box CLI
acceptance, and TS when browser integration needs it. Do not rewrite mature
suites solely to unify languages or add production debug APIs for external tests.

Loom owns development/package/evidence contracts; an-agent owns execution,
termination and custody enforcement. Keep paired acceptance narrow and record
both revisions and binary identities. Build once per selected revision and
target/configuration, then pass explicit binary paths to external runners.

The checks above remain required. Run affected Python/TS and real-host suites
separately: Cargo success alone does not mean full acceptance. Preserve coverage,
fault cases and gates during migration, measure build versus execution, and
never retry unknown execution outcomes automatically.
