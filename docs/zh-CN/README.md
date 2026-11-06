# an-loom

[English](../../README.md) · 中文译文，以英文为准。

Loom 是一套用于开发可复用领域 spool 的 Rust 工作台。

Loom 帮助开发者把领域知识、已有的 CLI 程序、技能和应用集成整理成可核查的发布包。它负责编写、组装、协调验证和打包。领域行为由 spool 项目实现。an-agent 负责运行时权限、模型与上下文服务、状态和产物保管。一次 spool 执行尝试是隔离与生命周期的单位。发布文件都带有 **silk** 基线标签。在同一授权环境中，可修改的派生副本和新生成的工作带有 **kit** 标签。OpenShell 是可选、可替换的后端，暂缓扩展。Shuttle 通过宿主授权接入应用。当前运行时工作优先支持经过审查的本地无界面 spool。无界面执行并不构成信任依据。宿主无法满足隔离要求时必须拒绝，禁止自动降级到约束更弱的原生环境。参见[职责边界、实现审计与迁移计划](docs/spool-boundary.md)。

初始实现从 an-waver 迁入选定的打包和验证代码，并把一个轻量 CLI silk 接入独立宿主。`loom.toml` 绑定选定的项目路径。操作员审查开发计划后，宿主才编译并测试选定的 Rust 源码。Loom 接收与计划相符的输出，生成固定的宿主声明，并组装可移植的本地发布包。

现有部署适配器只支持一份无依赖 Rust 源码、一个可执行文件、`file-kit-v1` 和宿主已有的隔离构建配置。参见[平台支持和验收矩阵](docs/platforms.md)。通用包可以包含其他语言和文件。新的构建或运行时配置需要宿主支持，并要有单独通过验收的适配器。通用多阶段工作流执行器和代码生成 Agent 仍未实现。

根据 [spool 内容契约](docs/spool-code.md)，每个已发布的基线文件都标为 **silk**，获授权的工作副本和新生成的工作都标为 **kit**。指令仍受保护，程序和配置可有获批的 kit 副本。标签说明来源和生命周期，执行权限另行授予。现有目录和 API 名称，以及通过验收的配置约束，继续兼容。

## 快速开始

```sh
cargo install --path . --locked
an-loom init ./my-spool
an-loom check ./my-spool
```

入门模板使用[现成的 file-list silk](examples/file-list/README.md)，直接复用已有程序。其纯转换和领域测试留在示例内，与 Loom 的通用实现分开。为自己的领域做适配时，可把 `generation.md` 提供给现有编程工具。

原生文本参考示例使用同样的检查和打包命令：

```sh
an-loom init-native ./my-spool
an-loom check ./my-spool
an-loom pack ./my-spool
# verify the archive path printed by pack
an-loom verify ARCHIVE
```

参见[原生 spool 示例](examples/native-spool/README.md)。打包此配置不会授予宿主执行权限。

