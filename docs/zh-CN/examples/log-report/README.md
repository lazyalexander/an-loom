# 日志报告工作流程

[English](../../../../examples/log-report/README.md) · 中文译文，以英文为准。 · [中文目录](../../README.md)

一个独立的CLI kit将合成的JSONL服务日志转换为可编辑的XLSX工作簿和一个原生的可编辑PPTX报告。它需要Python 3.9+以及已钉定的示例依赖项。对于业务计算，无需GPU、模型、Nextflow、Office安装、Loom或一个代理。

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt
.venv/bin/python kit/main.py workbook --input sample.jsonl --output report.xlsx
# Edit Owner and Review note in Excel or another compatible spreadsheet editor.
# Save a separate report-reviewed.xlsx.
.venv/bin/python kit/main.py slides --input report-reviewed.xlsx --output report.pptx
.venv/bin/python -m unittest discover -s tests -v
```

输出必须为新文件。该工作簿汇总了请求计数、HTTP 5xx 错误以及最近排名的 p95 延迟；其可编辑的审阅列会将内容导出为原生的 PowerPoint 文本和原生表格。该工作簿不会将幻灯片进行矢量化处理。摘要公式用于电子表格的显示；该幻灯片会读取已验证的服务行，而不是信任公式缓存。重新计算属于所选电子表格引擎的职责范围。小幻灯片布局支持三种服务，并拒绝溢出。

文档库是 `kit/workbook.py` 和 `kit/slides.py` 中可替换的适配器。`kit/data.py` 中的受限行模式拥有域含义。依赖版本被锁定，而非发布者认证或字节可复现的 OOXML 陈述。消费者本地检查限制 ZIP 展开，拒绝不支持的外部/宏部分和公式值的审阅单元格。它们不是通用的 Office 恶意软件扫描器。原生依赖安全性仍由操作员负责。

## 原生 spool 事件分发

原始 kit 保持独立可运行。`native.py` 使用这些相同的适配器，准备两个普通 Loom 项目，分别拥有工作簿和幻灯片生产过程的独立稳定条目。它将内容复制到新目录，且不会覆盖此示例。原生 v3 请求明确生成二进制产品和 Python 运行时；操作员需单独选择实际环境。不向 Loom 或宿主添加任何域实现。

```sh
# Install the pinned dependencies explicitly using the standalone steps above.
python3 native.py prepare /private/new-projects
python3 native.py runtime --python /selected/venv/bin/python \
  --output /private/new-runtime.local.json
an-loom check /private/new-projects/workbook
an-loom pack /private/new-projects/workbook
# Verify the returned archive and explicitly export it with export-native-spool-v3.
# Load each delivery through spool-load --runtime, review each start/delivery plan.
```

运行时辅助程序检查所选择的解释器，并验证已安装的版本是否符合requirements.txt中的要求，然后对所有站点包文件进行哈希处理。其本地JSON文件包含机器路径，必须不得进入公共包。宿主独立地锁定/重新检查所选的字节；标签和哈希不是供应链证书。应使用专用、稳定的环境。在受管理执行期间，站点启动钩子和.pth处理被禁用；需要此类钩子的依赖项位于此限定配置之外。

从Loom仓库的根目录开始，真实的配对接受是：

```sh
python3 -B tests/cli/native_documents.py --loom /selected/an-loom \
  --host /selected/an-agent-host --host-revision FULL_HOST_COMMIT \
  --python /selected/venv/bin/python --output /private/new-document-acceptance
```

它检查/打包/导出两个spools，生成并接受两个workbook版本，通过其精确的事件绑定交付第一个版本，接受/确认一个PPTX，然后收集私有工作/快照并重新读取产品。它验证原生文本/表格对象，而非截图。这是无头前台分发；它不声称拥有后台工作流程服务，实际GUI编辑，多输入状态组合，模型调用或通用Python环境管理。宿主接受拥有中断和运行时变化拒绝的覆盖。

## 可选宿主托管接受

宿主为生成可选。固定操作员套件此外还具备授权资格，不可变二进制版本，显式保存的编辑，钉定消费，取消和擦除收集。使用原生macOS以及协调的宿主修订版本，与工作区二进制输出和制品输入绑定。此审核工具配置不是敌对编码-代理沙箱。

```sh
python3 observe.py --host /selected/an-agent-host --host-revision FULL_COMMIT \
  --python /selected/venv/bin/python --output /private/new-acceptance \
  --loom /selected/an-loom
```

输出目录必须是新的，并且位于此包之外。操作员调用仅授权这些固定的fixture计划。生成的运行时配置文件、进程日志和工作文件保持私有。仅 `acceptance.json` 设计为可移植；在发布前请仔细检查。`documents/` 包含用于手动检查的已接受的工作簿/幻灯片版本。

该套件模拟使用文档库保存的操作员编辑。它不声称具有人类图形用户界面交互或认证的作者身份。在普通使用中，将该固定步骤替换为真实的保存工作簿：`artifact-checkout`创建一个工作副本，`artifact-edit`将批准的摘要提交到父修订版，并且下游的配置文件绑定所选修订版和已检出的快照。后续的编辑不会改变该绑定或自动触发执行。

仅取消模式的包装器会写入一个真实的PPTX文件并暂停。在活动期间接受，经过SIGINT确认的终止，以及一个晚来的候选替换被拒绝后，仍可接受。现有正式版本在擦除收集后仍可读。宿主将每个文档视为不可见的透明字节；文档布局和模式检查在此处保持不变。没有宿主、Loom、模型或引擎被`kit/`导入。

## Loom packaging

```sh
an-loom check .
an-loom build .
an-loom pack .
```

这是一个通用内容包，包含CLI源码、依赖项和领域检查。它不扩大现有的固定 Rust `file-kit-v1` 部署适配器，也不从包元数据中授予运行时权限。可选的 `tools/host.py` 适配器是一个显式的宿主合同依赖项；替换它时不要改变业务套件。

## 实桌面保存接受

可选的 [桌面操作工具](desktop/README.md) 保持图形界面交互的可替换性，并将实际保存记录与脚本固定_fixture 分开。参见 [测量的 LibreOffice roundtrip](acceptance/native-office/README.md) 以获取保存的修订证据，一个固定 v2 下游运行和加载/卸载观察。此手动接受补充 CI；CI 不声称操作一个真实桌面。
