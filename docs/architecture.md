# an-loom: spool development workbench

Status: accepted direction, 2026-10-24. Implementation and migration acceptance
are tracked separately; a target capability is not an implemented guarantee.

## Spool execution boundary

A spool attempt owns the granted execution environment and lifecycle. Its
internal **silks** are release-bound core code and pipeline skeleton; **kits** are
adaptable glue working copies. See [the code contract](spool-code.md). They do not each
receive a VM or independent grant. OpenShell is a replaceable host backend,
never a Loom or semantic dependency. Its expansion is paused; native headless
lifecycle work is the current priority. Backend suitability is host-owned and
fail-closed: package checking or a headless interface cannot justify weaker
isolation or an automatic native fallback. Existing one-program profiles are bounded
specializations, not a completed multi-silk runtime. See the
[implementation audit and staged migration](spool-boundary.md). Legacy `kit`
spellings and historical Silk communication records remain compatibility data.

## Purpose

an-loom helps developers turn domain knowledge, existing programs, skills and
application integrations into inspectable, reusable spool releases. It provides
project authoring, assembly, development-tool integration, validation orchestration
and delivery. Its implementation is Rust, with deterministic transformations at
the center and explicit adapters for effects.

The product goal remains natural assistance inside existing applications.
an-loom serves the developers who make that assistance possible. CLI support
comes first; a graphical workflow editor is optional future presentation.

## Implemented starting point

Loom provides allowlisted content-addressed packaging, archive verification,
bounded source inspection, starter templates, candidate/check identity binding,
build-result reception and bounded tasks for existing coding tools. The legacy
deployment adapter remains a fixed Rust source and two predefined profiles.
The separate [native spool adapter](native-spool.md) exports release-bound
silk/kit roles, headless runtime requests, explicit state slots and v3 events.
Generic packaging does not imply arbitrary controlled runtime support.

Paired acceptance qualifies real native instances, kit edits/reset, cancellation,
fixed artifacts/state, finite event operation and private-material collection.
The host owns those guarantees. Loom does not replace a coding tool's session,
the host's supervisor, or the domain project's own workflow/acceptance rules.
Use [development rounds](development-rounds.md) for authoring/check status and
[native spool](native-spool.md) for the qualified package/host handoff.

## Responsibilities

| Component | Owns |
| --- | --- |
| an-loom | Authoring contracts, scaffolds, explicit component bindings, code-tool adapters, build/check plans, package evidence, release assembly and delivery clients |
| an-agent host | Admission, grants, scoped model/context services, execution coordination, durable instance/checkpoint publication, artifact custody, delivery evidence, retention decisions and audit |
| Execution backend | Enforces the spool attempt boundary and reports environment lifecycle/termination; OpenShell is one replaceable implementation |
| Spool release | Domain workflow, skills, agent instructions, business decisions, state schemas, migrations and domain acceptance rules |
| Silk | Core code and pipeline skeleton with stable contracts; read-only during its owning attempt |
| Kit | Internal glue whose authorized working copies may be adapted; published baseline bytes remain immutable |
| Shuttle | Application integration, including native APIs or GUI interaction, target revision checks and actual-effect receipts |
| an-agent-shuttle repository | A collection of domain integrations and reusable components; it is not a runtime layer |

Domain behavior belongs to spool projects. an-loom understands how to build,
check and deliver them, without interpreting their engineering, artistic or
other domain rules. Checks supplied by a project run as untrusted code inside
an explicitly authorized development boundary.

## Development and execution flow

```mermaid
flowchart TB
    Author[Domain expert and developer] --> Project[Domain project: contracts, existing code, skills, checks]
    Project --> Loom[an-loom: plan, assemble, validate, package]
    Loom --> CodeTool[Existing coding tool: development agent loop]
    CodeTool --> Source[Editable project changes]
    Source --> Loom
    Loom --> DevRequest[Explicit build and check requests]
    DevRequest --> Host[an-agent host: authorization and evidence]
    Host --> Backend[Native or isolated execution backend]
    Backend --> Candidates[Candidate binaries and check reports]
    Candidates --> Host
    Host --> Accepted[Accepted development outputs]
    Accepted --> Loom
    Loom --> Release[Immutable spool release and evidence]
    Release --> Host
    Host --> Instance[Instance bound to release and checkpoint]
    Instance --> Run[Admitted spool execution]
    Run --> Backend
    Run --> Request[External capability request]
    Request --> Host
    Host --> Shuttle[Authorized application shuttle]
    Shuttle --> Application[Existing software and editable documents]
    Application --> Shuttle
    Shuttle --> Receipt[Actual-effect receipt]
    Receipt --> Host
```