[显式 v3 事件接口](../native-spool.md#explicit-v3-event-interfaces)为通过验收的宿主打包工作区事件输入和输出。订阅、交付授权和消费证据仍归 an-agent。Loom 没有引入业务流程图引擎或事件服务。

[候选包检查流程](docs/candidate-checks.md)冻结选中的全部项目字节，通过显式授权的宿主配置执行检查。确认执行已结束、产物已验收且记录相符后，才发布同一归档。旧版原生发布格式仍受支持，检查证据单独版本化。

受控的开发到发布流程：

```sh
# HOST_CLI and TOOLCHAIN_JSON are trusted, operator-owned paths.
# NEW_JOB stays outside the project's package allowlist.
an-loom develop PROJECT HOST_CLI TOOLCHAIN_JSON NEW_JOB
# Inspect the exact host commands, policies, environment, limits and inputs.
an-loom develop PROJECT HOST_CLI TOOLCHAIN_JSON NEW_JOB --approve REVIEWED_DIGEST
an-loom release PROJECT NEW_RELEASE_DIRECTORY
an-loom verify-release NEW_RELEASE_DIRECTORY
```

`release` 创建 `spool.tar.gz`、`release.json` 和 `deployment/` 目录。交付发生在本地。宿主另行验证、注册和授权执行，打包操作不会隐式触发远程发布或授予运行时权限。参见[完整操作流程](docs/development.md)。

## 契约与检查

- 显式包含清单、大小受限的普通文件、可移植路径及对符号链接的拒绝，共同防止遍历到项目无关的文件。
- 排序后的清单、确定性的归档元数据和 SHA-256 标识共同绑定包内容。验证涵盖实际内容、归档上限、gzip 尾部和意外追加的数据。
- 拒绝已知的凭据文件名、本地配置名和常见密钥模式。文本或二进制内容若包含已知的当前用户家目录路径，也会被拒绝。检查只能提供有限证据，无法作为隐私认证。
- 接收宿主构建结果时会核对成功状态、源码和程序标识，并要求记录完整、可移植。机器配置、私有计划和诊断信息留在发布包之外。哈希本身不提供签名或编译器证明。
- 声明用于申请能力。宿主独立验证请求，即使源码检查无法推断某种副作用，也会强制执行限制。
- 开发批准绑定源码、项目配置、工具链配置、选定的宿主可执行文件和确切的宿主预览。提交失败或结果不明时，不会自动重试或覆盖。

本地目录模型假设项目、作业和发布目录由操作员控制。它没有提供用于抵抗同一用户并发恶意替换的远程认证构建协议。本地交付不完整时，应先检查，再选用新目标重新交付。目录发布不保证断电时的原子性。

## 开发与审查

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
# Explicit native macOS host qualification; missing host configuration is an error.
AN_AGENT_HOST_BIN=/operator/selected/an-agent-host \
  cargo test --test host_handoff --locked -- --ignored
```

普通测试覆盖契约和合成的接收样例。真实宿主测试在普通测试中被显式忽略，在单独的 `host-handoff` CI 作业中必跑。缺少宿主环境变量时会报错。参见[验收证据](docs/acceptance.md)和[迁移来源](docs/migration.md)。

Loom 不链接 an-agent，也不导入它的实现。CI 固定宿主的源码修订版本并执行真实 CLI。运行时无需依赖 Loom。

参见[架构](docs/architecture.md)。项目优先支持 CLI，采用函数式核心与命令式外壳。英文文档为准，简体中文文档是配套译文。Agent 可在审查完成、适用验收和所有 CI 门禁通过后，将开发 PR 合入 `dev-advance`，随后通知维护者。疑似暴露私有信息时，发布或合并前须由用户审查。向 `main` 的发布仍由维护者审查与合并。参见 [CONTRIBUTING.md](CONTRIBUTING.md)。

使用 Agent 协助开发时，从 [AGENTS.md](AGENTS.zh-CN.md) 开始。[研究路线图](docs/roadmap.md)涵盖 spool 更新身份、现有工作流的文件生命周期，以及成熟编程工具集成。[Nextflow 观察](docs/nextflow-observation.md)已通过单独审查的宿主配置完成正常运行、中断和恢复的真实验收。[示例包](examples/nextflow/README.md)仍由宿主管理运行时权限，Loom 只打包选中的源码。路线图其余事项仍处于计划阶段，现有 file-kit 部署适配器沿用当前实现。

[共用的原生 MCP 示例](examples/finite-mcp/README.md)展示有限的宿主 kit 调用、源码提案、独立的审查构建和取消，同时保留编程工具自己的原生会话。[平台边界](docs/platforms.md)区分控制宿主分发、应用侧 Shuttle 和 CPU Linux 工作流。开发 PR 使用 `cargo quality --base origin/dev-advance`，稳定版发布使用 `cargo quality --base origin/main`，与 PR 的实际目标分支一致。两者都执行仓库的 [TOML 质量策略](docs/quality.md)。

[日志报告示例](examples/log-report/README.md)覆盖日志 → 可编辑 XLSX → 可编辑 PPTX，工作簿另存为独立修订，下游固定版本消费，取消后拒绝验收二进制输出。kit 可独立运行。[边界报告](docs/log-report-workflow.md)区分已有支持、显式适配器耦合和对领域实现的侵入。

[实现边界审计](../architecture.md#implementation-boundary-audit)把当前模块映射到开发者、宿主和后端的职责，并记录剩余限制。

## 已审查的原生 spool 包

[原生 spool 适配器](docs/native-spool.md)导出只读 silk 代码和 kit 基线，供 an-agent 已通过验收的无界面实例生命周期使用。随附的[轻量文本 spool](examples/native-spool/README.md)无需模型或安装钩子。打包独立于宿主授权和执行。

[CAD 绘画参考 spool](examples/cad-paint/DISTRIBUTION.md)将外观规划器、可编辑 kit 和固定的渲染 silk 一同分发。CAD 客户端固定分发版本，宿主验收分层绘画产物。

[日志报告的原生交接](../../examples/log-report/README.md#native-spool-event-handoff)通过同样的打包方式提供两个无界面 CPU Python spool。独立选择的宿主环境生成可编辑的 XLSX 和 PPTX，并通过工作区事件传递固定版本的二进制产物。业务适配器留在示例内。[日志累加器](examples/log-accumulator/README.md)组合单独选择的业务产物与已提交的检查点。Loom 打包这些角色，宿主负责固定版本、过期状态冲突和取消后的留存。

[原生 spool 验收](../native-spool.md#optional-weaver-and-the-two-instance-acceptance-path)也覆盖可选的宿主 kit 提案入口（沿用旧 Weaver 标识）、两个独立实例、固定产物交接、取消及恢复基线。Loom 负责打包，Agent 执行和运行时权限由宿主负责。

原生入门模板含选定的 `AGENTS.md`，说明文件角色和开发边界。参见[六轮验收状态](docs/development-rounds.md)，了解已实现的证据与下一次真实编程工具交接。

对于准备好的原生候选包，[develop-native](docs/develop-native.md)运行获批的检查到交付流程，并通过选定宿主加载生成的 spool。它复用现有契约，拒绝自动重新提交。

使用[原生 harness 示例](examples/weaver-log/README.md)准备领域材料，并接收来自已有编程工具的、经过审查的提案。真实 Codex 验收覆盖独立检查与受管产物保管。[历史 Kimi 交接](examples/kimi-native/README.md)保留证据和固定的复现源码，重复的启动器已经退役。

[Weaver 契约](docs/weaver.md)说明 Weaver 任务准备，以及 spool 独立运行和受管运行的区别。

## 中文文档目录

以下译文与原英文文件一一对应。运行命令、接口和行为约定以英文原文为准。

- [an-loom 贡献者契约](AGENTS.zh-CN.md) · `AGENTS.md`
- [贡献](CONTRIBUTING.md) · `CONTRIBUTING.md`
- [安全边界](SECURITY.md) · `SECURITY.md`
- [接受范围和复现](docs/acceptance.md) · `docs/acceptance.md`
- [an-loom: spool 开发工作台](docs/architecture.md) · `docs/architecture.md`
- [候选项托管检查](docs/candidate-checks.md) · `docs/candidate-checks.md`
- [与现有编码代理的工作](docs/code-agents.md) · `docs/code-agents.md`
- [原生开发到已加载的spool](docs/develop-native.md) · `docs/develop-native.md`
- [开发轮次：当前资格](docs/development-rounds.md) · `docs/development-rounds.md`
- [一个轻量级的 CLI silk 从开发到发布](docs/development.md) · `docs/development.md`
- [日志报告：支持、耦合和领域所有权](docs/log-report-workflow.md) · `docs/log-report-workflow.md`
- [迁移与源代码溯源](docs/migration.md) · `docs/migration.md`
- [原生 spool 打包和宿主交接](docs/native-spool.md) · `docs/native-spool.md`
- [Nextflow执行、中断和文件管理](docs/nextflow-observation.md) · `docs/nextflow-observation.md`
- [平台支持和资格认证](docs/platforms.md) · `docs/platforms.md`
- [仓库质量门](docs/quality.md) · `docs/quality.md`
- [研究与下一阶段接受切片](docs/roadmap.md) · `docs/roadmap.md`
- [会话和留存开发移交](docs/session-retention-handoff.md) · `docs/session-retention-handoff.md`
- [轮廓边界，内部丝线和后端独立性](docs/spool-boundary.md) · `docs/spool-boundary.md`
- [轮廓内容标签：丝线和套件](docs/spool-code.md) · `docs/spool-code.md`
- [Spool 识别与更新：研究决策记录](docs/spool-identity.md) · `docs/spool-identity.md`
- [Spool 发布到宿主的交接：首次实现切片](docs/spool-runtime-handoff.md) · `docs/spool-runtime-handoff.md`
- [Weaver：审查领域知识以进行spool开发](docs/weaver.md) · `docs/weaver.md`
- [CAD材料证据](examples/cad-material-evidence/README.md) · `examples/cad-material-evidence/README.md`
- [CAD绘画任务](examples/cad-paint/AGENTS.zh-CN.md) · `examples/cad-paint/AGENTS.md`
- [CAD绘画发布](examples/cad-paint/DISTRIBUTION.md) · `examples/cad-paint/DISTRIBUTION.md`
- [CAD绘画spool](examples/cad-paint/README.md) · `examples/cad-paint/README.md`
- [文件列表 silk](examples/file-list/README.md) · `examples/file-list/README.md`
- [内部 silk 开发合同](examples/file-list/generation.md) · `examples/file-list/generation.md`
- [从原生客户端使用现有的有限MCP服务](examples/finite-mcp/README.md) · `examples/finite-mcp/README.md`
- [历史 Kimi 有限-MCP 合格性](examples/kimi-kit/README.md) · `examples/kimi-kit/README.md`
- [历史 Kimi 原生提案资格认证](examples/kimi-native/README.md) · `examples/kimi-native/README.md`
- [显式状态日志累加器](examples/log-accumulator/README.md) · `examples/log-accumulator/README.md`
- [日志报告工作流程](examples/log-report/README.md) · `examples/log-report/README.md`
- [实际 LibreOffice 保存回环](examples/log-report/acceptance/native-office/README.md) · `examples/log-report/acceptance/native-office/README.md`
- [原生编辑器接受度（可选操作工具）](examples/log-report/desktop/README.md) · `examples/log-report/desktop/README.md`
- [日志摘要 spool](examples/log-summary/README.md) · `examples/log-summary/README.md`
- [原生 spool 开发合同](examples/native-spool/AGENTS.zh-CN.md) · `examples/native-spool/AGENTS.md`
- [原生文本参考](examples/native-spool/README.md) · `examples/native-spool/README.md`
- [Nextflow 生命周期和文件托管接受](examples/nextflow/README.md) · `examples/nextflow/README.md`
- [与现有套件一起进行域名准备](examples/weaver-log/README.md) · `examples/weaver-log/README.md`
- [公共日志摘要需求](examples/weaver-log/materials.md) · `examples/weaver-log/materials.md`
- [日志摘要领域合同 — 待开发者批准](examples/weaver-log/reviewed-example/DOMAIN.md) · `examples/weaver-log/reviewed-example/DOMAIN.md`
- [公共日志摘要需求](examples/weaver-log/reviewed-example/materials.md) · `examples/weaver-log/reviewed-example/materials.md`
- [测试布局](tests/README.md) · `tests/README.md`
- [CLI手off接受](tests/cli/README.md) · `tests/cli/README.md`
- [历史原生发布](tests/fixtures/README.md) · `tests/fixtures/README.md`
