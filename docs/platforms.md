# Platform support and qualification

Loom owns authoring, checking and package handoff. The host owns runtime support;
use the authoritative [an-agent runtime matrix](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/platforms.md)
instead of duplicating that table here. These two documents are the platform
entry points. Feature contracts link here; historical acceptance reports keep
their original OS, architecture and revision evidence.

The current coordinated delivery target is local, reviewed **headless** spools.
Existing Linux implementations and CI remain intact, with no Linux compatibility
expansion in this phase. Linux deployment remains a possible future profile,
not a default that converts today's native releases or grants broader access.

## Loom and handoff matrix

| Surface | macOS | Linux | Other platforms | Evidence and limits |
| --- | --- | --- | --- | --- |
| Rust authoring, checking, deterministic packaging and quality tools | CI | CI | Unqualified | [CI jobs](../.github/workflows/ci.yml); this is package tooling, not execution isolation |
| Native spool manifest/export adapter | Contract tests and real host handoff | Offline contract tests | Unqualified | Existing shell/Python target identifiers remain unchanged; producing a package does not enable its target runtime |
| Controlled build/development and native spool execution acceptance | Qualified host lane | No new execution lane | Unqualified | Exact host revisions in CI; [acceptance map](acceptance.md) and [CLI cases](../tests/cli/README.md) |
| Domain programs used independently | Example-specific tests | Example-specific tests where configured | Example-specific | Ordinary CLI success has no managed host-execution guarantee |
| Application shuttles and GPU workflows | Separately qualified | Separately qualified | Separately qualified | Application APIs, GUI sessions, drivers and external services are outside Loom's package runtime |

Per-target binaries simplify distribution but do not reproduce sandbox, filesystem,
GUI or driver behavior. OS and architecture are explicit target requirements.
Unsupported requests refuse; no automatic CPU fallback, unrestricted host shell
or VM GPU passthrough is inferred. A workflow may request a separately qualified
GPU/service profile or an application shuttle, with host-owned credentials,
cancellation and artifact handoff.

## Where platform decisions belong

- Loom's `adapters/host` validates the selected, versioned host contract. Existing
  `macos-native-*-v1` runtime names, target `os` fields and evidence backend IDs
  are compatibility data, not generic domain decisions. Do not rewrite old
  releases or relax a receipt check to make its name appear portable.
- Host backend and filesystem adapters own enforcement and OS-specific APIs.
  Loom must not add native launch, lifecycle or file-custody policy to compensate
  for an unsupported runtime.
- Python CLI cases consult the local `tests/cli/host_qualification.py` before
  fixture creation or host invocation. This retains the current prerequisite;
  it is not a sandbox probe, authorization or a substitute for real acceptance.
  Rust OS-specific tests and CI runner selection remain explicit test entry points.
- Domain code stays in the project; source language is not a host capability.
  The same headless lifecycle interfaces do not imply that every backend can
  pause, suspend, recover memory or interact with a display.

The host runtime matrix is authoritative for execution. This matrix is
authoritative for Loom tooling and paired handoff coverage. A new target requires
adapter work, an explicit selected host revision, real evidence and coordinated
updates to both entry points. Never turn a skipped or unsupported test into a
qualified result, remove a check, or broaden a trusted profile to pass a lane.
No new platform registry, generic pipeline DSL or execution backend is introduced.
