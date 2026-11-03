# Historical Kimi finite-MCP qualification

The provider-specific `run`/`cancel` driver and custom agent profile are retired.
New work uses the [shared finite-MCP fixture](../finite-mcp/README.md) with an
existing native client. Fixture preparation, host-result verification and reviewed
candidate acceptance remain maintained; model selection, authentication, client
startup and interruption belong to the native harness and its operator.

The current Codex route already has separate real development and cancellation
evidence. It adds no replacement launcher, SDK or process supervisor. Domain
source and tests stay in the kit. The host still owns finite grants, kit execution,
termination evidence and accepted artifacts. This retirement does not remove or
weaken the host MCP service.

## Existing operator directories

Use the [last maintained adapter and instructions at a fixed source revision](https://github.com/lazyalexander/an-loom/tree/ff984999b7146c48cb3c8e0e15b0cba364c61af1/examples/kimi-kit)
with the originally selected binaries to inspect an old Kimi-bound directory.
The shared fixture refuses client-bound settings rather than interpreting them
as a new native-client selection. Prepare a fresh directory for new work. Do not
resubmit an uncertain old request as a migration step.

## Evidence and coverage

The unchanged [portable Kimi acceptance](../../docs/acceptance/kimi-development.json)
records Kimi Code 2.3.1, K3 source/kit work, reviewed build/acceptance and K3-256k
active-kit cancellation. It remains evidence for that route, not qualification
of a new client or proof of remote model termination. The separate
[Codex acceptance](../../docs/acceptance/codex-mcp.json) records direct native MCP
use. Both records retain their original identities and limits.

Shared result checks remain in `examples/finite-mcp/tests/`: committed publication,
exact ungranted-handle refusal, candidate identity and rejection before build.
Credential-free MCP schema, capability, duplicate-invocation, cancellation and
transport-EOF regressions remain in the host's Linux/macOS CI. Removing a model
launcher removes its client process management; it does not replace those tests
with a prompt or a CLI exit code.
