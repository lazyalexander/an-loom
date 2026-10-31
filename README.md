# an-loom

A Rust development workbench for reusable domain spools.

Loom helps developers turn domain knowledge, existing CLI programs, skills and
application integrations into inspectable releases. It owns authoring, assembly,
validation orchestration and packaging. Domain behavior belongs to spool projects;
an-agent owns runtime authority, model/context services, state and artifact
custody. A spool execution attempt is the isolation and lifecycle unit. Internal **silks**
provide stable core steps; editable **kit** working copies provide bounded glue
within the same granted environment. OpenShell is an optional,
replaceable backend; shuttles connect applications through host authorization.
See [the boundary, implementation audit and migration](docs/spool-boundary.md).

The first slice migrates identified packaging and validation code from an-waver
and connects a lightweight CLI silk to the independent host. `loom.toml` binds
selected project paths. An operator reviews a development plan before the host
compiles and tests the selected Rust source. Loom receives matching output, emits
the fixed host declaration, and assembles a portable local release.

The implemented deployment adapter is deliberately narrow: one dependency-free
Rust source, one executable, `file-kit-v1`, and the host's existing macOS isolated
build. General packages may contain other languages and files. A new build or
runtime profile requires a separately qualified adapter and host support.
This is not yet a general multi-stage workflow executor or a code-generation agent.

The [spool code contract](docs/spool-code.md) distinguishes **silks**, the stable pipeline
skeleton and core domain code, from **kits**, adaptable internal glue. Published
packages remain immutable; agent edits apply only to authorized working copies.
This vocabulary does not add runtime enforcement or change legacy CLI/API tokens.

## Quick start

```sh
cargo install --path . --locked
an-loom init ./my-spool
an-loom check ./my-spool
```

The starter is [the existing file-list silk](examples/file-list/README.md), not a
model-generated program. Its pure transformation and domain tests live in the
example, outside Loom's generic implementation. Give `generation.md` to an
existing coding tool when adapting it to your own domain.

The native text reference uses the same check and pack commands:

```sh
an-loom init-native ./my-spool
an-loom check ./my-spool
an-loom pack ./my-spool
# verify the archive path printed by pack
an-loom verify ARCHIVE
```

See [examples/native-spool/README.md](examples/native-spool/README.md). Packing
this profile does not grant host execution.

