# Internal silk development contract

Develop an internal silk for the owning spool. Reuse existing programs where
appropriate. Do not create a VM, select an execution backend, acquire host
credentials or widen the spool grant from domain code. Call external capabilities
only through the host-authorized interface. Internal subprocess completion is
not whole-spool termination. Legacy paths and host profile names below remain
compatibility tokens; preserve them until a separately reviewed migration.


Use an existing code-generation tool to edit kit/main.rs if requirements
change. The initial scaffold is deterministic, not a model-generated result.
Keep one dependency-free Rust source with a pure transformation and unit tests.
Read input.txt (UTF-8, at most 8192 bytes), trim blank lines, deduplicate and sort
lexically, and write output.txt using create-or-truncate. The host supplies a
fresh run directory and may pre-create that exact output file on Linux. Never
remove, rename or create other files to work around the host boundary.
Do not use network, environment secrets, subprocesses, or other paths. Source
inspection is incomplete evidence: the host must enforce the execution boundary.

For untrusted source, ask the operator to preview and approve the supported
an-agent-host build-kit profile. That isolated build remains macOS-only. Linux
file runtime support does not authorize unrestricted compilation of generated
code. An operator may explicitly compile and test reviewed source in a trusted
development environment and provide its matching portable build record; do not
forge an isolated-build receipt. Declare, pack, verify, export and register
through the existing CLIs after the applicable build checks.
Do not repair failed tests by skipping them or replacing accepted artifacts.
