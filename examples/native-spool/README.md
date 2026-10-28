# Native text reference

This is the project a developer copies when the domain is a small text pipeline.
Sequencing and the report shape stay in two read-only silks. The transform
script is the one editable kit. Input is one selected text file. Output is
`report.txt`. Intermediate `transformed.txt` is scratch.

`loom.toml` `description` is the domain summary. `README.md` is the longer note.
`native-spool.toml` states roles, slots, capability requests, the editable kit
path, dependency labels, the qualified runtime and acceptance command records.
Those records are data. `check`, `pack` and `verify` do not spawn them, and they
do not resolve or download dependencies.

| Path | Role |
| --- | --- |
| `silk/main.sh` | Silk entry. Calls the kit, then the report silk. |
| `silk/report.sh` | Silk. Writes the `report.txt` shape. |
| `kit/transform.sh` | Kit baseline. The only editable working-copy path. |

Capability names are requests, not grants. This project requests `read-input`
and `write-declared-outputs`. The qualified host profile is still one selected
text file in and declared text outputs out. `runtime = "macos-native-process-v1"`
is the only runtime this adapter accepts. An empty `dependencies` list means
this project declares none.

Changing `kit/transform.sh` or the silk scripts is domain work. It does not
require a change to Loom. A silk change belongs in a new release. A kit change
can later be applied to a host working copy; this package does not do that by
itself.

## From an empty directory

```sh
an-loom init-native ./my-spool
an-loom check ./my-spool
an-loom pack ./my-spool
an-loom verify ARCHIVE
```

`pack` prints `archive`. Pass that path to `verify`. Both commands snapshot the
selected files. They do not run `acceptance`, project hooks or dependency
installers. `check` refuses a missing entry, a silk/kit path clash, a path that
escapes the project, and a file under `silk/` or `kit/` that the contract does
not list.

Packing and verification do not grant execution. Host loading, kit edits and
runs stay on the separately qualified `an-agent-host` profile. See
[the native contract](../../docs/native-spool.md). The optional weaver fixture
in that guide is deterministic test input, not a model integration.

`export-native-spool` copies the selected silk and kit bytes plus
`native-spool.json`. The host JSON keeps `schema`, `name`, `version`, `entry`,
`files` and `outputs`. Authoring fields stay in the source package.

## Bounded kit task

`task` writes an assignment in a new directory outside the project. The
workspace contains editable kit copies and a brief. Silk, `loom.toml` and
`native-spool.toml` stay protected. The brief grants nothing and includes no
host CLI.

```sh
an-loom task ./my-spool ./task
# A coding tool may edit only kit/transform.sh in ./task/workspace.
an-loom receive-task ./task kit/transform.sh ./replacement.sh
an-loom revoke-task ./task
```

`receive-task` copies the candidate under `task/candidate/` and leaves the
project and any release unchanged. It rejects a silk path, a stale project
baseline, a rewritten task contract, and any candidate that arrives after
`revoke-task`. These commands do not invoke a model. A passing test of this
gate is not a completed coding-tool adapter.

## Check evidence before packaging

See [candidate check custody](../../docs/candidate-checks.md) for the complete
operator sequence and its qualified scope. `check-task` freezes the complete
candidate project and prepares a nonexecuting host profile. Acceptance commands
run against its read-only input snapshot, with writable results in
`$AN_LOOM_WORK`. `receive-check TASK HOST_JOB` validates the actual retained host
attempt and formally accepted check output. Handwritten success JSON cannot
satisfy this contract. `pack-task` copies the already checked archive, rather than
selecting bytes again from the source project.

Legacy native v1 release contracts without authoring metadata remain readable,
packable and exportable. They cannot create coding tasks until the author adds
explicit editing and acceptance metadata. Old v1 candidate-check evidence must
be regenerated; it never established this stronger execution binding.
