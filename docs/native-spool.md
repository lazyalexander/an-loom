# Native spool packaging and host handoff

This qualified adapter exports a reviewed macOS shell spool. It does not implement
runtime authorization, supervision, cleanup or a domain pipeline engine. The
legacy file-kit deployment adapter and its wire names are unchanged.

## Authoring

Use a generic `loom.toml` package with an explicit include allowlist. Include
`native-spool.toml` and every selected code file. The authoring file has
`schema = "an-native-spool-v1"`, `runtime`, `entry`, `silk`, `kit`, `editable`,
`inputs`, `outputs`, `capabilities`, `dependencies` and `acceptance`.
`loom.toml` `description` carries the domain summary. See
`examples/native-spool/` for two domain silk steps and one transform kit, and
for `an-loom init-native`, `check`, `pack` and `verify`.

An entry is one declared silk script. Paths are one role directory plus a unique
ASCII basename; there are at most 12 files, 1 MiB each and 8 MiB total. Input
and output slots are bounded identifiers. `editable` is a subset of `kit`.
Capability names and acceptance commands are records, not grants and not
processes. Dependency values are unresolved labels: no URL, path or version
range is fetched. `check`, `pack` and `verify` refuse a missing entry, a path
listed in both roles, an escaping path, and a `silk/` or `kit/` file that is
not listed. They do not spawn acceptance commands or download dependencies.
`runtime` must be `macos-native-process-v1`; other runtime labels are refused.

```text
an-loom pack PROJECT
an-loom export-native-spool ARCHIVE NEW_DELIVERY
```

The exporter uses existing archive inventory, allowlist, secret and digest checks.
It emits only selected silk/kit bytes and `native-spool.json`; README, source build
files and authoring metadata are not copied into the execution package. The host
JSON keeps `schema`, `name`, `version`, `entry`, `files` and `outputs`. Runtime,
inputs, editable paths, capability requests, dependency labels and acceptance
records stay in the source package. Name and version come from the verified
package identity. The exact emitted manifest bytes and selected file hashes
determine the release digest. Existing output directories are refused. The host
parses and verifies this contract independently.

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


## Optional weaver and the two-instance acceptance path

The host provides a file proposal slot, independently of Loom. Request one kit with
`spool-weaver-request INSTANCE transform.sh`; forward only the returned data to an
external CLI adapter or an authorized shuttle. Import its proposal with
`spool-weaver-apply INSTANCE RESPONSE_JSON`, inspect the returned plan, then repeat
with `--approve HASH`. `spool-weaver-revoke INSTANCE REQUEST_UUID` prevents late
admission, but does not assert that an external tool has stopped. A proposal grants
no execution authority. Never give an untrusted agent the unrestricted host CLI.

The response schema is `an-weaver-proposal-v1` with `request` (the request UUID),
`request_sha256`, `kit` (the selected `kit/BASENAME`), and UTF-8 `content`. The host
binds it to the instance, release, original kit and run/edit generation. Changed,
revoked, duplicate, foreign or protected-path proposals are refused. Starting a run
requires a separate approval after application. The slot launches no model and
keeps model/context behavior outside Loom.

The real macOS acceptance now exercises this sequence using the actual release
exported from `examples/native-spool`, without a fake execution backend:

1. Load two independent instances of one release (two silks and one kit).
2. Import a deterministic file proposal into A, rejecting foreign and silk edits.
3. Run A and accept a versioned report; checkout that exact artifact revision.
4. Publish a newer report from A, refuse an old receipt claiming the new candidate,
   and run B against the earlier checked-out version.
5. Reclaim A's accepted scratch and unload it; load a fresh instance and verify the
   original kit behavior.
6. Hold B in real execution while the fresh instance completes another run; refuse
   B's unload, reclamation and weaver request while it is active.
7. Signal B's CLI, require confirmed cancellation, refuse cancelled output acceptance,
   accept B's successful downstream report before reclaiming it, and unload B.
8. Verify the pinned earlier artifact remains unchanged.

The test's file proposal is explicitly deterministic test input, not an LLM or a
claim of coding-tool security. `LOOM_NATIVE_EVIDENCE` retains identities, receipts,
fixed handoff revision, cancellation and independent execution evidence. This is
an operator validation route: run the ignored integration test with a qualified
`AN_AGENT_HOST_BIN`. Domain code is still packaged independently, and no runtime
Rust dependency connects Loom to an-agent.
