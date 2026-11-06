# 与现有套件一起进行域名准备

[English](../../../../examples/weaver-log/README.md) · 中文译文，以英文为准。 · [中文目录](../../README.md)

该示例拥有其公共材料、域名指令和接受规则。`weaver.py` 负责准备并审核这些数据，然后通过 Loom 接收一个候选者。您的现有套件拥有模型选择、身份验证、对话历史、生成和取消操作。本适配器不导入、启动或监督任何套件。CI 不需要任何模型服务。

## 准备、生成和审核

一次构建 Loom。使用一个明确的、新的绝对审核目录，并在调用之前选择原生套件/模型和披露策略。这些命令演示了 Codex 的[非交互式接口](https://learn.chatgpt.com/docs/non-interactive-mode)；它们是操作员命令，而不是一个新的运行时适配器或一个包含声明。`CODEX_MODEL` 是一个操作员选择；示例不选择任何模型。

```sh
python3 -B examples/weaver-log/weaver.py prepare --loom /path/to/an-loom \
  --output /new/review-directory
export REVIEW=/new/review-directory
# Set CODEX_MODEL and review the native client's effective configuration first.
codex exec --ignore-user-config --sandbox read-only --skip-git-repo-check \
  --ephemeral --model "$CODEX_MODEL" --cd "$REVIEW/client" \
  --output-last-message "$REVIEW/DOMAIN.md" - < "$REVIEW/draft-prompt.txt"
# Inspect and, if needed, edit DOMAIN.md, materials.md and acceptance.sh.
python3 -B examples/weaver-log/weaver.py review --output "$REVIEW"
python3 -B examples/weaver-log/weaver.py approve --output "$REVIEW" --approve REVIEW_SHA256
codex exec --ignore-user-config --sandbox read-only --skip-git-repo-check \
  --ephemeral --model "$CODEX_MODEL" --cd "$REVIEW/client" \
  --output-last-message "$REVIEW/candidate.sh" - < "$REVIEW/prompt.txt"
# Inspect the complete script and its digest; no code runs during reception.
python3 -B examples/weaver-log/weaver.py receive --output "$REVIEW" --source-sha256 CANDIDATE_SHA256
python3 -B examples/weaver-log/weaver.py standalone --output "$REVIEW"
```

一个编辑器，另一个原生套件或人类可以提供相同的DOMAIN.md和candidate.sh。准备好的提示注入了选定的领域知识；套件不会成为领域规则的所有者。原生配置、插件和外部服务披露仍属于操作方的责任。只读壳政策不建立工具自由执行或阻止所有外部服务效果。提示请求不使用工具，但不执行强制。

审核绑定选定的材料、规范、检查和快照的Loom二进制文件。批准也绑定准备好的编码提示和精确的Loom任务。更改这些字节需要新的审核/任务。候选接收绑定操作方审核的摘要，并仅允许kit/transform.sh；Loom单独检查当前项目基准、撤销、常规文件限制和路径。候选字节无论由谁生成都不可信。接收不验证模型完成、作者或提供者政策。原生客户端的成功消息不能替代候选检查或宿主接受。

请保持原生请求/结果私密，并在任何显式新提交之前，使用该客户端检查不确定的运行。此适配器从不提交或重试模型请求，不读取原生会话存储，也不创建第二个对话。原生套件拥有其会话；宿主拥有管理执行和保管权。

## 独立和管理检查

独立阶段冻结接收的候选，使用操作方原生权限运行项目拥有的检查，验证快照并导出它。然后将导出的spool通过其现有的CLI/环境合同进行处理。超时保留证据，永不触发自动重试。原生进程退出不证明远程或分离进程的终止。

```sh
an-loom develop-native REVIEW/task /path/to/an-agent-host NEW_CHECK_JOB NEW_DELIVERY
# Inspect and approve that exact plan with --approve.
an-agent-host spool-start NEW_DELIVERY/instance demo INPUT 15
# Inspect and approve the start, then spool-accept for formal artifact custody.
```

## 迁移与证据

原来的 `draft`、`generate` 和 `--kimi` 命令已被移除。请使用 `prepare` 并直接调用您选择的原生套件。新的审核记录使用 v2；旧的目录将被拒绝，而不是重新解释。将旧目录与原已钉接的适配器一起完成，或者准备一个全新的目录。单独的旧 [kimi-native launcher](../kimi-native/README.md) 也已退役；其历史证据和修复的再现源仍可获取。

[历史 Kimi 总结](../../../../examples/weaver-log/acceptance/macos.json) 和 [审核源码](reviewed-example/DOMAIN.md) 保持不变。它们描述的是原来的适配器，而非 Codex 或 v2 接受的版本。[Codex v2 总结](../../../../examples/weaver-log/acceptance/codex.json) 记录了两个实际的原生 CLI 操作和相同的候选者通过独立检查、管理检查和正式运行时 artifact 接受。助理在检查生成的草案后选择了现有的审核域规范；不声称有任何新的人类审批或新的域决策。两个操作均未发出工具调用。这并非任意客户端工具、取消或远程终止的资格。CI 会使用外部提供的 fixture 字节来执行当前的审核门禁并验证 qualify_fixture.py。它拒绝受保护的路径，并证明 standalone 成功无法满足宿主托管。无需已安装的模型客户端或凭证。Linux 单独检查无法满足管理的 macOS 运行时的资格。
