# Nextflow 生命周期和文件托管接受

[English](../../../../examples/nextflow/README.md) · 中文译文，以英文为准。 · [中文目录](../../README.md)

此操作者拥有的示例将现有的工作流引擎连接到宿主的已审核本地工作区配置文件。它不使用任何模型 API，不将工作流调度器安装在 Loom 中，并保留 Nextflow 原生任务缓存/恢复语义。此示例是一个通用的 Loom 内容包，而不是 `file-kit-v1` 发行版。

## 源代码和要求

- Nextflow `26.10.3`，[官方发行版](https://github.com/nextflow-io/nextflow/releases/tag/v26.10.3)，
  资产 `nextflow-26.10.3-dist`，SHA-256
  `e0d46f3274882e73abfb0672cf84b04c6f408b1882d11b51e50dcd533b707da6`。
- 一个兼容的 ARM JDK；原生接受使用 Homebrew OpenJDK 21.0.13.0.2。
  请参阅 [Nextflow 安装](https://docs.seqera.io/nextflow/install)。
- Python 3 标准库用于观察固定装置。它避免了额外的编排依赖；产品规划和宿主托管仍保持 Rust。
- 原生 macOS 允许 Seatbelt 激活，独立构建的宿主暴露 `workspace-run`，`workspace-resume`，`workspace-accept` 和
  `workspace-reclaim`。请参阅 [宿主合同](https://github.com/lazyalexander/an-agent/blob/fa8cc05e981e743bcc3cfc94fd04efd732827a6b/crates/an-agent-host/WORKSPACE-JOBS.md)
  来自 [配套宿主 PR #100](https://github.com/lazyalexander/an-agent/pull/100)。没有非沙箱回退。

`upstream-hello.nf` 与上游 `nextflow-io/hello/main.nf` 完全一致，
在 `3c2cdc9823c2b4636e5e3e73e223878099ff5dc9`；其 Apache-2.0 许可证保留为 `LICENSE.nextflow-hello`。`upstream-driver.sh` 仅将标准输出重定向到显式候选文件。`files.nf` 是一个独立的本地文件生成固定装置，而不是上游管道。它有两个准备任务和两个完成任务，每次只执行一个完成任务，以便在完成有用工作后可以中断。这建立了引擎/文件边界，而不是生产领域工作流质量。

## 显式运行

从执行中单独安装 Java 并下载固定发行版。在使发行版可执行之前检查 SHA-256。不要提交运行时、机器特定配置文件、生成的工作/缓存或原始执行日志。

```sh
python3 examples/nextflow/observe.py \
  --host /operator/selected/an-agent-host \
  --nextflow /operator/dependencies/nextflow-26.10.3-dist \
  --java-home /operator/dependencies/jdk/Contents/Home \
  --output .local/nextflow-acceptance
```

使用一个全新的输出目录。该 harness 是一个受信任的操作员接受驱动程序：
其显式调用授权其固定的 fixture 计划。它生成私有配置文件，获取宿主预览并批准其确切摘要。不要将其用作任意项目提供的配置文件或源代码的无人值守启动器。
运行时根目录是只读的，环境被清除且执行是离线的。

harness 运行未更改的上游 hello 工作流和本地文件 fixture，然后观察第二个文件 fixture 运行，同时一个结果已完成，另一个完成任务正在运行。在中断情况下，该任务创建一个就绪标记并等待超过宿主截止时间。观察者可以检查文件并验证主动回收拒绝，而无需与正常完成竞争。它向真实宿主 CLI 发送 SIGINT 并检查确认终止和部分输出的拒绝。显式后续任务保留就绪标记并使用 Nextflow `-resume`。其跟踪必须显示三个 `CACHED` 任务和一个 `COMPLETED` 任务。

声明的输出通过现有的产物修订目录接收。
harness 检出每个固定版本，显式放弃恢复并收集临时文件，然后再次检出相同版本并比较字节。
后续的恢复请求必须失败。不需要转录摄入或模型调用。
这不是一个自动删除/保留服务。

## 证据和打包

私有输出包括每个案例的配置文件、计划、意图、进程收据、完整的相对路径库存、本地跟踪观察、接受的修订绑定以及前后检出。并发库存是尽力而为的观察，不是一致的检查点；更改或消失的文件会被标记。
本地缓存符号链接会被记录，而无需遍历它们。原始日志和生成的脚本可以包含绝对路径并保持私有。

[观察报告](https://github.com/lazyalexander/an-loom/blob/main/docs/nextflow-observation.md) 描述了测量的文件类别、适应失败、运行时限制和剩余差距。公共接受 JSON 仅包含可移植的计数、断言、源/工具身份和显式选择的相对路径和输出摘要。

fixture 可以在没有 JVM 或 -agent 的情况下进行库存和打包：

```sh
an-loom check examples/nextflow
an-loom build examples/nextflow
an-loom pack examples/nextflow
```

这些命令会快照选定的源字节；它们不会运行Nextflow，也不会提供运行时访问权限。`observe.py` 结果是宿主接受的证据，而不是Loom运行时服务，也不是证明其他JVM工作流可以安全执行的证明。
