# Spool identity and updates: research decision record

Status: research and proposed direction, 2026-10-24. No new manifest fields,
publisher verification, upgrade operation or signature support are implemented
by this document.

## What the current implementation establishes

`src/manifest.rs` binds package name, version, description and a sorted file
inventory to a release digest. `src/release.rs` also checks archive, program,
source and declaration hashes. This detects content mismatch within the local
release contract. It does not authenticate a publisher, reserve a namespace or
prove that two releases belong to an authorized update lineage. The version
validator accepts a bounded numeric form; do not claim full SemVer support.

## Four different questions

| Question | Proposed evidence | What it cannot establish alone |
| --- | --- | --- |
| Is this the same exact release? | Verified inventory and payload digest | Publisher identity or behavioral safety |
| Is this an authorized successor of the same spool? | Stable publisher-scoped identity, authorized release relation and verified authority | Compatibility of domain behavior |
| Can this instance upgrade? | Runtime/input/output contracts, state migration evidence and permission review | Identical future model responses |
| Is this the same running instance? | Host instance identity and committed checkpoint lineage | Equality of release bytes |

The stable identity should survive an ordinary release, repository rename or
approved publisher-key rotation. A copied identifier or parent hash is only a
claim until checked against trusted publisher/registry authority. An authorized
fork receives a new identity and may declare its origin; it does not silently
inherit update rights over the original instances.

## Reuse and limits

- [SemVer](https://semver.org/) communicates compatibility intent for a declared
  public interface. It cannot decide domain equivalence, and it is not update
  authorization. Define the spool's public contracts before interpreting a bump.
- [Package-URL](https://tc54.org/purl/) supplies a standard package-coordinate
  vocabulary. Study its namespace/name/version model rather than inventing an
  incompatible locator or an unregistered ecosystem type. Coordinates are not
  authenticated publisher credentials.
- [SLSA build provenance](https://slsa.dev/spec/v1.2/build-provenance) separates
  artifact subjects from build definitions, dependencies and run details.
  Reuse that separation for evidence. Existing Loom hashes do not claim SLSA
  conformance or a trusted build attestation.

Start with an operator-owned local authority record and digest-pinned releases
for the first acceptance. Before remote update discovery, evaluate an existing
signature/update-metadata system, including key rotation, revocation and rollback
protection. Do not introduce an unreviewed home-grown signing protocol.

## Proposed responsibility split

Loom validates authoring metadata, computes inventories and attaches identified
checks and proposed lineage. Publisher-controlled release infrastructure supplies
authorization evidence. The host independently validates admission and selects a
release for an instance. The spool supplies domain compatibility and migration
rules. A backend snapshot is neither a publisher credential nor a state schema.

Do not treat a newer version or the same logical identity as permission to expand
capabilities. Compare effective requests, entrypoints, dependencies, agent
instructions, tools and network requirements. An LLM-generated compatibility
report remains advice; domain tests and the operator's upgrade policy decide.

An instance upgrade must bind the old release/checkpoint and the proposed new
release, with explicit state-schema preconditions. Migration executes in a fresh
authorized attempt; failure preserves the old committed checkpoint. Switching
code back after a successful migration does not automatically reverse state or
external effects. Downstream consumers continue to pin accepted artifact
revisions, independently of a producer upgrade.

## First acceptance before adding a general version service

1. Publish two different releases under one locally authorized logical identity.
2. Refuse a different publisher, an unauthorized fork and reused version labels
   with different digests; permit a documented authority transfer only by policy.
3. Show a rename changes neither logical identity nor existing pins.
4. Require review of a capability increase and reject a stale instance checkpoint.
5. Demonstrate one compatible update and one failing state migration with the
   old committed state retained. Do not infer arbitrary behavioral equivalence.

Open decisions: logical-ID encoding, authority storage, dependency lock format,
multi-platform release variants, key management and migration evidence format.
Keep these research questions separate from the current package schema.
