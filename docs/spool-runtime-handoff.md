# Spool release-to-host handoff: first implementation slice

Status: proposed wire contract. No new runtime guarantee is implemented by this
specification. The current native v1 reader/exporter remains unchanged.

## Problem observed in the current implementation

Native authoring records runtime, inputs, editable kit paths, capabilities and
dependencies, but export-native-spool drops these from the execution manifest.
The host supplies a fixed reviewed macOS shell profile. This is sufficient for
its qualified fixture, but cannot represent a generally loadable spool contract.

## Ownership

Loom validates and exports requested requirements. The host independently checks
the complete selected release and decides admission and execution grants. Neither
manifest instructions nor package validation grant authority. Domain acceptance
stays in the spool. Platform-specific launch and containment stay in backends.

## Proposed native v2 extension

Retain the v1 fields and add one required `requirements` object for v2:

| Field | Meaning | Initial qualification |
| --- | --- | --- |
| runtime | Selected execution profile identity | macos-native-process-v1 |
| os | Target operating system | macos |
| arch | Target CPU architecture or explicitly portable source | portable-source for the reviewed shell fixture |
| interaction | headless or external-application | headless |
| gpu | none or required, with provider/runtime constraints | none |
| inputs | Declared input slots | One text slot for the existing host route |
| editable | Release-relative kit paths allowed as working copies | Subset of selected kit payload |
| capabilities | Requested effects | Only the effects actually supplied by the fixed profile |
| dependencies | External runtime dependencies to resolve before execution | No unresolved additional dependencies |

A concrete encoding and vocabulary must be frozen with producer/consumer tests
before implementation. Do not infer OS/architecture/GPU capabilities from package
names. Do not add an independent YAML authority file: any human-readable YAML
must derive from, or be explicitly bound into, this one content-identified
contract. Labels alone do not establish compatibility or dependency availability.

## Admission and execution

Validate schema, roles, paths and complete inventory before creating an instance.
Validate target environment and required capabilities before launching. Unknown
required capabilities, unsupported GPU/interaction requirements and unresolved
dependencies must refuse explicitly, never silently fall back to native access.
Show requested requirements separately from effective grants. Recheck instance,
release, inputs, kit revisions and execution environment when approving a start.

Headless and GPU are independent dimensions. A GPU CLI may run directly on a
qualified compute backend. A shuttle mediates authorized external application or
document interaction, whether or not that application uses a GPU. A microVM is
an optional backend; GPU availability must be qualified for that actual backend.
An external job needs its own settlement evidence; local process exit is not
proof that an application or remote worker stopped.

Linux must reuse release/instance/attempt identities and artifact custody. Keep
macOS paths and launch policy in platform adapters. Unsupported Linux execution
must remain unsupported until tested; portable packaging does not imply portable
execution. The first slice adds no new backend or dependency installer.

## Compatibility

Keep native v1 export/load and existing CLI behavior available. Use explicit v2
selection for new requirements; do not reinterpret historical v1 bytes or invent
missing permissions. A v1 host must reject v2. V2 requirements participate in
release identity, so permission changes cannot retain an old release identity.
Never automatically inherit an old execution approval for a changed release.

## Coordinated implementation and acceptance

1. Freeze a small public valid/invalid v2 fixture set and matching producer/host
   tests. Keep both repositories runtime-independent.
2. Add explicit v2 export in Loom and strict v2 validation/admission in the host;
   preserve v1 fixtures and deterministic identities.
3. Run a real host loop with the same narrow text spool: export, load, preview,
   approve, execute and accept fixed-version output.
4. Verify refusal for OS mismatch, unresolved dependency, unknown capability,
   protected editable path, GPU requirement and external-application requirement.
   Refusal must not launch or publish output. Verify changed requirements
   invalidate prior release/approval identity.
5. Publish paired PRs with pinned cross-repository CI commits and actual evidence.
   Do not mark the first slice complete until this execution acceptance passes.

Loom's current shell check route has separate 15-project-file and 30-second
limits; do not expand them to accommodate a new workflow during this slice.
