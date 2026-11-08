# an-loom

[English](../../README.md) · 简体中文

可复用领域 spool 的开发与打包工作台。

Loom 准备有边界的编码任务、校验选定源码及证据、打包已审阅内容。
编辑和模型循环由现有编码工具负责；an-agent 负责运行权限、调度、生命周期与产物保管；
an-spool 保存成熟领域实现，Shuttle 连接应用。

## 开始

```sh
cargo install --path . --locked
an-loom init-native ./my-spool
an-loom check ./my-spool
an-loom pack ./my-spool
an-loom verify ARCHIVE
```

将 `ARCHIVE` 替换为打包返回的路径。打包不授予执行权限。
见[开发流程](docs/development.md)、[原生交付](docs/develop-native.md)和[编码工具契约](docs/code-agents.md)。

发布基线为 **silk**；获准的可变副本和新增工作为 **kit**。指令仍受保护。
实际操作由 host 检查，标签和源码名称不构成运行权限。
见[内容契约](docs/spool-code.md)和[职责边界](docs/spool-boundary.md)。

## 示例：CAD → Krita

Loom 打包把 CAD 快照变成分层绘画的领域 spool。成熟源码由 an-spool 维护，应用交互由 Shuttle 实现。

![CAD 绘画](https://raw.githubusercontent.com/lazyalexander/an-shuttle/0dfd646/examples/cad-paint/sample/vise.png)

[打包交付](examples/cad-paint/DISTRIBUTION.md) ·
[应用配置与验收](https://github.com/lazyalexander/an-shuttle) ·
[领域源码](https://github.com/lazyalexander/an-spool)

## 开发与契约

显式包含列表、确定性清单和有边界的归档绑定发布内容。源码分析提供诊断，host 执行操作限制。
哈希不等于发布者签名，未知执行结果不可自动重试。

见[架构](docs/architecture.md)、[平台](docs/platforms.md)、[验收](docs/acceptance.md)和[质量门禁](docs/quality.md)。
提交需通过格式、锁定测试、严格 clippy、质量检查和质量工具测试；执行保证另需真实 host 验收。
