# 原生 spool 打包和宿主交接

[English](../../native-spool.md) · 中文译文，以英文为准。 · [中文目录](../README.md)

内容标签遵循 [修订版 2 合约](spool-code.md)：每个发布文件都是 silk，包括位于历史 `kit/` 路径下的文件。可变衍生品和新工作为 kit。下面的路径绑定描述了合格的旧版配置文件，而不是这些标签的定义。指令资源保持受保护；标签不授权新文件、配置更改或执行。

此合格适配器导出经过评审的 shell spools 和显式的 v3 Python 请求，包含二进制产品。请参阅 [平台矩阵](platforms.md) 以了解当前的导出和真实宿主覆盖情况。它不实现运行时授权、监督、清理或域管道引擎。旧版文件-kit 部署适配器及其 wire 名称保持不变。

## 编写

使用一个通用的 `loom.toml` 包，并使用显式的 include 允许列表。包含 `native-spool.toml` 和每一个选定的代码文件。编写文件包含 `schema = "an-native-spool-v1"`、`runtime`、`entry`、`silk`、`kit`、`editable`、`inputs`、`outputs`、`capabilities`、`dependencies` 和 `acceptance`。`loom.toml` `description` 包含域摘要。请参阅 `examples/native-spool/` 了解两个域 silk 步骤和一个转换 kit，以及 `an-loom init-native`、`check`、`pack` 和 `verify`。

一个条目是一个声明的 silk 脚本。路径是一个角色目录加上唯一的 ASCII 基名；最多有 12 个文件，每个文件最多 1 MiB，总计 8 MiB。输入和输出槽是有限标识符。`editable` 是 `kit` 的子集。能力名称和接受命令是记录，而不是授权和进程。依赖项值是未解析的标签：不会获取 URL、路径或版本范围。`check`、`pack` 和 `verify` 拒绝缺失的条目、在两个角色中列出的路径、转义路径，以及未在列表中列出的 `silk/` 或 `kit/` 文件。它们不会生成接受命令或下载依赖项。`runtime` 必须是 `macos-native-process-v1`，或者 `macos-native-python-v1` 并使用下面描述的显式 v3 接口。其他运行时标签将被拒绝。

```text
an-loom pack PROJECT
an-loom export-native-spool ARCHIVE NEW_DELIVERY
```

导出程序使用现有的归档库存、允许列表、密钥和摘要检查。
它仅输出选定的silk/kit字节和`native-spool.json`；README、源构建文件和创作元数据不会被复制到执行包中。宿主
JSON保留`schema`、`name`、`version`、`entry`、`files`和`outputs`。运行时、输入、可编辑路径、能力请求、依赖项标签和接受记录保留在源包中。名称和版本来自经过验证的包标识。精确输出的清单字节和选定文件哈希决定了发布摘要。现有输出目录将被拒绝。宿主
独立解析并验证此合同。

导出结果记录源包的摘要。它不授予运行时权限，不证明发布者身份，也不解析版本范围。这是一个
显式的全新配置文件，而不是对旧版`[kit]`/`[silk]`适配器的重新解释。

## 旧版兼容性

仅包含`schema`、`entry`、`silk`、`kit`和`outputs`的历史原生v1文件仍然适用于打包、验证和导出。不会生成任何创作元数据，且现有发布标识将被保留。编码任务需要显式的完整创作元数据。部分添加将被拒绝，而不是静默地视为旧版合同。候选检查计划和证据具有单独的版本化合同，详见
[candidate check custody](candidate-checks.md)。

## 宿主使用

使用带有`an-native-spool-v1`合同的无代理宿主：

```text
an-agent-host spool-load DELIVERY NEW_INSTANCE
an-agent-host spool-show INSTANCE
an-agent-host spool-kit-update INSTANCE transform.sh EDITED_FILE
an-agent-host spool-kit-update INSTANCE transform.sh EDITED_FILE --approve DIGEST
an-agent-host spool-start INSTANCE run-1 INPUT_FILE 30
an-agent-host spool-start INSTANCE run-1 INPUT_FILE 30 --approve DIGEST
an-agent-host spool-accept INSTANCE run-1
an-agent-host spool-accept INSTANCE run-1 --approve DIGEST
an-agent-host spool-reclaim INSTANCE run-1
an-agent-host spool-reclaim INSTANCE run-1 --approve DIGEST
an-agent-host spool-unload INSTANCE
an-agent-host spool-unload INSTANCE --approve DIGEST
```

