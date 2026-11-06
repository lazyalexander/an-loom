# 日志摘要 spool

[English](../../../../examples/log-summary/README.md) · 中文译文，以英文为准。 · [中文目录](../../README.md)

现有原生 spool 路径的第二域资格认证。可编辑的 kit 使用系统 awk CLI 记录 INFO、WARN 和 ERROR 级别的信息。空行会被忽略；未知级别会出错。稳定的 silks 协调 kit 并添加报告头。这是一个经过审核的无头 macOS shell 工作流，不涉及模型调用、网络访问或依赖安装。

```sh
cargo build --locked
python3 -B -m unittest discover -s examples/log-summary/tests -v
python3 -B examples/log-summary/qualify.py \
  --loom "$PWD/target/debug/an-loom" \
  --host /path/to/an-agent-host \
  --output /path/to/new-qualification-directory --execute
```

`--execute` 授权可信操作员驱动程序检查并批准现有的受限计划。不要将此驱动程序作为无限制的代理工具暴露出来。
输出必须不存在；失败或未知的尝试会保留证据，且不能自动重新提交。客户端超时并不能证明宿主终止。
在授权单独的新任务之前，请检查保留的命令/收据。

驱动程序搭建项目，任务创建前提供领域接受度，接收一个确定性的工具候选包，运行 `develop-native`，并单独执行加载的 spool 两次。修改输入会产生不同的固定产物版本。在擦除回收和卸载后，两个报告仍可通过固定版本读取。领域接受度保留在示例中；不需要宿主或 Loom 业务分支。这不构成对抗性代理隔离，也不代表新的运行时、日志流或完整的生产可观测性系统。

`acceptance.json` 是一个可移植的摘要；项目、任务、交付、原始命令输出和路径是本地操作证据。不要发布这些原始文件。
开发任务使用现有的文本骨架并保留其包名；它是一个内容识别的独立发布，而不是新的运行时配置文件。

驱动程序还会作为第三次运行提交一个未知的日志级别。其真实执行必须以确认终止失败；正式接受和产物发现必须拒绝。只有在之后才会回收擦除，且在卸载后，两个先前成功的版本会被再次检查。部分报告不是产品。
