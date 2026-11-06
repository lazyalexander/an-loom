# 从原生客户端使用现有的有限MCP服务

[English](../../../../examples/finite-mcp/README.md) · 中文译文，以英文为准。 · [中文目录](../../README.md)

Codex直接连接到`an-agent-tools`通过其原生stdio MCP支持。
官方`rmcp` SDK已经提供了服务器传输。无需Codex SDK、代理、代理循环、回复解析器或新的宿主工具。现有的文件列表 kit拥有域名代码和测试；共享提示选择任务。宿主拥有授予的kit执行、终止和正式产物发布的权限。

## 准备并检查现有的固定装置

使用原生的macOS，已构建的Loom/宿主/tools二进制文件和在[开发](../../docs/development.md)中描述的选定的Rust工具链配置。
现有的准备构建只生成公共排序kit和取消变体。此步骤中未选择或调用任何模型客户端。该装置仅暴露`prepare`、`check`和`accept`；它从不启动、等待或通知编码客户端。这使得没有用于维护的每个提供者的启动命令。

```sh
python3 -B examples/finite-mcp/qualify.py prepare --output /new/mcp-fixture \
  --loom /selected/an-loom --host /selected/an-agent-host \
  --bridge /selected/an-agent-tools --toolchain /selected/toolchain.local.json
```

检查 `development-approval.local.json` 和公共目录
[开发提示](../../../../examples/finite-mcp/development-prompt.txt)。将 `REVIEW` 作为新的绝对固定目录，将 `BRIDGE` 作为相同的工具二进制，将 `APPROVAL` 作为精确审查过的摘要，将 `CODEX_MODEL` 作为你选择的模型。保护固定目录免受并发编辑。保持原生认证在原始客户端存储中。

## 单次原生客户端调用

使用 Codex 0.166.0 的合格命令。这些是原生配置标志，而不是 Loom 提供者 API。对于不同的客户端/版本，需要检查其实际配置并重新合格，而不是假设标志的语义匹配。[历史 Kimi 路径](../kimi-kit/README.md) 保留其固定适配器和证据；其客户端绑定的目录不会由此固定目录迁移。

```sh
mkdir "$REVIEW/development-client" || exit 1
# Exclusive submission marker: do not remove it to replay an uncertain request.
mkdir "$REVIEW/development-submitted" || exit 1
codex exec --ignore-user-config --strict-config --sandbox read-only \
  --skip-git-repo-check --ephemeral --model "$CODEX_MODEL" \
  --cd "$REVIEW/development-client" --json \
  --disable shell_tool --disable multi_agent --disable plugins --disable apps \
  --disable hooks --disable browser_use --disable computer_use \
  --disable image_generation --disable view_image --disable memories \
  --disable skill_search --disable skill_mcp_dependency_install --disable goals \
  -c 'web_search="disabled"' -c 'project_doc_max_bytes=0' \
  -c 'skills.include_instructions=false' \
  -c "mcp_servers.host.command=\"$BRIDGE\"" \
  -c "mcp_servers.host.args=[\"$REVIEW/development-grant.local.json\",\"--approve\",\"$APPROVAL\"]" \
  -c 'mcp_servers.host.enabled_tools=["read_source","run_kit","propose_source"]' \
  -c 'mcp_servers.host.required=true' -c 'mcp_servers.host.tool_timeout_sec=45' \
  -c 'mcp_servers.host.default_tools_approval_mode="prompt"' \
  -c 'mcp_servers.host.tools.read_source.approval_mode="approve"' \
  -c 'mcp_servers.host.tools.run_kit.approval_mode="approve"' \
  -c 'mcp_servers.host.tools.propose_source.approval_mode="approve"' \
  --output-last-message "$REVIEW/development-last.txt" - \
  < examples/finite-mcp/development-prompt.txt \
  > "$REVIEW/development-events.jsonl" 2> "$REVIEW/development-stderr.txt" || exit 1
python3 -B examples/finite-mcp/qualify.py check --output "$REVIEW"
```

检查完整 `development-session/calls/edit-1/candidate.rs` 及其
摘要前，请先查看：

```sh
python3 -B examples/finite-mcp/qualify.py accept --output "$REVIEW" \
  --source-sha256 REVIEWED_CANDIDATE_DIGEST
```

只有三个命名的MCP工具会收到客户端的批准。服务器仍然检查精确的宿主授权、处理字段和调用身份。客户端批准不能编译提交的候选者或扩大这些能力。`check`阶段读取持久的宿主结果，而不是模型的最终消息。`accept`单独构建/测试、打包、注册和执行已审查的源码。其模型字段对于外部调用的客户端是未知的；选择一个原生客户端并不能证明候选者的作者身份。

## 证据和限制

未改变的[可移植接受](../../../acceptance/codex-mcp.json)记录通过现有宿主实际执行了Codex调用，一个未授权的处理字段拒绝，独立构建的候选输出和固定版本检查出错。第二次原生转义在观察到的Kit活动后被中断；宿主确认取消和收割，但未发布。早期接受的artifact仍然可读。其验证器摘要在共享固定位移动之前识别了[合格脚本](https://github.com/lazyalexander/an-loom/blob/5068ef8442795b42d99573052ec1476c79cef0ac/examples/kimi-kit/qualify.py)；退休不会重写该历史记录或声称另一个模型调用。

这与一个可信的原生客户端进行互操作，而不是对该客户端的限制。功能标志限制了偶然的工具；只读的shell策略和MCP允许列表并不能证明所有其他原生工具都不存在。公共源码和宿主工具结果进入所选模型连接。原生日志、计划和机器路径保持私密。没有原生会话存储被读取或复制。

取消固定使用单独审查的取消授权和一个新的原生调用。调用 `run_kit` 一次，使用调用 `cancel-1`，处理字段 `wait` 和 `wait`，然后跟一个换行符。仅在实际原生运行写入其 `pending` 标记后，才发送SIGINT。要求持久的工具结果和原生确认以报告取消、收割和未接受的artifact。CLI退出、超时或缺少响应单独不能是成功；永远不要重试不确定性。这验证了本地kit取消，而不是远程模型终止或回滚。

官方客户端参考：[MCP配置](https://learn.chatgpt.com/docs/extend/mcp) 和 [功能/配置参考](https://learn.chatgpt.com/docs/config-file/config-reference)。
