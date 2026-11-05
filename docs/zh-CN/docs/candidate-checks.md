# 候选项托管检查

[English](../../candidate-checks.md) · 中文译文，以英文为准。 · [中文目录](../README.md)

这是一个资格本地壳项目路线，不是一般构建执行器或认证远程验证。操作员拥有任务权限和宿主账本。编码代理仅接收其分配的工作副本，而不是存储，也不是无限制的宿主CLI。

## 所有权和身份

1. 云朵接受一个可编辑的Kit替换，针对完整的项目基准。
2. `check-task` 冻结所有选定文件、模式和manifest为候选档案。它创建一个新鲜的UUID检查请求，一个完整的发布摘要，一个档案摘要，一个runner和一个profile。这些是不同的身份。
3. 操作员在授予宿主执行之前审查profile和接受命令。宿主将候选树的完整内容以只读方式快照，监督过程并记录自己的尝试UUID、计划和执行摘要、退出状态和进程组终止观察。
4. 操作员通过宿主正式接受`checks.txt`。宿主将该不可变输出绑定到尝试接收和profile。
5. `receive-check TASK HOST_JOB` 验证链路是否符合冻结的计划。
   `pack-task` 重复检查并逐字复制冻结的档案。它还为后续开发创建一个独立的可编辑项目副本。源项目和历史发布保持不变。

在Kit内部不添加第二权威或过程监督者。Loom拥有候选选择和发布；宿主拥有授权、执行、终止和制品托管。领域命令及其正确性属于项目作者。一个测试如果谎报或遗漏了需求仍然是一个糟糕的项目测试：执行证据无法证明领域完整性。

## 操作员序列

从一个已接收候选且未被撤销的任务开始：

```sh
an-loom check-task ./task
# Review authority/check-plan.json, check.sh and host-profile.json.
# The profile contains local paths and must not be published.
an-agent-host workspace-run ./task/authority/host-profile.json ./check-job
an-agent-host workspace-run ./task/authority/host-profile.json ./check-job --approve REVIEWED_DIGEST
# Use the host attempt ID printed by execution, not the Loom check request UUID.
an-agent-host workspace-accept ./check-job HOST_ATTEMPT
an-agent-host workspace-accept ./check-job HOST_ATTEMPT --approve REVIEWED_ACCEPT_DIGEST
an-loom receive-check ./task ./check-job
an-loom pack-task ./task ./checked-project
```

最终命令返回了`checked-project/checked-release.tar.gz`。这是定义了检查候选项的相同档案。正常`pack PROJECT`仍是一个未上闸的打包操作；它并不声称项目检查已运行。

命令以完整的候选快照作为当前目录执行。`AN_LOOM_WORK`指向可写入的宿主临时目录。需要生成文件的测试必须在此处写入；它们不能覆盖所选候选。固定运行器会通过`/bin/bash -eu -c`执行每个接受命令，一旦失败就会停止，并且只有在每个命令成功退出后才会记录完成状态。领域测试必须自身传播失败（例如，当需要启用`pipefail`时）。`checks.txt`捕获其组合输出和完成标识。这是一项检查产物，而不是声称spool的生产输出是生成的。

## 接入与发布门禁

- 全局基准、候选、冻结档案、运行器和配置文件必须仍然匹配。
- 每个选定的文件都绑定到宿主快照，而不仅仅是编辑的打包器。
- 最多只能保留一次保留宿主尝试。一个被恢复、取代或回收的作业不能被此适配器使用。没有自动重试。
- 一个失败或取消的退出、未知终止、需要清理的组、缺失的宿主批准、不匹配的执行映射或缺失的正式接受会失败。
- 一个较旧检查的成功的作业不能满足一个新的检查请求，即使两个请求具有相同的发布字节。
- 已接受的检查输出从其固定的宿主镜像版本进行验证，包括接收/配置文件的来源。手写v1证据目录不足以满足要求。失败的准备会留下证据用于诊断；应创建一个新的任务，而不是修改已提交的检查请求。

保持源基准、任务权威和保留宿主作业直到发布完成。这些是本地可信存储，而不是对恶意操作员或同用户恶意修改的密码学防御。哈希值不是签名。

## 兼容性和资格范围

包含仅 schema、entry、silk、kit 和 outputs 的遗留 `an-native-spool-v1` 合同仍可检查、打包、验证和导出。它们的字节不会重写。在创建编码任务之前，必须显式指定作者字段；Loom 不会发明编辑范围或接受命令。部分作者元数据被拒绝。现有 v1 部署 JSON 和 CLI 名称保持不变。 provisional v1 候选检查计划/证据必须重新生成为 v2：接受它作为等价物将保留原始缺陷。

执行适配器使用了限定的宿主工作区v2 ([平台矩阵](platforms.md))，并显式指定 `input_layout`，最多15个选中的项目文件加上一个运行器，一个30秒的墙时间限制，没有额外的运行时读根和网络连接。更广泛的包格式仍然保持独立支持。此路径适用于已审查的本地工具，不进行脱机操作，而不是一个未信任的代码代理沙箱。依赖项未安装，模型未被调用，且项目钩子未被隐式运行。

## 接收证据

`tests/candidate_snapshot.rs` 用于验证完整内容与请求身份是否一致，以及过期的silk变更、冻结字节的变异、撤销和遗留兼容性。
`tests/task_handoff.rs` 拒绝裸操作符的成功记录。
`tests/native_spool.rs::candidate_check_uses_real_host_execution_before_packaging` 使用实际的macOS宿主并验证：

- 仅对替换套件有效域检查通过；
- 尝试写入候选快照被拒绝；
- 语法错误的候选失败，尽管原始源码有效；
- 无正式输出接受的退出成功不足以满足条件；
- 缺失终止、取消/过期/不匹配的凭证以及较旧的作业失败；
- 实际取消解决了组别，并且无法授权发布；
- 最终归档与冻结归档完全一致，后续快照的修改将被拒绝。

使用 `AN_AGENT_HOST_BIN` 选择协调宿主构建：

```sh
cargo test --test native_spool --locked -- --ignored --nocapture
```

`LOOM_CANDIDATE_EVIDENCE` 可选地保留私有观察 JSON。候选者为确定性测试替换；此接受并不声称实现了真实的编码模型集成。
