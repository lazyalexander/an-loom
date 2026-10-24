# Acceptance scope and reproduction

The initial domain example is the existing file-list CLI kit. Its algorithm and
tests remain in `examples/file-list/kit/main.rs`. No generated binary, private
job directory or machine toolchain configuration is committed.

Ordinary `cargo test --locked` covers package/manifest contracts, failed or
tampered reception, secret/private metadata rejection, nondefault kit bindings,
portable release relocation, stale evidence, archive integrity and plan binding.
Some reception/release tests use explicitly synthetic program bytes and never
execute them; they do not qualify native compilation or domain correctness.

The separate native macOS host acceptance requires `AN_AGENT_HOST_BIN`. Run:

```sh
AN_AGENT_HOST_BIN=/operator/selected/an-agent-host \
  cargo test --test host_handoff --locked -- --ignored --nocapture
```

The positive test follows the actual CLI path: scaffold -> change a source path
binding -> preview -> refuse a stale approval -> approved isolated host build and
domain tests -> receive -> declare -> local release -> verify -> host check and
register -> runtime preview -> controlled execution -> accepted file artifact.
It verifies no preview job, no duplicate build, unchanged input, exact output,
reaped process, output hash and absence of the private job and current-user home paths in package payloads.

The negative native test deliberately fails a domain test. It checks that the
host failure occurred specifically at the test stage, retained a receipt, and
produced no received program, declaration or release. An environment that cannot
launch the native boundary cannot count as successful qualification.

These tests are explicitly ignored by the ordinary suite and explicitly selected
by the mandatory `host-handoff` CI job. Missing host configuration is an error.
CI builds public host revision `deb3721e0429bd31f325ba3b55d8c646af1eacdf` without
importing its implementation into Loom. This pin includes the separate host
build-debug-metadata fix; that host change requires its own review.

Set `LOOM_ACCEPTANCE_DIR` to a new operator-selected directory to preserve local
evidence. The closed `acceptance.json` summary can be shared; full host plans,
toolchain profiles, builds and audit logs stay local. The CI upload includes only
that summary. The summary is observed evidence, not a signed attestation.

No live model generation, third-party workflow compatibility, Linux isolated
build, whole-program safety proof or multi-stage runtime is claimed. Renaming
the source binding qualifies configurable layout, not an independent domain.

The [qualified local summary](acceptance/local-run.json) records the successful
macOS ARM64 route using the pinned host privacy fix. Nineteen ordinary checks
and both explicit native host tests pass locally; formatting and clippy pass.
Remote CI independently qualifies the PR commit. The first, nested execution
environment refused native sandbox creation, which was recorded locally and was
not counted as successful native qualification.