每个摘要属于其立即前一个预览，而不是另一个命令或实例。SIGINT/SIGTERM 用于取消正在运行的宿主 CLI 请求。宿主拥有有效的策略、选定代码的快照、终止和保管权。向另一个目录进行全新加载会恢复套件的基线，同时保留先前的产品。回收影响工作临时空间；释放、输入证据和正式产品不会被隐式删除。输出仍保留在每个运行的宿主产物目录中。

对于独立监督的启动，预启动取消仅在宿主验证原始工作者的请求绑定无启动完成以及无执行/输入声明记录时才是最终的。之后，它允许关闭或卸载，但不允许接受产物、收集或重放。缺失或矛盾的证据仍保持未知。Loom 既不会从取消的客户端推断终止，也不会修复宿主记录；宿主拥有此故障矩阵及其真实工作者 CLI 的资格。不需要进行任何释放格式或打包变更。

在运行之间允许对套件进行编辑。运行时代码快照是只读的。域脚本使用 `AN_SPOOL_SILK_DIR`、`AN_SPOOL_KIT_DIR` 和 `AN_SPOOL_INPUT`；其当前目录是可写的临时空间。这并非是实时代理的自我修改或对恶意本地代码的强隔离。实时进程暂停/挂起、编织器执行、依赖解析和自动保留功能不会由此适配器实现。

## 显式状态交接

宿主的可选 [原生状态合同](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-state.md) 与现有的 v1/v2 导出兼容。作者定义了状态格式及其验证方式，选择一个现有的文本输出作为下一个状态，并使用所声明的输入用于选定的状态文档。不会引入额外的 Loom 状态模式、状态解释器或执行监督器。

操作员通过 `spool-state-init INSTANCE OUTPUT FILE` 初始化状态，通过 `spool-start-state INSTANCE RUN REVISION SECONDS` 选择一个确切的已提交版本，正式接受该运行，然后显式地 `spool-state-commit INSTANCE EXPECTED RUN`。每个变更都需要自己的宿主预览/批准。普通的 `spool-start` 保持不变，不能仅仅因为其输入字节恰好匹配一个检查点，就通过状态提交的方式提升其输出。

宿主拒绝过时的、未成功的、被取消的或未被接受的状态候选。临时集合保留已提交的状态和已已验收的产物。新的运行不会继承临时集合；可以从检出的固定状态显式地加载一个新的初始状态，创建一个新的历史记录，而不隐含迁移或血缘关系。状态的含义、业务接受度和跨版本兼容性仍由作者拥有。

[narrow CLI 限定](../tests/cli/README.md) 将现有的文本引用打包，并通过两个与状态绑定的宿主运行来验证两种输出格式，即提交前的回收和卸载后的检查。宿主测试拥有完整的失败矩阵。这是协议证据，而不是新的领域工作流或通用的 Loom 状态管理功能。

### 可选的检查点依赖声明

领域作者可以通过 `spool-state-dependencies INSTANCE REVISION REQUEST` 提供一个由宿主审核的已接受制品列表，这些制品被一个确切的检查点引用。
请求模式是 `an-native-checkpoint-dependencies-v1`；其 `artifacts` 条目标识生产者 `instance`、`run`、`output` 和确切的 `revision`。
宿主验证源的保管权并拥有不可变的注册/源指针。
Loom 不解析状态来推断依赖关系、授予文件访问权限或授权收集。现有的发布格式和独立执行保持不变。

