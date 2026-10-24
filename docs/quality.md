# Repository quality gates

`cargo quality --base origin/main` runs the pinned SPACE architecture checker
through a small Rust operator adapter. The checker is a development dependency
in its own workspace; it is not linked into Loom or a deployed spool. Upstream
source and license remain at
[SPACE revision 57c7c0c](https://github.com/perplexityai/SPACE/tree/57c7c0c47a73373a974c5b542d180599795465da/crates/tools/archcheck).
This reuse adds no SPACE runtime or control plane dependency.

- `ARCHITECTURE.toml` caps production Rust files at 500 lines, confines new command
  construction to `src/adapters/`, and rejects vague crate names.
- `architecture-baseline.toml` starts with no exceptions. Required ratchets reject
  new exceptions, raised caps, growth and stale entries. Rename comparison is
  against the explicit Git base.
- `quality.toml` prohibits runtime dependencies on host/core/domain implementation,
  the architecture checker and direct HTTP orchestration. Loom delegates execution
  and models through explicit contracts rather than owning another runtime.

The wrapper fails closed on missing history, unknown policy fields and skipped
required checks. Policy changes require a separate reviewed migration. The initial
bootstrap recognizes only the two pre-policy repository trees, including an
equivalent merge/squash; it accepts no baseline exceptions. Future bases must
carry committed rules. It cannot silently bootstrap from a moving branch name.

Both Linux and macOS CI run the gate and checker tests. Mutation tests demonstrate
refusal of an oversized source, unmediated command construction and weakened
policy. The isolated checker also gets fmt/test/clippy. Runtime tests and the
existing real host/Nextflow acceptance jobs remain required by review discipline.
No branch protection setting is weakened by this change.

These are source structure and dependency checks, not semantic sandbox proofs.
Do not split a coherent file mechanically or add a baseline waiver to make a
feature pass. Review the responsibility boundary and move effects to explicit
adapters. Arbitrary domain payloads do not become Loom implementation merely
because they are packaged by Loom.
