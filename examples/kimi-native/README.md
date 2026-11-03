# Historical Kimi native proposal qualification

The provider-specific launcher is retired. New work uses the
[native-harness Weaver example](../weaver-log/README.md), or the existing
`task`, `receive-task` and `develop-native` commands directly. Loom receives
reviewed candidate bytes; it does not need to start a model client or parse its
rendered reply. Domain materials, instructions and checks remain spool-owned.

The removed adapter pinned Kimi Code 2.3.1, launched one tool-free K3 request,
normalized its JSON rendering and tracked a local generation outcome. Those
responsibilities now stay with the operator's selected native harness. There is
no replacement launcher, provider SDK, process supervisor or model service.

## Existing operator directories

Use the [original adapter and instructions at the fixed source revision](https://github.com/lazyalexander/an-loom/tree/58f2668c69528b02fdb7b90b5fcdae525d75284c/examples/kimi-native)
to inspect or finish an old directory with its original selected binaries.
The current Weaver v2 route does not reinterpret its generation/approval files.
Prepare a fresh task for new work. Never resubmit an uncertain old model request
as a migration step; inspect its native evidence first.

## Evidence and coverage

The unchanged [portable acceptance](../../docs/acceptance/kimi-native.json)
records the actual K3 proposal, domain checks, host admission, formal output and
retention after cleanup. It remains historical evidence for that exact route,
not qualification of a newer client or proof of remote cancellation.
The current [Codex qualification](../weaver-log/acceptance/codex.json) separately
records the direct native-harness route and its managed artifact acceptance.

| Retired adapter check | Current responsibility and coverage |
| --- | --- |
| Parse Kimi's JSON, bullet and code-fence rendering | Removed with the parser; the operator reviews a candidate file before reception |
| Fence repeated generation and refuse unknown generation extraction | No model submission or extraction exists in the replacement adapter; native outcomes require operator reconciliation |
| Refuse an unreviewed candidate digest | `examples/weaver-log/tests/test_review.py` tests refusal before Loom is called |
| Keep proposals within the editable kit and reject stale/revoked tasks | `tests/task_handoff.rs`, `tests/candidate_snapshot.rs` and the current Weaver fixture retain these checks |
| Check and deliver through host custody | Existing real `host-handoff` and `native-spool` CI jobs remain required |

The separate [Kimi finite-tool qualification](../kimi-kit/README.md) also retains
historical evidence for host-authorized MCP capabilities and cancellation.
New finite-tool work uses the [shared native MCP route](../finite-mcp/README.md);
both provider launchers are retired without removing the host capability boundary.
