# 历史 Kimi 原生提案资格认证

[English](../../../../examples/kimi-native/README.md) · 中文译文，以英文为准。 · [中文目录](../../README.md)

特定提供方的启动器已停用。新工作使用
[native-harness Weaver 示例](../weaver-log/README.md)，或直接使用现有的
`task`、`receive-task` 和 `develop-native` 命令。Loom 接收经过审核的候选字节；它不需要启动模型客户端或解析其渲染的回复。领域材料、指令和检查仍由 spool 所有。

已移除的适配器固定了 Kimi Code 2.3.1，启动了一个无需工具的 K3 请求，规范化了其 JSON 渲染并跟踪了本地生成结果。这些职责现在由操作员选择的原生 harness 承担。目前没有替代启动器、提供方 SDK、进程监督器或模型服务。

## 现有操作员目录

使用 [原始适配器和固定源版本的指令](https://github.com/lazyalexander/an-loom/tree/58f2668c69528b02fdb7b90b5fcdae525d75284c/examples/kimi-native) 来检查或完成一个旧目录及其原始选定的二进制文件。当前的 Weaver v2 路径不会重新解释其生成/批准文件。为新工作准备一个全新的任务。绝不要将不确定的旧模型请求作为迁移步骤重新提交；请先检查其原生证据。

## 证据和覆盖范围

未改变的 [便携式接受](../../../acceptance/kimi-native.json)
记录了实际的 K3 提案、领域检查、宿主准入、正式输出以及清理后的保留。它仍然是该确切路径的历史证据，而不是对较新客户端的资格认证或远程取消的证明。当前的 [Codex 资格认证](../../../../examples/weaver-log/acceptance/codex.json) 分别
记录了直接原生 harness 路径及其管理的产物接受。

| 已停用的适配器检查 | 当前责任和覆盖范围 |
| --- | --- |
| 解析 Kimi 的 JSON、项目符号和代码块渲染 | 随着解析器的移除；操作员在接收前审查候选文件 |
| 防止重复生成和拒绝未知生成提取 | 替代适配器中不存在模型提交或提取；原生结果需要操作员协调 |
| 拒绝未经审查的候选摘要 | `examples/weaver-log/tests/test_review.py` 在 Loom 被调用前测试拒绝 |
| 保持提案在可编辑的工具包内并拒绝过期/已撤销的任务 | `tests/task_handoff.rs`、`tests/candidate_snapshot.rs` 和当前的 Weaver 测试套件保留这些检查 |
| 通过宿主托管进行检查和交付 | 现有的真实 `host-handoff` 和 `native-spool` CI 任务仍需保留 |

单独的 [Kimi 有限工具资格认证](../kimi-kit/README.md) 也保留了宿主授权的 MCP 能力和取消的历史证据。新的有限工具工作使用 [共享原生 MCP 路径](../finite-mcp/README.md)；两个提供方启动器均已停用，但不会移除宿主能力边界。
