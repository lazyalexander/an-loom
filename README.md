# an-loom

A Rust development workbench for reusable domain spools.

Loom helps developers turn domain knowledge, existing CLI programs, skills and
application integrations into inspectable releases. It owns authoring, assembly,
validation orchestration and packaging. Domain behavior belongs to spool projects;
an-agent owns runtime authority, model/context services, state and artifact
custody. Execution backends enforce isolation; shuttles connect applications.

The first slice migrates identified packaging and validation code from an-waver
and connects a lightweight CLI kit to the independent host. `loom.toml` binds
selected project paths. An operator reviews a development plan before the host
compiles and tests the selected Rust source. Loom receives matching output, emits
the fixed host declaration, and assembles a portable local release.

The implemented deployment adapter is deliberately narrow: one dependency-free
Rust source, one executable, `file-kit-v1`, and the host's existing macOS isolated
build. General packages may contain other languages and files. A new build or
runtime profile requires a separately qualified adapter and host support.
This is not yet a general multi-stage workflow executor or a code-generation agent.

## Quick start

```sh
cargo install --path . --locked
an-loom init ./my-kit
an-loom check ./my-kit
```

The starter is [the existing file-list kit](examples/file-list/README.md), not a
model-generated program. Its pure transformation and domain tests live in the
example, outside Loom's generic implementation. Give `generation.md` to an
existing coding tool when adapting it to your own domain.

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
a functional core / imperative shell design. All repository prose is English;
maintainers review and merge pull requests.

For agent-assisted development, start with [AGENTS.md](AGENTS.md) and the
[working guide](agent.md). The [research roadmap](docs/roadmap.md) covers spool
update identity, an existing workflow's file lifecycle and mature coding-tool
integration. These are planned acceptance slices, not supported runtime features.
