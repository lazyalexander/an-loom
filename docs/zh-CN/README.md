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

发布基线为 **silk**；agent 修改过的副本和新增工作为 **kit**。指令仍受保护。
实际操作由 host 检查，标签和源码名称不构成运行权限。
见[内容契约](docs/spool-code.md)和[职责边界](docs/spool-boundary.md)。

## 开发与契约

显式包含列表、确定性清单和有边界的归档绑定发布内容。源码分析提供诊断，host 执行操作限制。
哈希不等于发布者签名，未知执行结果不可自动重试。

见[架构](docs/architecture.md)、[平台](docs/platforms.md)、[验收](docs/acceptance.md)和[质量门禁](docs/quality.md)。
提交需通过格式、锁定测试、严格 clippy、质量检查和质量工具测试；执行保证另需真实 host 验收。

## 案例 · 机械动画

### 水墨动画与手绘蒸汽机对比

| 水墨动画 | 手绘版本 · 6 秒 |
| --- | --- |
| ![水墨动画](https://raw.githubusercontent.com/lazyalexander/an-agent/d94f574197d8a7f56d30ca78d9cad8d81039d65d/assets/demos/mechanical-ink.gif) | ![手绘蒸汽机](https://raw.githubusercontent.com/lazyalexander/an-agent/d94f574197d8a7f56d30ca78d9cad8d81039d65d/assets/demos/painted-steam-engine.gif) |

Loom 将选定几何、运动、图片与验收命令准备为经过审阅的发布包。动画 spool 打包和宿主生命周期验收待完成。
[教学流程与复现证据](https://github.com/lazyalexander/an-agent/blob/d94f574197d8a7f56d30ca78d9cad8d81039d65d/doc/zh-CN/doc/cases.md)。

### 补充案例 · 四缸发动机

<img src="https://raw.githubusercontent.com/lazyalexander/an-agent/d94f574197d8a7f56d30ca78d9cad8d81039d65d/assets/demos/painted-inline-four.gif" alt="四缸发动机" width="420">

40 个 CAD 零件复用手绘流程，完整周期为 8 秒。两段手绘 GIF 为 720 × 405、12 fps 循环预览。
素材和教学说明引用 an-agent 的固定版本。

## 示例：CAD → Krita

Loom 打包把 CAD 快照变成分层绘画的领域 spool。成熟源码由 an-spool 维护，应用交互由 Shuttle 实现。

![CAD 绘画](https://raw.githubusercontent.com/lazyalexander/an-shuttle/0dfd646/examples/cad-paint/sample/vise.png)

[打包交付](examples/cad-paint/DISTRIBUTION.md) ·
[应用配置与验收](https://github.com/lazyalexander/an-shuttle) ·
[领域源码](https://github.com/lazyalexander/an-spool)
