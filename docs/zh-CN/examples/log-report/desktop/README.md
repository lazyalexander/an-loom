# 原生编辑器接受度（可选操作工具）

[English](../../../../../examples/log-report/desktop/README.md) · 中文译文，以英文为准。 · [中文目录](../../../README.md)

这些工具用于阶段化和检查真实的桌面保存。它们从不编辑工作簿单元格。
独立的工具仍然只需要其自身的 `requirements.txt`；Loom 或 an-agent 不依赖于 LibreOffice、psutil 或 matplotlib。

## 边界条件

操作员可替换的桌面适配器打开检查出的 XLSX 文件，编辑单元格并调用保存。可以使用 Excel 或 LibreOffice。示例验证选定的业务字段为只读；宿主接受精确批准的字节到其现有的制品目录中。编辑器生产者属性只是一个提示，而非作者身份的证明。Loom 打包文件；宿主拥有修订的保管权、固定输入绑定和已解决的工作区回收。这不会引入穿梭 ABI、执行后端或不受信任代理的桌面沙盒。

使用一个专用的编辑器会话，不包含其他文档。从本目录的 requirements 安装观察者依赖项到一个单独的环境中。使用该环境调用 `session.py`。保留业务环境的 Python 启动器路径；不要解析其虚拟环境符号链接。

## 手动原生往返操作

1. 运行 `session.py prepare --root NEW_DIRECTORY --host HOST_BINARY
   --host-revision FULL_COMMIT --python KIT_VENV/bin/python
   --observer-python OBSERVER_VENV/bin/python --editor-executable EXACT_EXECUTABLE`。
   可选地使用 `--working-directory VISIBLE_DIRECTORY` 用于原生文件对话框，这些对话框不接受隐藏的项目路径。`report.xlsx` 必须不存在。
   使用规范路径（在 macOS 上，使用 `/private/tmp` 而不是 `/tmp`）。
2. 通过真实的桌面 UI 打开返回的工作副本。仅在对应操作实际被观察到时标记 `editor.window.ready`、`document.open.requested` 和 `document.opened`。
3. 编辑 `Report!E8`（所有者）和 `F8`（审查注释）。在对应操作实际被观察到时标记 `document.save.v2.requested`，保存 XLSX 文件并在应用程序中验证完成，然后标记 `document.saved.v2`。
4. 哈希保存的文件，审查选定的字节并运行 `session.py commit
   --root DIRECTORY --label v2 --approve SHA256 --expected-note EXACT_CELL_TEXT`。
5. 再次在真实 UI 中更改 F8，保存并以相同方式提交 v3，使用 v3 事件。
6. 标记 `document.close.requested`，关闭文档，标记 `document.closed`。
   标记 `editor.quit.requested`，退出专用应用程序并验证退出。
7. 运行 `session.py finish --root DIRECTORY`。它需要两个保存的修订版本和一个观察到的编辑器退出。它在 v3 存在后选择 v2，运行展示工具包，验证 v2 的来源和注释文本，回收已解决的临时文件并检查所有三个正式修订版本是否仍然可读。
8. 在观察者环境中运行 `report.py --root DIRECTORY --output NEW_REPORT_DIRECTORY`。不完整或过期的捕获无法成为成功报告。

使用 `session.py mark --root DIRECTORY --event EVENT` 记录观察到的 UI 步骤。
这些标记是操作员的认证，而非自动验证的 GUI 收据。
绝不要将尝试的操作标记为完成。不要自动重试不确定的宿主操作。首先进行协调。

## 观察和限制

250 毫秒的采样器仅观察选定的编辑器可执行文件、显式监控的宿主进程及其子进程。进程标识包括创建时间。
私有监控记录包含 PID；便携式样本不包含命令行、环境、PID 或机器路径。在发布前检查任何接受总结。
实际文档可能包含编辑器添加的作者元数据，并应保持本地，除非单独审查。

图表区分进程存在性、GUI文档状态、临时数据保留和正式产物保留。它们不展示VM/spool热加载。RSS总和可能会将共享页面计算多次。短任务和最终CPU爆发可能被遗漏。CPU采样的是进程时间，而非整个系统的能耗。UI窗口包括操作员和自动化延迟，因此时间不是应用程序的基准。热重启和首次运行设置必须单独报告。观察者过期会留下私有诊断证据，不得报告为完成验收。
