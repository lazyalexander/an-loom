# Native spool packaging and host handoff

This qualified adapter exports a reviewed macOS shell spool. It does not implement
runtime authorization, supervision, cleanup or a domain pipeline engine. The
legacy file-kit deployment adapter and its wire names are unchanged.

## Authoring

Use a generic `loom.toml` package with an explicit include allowlist. Include
`native-spool.toml` and every selected code file. The separate native contract has
exactly `schema = "an-native-spool-v1"`, `entry`, `silk`, `kit` and `outputs`.
See `examples/native-spool/` for two domain silk steps and one transform kit.
An entry is one declared silk script. Paths are one role directory plus a unique
ASCII basename; there are at most 12 files, 1 MiB each and 8 MiB total. Output
slots are bounded top-level text filenames. No source hook is implicitly executed.

```text
an-loom pack PROJECT
an-loom export-native-spool ARCHIVE NEW_DELIVERY
```

The exporter uses existing archive inventory, allowlist, secret and digest checks.
It emits only selected silk/kit bytes and `native-spool.json`; README, source build
files and authoring metadata are not copied into the execution package. Name and
version come from the verified package identity. The exact emitted manifest bytes
and selected file hashes determine the release digest. Existing output directories
are refused. The host parses and verifies this contract independently.

The export result records the source bundle digest. It grants no runtime rights,
proves no publisher identity and does not resolve version ranges. This is an
explicit new profile, not a reinterpretation of the legacy `[kit]`/`[silk]` adapter.

## Host usage

Use an an-agent host with the `an-native-spool-v1` contract:

```text
an-agent-host spool-load DELIVERY NEW_INSTANCE
an-agent-host spool-show INSTANCE
an-agent-host spool-kit-update INSTANCE transform.sh EDITED_FILE
an-agent-host spool-kit-update INSTANCE transform.sh EDITED_FILE --approve DIGEST
an-agent-host spool-start INSTANCE run-1 INPUT_FILE 30
an-agent-host spool-start INSTANCE run-1 INPUT_FILE 30 --approve DIGEST
an-agent-host spool-accept INSTANCE run-1
an-agent-host spool-accept INSTANCE run-1 --approve DIGEST
an-agent-host spool-reclaim INSTANCE run-1
an-agent-host spool-reclaim INSTANCE run-1 --approve DIGEST
an-agent-host spool-unload INSTANCE
an-agent-host spool-unload INSTANCE --approve DIGEST
```

Each digest belongs to its immediately preceding preview, not another command or
instance. SIGINT/SIGTERM to the running host CLI request cancellation. The host
owns effective policy, selected-code snapshots, termination and custody. A fresh
load to another directory restores the kit baseline while retaining prior products.
Reclaim affects work scratch; release, input evidence and formal products are not
implicitly deleted. Outputs remain in the per-run host artifact catalog.

Kit edits are allowed between runs. Runtime code snapshots are read-only. Domain
scripts use `AN_SPOOL_SILK_DIR`, `AN_SPOOL_KIT_DIR` and `AN_SPOOL_INPUT`; their current
directory is writable scratch. This is not live agent self-modification or strong
containment for malicious native code. Pause, suspend, weaver execution, dependency
resolution and automatic retention are not implemented by this adapter.

## Acceptance and next ownership

`cargo test --test native_spool --locked` checks deterministic exclusive export and
invalid role/output bindings. The ignored real host test requires a qualified
`AN_AGENT_HOST_BIN`; it exercises actual CLI load, kit edit, native execution,
acceptance, reclaim, unload, baseline reload and SIGTERM cancellation. No fake host substitutes for that
claim. CI pins the host implementation separately from legacy acceptance profiles.
Optional `LOOM_NATIVE_EVIDENCE` selects a private JSON evidence file; it can contain
local paths and must not be committed publicly.

Loom may later prepare bounded weaver edit tasks and resolve declared dependencies.
The host must still grant and enforce execution, checkpoint, artifact and shuttle
access. Supporting a code tool must not move its model loop into Loom or grant it
unrestricted host CLI access.
