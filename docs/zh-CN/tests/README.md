# 测试布局

[English](../../../tests/README.md) · 中文译文，以英文为准。 · [中文目录](../README.md)

将测试放在它们所验证的契约旁边。不要将领域算法或操作观察者移动到Loom的通用实现中以共享测试代码。

| 位置 | 责任 |
| --- | --- |
| `tests/cli/native_state.py` | 未改变的v1/v2导出的实测macOS宿主状态转移；显式构建的二进制文件，无宿主失败矩阵的重复 |
| `tests/cli/native_release.py` | 两个实际的v1/v2发布导出，宿主差异审查和独立状态延续；无继承或迁移声明 |
| `tests/cli/native_events.py` | 实际的v3接口包装和宿主事件交付/消耗；无宿主失败矩阵的重复 |
| `tests/native_events.rs` | 显式事件导出，降级拒绝和不支持的接口元数据 |
| `src/manifest/tests.rs` | 私有manifest/路径/秘密/摘要单元检查；仅导入于`cfg(test)` |
| `src/package/tests.rs` | 私有包库存和档案单元检查 |
| `tests/project_release.rs` | 稳定的Cargo集成测试入口点 |
| `tests/project_release/authoring.rs` | Silk/遗留作者和声明歧义 |
| `tests/project_release/approval.rs` | 开发计划身份和显式批准 |
| `tests/project_release/analysis.rs` | 有限的源信号，包括不支持的操作 |
| `tests/project_release/release.rs` | 发布/归档身份，篡改，可移植性和遗留字节 |
| `tests/project_release/support.rs` | 仅在该集成套件内共享的合成固定数据/检查点包装；从不作为可执行的验收证据 |
| `tests/build_reception.rs` | 宿主构建结果接收和关闭源/证据检查 |
| `tests/support/native_development.rs` | 选择性实单提交检查，交付，加载和独立执行；无模型调用 |
| `tests/native_spool.rs` | 可移植的原生导出检查和选择性实spool生命周期CLI接受 |
| `tests/candidate_snapshot.rs` | 完整的快照身份，修改拒绝和遗留原生兼容性 |
| `tests/support/candidate_host.rs` | 实际候选检查，只读执行，取消和正式保管；由native_spool调用 |
| `tests/task_handoff.rs` | 原生套件任务准备，候选隔离，检查证据和撤销 |
| `tests/host_handoff.rs` | 选择性实macOS宿主接受；在CI中显式启用 |
| `examples/log-report/tests/` | 由示例所有权的领域文档检查 |
| `tests/cli/native_documents.py` | 实际的v3包装和宿主绑定的Python/二进制文档转移；领域适配器仍留在示例中 |
| `tests/cli/native_data_state.py` | 实际固定数据/检查点包装和宿主转移；宿主拥有取消/故障覆盖 |
| `examples/log-accumulator/tests/` | 独立的累积日志业务验证，不在宿主和Loom实现之外 |
| `examples/log-report/desktop/tests/` | 观察资格检查；非真实GUI接受 |
| `examples/weaver-log/tests/` 和 `qualify_fixture.py` | 原生Harness审查，精确候选接收和保护路径检查；无提供者启动器或模型要求 |
| `examples/finite-mcp/tests/` | 共享有限-MCP结果检查：发布、精确拒绝、候选身份和拒绝遗留客户端绑定设置；无模型要求 |
| `tools/quality/tests/` 和其测试模块 | 独立锁定质量工具检查 |

`cargo test --locked` 总是运行始终开启的 Rust 检查。它列出但不执行忽略的实宿主测试。`cargo test --test host_handoff --locked -- --ignored
--nocapture` 需要文档化的原生宿主环境和实宿主强制执行；其缺失是错误，而非通过替代。现有 Cargo 目标名称保持稳定，以便 CI 和操作员命令能够持续发现相同的套件。

