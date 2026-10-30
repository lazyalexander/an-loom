# Test layout

Keep tests beside the contract they verify. Do not move domain algorithms or
operator observations into Loom's generic implementation to share test code.

| Location | Responsibility |
| --- | --- |
| `src/manifest/tests.rs` | Private manifest/path/secret/digest unit checks; imported only with `cfg(test)` |
| `src/package/tests.rs` | Private package inventory and archive unit checks |
| `tests/project_release.rs` | Stable Cargo integration-test entrypoint |
| `tests/project_release/authoring.rs` | Silk/legacy authoring and declaration ambiguity |
| `tests/project_release/approval.rs` | Development-plan identity and explicit approvals |
| `tests/project_release/analysis.rs` | Bounded source signals, including unsupported operations |
| `tests/project_release/release.rs` | Release/archive identity, tampering, portability and legacy bytes |
| `tests/project_release/support.rs` | Synthetic fixtures shared only within that integration suite; never executable acceptance evidence |
| `tests/build_reception.rs` | Host build-result reception and fail-closed source/evidence checks |
| `tests/support/native_development.rs` | Opt-in real single-submission check, delivery, load and independent execution; no model invocation |
| `tests/native_spool.rs` | Portable native export checks and opt-in real spool lifecycle CLI acceptance |
| `tests/candidate_snapshot.rs` | Full snapshot identity, mutation rejection and legacy native compatibility |
| `tests/support/candidate_host.rs` | Real candidate checks, readonly enforcement, cancellation and formal custody; called by native_spool |
| `tests/task_handoff.rs` | Native kit task preparation, candidate isolation, check evidence and revocation |
| `tests/host_handoff.rs` | Opt-in real macOS host acceptance; explicitly enabled in CI |
| `examples/log-report/tests/` | Domain document checks owned by the example |
| `examples/log-report/desktop/tests/` | Observation qualification checks; not real GUI acceptance |
| `tools/quality/tests/` and its test modules | Independently locked quality-tool checks |

`cargo test --locked` runs always-on Rust checks. It lists but does not execute
ignored real-host tests. `cargo test --test host_handoff --locked -- --ignored
--nocapture` requires the documented native host environment and real enforcement;
its absence is an error, not a passing substitute. Existing Cargo target names
remain stable so CI and operator commands keep discovering the same suites.

Use adjacent `tests.rs` or `tests/` modules for private unit tests instead of
large inline test bodies in production source. Use an existing integration target
with focused submodules when public behavior spans several modules. Keep shared
fixtures local to the suite; do not create a production helper module for them.
Retain assertions, ignored-test conditions and discoverability when relocating.

Generated builds, private profiles and machine transcripts belong in ignored
local directories. Portable historical acceptance reports remain under the
corresponding example's `acceptance/` directory: they are evidence, not tests or
fixtures, and must not be rewritten just to match a new test layout.

## Language and cross-repository ownership

Use Rust for internal invariants, archive/identity rules and private fault seams.
Prefer Python for new black-box CLI/process/filesystem scenarios; use TS for
browser integration where appropriate. Existing Rust acceptance remains valid;
migration is selective and requires equivalent assertions and measured benefit.
Do not add production APIs merely to expose private test seams to another language.
Place new external CLI suites under `tests/cli/` with a local command/prerequisite
README; this convention does not mean the existing suites have been migrated.
Domain suites remain under their examples.

Loom verifies authoring, frozen candidates, packaging and host-evidence reception.
The host verifies authorization, execution lifecycle, cancellation, recovery and
artifact custody. Paired acceptance verifies the release-to-host interface; it
must not grow into a duplicate of the host's full stress/failure matrix. Preserve
existing coverage until a replacement is demonstrated, including legacy formats.

## Binary preparation and verification

Build the selected product binaries in a preparation step and pass explicit
paths to external suites. Do not rebuild or use `cargo run` in each scenario,
except when building is itself the behavior under test. Reuse a binary only for
the same revision, target and build configuration. Record Loom and host source
revisions, binary digests, OS/architecture and relevant contract identities.
Historical host suites can pin different revisions; sharing one binary across
those suites is not automatically correct.

`cargo test --locked` does not run Python suites. Existing example checks include:

```sh
python -B -m unittest discover -s examples/log-report/tests -v
python -B -m unittest discover -s examples/log-report/desktop/tests -v
```

Install the corresponding documented example requirements first. Desktop
observation tests do not prove a real GUI edit. Run affected external suites and
explicit real-host acceptance in addition to the contributor checks, and report
unsupported platforms or skipped prerequisites rather than claiming a pass.

## Migration and performance evidence

Map old assertions/fault cases to replacement tests before removing a suite.
Preserve cancellation, fixed-version custody, stale-result rejection and cleanup
checks where applicable; compare both implementations during migration. Update
CI and documented entrypoints together without weakening required gates. Bound
external waits, retain failure output and clean up only owned resources after
termination is confirmed. A client timeout is not termination evidence and must
not trigger automatic resubmission of an unknown operation.

Measure compilation/linking and execution separately, distinguishing cold and
warm cache runs. Record platform, toolchain, revisions and repeated observations.
Changing test language does not remove compilation of the product or its
dependencies. Optimize measured bottlenecks; do not infer savings from a single
combined CI duration. Current workflows remain unchanged by this document.
