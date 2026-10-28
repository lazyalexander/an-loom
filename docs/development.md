# A lightweight CLI silk from development to release

## Requirements

Loom uses Rust 2024 and Cargo, with toolchain 1.98.0 pinned for qualification. Generic packaging and contract tests are portable.
The first controlled build adapter requires native macOS, an installed Rust
toolchain and Xcode or Command Line Tools. The operator supplies an independent
`an-agent-host` binary supporting `build-kit` and `file-kit-v1`. CI currently pins
host revision `deb3721e0429bd31f325ba3b55d8c646af1eacdf`, which removes
linker debug paths before accepting compiled programs. The corresponding host
PR must be reviewed before adopting this integration as the default.

No model API, model download, Docker, GUI application or an-agent runtime library
dependency is needed for this acceptance. Existing mature pipelines can be
packaged as explicitly selected files; executing their build systems needs a
qualified host adapter. Loom does not imply support for arbitrary Cargo build
scripts, dependencies or project commands.

## Author the project

Run `an-loom init PROJECT` to create a copy of the existing file-list example.
Develop in the copy; keep the checked-in example unchanged. The developer or an
existing coding tool edits domain code and its acceptance tests using the
generated `generation.md`. Generation is deterministic here; no live code-tool
or model run is claimed.

```toml
schema_version = 1
name = "file-list"
version = "0.1.0"
description = "A developer-owned CLI component"
include = ["loom.toml", "README.md", "kit", "generation.md"]

[silk]
source = "kit/main.rs"
program = "kit/program"
build_record = "kit/build.json"
declaration = "kit/spool.yaml"
analysis = "kit/analysis.json"
profile = "file-kit-v1"
```

The canonical authoring key is `[silk]`; old `[kit]` remains readable, but
using both is refused. The legacy `kit/` paths and `file-kit-v1` token do not
create a separate runtime boundary. See [the spool contract](spool-boundary.md).

These paths are explicit authoring bindings, not hardcoded source names. Every
binding must be inside the include list; files must be distinct and cannot
replace package metadata. The runtime ABI remains fixed: read `input.txt`, write
`output.txt`. The host maps those slots to a private run directory.

`an-loom init-package DIRECTORY` creates a generic package without a deployment
adapter. It can contain developer-owned non-Rust components. Package construction
snapshots bytes and does not execute them.

`an-loom init-native DIRECTORY` copies the reviewed native text reference.
Its `native-spool.toml` states silk and kit files, input and output slots,
capability requests, editable kit paths, dependency labels, the qualified
runtime and acceptance command records. `check`, `pack` and `verify` read those
records. They do not spawn the commands or download the labels. See
[the native reference](../examples/native-spool/README.md).

## Select the trusted host and toolchain

Keep operator configuration outside PROJECT. A local directory such as `.local/`
is ignored by Git and rejected as a package input. Use an existing directory for
the job's parent, and a new name for the job itself.

The host toolchain profile is JSON with concrete absolute `rustc`, `sdk` and
`linker` paths. Discover them locally with `rustc --print sysroot`,
`xcrun --show-sdk-path` and `xcrun --find clang`. `rustc` must identify the actual
compiler under that sysroot, not the rustup shim. Never commit the resulting
machine profile. Public acceptance includes only portable digests and outcomes.

## Plan, approve and build

```sh
an-loom check PROJECT
an-loom develop PROJECT HOST_CLI TOOLCHAIN_JSON NEW_JOB
```

Preview reads the selected inputs and requests the host's own build preview. It
creates no job, program or declaration. It invokes the operator-selected trusted
host CLI, never a command supplied by project metadata.

The result contains the exact host commands, environment, working directories,
policies, resource limits and an outer `approval_sha256`. Review those values,
then submit the same selection:

```sh
an-loom develop PROJECT HOST_CLI TOOLCHAIN_JSON NEW_JOB --approve REVIEWED_DIGEST
```

Loom reconstructs the plan and refuses a changed source, authoring contract,
toolchain profile or host identity. The selected host recompares its own approval
and owns the controlled compiler, tests, cancellation and evidence. The adapter
does not implement a second process/VM supervisor. CLI disconnects may leave a
settling host job; consult that job before doing anything else.

Successful host output is source-bound and received into fresh program/build
record destinations. Loom then emits a deterministic YAML declaration and a
bounded source report. The report explicitly records unknown native behavior;
it is not whole-program effect inference. A source alias or macro cannot grant
permissions that the host does not enforce.

Failure never triggers automatic regeneration, reexecution, receipt fabrication
or package publication. A completed host job whose reception was interrupted can
be inspected and handed to `an-loom receive-build PROJECT HOST_JOB`, then
`an-loom declare PROJECT`. Partial destination files require inspection and fresh
destinations. Do not resubmit an ambiguous job automatically.

The profile accepts one std-only Rust source. It does not provide a Linux isolated
compiler, dependency installation or an arbitrary workflow executor. Unsupported
host profiles refuse execution.

## Assemble and verify a portable release

```sh
an-loom release PROJECT NEW_RELEASE_DIRECTORY
an-loom verify-release NEW_RELEASE_DIRECTORY
```

The local delivery contains:

```text
release/
  release.json
  spool.tar.gz
  deployment/
    spool.yaml
    program
```

`spool.tar.gz` carries the selected source, program, domain instructions and
portable evidence, plus its bundle inventory. `release.json` binds that archive
and the fixed deployment to source, program, declaration and package identities.
It contains no host job paths, plans, diagnostics or credentials. Verification
rechecks archive contents and the source/build/declaration relationship, compares
deployment bytes and refuses unregistered delivery entries.

Releases can be moved and verified without an installed host. A passing release
check identifies bytes; it does not grant registration, certify domain quality
or prove the binary was produced by an authenticated compiler.

## Register and run through the host

```sh
an-agent-host check RELEASE_DIRECTORY/deployment
an-agent-host register RELEASE_DIRECTORY/deployment HOST_STORE
# Use the host-returned deployment digest, not the archive or bundle digest.
an-agent-host run HOST_STORE DEPLOYMENT_DIGEST INPUT_FILE
# Review the host's runtime plan before authorizing execution.
an-agent-host run HOST_STORE DEPLOYMENT_DIGEST INPUT_FILE --execute
```

The host accepts the output and retains its receipt/audit. The file-list example
trims, deduplicates and sorts lines; the host does not implement that behavior.
The original input remains unchanged. No external application effect, rollback,
VM snapshot, general state migration or automatic garbage collection is claimed.
