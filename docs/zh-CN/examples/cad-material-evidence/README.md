# CAD材料证据

[English](../../../../examples/cad-material-evidence/README.md) · 中文译文，以英文为准。 · [中文目录](../../README.md)

当操作员提供一个确切的被接受的绘画参考时，使用本地适配器。它从宿主的接受ORA中选择一个部件`data/scene.json`，检查艺术品摘要和图纸身份，并打印带有其来源的材料。它通过宿主目录读取被接受的版本。

```sh
python3 -B examples/cad-material-evidence/read_material.py \
  --host /path/to/an-agent-host --instance /path/to/accepted/instance \
  --paint-release "$APPROVED_PAINT_RELEASE" \
  --artifact-revision "$ACCEPTED_ORA_REVISION" --document handwheel \
  --epoch "$CAD_EPOCH" --revision "$CAD_REVISION" --part grip
```

配对的本地工作者资格使用与有界对等上下文相同的证据，具有精确的工作者发布版本和独立的准入。工作者接收材料及其来源。绘制网格、ORA、其他部分和实例路径仍保留在操作员处。经过验证的工作者回复是主代理的证据，主代理决定是否重用完成结果。

`tests/cli/cad_material_read.py` 使用 Loom 将工作者打包，并在原生宿主上验证其接受的结果。测试使用一个接受的合成 CAD 元数据 fixture。CLI 可以独立检查之前接受的绘画，使用管理的 CAD 元数据格式。

## 与托管 harness 进行比较

`remote_compare.py` 是 `public-grip.json` 的一个可选实验。其本地输入来自原生资格的 `verified.json`。它将该证明中的抓取材料和绘制修订与公共 fixture 进行比较，然后从公共 fixture 字节中投影出一个通用引用。宿主实例 ID、运行时路径、ORA 字节和本地产物修订哈希值保留在本地。

运行脚本一次以预览精确的请求和允许的工具结果。

```sh
python3 -B examples/cad-material-evidence/remote_compare.py \
  --local-proof .local/cad-material-read/verified.json \
  --model gpt-6-astra --output .local/cad-remote-read
```

对于实时运行，请安装已审核的 OpenAI Python SDK 接口与
`python3 -m pip install openai==3.46.0`。使用
一个 Agents API 密钥在环境变量中，并将预览的 SHA-256 作为
`--approve` 传递给相同的命令。实时运行开启一个回合与
`environment.type = none`。预览包含一个五美分
提供者会话消费限额。其唯一功能可以返回固定公共
记录一次。输出目录包含意图、会话和回合 ID、披露意图以及验证结果、经过时间及可用使用情况。
丢失的流会留下证据供检查。每个输出目录只能使用一次。工具结果预算与探针相关。使用情况从提供者的完成事件开始计算。缺失的使用值记录为 null。
报告将请求到动作的时间与工具提交到完成的时间延迟分开。这些间隔使用宿主的时钟，包括流的传输。

原生资格认证会在验证证明旁边写入 `local-measurement.json`。它从第一个宿主准入请求开始计算，经过证明检查和验证，不包括固定装置创建和 spool 加载。此测量与远程报告覆盖相同的公共固定装置，且两者都不包括 CAD 渲染。

[OpenAI Agents API 函数工具](https://developers.openai.com/api/docs/guides/agents-api/tools/functions)

CAD 工作台通过 `--delegated` 在访问产物前核验 `delegation.json`。第 2 版显式列出披露模式；本地调用仍默认 `local-only`。Codex 调用还须指定 `--disclosure codex-selected-material`，且声明必须允许该模式。旧的第 1 版仅限本地声明仍可读取，但不能授权远程调用。工作台在提交任何 Codex 任务前，通过 `--check-delegation --document handwheel --part grip --disclosure codex-selected-material` 预检披露许可；预检不会读取产物。

Codex 模式允许发送所选图纸、部件、已验收修订、产物修订和材质元数据，以及 Shuttle 生成的本次证据编号。ORA、网格、其他材质、文档世代、发布编号、文件路径和原生记录均留在本地。操作员在工作台选择历史查证时授权这项披露。读取器核验已验收产物、文档世代和确切修订；Shuttle 核验单次读取范围与证据采纳。声明须放在读取器旁，与打包的原生同级读取契约各自独立。Codex 承担 agent 过程；读取器只解析证据。
