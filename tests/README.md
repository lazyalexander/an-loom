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
| `tests/native_spool.rs` | Portable native export checks and opt-in real spool lifecycle CLI acceptance |
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