引脚在 HEAD 前进、取消尝试的引用释放和私有集合中得以保留。缺失注册完成块的使用该检查点，而不是重试。未声明的状态不是无依赖的；后续检查点不会隐式继承声明。宿主操作员可以选择 `--registered` 在状态初始化或新提交版本时。已注册的后续检查点继承读取器策略。宿主可以关闭该确切检查点，然后在可以证明所有读取器已完成时，单独释放其声明的引脚。具有不完整读取器普查的历史状态保留其引脚。参见 [宿主合同](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-state.md#explicit-checkpoint-retirement)。

宿主无头测试拥有中断/保护矩阵；Loom 的现有包到宿主资格绑定对应宿主版本。

### 轮次边界宿主控制

合格的宿主还暴露 `spool-pause`、`spool-suspend INSTANCE REVISION` 和 `spool-resume`，每个转换都有单独的预览/批准。暂停后，仅在所有已启动的尝试确认结算且成功输出获得正式接受后，才设置暂停围栏，阻止新的开始、状态突变和工具编辑。暂停还会锁定当前提交的检查点和工作代码标识。检查和现有受保护的产物回收仍然可用。

恢复验证保留的绑定并重新打开准入，而无需执行或注入状态。下一轮明确选择其状态输入并获得新的批准。宿主的控制生成拒绝暂停前的执行和编辑提案。配对的 v1/v2 CLI 资格通过现有源包行使此路径；Lorem 不添加任何模型循环或生命周期监督器。

激活升级将宿主的私有实例记录升级，使得旧宿主拒绝它而不是忽略控制。包标识和v1/v2导出格式不会改变。安全点是一个已完成的轮次；正在进行中的信号、进程/虚拟机镜像和自动重启仍不支持。请参阅宿主的[native control contract](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-spool.md#native-control-at-round-boundaries)。

### 私有快照保留

宿主的`spool-storage INSTANCE`观察逻辑字节和可用空间。在显式工作收集之后，`spool-reclaim-snapshots INSTANCE RUN`可以预览并在现有证据/批准边界下收集私有输入体。正式产物、检查点对象和执行/控制记录仍会被保留。配对的v1/v2资格在状态提交前收集输入快照，然后从独立检查点继续。没有添加Loom保留引擎。

消费者使用固定产物检出或显式状态选择，而不是私有快照路径。这不会启用通用产物GC或隐式发现不透明业务状态中的引用。原生准备添加文件系统空间检查；这些是准入压力，而不是运行时配额。请参阅[宿主存储合同](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-storage.md)。

## 接受和下一所有权

### 发布审查和独立继续

分别导出旧版本和候选存档，然后使用宿主的只读`spool-release-diff OLD_DELIVERY CANDIDATE_DELIVERY`。它独立验证两个负载，并显示确切的代码/合同/请求差异，包括不支持的请求。相同名称或版本标签不建立更新权限。导出或比较不会授予执行权或验证后续者。

对于合格的路径，将候选者载入一个全新的实例中。如果域合约允许，显式地检出一个固定的旧检查点，并从该文件初始化新的历史记录，然后分别批准其执行。旧状态和产品仍保留在原始实例中。这是独立的延续，而不是原地升级、自动迁移或状态模式兼容性证明。编辑后的工作套件需要单独审查；比较使用发布基线。

真正的 `tests/cli/native_release.py` 包含通过普通检查/打包/验证流程以及 v1/v2 导出器进行的两个版本，比较它们并执行独立的状态历史记录，同时保留旧的绑定。宿主测试自身不支持的请求、过时的批准和失败候选者的拒绝。没有引入新的 Loom 发布格式、更新程序、依赖解析器或运行时权限。详见
[宿主发布合约](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-state.md#native-release-review-and-independent-continuation)。

`cargo test --test native_spool --locked` 检查确定性的独占导出和无效的角色/输出绑定。被忽略的真实宿主测试需要一个合格的 `AN_AGENT_HOST_BIN`；它实际执行 CLI 加载、套件编辑、原生执行、接受、回收、卸载、基线重新加载和 SIGTERM 取消。没有假宿主来替代该声明。CI 将宿主实现与旧的接受配置文件分开进行固定。

可选的 `LOOM_NATIVE_EVIDENCE` 选择一个私有的 JSON 证据文件；它可以包含本地路径，但不得公开提交。

Loom 以后可能会准备有限的编织者编辑任务并解析声明的依赖项。宿主仍必须授予并强制执行执行、检查点、产物和穿梭访问权限。支持代码工具不得将其模型循环移入 Loom 或授予其不受限制的宿主 CLI 访问权限。


## 可选编织者和双实例接受路径

宿主提供一个文件提案插槽，与Loom无关。请求一个包含`spool-weaver-request INSTANCE transform.sh`的套件；仅将返回的数据转发给外部CLI适配器或授权的穿梭工具。使用`spool-weaver-apply INSTANCE RESPONSE_JSON`导入其提案，检查返回的计划，然后使用`--approve HASH`重复此过程。`spool-weaver-revoke INSTANCE REQUEST_UUID`防止晚期准入，但并不保证外部工具已停止。提案不授予执行权限。绝不要将不受信任的代理授予不受限制的宿主CLI。

响应模式为`an-weaver-proposal-v1`，包含`request`（请求UUID）、`request_sha256`、`kit`（所选的`kit/BASENAME`）以及UTF-8的`content`。宿主将其绑定到实例、发布、原始套件和运行/编辑生成版本。更改、撤销、重复、外来或受保护路径的提案将被拒绝。启动运行需要在应用之后单独批准。插槽不会启动任何模型，并且保持模型/上下文行为在Loom之外。

真实的原生接受现在使用实际的发布版本执行此序列，该版本从`examples/native-spool`导出，而无需使用假的执行后端：

1. 加载一个发布版本的两个独立实例（两个丝线和一个套件）。
2. 将一个确定性的文件提案导入A，拒绝外来和丝线编辑。
3. 运行A并接受一个版本报告；检出该确切的产物修订版本。
4. 从A发布一个更新的报告，拒绝声称新候选的旧收据，并将B运行在较早检出的版本上。
5. 回收A的接受草稿并卸载它；加载一个新鲜实例并验证原始套件行为。
6. 在新鲜实例完成另一轮运行的同时，保持B在真实执行中；在B处于活动状态时，拒绝B的卸载、回收和编织请求。
7. 向B的CLI发送信号，要求确认取消，拒绝取消输出的接受，接受B成功的下游报告并在回收B之前接受它，然后卸载B。
8. 验证固定较早的产物保持不变。

测试的文件提案是明确的确定性测试输入，而不是一个大型语言模型（LLM）或编码工具安全性的声明。`LOOM_NATIVE_EVIDENCE`保留身份、收据、固定交接版本、取消和独立执行证据。这是一个操作员验证路径：运行被忽略的集成测试，使用合格的`AN_AGENT_HOST_BIN`。领域代码仍然独立打包，且运行时Rust依赖不会将Loom连接到一个代理。

## 显式v2宿主交付

使用`an-loom export-native-spool-v2 ARCHIVE NEW_DIRECTORY`来在宿主发布中保留作者要求。普通导出仍为v1且字节兼容。v2需要显式的作者元数据；它不会为旧的六字段合约伪造声明。生成的原生清单具有架构`an-native-spool-v2`，并包含一个必需的`requirements`对象，其中包含运行时、输入、可编辑路径、能力及依赖项等作者信息。

这个合格的导出器将其现有的macOS shell运行时映射到操作系统`macos`、架构`portable-source`、交互`headless`和GPU`none`。这些是固定的配置文件事实，而不是检测到的设备能力或新的通用作者字段。不声称任意的可执行文件、GUI或GPU配置文件。声明标签是请求；宿主在接纳前会检查其支持的集合。保留额外的依赖项标签，以便宿主可以拒绝未解决的需求，而不是静默丢弃它们。宿主独立验证v2，拒绝不支持的需求，并将接受的需求绑定到发布身份。空的可编辑范围使打包的工具包不可编辑。

这是可执行运行时准入的元数据，与源包指令和接受命令分开。它不是扫描证书或授权。请参见`spool-runtime-handoff.md`了解计划中的更广泛合约和未来适配器。

相同的实际宿主生命周期验收测试同时适用于 v1 和 v2：两个独立的实例，受约束的 kit 提案应用，固定的产物检出，取消时的组结算，活动期间拒绝收集，卸载和重新加载新鲜基线。v2 并不单纯从其准入测试继承生命周期正确性。当 LOOM_NATIVE_EVIDENCE 被设置时，v2 会记录一个单独的 `.v2.json` 证据文件。

## 显式的 v3 事件接口

将以下表格添加到完整的原生编写文件中，并使用
`export-native-spool-v3 ARCHIVE NEW_DIRECTORY`:

```toml
[events]
consumes = ["artifact.accepted"]
produces = ["artifact.accepted"]
```

每个数组可能为空或包含该唯一支持的名称。未知名称、重复项和未知字段将被拒绝。V3 保留 v2 的要求，并将接口绑定到发布摘要中。V1/v2 不包含事件编写功能的保持不变；将事件感知项目导出为 v1/v2 会拒绝而不是丢弃其合同。旧版宿主拒绝 v3。现有的 `develop-native` 方便路由仍然导出 v2，并且不适用于事件感知项目；请使用显式打包和 v3 导出。这不会扩大该路由的构建权限或依赖支持。

这些声明是接口，而不是订阅或执行权限。宿主独立授权静态工作区路由，验证已验收的产物事实，并在精确版本上承认交付。收据和输出确认仍然由宿主拥有。没有路由服务、运行时依赖、业务谓词、循环或全局状态被添加到 Loom。领域工作流仍保留在 silk/kit 中。

配对的 `tests/cli/native_events.py` 包装现有的两个 silk 引用，加载两个实际的 v3 实例，发布一个已验收的产物事件，批准一次交付，接受并确认其结果，并在 scratch/snapshot 收集后验证固定产品和重复确认。宿主在其独立尝试监督下运行批准的交付。宿主测试自身的取消、撤销和证据缺失情况。请参见 [宿主事件合同](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-events.md)。

## 原生 Python 和字节产品

V3 编写可以声明 `binary_outputs`，这是 `outputs` 的一个独特子集。省略保持文本行为。V1/v2 导出拒绝这些声明，而不是丢弃它们；现有的源代码和导出字节保持不变。一个二进制文件对宿主来说是一个不透明的产品，而不是解析 Office 文档的请求。现有的字节产物限制仍然适用；消费者拥有格式/领域检查。

`runtime = "macos-native-python-v1"` 请求具备完整 v3 事件接口的合格 CPU/headless 原生 Python 配置文件。依赖标签保持为可移植数据。Loom 不发出本地解释器路径，不安装任何内容，并不授予任何运行时权限。宿主操作员单独选择一个已安装的环境，该环境具有匹配的标签、解释器摘要和完整的站点包库存：

```text
an-agent-host spool-load DELIVERY NEW_INSTANCE --runtime LOCAL_RUNTIME_JSON
```

宿主验证在加载/启动前的绑定以及产物接受情况。Python 在站点启动时运行，禁用字节码写入，选择只读依赖根目录并尝试私有代码/数据快照。声明的输入槽位提供其快照文件名；`source.xlsx` 不需要继承目录的 `content.bin` 名称。输入/代码基础名冲突将被拒绝。运行时选择不是发布者认证、安装包权限或敌对框架沙盒。系统标准库仍然是操作者拥有的依赖项。参见 [宿主合约](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-spool.md#operator-selected-native-python-and-binary-products) 获取严格的 JSON、库存算法和限制。

[日志报告示例](../../../examples/log-report/README.md#native-spool-event-handoff) 通过复制其现有的独立文档适配器来准备两个项目。Loom 检查/归档/导出它们，不涉及领域分支；宿主将每个作为独立的 spool 执行，并通过事件交付精确接受的 XLSX 版本。消费者创建一个三页的 PPTX，包含原生可编辑的文本/表格对象。配对的 `tests/cli/native_documents.py` 在新的上游结果后确认旧版本的消费，并在收集后保留产品。它不执行任何 GUI 编辑或模型调用。宿主测试自身的取消/运行时变异拒绝情况。

使用显式的 `check`、`pack`、`verify` 和 `export-native-spool-v3` 来处理此路径。现有的 `develop-native` 方便路由仍针对 v2 shell 检查；它不具备提供 Python 依赖项或交付 v3 项目的资格。

## 分离业务数据和检查点输入

V3 编写可能添加 `state_input = "checkpoint.txt"`，与一个业务 `inputs` 槽位和所有代码基础名分开。Loom 在宿主需求和编码任务简报中保留名称；它在现有任务合约下保护所选的编写文件。V1/v2 导出无法删除此要求。没有该字段的现有声明将保留其字节和语义。

宿主独立地通过`spool-start-data-state`绑定一个固定的 数据产物 加上一个显式选择的已提交状态，或者将`--state REVISION`添加到产物-事件交付中。`AN_SPOOL_INPUT`和`AN_SPOOL_STATE`分别命名了两个只读快照。Loom中不添加任何HEAD查找、状态合并、领域解释或重试机制。启动、接受、检查点提交和事件完成每个都保持为独立的宿主操作。过时状态不能覆盖HEAD；即使其提议的状态存在冲突，已接受的报告仍然有效。

[日志累加器](../examples/log-accumulator/README.md) 重用了现有的摘要转换，并添加了领域拥有的累计计数。其独立测试只需Bash、awk和Python标准库。配对的CLI资格检查实际v3打包、直接/事件输入角色、一个过时提交以及私有收集和卸载后的固定输出/状态。宿主接受负责实际取消、更改选择和缺失证据的情况。参见[宿主状态合同](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-state.md#independent-business-data-and-state) 了解严格的筛选器、冻结预览和交付身份规则。

## 运行时产物发现

宿主的`spool-artifacts INSTANCE RUN` 返回经过验证的固定版本绑定，用于正式已验收的输出。Loom声明输出槽；运行时发现、保留和内容/来源验证仍由宿主负责。查询结果在临时收集和卸载后仍然可用；使用现有的产物检出命令创建一个显式的下游输入。源工作副本和临时文件名绝不会作为验收证据替代。

宿主的 `spool-status INSTANCE RUN` 暴露了基于证据的运行阶段以及独立的执行/终止/产物观察。Loom 的真实 v1/v2 CLI 接受观察运行中的不确定性、已确认的取消、已验收的输出以及回收的临时空间，包括在卸载之后。没有 PID 猜测、锁存在或状态响应可以授予执行或回收的权限。运行时监控保留在宿主端；通用的观察词汇表并不意味着后端对 GPU、GUI、暂停或挂起有相同的支持。无头模式仍然是第一个合格的模式。

对于中断实例的接受移交，使用宿主的显式 `spool-reconcile-accept INSTANCE RUN` 预览/批准流程。Loom 的真实 CLI 测试在工作区输出发布后创建一个实际的 facade-write 失败，检查未确认的托管权，进行协调，并在正常下游检出前验证未更改的尝试次数、修订次数和工作区接受字节。这不是一个过程重试、第二次发布或断电持久性声明。不完整的工作区接受和不确定的执行仍不支持恢复。宿主拥有协调功能；Loom 不拥有运行时修复状态机。

原生资格认证还对显式固定宿主版本执行 spool-kit-reset：预览是无害的，恢复套件会保留实例身份，新的运行使用 release-baseline 代码，而之前固定的产物仍然可读。这是宿主生命周期的功能，而不是 Loom 对套件变异的实现或额外授权。其他历史宿主任务保留自己的固定版本。

## 宿主拥有的事件库存

宿主提供 `spool-events-pending WORKSPACE` 和前台 `spool-events-watch WORKSPACE SECONDS INTERVAL_MS`。库存将匹配的固定事件和订阅与准入拒绝和基于证据的交付阶段相关联。Watch 会发出初始/变更的 NDJSON 快照和最终的停止摘要；它不承诺每个中间转换。任何命令都不会准备运行、选择状态、确认消费或授予执行。停止观察不会取消一个独立准入的交付。

Loom 仍然仅声明/打包接口。运行时观察、轮询、订阅准入和执行仍保留在代理中。不要在 Loom 中添加第二个调度器，也不要把编码任务指令视为运行时权限。
配对的原生事件接受观察实际 Loom 构建版本的待处理、接受、确认和已消费阶段。宿主接受拥有监控信号、截止时间和证据丢失情况。此 P1 切片不添加自动调度、最新状态选择或广泛批量授权。

## 具有固定状态的有限宿主调度

宿主的 `spool-events-dispatch WORKSPACE REQUEST_JSON [--approve HASH]` 接受
1..16 明确选择的现有事件/订阅对，每个都具有所需的精确 `state_revision`。预览和哈希批准绑定每个交付的开始计划；不推断最新状态、通配符或未来事件授权。SIGTERM 排空所选条目并停止进一步选择；SIGINT 请求其受监督的取消。持久意图将整个批次防止重放/恢复，包括在缺失结果或账本失败之后。观察可通过 `spool-events-dispatch-show` 获得；重复记录的请求永远不会运行剩余部分。产品/状态接受和完成保持显式。

Loom 不添加运行时调度器、信号处理程序或状态策略。现有的日志累积配对接受打包相同的 v3 合约，通过宿主调度器提交一个固定的状态事件，独立接受/提交/完成它，并检查固定先前的产品，且在收集后不重复尝试。宿主接受拥有多条目停止/取消、真实账本故障和未知结果围栏。现有的独立和直接交付路径仍受支持。

## 宿主订阅准入控制

`spool-event-pause` 和 `spool-event-resume` 是显式批准的宿主操作，适用于固定订阅。暂停会阻止新任务的准入，但不会取消正在运行的工作或阻止已准入的接受/提交/完成。恢复需要重新交付/批次批准，并且目标实例/生成版本必须相同；它不会重放记录的批次或撤销永久撤销。

宿主保留路由/交付身份，并记录前驱绑定的控制生成版本。首次使用会升级工作区元数据到 v2，以便早期宿主拒绝新的打开操作；现有的 v1 工作区如果没有控制则继续支持。同一升级工作区上的混合旧/新宿主进程不被允许。这不会改变 Loom 发布格式，也不会允许项目声明授权运行时操作。配对的日志-磁带接受检查会暂停/恢复，拒绝过期的批次批准，保持交付身份不变，暂停期间保留 custody 和保留产品/检查点。宿主套件拥有并发准入、历史错误和旧宿主资格。

## 宿主调度摘要重建

宿主显式批准的 `spool-events-dispatch-reconcile WORKSPACE NAME` 可以仅从所有原始条目记录、精确的交付/开始绑定和确认成功的本地接收记录中重建缺失的已完成批次摘要。它从不运行进程、恢复部分批次、接受输出或提交状态。缺失或矛盾的证据会被拒绝。Loom 的配对日志-磁带烟雾在 custody/collection 之后仅丢失已知的已完成批次投影，重建它并验证尝试次数仍为一。这不是任意崩溃恢复。运行时协调仍由宿主拥有。

## 宿主接收者继任

`spool-event-replace WORKSPACE OLD NEW ROUTE_JSON [--approve HASH]` 重用了现有的严格路由请求格式，仅替换未来的接收方准入。
生产者/事件槽位保持不变，后续使用一个全新的名称，旧的路由以及所有已准入的交付标识仍然保留。替换过程不会运行进程、取消旧工作或迁移状态。一个不可变的目录提交会淘汰旧的准入，并绑定一个固定的日志前缀切换点；之前的事件无法通过后续的实例进行重放。新的事件仍然需要显式的执行批准和一个选定的检查点。旧的待处理/批量批准无法将工作重新定向到新的实例，且恢复旧的准入会被拒绝。

首次使用将升级宿主工作区元数据到 v3；旧的宿主拒绝新的打开请求。
先前的 v1/v2 工作区仍然支持，v3 保留现有的控制历史。保留旧的实例目录以供历史托管；不要在升级后的工作区中混合已经运行的旧版和新版宿主。这不是自动发布升级、状态迁移、路由历史垃圾回收或任意拓扑编辑。现有的边界仍然适用于已退休的声明。一个无关且单独批准的路由仍然是一个独立的权威，而不是全局恰好一次行为的承诺。

打包的日志池烟雾测试会加载相同精确版本的第二个实例，显式地从一个固定的检查点进行初始化，拒绝通过后续实例进行历史交付，然后通过有限的分发处理新的事件并验证独立的状态/产品。Loom 声明/检查/打包现有的接口；切换点、权威、监督和托管仍然由一个代理负责。宿主套件拥有在途替换、过期批准、不确定结果和旧宿主拒绝等情形的处理权。

## 有限的私有材料维护

宿主的 `spool-storage-collect INSTANCE REQUEST_JSON [--approve HASH]` 组
1..16 个选定的运行用于工作或私有输入快照收集。每个现有运行的预览在删除前都会被验证，并受整体批准的约束。原始操作保留其锁、精确的库存和持久的一次性删除边界。工作和快照是两个明确批准的阶段。取消/未知结果会停止进一步的项；这不是一个原子批次、自动恢复或回滚服务。正式产物、状态、发布/套件和证据仍会被保留。Loom 不添加运行时收集器或保留权限。

打包的日志-日志接受收集在每个阶段收集两个接受的运行，然后验证显式状态提交、过时状态拒绝和固定下游产品，如之前一样。宿主套件拥有更大的多轮输入、并发消费者进度、取消和丢失收集接收边界。原生正式负载删除仍然需要宿主拥有的引用/释放合同；Loom 声明或未观察到的读者不存在不能授权它。

## 宿主产物输入准入退休

`spool-artifact-retire INSTANCE RUN OUTPUT [--approve HASH]` 关闭一个已验证的接受原生输出，不删除负载或更改其身份，以接受基于新修订的输入。新的检出/编辑父选择、原生业务输入/事件交付和工作区产物准备遵守宿主边界。关闭后，旧的直接/事件/批处理预览不能授权新的获取。其他输出和新修订是独立的。已经获取的输入、输出接受、状态提交、因果确认和私有收集仍可以验证保留的证据。一个提交的检查点是一个独立的版本。

关闭写入操作使用目录现有的锁和独占创建。不可变输入读取在获取前后检查单调性围栏，以保持并发读取。已经获取的字节可以完成；并发关闭操作可能会拒绝准备，而无需授权重试。
撕裂/无效的记录会失败关闭，并且不会作为重试进行覆盖。它不是原始文件、独立副本或旧宿主可执行文件的访问撤销；请使用合格的宿主。没有完整的参考普查、自动GC或删除授权。原生释放模式和旧包字节不会改变。请参见 [宿主存储合同](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-storage.md#retiring-new-artifact-input-admission)。

打包的日志-围栏烟雾关闭一个旧的生产者输出，拒绝新的有状态输入选择，并保留现有的消费证明、独立状态和保存的报告。宿主测试拥有并发退休、取消、并发读取行为和标记错误。Loom 不添加任何运行时退休策略。

### 原生输入引用移交

宿主现在在接纳新的原生运行时注册精确的管理产物/检查点输入。`spool-input-references INSTANCE RUN` 报告运行绑定的声明；预览和原始文件选择不会获取它们。打包的日志-围栏烟雾检查其数据/检查点修订和释放是否与声明匹配，这些声明在私有收集、状态提交和卸载后仍然存在。不需要更改释放格式或清单，Loom 没有运行时引用存储。

宿主的 `spool-input-release INSTANCE RUN [--approve HASH]` 保留了工作和快照收集后的已确认取消所有者合同。新增的 `spool-input-release INSTANCE RUN --delivery WORKSPACE SUBSCRIPTION [--approve HASH]` 支持在接收输出、原始消费确认和私有收集后成功实现无状态事件消费者。配对的事件案例检查显式释放、未更改的正式产品和无执行的重复确认。未完成的注册仍被保留。宿主 CI 拥有中断和释放故障。源观察不授权负载收集。

剩余的 [保留/会话交接](session-retention-handoff.md) 是一个计划。
成功的组合数据/状态事件可能使用 `--delivery WORKSPACE SUBSCRIPTION --coverage FILE` 并通过单独的宿主批准来释放其两个尝试声明。
Spool 会发出一个声明的 `checkpoint-input-policy.json`，与状态输出一同发出，将每种管理角色分类为 `checkpoint-pin` 或 `not-required`。操作员批准精确的版本匹配。宿主从原始交付中读取接受的策略，并检查提交、输入槽位、独立引脚、原始确认和私有收集。对于批次 A 和父状态 B，A 上的检查点引脚在两个临时声明释放后仍能存活。不透明的领域语义属于作者。宿主拥有验证、释放证据和不确定结果的拒绝。参见 [宿主保管合同](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-storage.md)。
旧目录仍被保留。宿主可以在其 [存储合同](https://github.com/lazyalexander/an-agent/blob/dev-advance/doc/native-storage.md#registered-copies-and-explicit-payload-deletion) 下明确收集新注册副本输出。封装的事件烟雾在产品接受前关闭其宿主会话。未知结果不会重试。

### 观察与控制并发

合格的宿主待处理/显示/观看路径读取一个有界的固定日志，而无需获取其工作区变异锁或修复工作区投影。其附加的`observation`元数据标识日志，并报告在结束时是否发生了变化；外部证据仍非原子性。Loom 不得将观察到的阶段转换为执行、取消确认或收集权限。变异宿主命令仍在其原始锁下重新验证。格式错误的捕获证据将失败并关闭；未知的工作内容永远不会重试。宿主 CI 拥有并发观察者/撤销矩阵；Loom 保留其狭窄的封装交付/状态烟雾测试。