an-loom constructs development plans; an-agent admits and supervises actual
controlled executions. A package remains usable without linking the developer
tool, and a silk can retain a standalone CLI. Offline authoring and package
verification do not require a running an-agent service.

## Project, release and instance

Keep these objects distinct:

- **Project:** editable source, domain documentation, component bindings,
  generation instructions, fixtures and development commands.
- **Release:** an immutable inventory of selected package bytes and dependencies,
  entrypoints, capability requests and identified validation evidence.
- **Instance:** a host-managed identity bound to a release, permissions and a
  committed checkpoint. Live state and credentials are not release payloads.
- **Attempt:** one admitted execution with its own identity and outcome.

A spool may contain several silks and agent instructions. It requests shuttle
capabilities; every application bridge does not need to be copied into every
spool. A bounded internal workflow may initially use an ordinary reviewed runner
inside the spool. A new runtime workflow language is not required for the first
development workbench.

## Interfaces and authority

Start with explicit CLI requests and machine-readable records at the host
boundary. Use the host's existing authoritative descriptor validation and
execution profiles. Add a missing host operation only when a real integration
demonstrates the gap; never replace unsupported authority with a generated shell
command or an apparently successful package check.

Authoring declarations are developer requests. Source analysis reports observed
signals and unknowns. A host grant is a separate decision. Build evidence binds
source, executable, dependency and configuration identities to the extent actually
observed; a digest does not prove safety or reproducible compilation.

The developer configuration may use TOML, while deployment uses the host's YAML
contract. Reuse existing schemas and serializers where they are adequate. The
first workbench should not invent a parallel runtime schema for the same rules.

No reverse implementation dependency is required from an-agent to an-loom.
Host compatibility is checked through versioned public contracts and pinned
integration acceptance. Both repositories may change in coordinated PRs without
sharing business logic or requiring a common release cycle.

## Mature coding tools

External coding tools keep their own model loops, context compaction and editing
mechanisms. an-loom supplies a bounded task, relevant project references, allowed
silk entrypoints and acceptance criteria, and receives editable changes and
identified results. It does not implement a competing general coding agent.

They also own their native session tape. The host retains execution, authority,
termination and artifact handoff receipts; it need not ingest every model/tool
event. Direct model API envelopes are an optional integration choice. Filesystem
backup does not establish external effects or billing. See the proposed
[coding-tool contract](code-agents.md) and the scoped
[Kimi qualification](../examples/kimi-kit/README.md). The native CLI is trusted
operator software; its model requests receive only finite host capabilities.

Adapters declare whether a tool's model session is managed by the tool or
forwarded through a host service. Tool-managed model calls are outside any claim
of complete host model-call supervision. Credentials stay in the corresponding
trusted service and do not enter spool payloads or untrusted build environments.
Deployed spools request host model services under their runtime grants.

Invoking an existing silk should be simpler than asking a coding tool to recreate
it. Expose its contract, approved host invocation and bounded result references.
Prompt instructions are guidance; the execution boundary enforces permissions.

## State, artifacts and recovery

The spool defines its state schema, transitions and migrations. The host holds
committed state bytes, checks revision preconditions and records publication.
The backend owns process/VM state and any supported snapshot operation. A VM
snapshot is an optional execution optimization, not the portable business state.

Only explicitly delivered and host-accepted outputs become formal artifacts.
Scratch files are disposable after confirmed termination and retention checks.
Artifacts referenced by retained checkpoints or awaiting required consumption
remain retained. Delivery acknowledgements derive from committed consumption
evidence; missing acknowledgements alone do not authorize duplicate execution.

Continuing from a release and checkpoint also requires available dependencies,
authorized services and valid referenced inputs. It does not guarantee restoration
of external systems, model behavior or uncommitted actions. Unknown external
outcomes require reconciliation, never unconditional replay.

An existing workflow may need internal cache and work files together to resume.
Retain these only under an explicit host-visible resume lease; otherwise the
default remains disposable scratch after confirmed termination and retention
checks. A workflow's internal filenames are not downstream artifact contracts.
The proposed observation and file-management acceptance are in the
[roadmap](roadmap.md).

