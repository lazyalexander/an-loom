# 显式状态日志累加器

[English](../../../../examples/log-accumulator/README.md) · 中文译文，以英文为准。 · [中文目录](../../README.md)

这个无需依赖的无头CLI工具将选定的日志批次与选定的检查点结合。它原样复用日志摘要转换器；`kit/accumulate.sh` 添加了四个经过验证的计数。Silk仅将输入连接到该领域代码。Loom和an-agent都不解释计数格式。

```sh
# From a new work directory; paths must identify the selected source files.
bash /selected/project/kit/accumulate.sh /selected/batch.txt /selected/initial-state.txt
# state.txt and report.txt are new outputs; tmp/ is private work.
```

状态中有精确的总/INFO/WARN/ERROR整数计数器，每个计数器在输入时最多为一百万，total等于其他三个计数器之和。未知级别、重复/缺失键或不一致状态将拒绝处理且不发布输出。受约束的业务规则和测试应放在这里，而不是在宿主或Loom中。

```sh
python3 -B -m unittest discover -s examples/log-accumulator/tests -v
an-loom check examples/log-accumulator
an-loom pack examples/log-accumulator
an-loom verify RETURNED_ARCHIVE
an-loom export-native-spool-v3 RETURNED_ARCHIVE NEW_DELIVERY
```

v3 编写合约声明了 `inputs = ["batch.log"]` 和一个单独的
`state_input = "checkpoint.txt"`。在运行时，域条目会接收到只读的
`AN_SPOOL_INPUT` 和 `AN_SPOOL_STATE` 快照路径。不会暴露 HEAD、状态目录或
上游工作区。现有的单输入发布保持不变。
操作员必须显式地使用 initial-state.txt 和声明的文本输出 state.txt 初始化实例。普通启动不能省略此发布的状态。

直接宿主选择使用 `spool-start-data-state INSTANCE RUN SELECTION SECONDS`，
其中 SELECTION 是严格的操作员 JSON：

```json
{"schema":"an-native-data-state-v1","data":{"catalog":"/selected/catalog","revision":"EXACT_DATA_REVISION"},"state_revision":"EXACT_COMMITTED_STATE_REVISION"}
```

或者用 `spool-event-deliver WORKSPACE EVENT SUBSCRIPTION --state EXACT_COMMITTED_STATE_REVISION` 交付一个已验收的产物事件。每个突变都需要自己的宿主预览/批准。没有隐式的最新输入、状态注入、提交或执行重试。正式接受、预期版本状态提交和事件消费完成是独立的决策。正常合格示例会在完成消费前提交状态；冲突不会静默地重做候选内容或丢弃其已接受的报告。

从仓库根目录运行配对的验收，使用已构建的二进制文件：

```sh
python3 -B tests/cli/native_data_state.py --loom /selected/an-loom \
  --host /selected/an-agent-host --host-revision FULL_HOST_COMMIT \
  --output /private/new-log-state-acceptance
```

它打包该项目，运行直接和事件绑定的数据/状态选择，检查陈旧提交冲突，并在收集后验证固定产品/检查点。宿主自身的套件拥有真实的取消、不确定交付和保留故障情况。不使用任何模型、GUI、后台工作流引擎或新后端。保持生成的机器路径和命令日志私有。
