# 一个轻量级的 CLI silk 从开发到发布

[English](../../development.md) · 中文译文，以英文为准。 · [中文目录](../README.md)

## 要求

Loom 使用 Rust 2024 和 Cargo，工具链 1.98.0 被固定用于资格认证。
参见 [平台矩阵](platforms.md) 了解打包和执行覆盖范围。
第一个受控构建适配器需要原生 macOS，已安装的 Rust 工具链和 Xcode 或命令行工具。操作员提供一个独立的 `an-agent-host` 二进制文件，支持 `build-kit` 和 `file-kit-v1`。精确的合格宿主版本在 [CI](../../../.github/workflows/ci.yml) 中固定；构建适配器在接受编译后的程序之前会移除链接器调试路径。

本验收测试不需要模型 API、模型下载、Docker、GUI 应用程序或 an-agent 运行时库依赖。现有的成熟流水线可以被包装为显式选择的文件；执行其构建系统需要合格的宿主适配器。Loom 不暗示对任意 Cargo 构建脚本、依赖项或项目命令的支持。

## 编写项目

运行 `an-loom init PROJECT` 来创建现有文件列表示例的副本。
在副本中进行开发；保持已提交的示例不变。开发者或现有的编码工具使用生成的 `generation.md` 编辑领域代码及其验收测试。此处生成是确定性的；不声称需要实时代码工具或模型运行。

```toml
schema_version = 1
name = "file-list"
version = "0.1.0"
description = "A developer-owned CLI component"
include = ["loom.toml", "README.md", "kit", "generation.md"]

[silk]
source = "kit/main.rs"
program = "kit/program"
build_record = "kit/build.json"
declaration = "kit/spool.yaml"
analysis = "kit/analysis.json"
profile = "file-kit-v1"
```

规范的作者密钥是 `[silk]`；旧的 `[kit]` 仍然可读，但
同时使用两者会被拒绝。旧的 `kit/` 路径和 `file-kit-v1` 令牌不会
创建独立的运行时边界。请参见 [spool 合约](spool-boundary.md)。

这些路径是显式的作者绑定，而不是硬编码的源名称。每个
绑定都必须包含在 include 列表中；文件必须是唯一的且不能
替换包元数据。运行时 ABI 保持不变：读取 `input.txt`，写入
`output.txt`。宿主将这些槽位映射到一个私有运行目录。

`an-loom init-package DIRECTORY` 创建一个不带部署适配器的通用包。它可以包含开发者拥有的非 Rust 组件。包构建
快照字节并不会执行它们。

`an-loom init-native DIRECTORY` 复制了已审阅的原生文本参考。
其 `native-spool.toml` 说明了 silk 和 kit 文件、输入和输出槽位、
能力请求、可编辑的 kit 路径、依赖标签、合格的
运行时和接受命令记录。`check`、`pack` 和 `verify` 读取这些
记录。它们不会生成命令或下载标签。参见
[原生参考](../examples/native-spool/README.md)。

## 选择受信任的宿主和工具链

将操作员配置保留在 PROJECT 之外。例如本地目录 `.local/`
会被 Git 忽略并被拒绝作为包输入。为作业的父目录使用现有目录，
为作业本身使用新名称。在作业名称处的符号链接，包括悬空的符号链接，
都会被拒绝。在批准提交之前，Loom 会在项目外部写入一个独占的、同步的 `.JOB.loom-submitted.json` 兄弟围栏。
失败、丢失响应或缺失作业不会移除该围栏。检查并协调保留的证据；重复的命令永远不会重新提交该作业。

宿主工具链配置文件是包含具体绝对路径的 JSON 文件，包含 `rustc`、`sdk` 和 `linker` 路径。可以通过 `rustc --print sysroot`、`xcrun --show-sdk-path` 和 `xcrun --find clang` 本地发现这些路径。`rustc` 必须识别出该 sysroot 下的实际编译器，而不是 rustup 的 shim。不要提交生成的机器配置文件。公共接受仅包括可移植的摘要和结果。