Release hashes identify bytes, not authorized spool lineage or upgrade
compatibility. Keep publisher-scoped logical identity, immutable release identity
and host instance/checkpoint identity separate. The
[identity research](spool-identity.md) must precede a new update schema.

## GUI boundary

A shuttle may use an application API or a GUI execution service. GUI capability
authorization, observation identity, target freshness and actual-effect readback
must remain explicit. Disconnecting or timing out a call does not establish that
an input action stopped. A GUI service is not interchangeable with a file-kit
backend solely because both execute requests.

## Minimal implementation shape

Begin with one Rust workspace and a small CLI. Establish module boundaries before
creating numerous public crates:

- `project`: parse and normalize authoring inputs and component bindings.
- `plan`: derive explicit generation, build, check and release plans.
- `adapters`: invoke existing coding tools and host CLI operations.
- `evidence`: validate bounded reports and their input/output identities.
- `release`: inventory, verify, assemble and export identified package bytes.

Domain templates and example projects live outside these generic modules.
Runtime state, delivery ledgers, provider credentials and VM controllers remain
outside the workbench.

## Migration and first acceptance

1. Establish the public project contract, README, architecture and contributor
   rules. Use the `an-loom` name consistently for the repository, package and CLI.
2. Port useful an-waver validation and archive code with its license and source
   attribution. Reorganize around projects and plans; avoid copying all historical
   commands or changing the host deployment format without demonstrated need.
3. Adopt an existing lightweight file silk as a project. Generate a reviewable
   change using one mature coding tool, invoke host-controlled build/check,
   receive verified output, assemble a release, register and run it through the
   real host. Record unsupported host capabilities as gaps.
4. Validate the same authoring-to-release route with an independently structured
   existing silk. This is the evidence for generality before adding a visual
   editor, provider catalog or a more elaborate workflow language.
5. Move required examples, CI references and design evidence. Then mark an-waver
   superseded, make it private and archive it. Preserve history; avoid deleting
   material needed to reproduce old releases.

The first milestone is complete when a developer can inspect and reproduce the
project-to-release route, replace its domain code without editing an-loom, and
consume the release without depending on the workbench. Supported refusal,
cancellation and acceptance behavior must come from the actual host contract.

## Platform policy

See [platform boundaries](platforms.md). Host, shuttle and workload targets are
independent. Local reviewed headless execution is the current delivery target;
future Linux profiles require independent qualification. A per-target binary
does not imply portable sandbox or GUI semantics. GPU/platform variants must be
explicit. Loom describes and validates requirements; the host grants execution
and the backend enforces the selected boundary.

## Support, coupling and invasion

The [log report acceptance](log-report-workflow.md) distinguishes generic support
from explicit adapter coupling and domain invasion. Host byte/version/execution
contracts support any admitted silk; document libraries and format rules stay in
the example. An optional CLI adapter may depend on a qualified host contract,
while the silk remains independently callable. New backend types are not required
to demonstrate this division.

## Implementation boundary audit

The developer/runtime division remains intentional: Loom has no runtime-crate or
model-client dependency; an-agent does not depend on Loom. The current build
handoff and file deployment profile are qualified adapter contracts, not a generic
runtime implementation. Domain source and acceptance rules stay in the project.

| Module | Owns |
| --- | --- |
| `project`, `manifest`, `package` | Editable declarations, include/path/secret rules, inventory, archive identity and verification |
| `files` | Bounded regular-file reads shared by authoring and packaging; no host protocol |
| `plan` | Pure development-request binding and explicit operator approval checks; no runtime grant |
| `adapters/host/development` | One approved build request and reception/declaration sequencing |
| `adapters/host/transport` | Explicit invocation of the operator-selected host CLI |
| `adapters/host/build_artifacts` | Qualified build-receipt validation and portable artifact reception |
| `adapters/host/file_profile` | Source signals, deterministic file-profile YAML, validation and deployment export |
| `release` | Exclusive local release delivery and identity checks; no host activation |

Public `build_artifacts`, `spool`, and `adapters::develop` paths remain compatible
facades. Existing profile strings, YAML bytes, release digests and CLI commands
are unchanged. The generic package code uses a deliberate offline profile-check
call; it never launches the host while inspecting or verifying an archive. The
profile adapter can in turn request package inspection for explicit authoring
operations. This is bounded implementation coupling, not a new extension system.