使用相邻的 `tests.rs` 或 `tests/` 模块进行私有单元测试，而不是生产源代码中的大内联测试体。当公共行为跨越多个模块时，使用已有的集成目标并聚焦子模块。将共享的固定值本地化到套件中；不要为它们创建生产辅助模块。在迁移过程中保留断言、忽略测试条件和可发现性。

生成的构建、私有配置和机器日志应存放在忽略的本地目录中。可移植的历史接受报告仍保留在对应的示例的 `acceptance/` 目录下：它们是证据，而非测试或固定值，必须不被仅为匹配新测试布局而重写。

已退役的 Kimi 原生启动器的解析器和提交测试是特定于不再存在的代码。其 [退休映射](../../../examples/kimi-native/README.md#evidence-and-coverage) 识别保留的候选者、过时/撤销任务和宿主托管覆盖范围。没有必需的 CI 工作或原生取消/清理套件被移除。

## 语言和跨仓库所有权

使用 Rust 对内部不变量、归档/身份规则和私有故障缝进行编码。偏好 Python 对新黑盒 CLI/进程/文件系统场景；使用 TS 对浏览器集成适当。现有 Rust 接受保持有效；迁移是选择性的，并需要等效断言和衡量的益处。不单纯为暴露私有测试缝而添加生产 API。将新外部 CLI 套件放在 `tests/cli/` 下，并附带本地命令/前提条件的 README；此惯例并不意味着现有套件已被迁移。

Loom 验证作者、冻结候选者、包装和宿主证据接收。宿主验证授权、执行生命周期、取消、恢复和产物托管。配对接受验证发布到宿主接口；它必须不增长成宿主的完整压力/失败矩阵的复制。保留现有覆盖直到演示替代方案，包括遗留格式。

## 二进制准备和验证

在准备步骤中构建选定的产品二进制，并显式指定外部套件的路径。除非构建本身是测试的行为，否则不要在每个场景中重建或使用 `cargo run`。仅复用二进制用于相同修订、目标和构建配置。记录 Loom 和宿主源代码修订、二进制哈希、操作系统/架构和相关合同标识。历史宿主套件可以锁定不同修订；共享一个二进制跨这些套件并不自动正确。

`cargo test --locked` 不运行 Python 套件。现有示例检查包括：

```sh
python -B -m unittest discover -s examples/log-report/tests -v
python -B -m unittest discover -s examples/log-report/desktop/tests -v
```

安装对应的文档示例要求。桌面观察测试不证明一个真实的GUI编辑。运行受影响的外部测试套件，并在贡献者检查之外，显式地进行真实宿主验收测试，而不是声称通过。

## 迁移与性能证据

在移除一个套件之前，将旧的断言/故障案例映射到替换测试中。在适用的情况下，保留取消、固定版本维护、过期结果拒绝和清理检查；在迁移过程中比较两种实现。同时更新CI和文档入口点，而不削弱所需的门禁要求。保留外部等待，保留失败输出，并仅在终止确认后清理仅有的所有者资源。客户端超时不是终止的证据，必须不触发自动重试未知操作。

测量编译/链接和执行分别，区分冷和暖缓存运行。记录平台、工具链、修订版本和重复观察。更改测试语言不会删除产品的编译或其依赖项的编译。优化测量瓶颈；不从单一组合CI持续时间中推断节省。当前工作流不受本文件影响。

## CI缓存纪律

使用仓库工具链作为单一版本源和默认编译器，包括checkout外的命令。仅在临时托管运行器上移除未使用的预安装工具链：rust-cache指纹每个已安装的编译器，因此无关的稳定版本可以否则无效相同的源缓存。请勿在开发机器上执行此清理。

仅依赖的Rust缓存覆盖Loom，独立的质量工作区和每个资格测试的显式指定宿主检查出。作业身份保持不同宿主修订版本分离。在所有命中上保持所有检查和制品验证激活。不缓存接受记录、工作文档或凭证。在相同提交上比较一个缺失和精确命中，包括恢复/保存时间，并记录运行器镜像差异。单对是一次性观察，而非性能保证。