## 计划、批准和构建

```sh
an-loom check PROJECT
an-loom develop PROJECT HOST_CLI TOOLCHAIN_JSON NEW_JOB
```

预览会读取所选输入，并请求宿主自身的构建预览。它不会创建任何任务、程序或声明。它会调用操作员选择的可信宿主CLI，而不是项目元数据中提供的命令。

结果包含精确的宿主命令、环境、工作目录、策略、资源限制以及一个外层 `approval_sha256`。查看这些值，然后提交相同的选项：

```sh
an-loom develop PROJECT HOST_CLI TOOLCHAIN_JSON NEW_JOB --approve REVIEWED_DIGEST
```

Loom 重构计划并拒绝更改的源代码、作者合同、工具链配置或宿主标识。所选宿主重新比较其自身的批准，并拥有受控的编译器、测试、取消和证据。适配器不实现第二个进程/虚拟机监督程序。CLI 断开可能留下一个正在稳定中的宿主任务；在做任何其他操作之前，请先查看该任务。

成功的宿主输出是源代码绑定的，并接收至新的程序/构建记录目标。Loom 然后会发出一个确定性的 YAML 声明和一个有限的源代码报告。该报告明确记录了未知的本地行为；它不是全程序效果推断。源代码别名或宏不能授予宿主不强制执行的权限。

失败永远不会触发自动重生成、重新执行、接收伪造或包发布。一个接收被中断的已完成宿主任务可以被检查并传递给 `an-loom receive-build PROJECT HOST_JOB`，然后
`an-loom declare PROJECT`。部分目标文件需要检查并使用新的目标。不要自动重新提交一个模糊的任务。

该配置文件接受一个仅标准库的 Rust 源代码。它不提供 Linux 隔离的编译器、依赖安装或任意工作流执行器。不支持的宿主配置文件拒绝执行。

## 拼装并验证可移植的发布版本

```sh
an-loom release PROJECT NEW_RELEASE_DIRECTORY
an-loom verify-release NEW_RELEASE_DIRECTORY
```

本地投递包含：

```text
release/
  release.json
  spool.tar.gz
  deployment/
    spool.yaml
    program
```

`spool.tar.gz` 包含所选的源代码、程序、领域指令和可移植证据，以及其捆绑库存。`release.json` 将该存档和固定部署绑定到源代码、程序、声明和包标识。它不包含宿主作业路径、计划、诊断或凭证。验证会重新检查存档内容和源代码/构建/声明关系，比较部署字节，并拒绝未注册的交付条目。

可以在没有安装宿主的情况下移动和验证发布。通过发布检查仅标识字节；它不会授予注册、认证领域质量或证明二进制文件是由经过认证的编译器生成的。

## 注册并通过宿主运行

```sh
an-agent-host check RELEASE_DIRECTORY/deployment
an-agent-host register RELEASE_DIRECTORY/deployment HOST_STORE
# Use the host-returned deployment digest, not the archive or bundle digest.
an-agent-host run HOST_STORE DEPLOYMENT_DIGEST INPUT_FILE
# Review the host's runtime plan before authorizing execution.
an-agent-host run HOST_STORE DEPLOYMENT_DIGEST INPUT_FILE --execute
```

宿主接受输出并保留其接收/审计记录。文件列表示例会进行修剪、去重和排序行；宿主不会实现该行为。原始输入保持不变。不声称有任何外部应用程序效果、回滚、虚拟机快照、通用状态迁移或自动垃圾回收。

## 归档验证边界

归档包含常规有效载荷条目和`bundle.json`，且恰好位于一个gzip成员内。验证将整个解压流进行限制，包括tar头和填充内容，并拒绝在解析或分配其正文之前使用的PAX/GNU长名称扩展。即使第二个gzip成员仅扩展为零填充内容，也会被拒绝。第一个成员的CRC、有效载荷限制、清单和哈希仍然适用。这有意拒绝了基于扩展的归档，这些归档格式不在Loom的格式范围内。