The [explicit v3 event interface](docs/native-spool.md#explicit-v3-event-interfaces)
packages workspace-event inputs/outputs for a qualified host. Subscriptions,
delivery authorization and consumption evidence stay in an-agent; Loom adds no
business graph engine or event service.

The [candidate check route](docs/candidate-checks.md) freezes all selected project
bytes, checks them through an explicitly authorized host profile, and publishes
the same archive only after reconciling settled execution and accepted output.
Legacy native release formats remain supported; check evidence is versioned
separately.

For the controlled development-to-release route:

```sh
# HOST_CLI and TOOLCHAIN_JSON are trusted, operator-owned paths.
# NEW_JOB stays outside the project's package allowlist.
an-loom develop PROJECT HOST_CLI TOOLCHAIN_JSON NEW_JOB
# Inspect the exact host commands, policies, environment, limits and inputs.
an-loom develop PROJECT HOST_CLI TOOLCHAIN_JSON NEW_JOB --approve REVIEWED_DIGEST
an-loom release PROJECT NEW_RELEASE_DIRECTORY
an-loom verify-release NEW_RELEASE_DIRECTORY
```

`release` creates `spool.tar.gz`, `release.json` and a `deployment/` directory.
It is a local delivery operation. The host separately validates, registers and
authorizes execution; no remote release or runtime grant is created implicitly.
See [the complete walkthrough](docs/development.md).

## Contracts and checks

- Explicit include lists, bounded regular files, portable paths and symlink
  rejection prevent accidental traversal into unrelated project files.
- Sorted inventories, deterministic archive metadata and SHA-256 identities
  bind package contents. Verification checks payloads, archive limits, gzip
  trailers and unexpected appended data.
- Known credential filenames, local configuration names and common secret
  patterns are rejected. Known current-user home paths in text or binary payloads
  are also refused. This is incomplete evidence, not a privacy certification.
- Host build reception checks successful status, source/program identities and
  a closed portable record. Machine profiles, private plans and diagnostics
  stay outside releases. Hashes are not signatures or compiler attestations.
- Declarations request capabilities. The host validates them independently and
  enforces restrictions even when source inspection cannot infer an effect.
- Development approval binds source, project configuration, toolchain profile,
  selected host executable and the exact host preview. Failed or uncertain
  submissions have no automatic retry or overwrite path.

The local directory model assumes operator-controlled project, job and release
directories. Concurrent hostile same-user replacement is not an authenticated
remote build protocol. A partial local delivery must be inspected and replaced
with a fresh destination; directory publication is not claimed power-loss atomic.

## Develop and review

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
# Explicit native macOS host qualification; missing host configuration is an error.
AN_AGENT_HOST_BIN=/operator/selected/an-agent-host \
  cargo test --test host_handoff --locked -- --ignored
```

Ordinary tests cover contracts and synthetic reception fixtures. Real host tests
are explicitly ignored there and mandatory in the separate `host-handoff` CI job;
they never silently pass because a host environment variable is absent.
See [acceptance evidence](docs/acceptance.md) and [migration provenance](docs/migration.md).

Loom does not link an-agent or import its implementation. CI pins the host source
revision and exercises the real CLI. The runtime does not depend on Loom.

See [the architecture](docs/architecture.md). This project is CLI-first and follows
a functional core / imperative shell design. All repository prose is English.
Agents may merge development PRs into `dev-advance` after review, applicable
acceptance and all CI gates pass, then notify the maintainer. Suspected privacy
exposure requires user review before publication or merge. Promotions into `main`
remain maintainer-reviewed and merged. See [CONTRIBUTING.md](CONTRIBUTING.md).

For agent-assisted development, start with [AGENTS.md](AGENTS.md). The [research roadmap](docs/roadmap.md) covers spool
update identity, an existing workflow's file lifecycle and mature coding-tool
integration. The [Nextflow observation](docs/nextflow-observation.md) now has real
normal/interrupted/resumed acceptance through a separately reviewed host profile.
Its [example package](examples/nextflow/README.md) leaves runtime authority with
the host; Loom only packages its selected sources. Other roadmap slices remain
planned, and the existing file-kit deployment adapter is unchanged.

A [qualified Kimi example](examples/kimi-kit/README.md) demonstrates finite host
kit calls, source proposals, separate reviewed builds and cancellation without
replacing the coding tool's native session. [Platform boundaries](docs/platforms.md)
distinguish control-host distribution, application shuttles and CPU Linux workflows.
Use `cargo quality --base origin/dev-advance` for development PRs and
`cargo quality --base origin/main` for stable promotions, matching the actual PR
target. Both enforce the repository's [TOML quality policy](docs/quality.md).

The [log report example](examples/log-report/README.md) exercises log → editable
XLSX → editable PPTX with a separately saved workbook revision, pinned downstream
consumption and cancelled binary-output refusal. The kit runs independently;
[the boundary report](docs/log-report-workflow.md) distinguishes support, explicit
adapter coupling and domain invasion.

The [implementation boundary audit](docs/architecture.md#implementation-boundary-audit) maps current modules to developer, host, and backend responsibilities and records the remaining limits.

## Reviewed native spool packages

The [native spool adapter](docs/native-spool.md) exports read-only silk code and
kit baselines for an-agent's bounded macOS instance lifecycle. The included
[lightweight text spool](examples/native-spool/README.md) needs no model or install
hook. Packaging remains independent from host authorization and execution.

The [log-report native handoff](examples/log-report/README.md#native-spool-event-handoff)
uses the same packaging path for two CPU/headless Python spools. An independently
selected host environment produces editable XLSX/PPTX and passes fixed binary
versions through workspace events; business adapters remain in the example.
The [log accumulator](examples/log-accumulator/README.md) combines a separately
selected business artifact and committed checkpoint. Loom packages its roles;
the host owns fixed versions, stale-state conflicts and cancellation retention.

The [native spool acceptance](docs/native-spool.md#optional-weaver-and-the-two-instance-acceptance-path) also covers the optional host kit proposal slot (with legacy Weaver tokens), two independent instances, fixed artifact handoff, cancellation and baseline restoration. Loom supplies packaging; it does not run the agent or own runtime permission.

The native starter includes a selected `AGENTS.md` describing file roles and
development boundaries. Follow [the six-round qualification status](docs/development-rounds.md)
for implemented evidence and the next real coding-tool handoff.

For a prepared native candidate, [develop-native](docs/develop-native.md) runs
the approved check-to-delivery path and loads the resulting spool through the
selected host. It reuses existing contracts and refuses automatic resubmission.

A [real Kimi native handoff](examples/kimi-native/README.md) demonstrates a
reviewed model proposal becoming a checked, loaded and independently executed
spool. Model calls are opt-in operator acceptance, outside Loom's runtime.

Weaver task preparation and independent versus managed spool operation are
described in [the Weaver contract](docs/weaver.md).
