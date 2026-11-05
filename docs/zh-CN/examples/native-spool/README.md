# 原生文本参考

[English](../../../../examples/native-spool/README.md) · 中文译文，以英文为准。 · [中文目录](../../README.md)

这是开发者在领域为小型文本流水线时复制的项目。
排序和报告形状保留在两个只读的丝线中。转换脚本是唯一可编辑的工具包。输入是一个选定的文本文件。输出是`report.txt`。中间的`transformed.txt`是临时文件。

`loom.toml` `description` 是领域摘要。`README.md` 是更长的注释。
`native-spool.toml` 说明角色、槽位、能力请求、可编辑工具包路径、依赖标签、合格的运行时和接受命令记录。
这些记录是数据。`check`、`pack` 和 `verify` 不会生成它们，也不会解析或下载依赖项。

| 路径 | 角色 |
| --- | --- |
| `silk/main.sh` | 丝线入口。调用工具包，然后调用报告丝线。 |
| `silk/report.sh` | 丝线。写入 `report.txt` 形状。 |
| `kit/transform.sh` | 工具包基线。唯一的可编辑工作副本路径。 |

能力名称是请求，而非授予。此项目请求 `read-input` 和 `write-declared-outputs`。合格的宿主配置文件仍然是一个选定的文本文件，输入在内，声明的文本输出在外。`runtime = "macos-native-process-v1"` 是此适配器唯一接受的运行时。一个空的 `dependencies` 列表意味着此项目声明无任何内容。

更改 `kit/transform.sh` 或丝线脚本是领域工作。它不需要更改 Loom。丝线更改应包含在新版本中。工具包更改可以后来应用于宿主工作副本；此包不会自行执行此操作。

## 从一个空目录开始

```sh
an-loom init-native ./my-spool
an-loom check ./my-spool
an-loom pack ./my-spool
an-loom verify ARCHIVE
```

`pack` 输出 `archive`。将该路径传递给 `verify`。这两个命令都会对选中的文件进行快照。它们不会运行 `acceptance`，项目钩子或依赖项安装程序。`check` 拒绝缺失的条目、silk/kit 路径冲突、超出项目范围的路径，以及合同中未列出的 `silk/` 或 `kit/` 下的文件。

打包和验证不会授予执行权限。宿主加载、kit 编辑和运行仍保留在单独认证的 `an-agent-host` 配置文件中。参见 [原生合同](../../docs/native-spool.md)。该指南中的可选 weaver 固定装置是确定性测试输入，而不是模型集成。

`export-native-spool` 会复制选中的 silk 和 kit 字节以及 `native-spool.json`。宿主 JSON 保留 `schema`、`name`、`version`、`entry`、`files` 和 `outputs`。作者字段仍保留在源包中。

## 有界 kit 任务

`task` 在项目外部的新目录中写入一个分配。工作区包含可编辑的 kit 副本和简要说明。silk、`loom.toml` 和 `native-spool.toml` 保持受保护状态。简要说明不授予任何权限，也不包含任何宿主 CLI。

```sh
an-loom task ./my-spool ./task
# A coding tool may edit only kit/transform.sh in ./task/workspace.
an-loom receive-task ./task kit/transform.sh ./replacement.sh
an-loom revoke-task ./task
```

`receive-task` 复制 `task/candidate/` 下的候选内容，并保持项目及任何发布不变。它拒绝silk路径、过时的项目基线、重写的任务合同以及任何在 `revoke-task` 之后到达的候选内容。这些命令不会调用模型。通过此关卡的测试并不代表完成编码工具适配器。

## 在打包前检查证据

查看 [候选检查托管](../../docs/candidate-checks.md) 以获取完整的操作序列及其合格范围。`check-task` 冻结完整的候选项目，并准备一个非执行的宿主配置文件。接受命令针对其只读输入快照运行，可写的结果存放在 `$AN_LOOM_WORK` 中。`receive-check TASK HOST_JOB` 验证实际保留的宿主尝试并正式接受检查输出。手写的成功 JSON 无法满足此合同。`pack-task` 复制已经检查过的存档，而不是再次从源项目中选取字节。