The file reader no longer lives inside build-evidence reception: general archive
and project operations should not depend on a particular host's receipt format
just to read a file. Conversely, moving a host schema behind an adapter does not
make its validation authoritative. an-agent must still validate/admit the release,
supervise execution and accept outputs under its own grants and settlement gates.

The host CLI adapter invokes trusted operator software. Its output cap is checked
after capture, and it is not an untrusted-agent sandbox or a competing cancellation
supervisor. Backend cancellation, termination evidence, artifact versions and
retention stay in an-agent. Native coding-agent context and logs stay with those
tools unless an explicit integration contract says otherwise.

Verification keeps the existing test target names and ignored real-host suites.
In addition to ordinary unit/contract checks, run the current Loom against the
current host through `tests/host_handoff.rs` before claiming coordinated acceptance.
The CI-pinned historical host remains a separate compatibility check; do not
rewrite its historical evidence or silently turn its pin into a moving branch.

## First release-bound native spool slice

Spool interaction uses workspace events and, when separately qualified, channels.
Business branches, loops, subtask delegation and result interpretation belong to
silk/kit. The host routes and authorizes event-driven attempts; Loom validates
interfaces and packages code without defining a global executable business graph.
The native v3 artifact-event adapter is a bounded foreground handoff, with no
background listener, GUI action or low-latency data plane. See native-spool.md.

The [native package adapter](native-spool.md) exports bounded shell-code releases
and explicit Python/binary requests with separately selected host environments
for an-agent's reviewed native profile. Host instances own mutable kit copies,
fixed-code attempts and custody. Loom does not manage running instances. This
adds no general workflow engine, dependency resolver or agent runtime.

The log-report example qualifies two headless spools exchanging fixed XLSX/PPTX
versions through workspace events. Document parsing stays in the example;
environment selection, execution and custody remain host-owned.

## Candidate-to-release custody

Loom freezes selected candidate bytes and their full release manifest before
execution. The host receives an explicitly reviewed profile, maps those inputs
into a read-only snapshot, supervises the process and accepts declared output.
Loom reconciles that retained ledger and publishes the same frozen archive.
Content identity, check request identity and host execution attempt are distinct.
Neither a model assertion nor a standalone operator success record substitutes
for the qualified execution evidence. See [the contract](candidate-checks.md).

## Planned session and retention coordination

See [the development handoff](session-retention-handoff.md) for the next bounded
work: host-owned artifact references and collection, then a headless session/thread
skeleton and explicit context selection. These are proposed extensions, not
additional Loom runtime responsibilities. GUI and TUI remain deferred.

## Context and work handoff principles

The [host architecture](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/architecture.md)
and Loom share four rules for future development/harness consumers:

1. Filter context by visibility before summarizing or paging it. Preserve source,
   exact revision, purpose, audience and read pointers; summaries inherit disclosure
   restrictions. These are design constraints, not a generic context router claim.
2. Distinguish context delivery, a work request and an execution grant. Subscriptions
   and project instructions cannot increase permissions. Workspace holds materials
   and facts; directed spool messages travel through authorized protocols.
3. Keep event handling ownership separate from authorization provenance. Who must
   handle, acknowledge or return information does not determine who may disclose
   it or execute a side effect. Correlation supplies context only.
4. Keep a continuing work goal and handoff state separate from each admitted
   attempt. Exact inputs/state and termination evidence belong to the host. A
   pending goal or recovery prompt cannot authorize retrying an unknown execution.

Weaver/harness adapters may prepare role prompts, paged context, human handoffs
and recovery inspection suggestions. Loom prepares and validates development
materials and packages the selected bytes. The host grants access, supervises
execution and owns state/artifact custody; no model loop or runtime supervisor
moves into Loom. An external application's discussion "Thread" is not an OS thread.

Design reference: [dsh-superagent-prototype at 6337fc8](https://github.com/deepseek-ai/dsh-superagent-prototype/tree/6337fc823ce20251c8de8085948165c81aaa701e).
Its domain/authorization/execution documents and effect-queue tests are useful
when implementing a matching need. Full-access execution and automatic restart
reassignment are not inherited guarantees. This is a reference, not a dependency
or code import. GitHub/Feishu integration is deferred and may later serve as an
optional capability test, not the primary delivery goal.
