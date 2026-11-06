# CAD绘画发布

[English](../../../../examples/cad-paint/DISTRIBUTION.md) · 中文译文，以英文为准。 · [中文目录](../../README.md)

此目录包含CAD绘画代理策略、Python silks、可编辑外观套件和域检查。Web CAD和Krita集成位于an-shuttle中。源代码从[an-shuttle PR 4](https://github.com/lazyalexander/an-shuttle/pull/4)迁移至`e6a4a882730a2305cf7097887f923785a55c04cf`。v0.1.0 Loom包保留发布摘要`1a380dff33ca0fc569d306e6de67af505544bb4185d7db086f600b9bc63ee4a1`。

[版本0.1.0客户端分发](../../../../examples/cad-paint/releases/an-loom-cad-paint-spool-0.1.0.tgz)包含经过验证的Loom存档和一个用于TypeScript客户端的编译后的ES模块。原生代码和需求列表均来自同一经过验证的包。通过固定Git提交的URL进行安装，并将npm锁文件与其完整性哈希一起提交。安装过程中不会运行包钩子。宿主可以将文件导出为`@an-loom/cad-paint-spool/archive`传递给`an-loom verify`和`export-native-spool-v3`。

要从此目录检查已审核的客户端，请使用Python 3.12+、npm和构建好的Loom CLI：

```sh
npm ci --ignore-scripts --no-audit --no-fund
python3 pack-client.py --loom ../../target/debug/an-loom --tsc node_modules/.bin/tsc --output releases --check
```

该检查将分布式文件和嵌入式存档与独立重建的包进行比较。对于新版本发布，设置两个包版本，运行相同的命令而不使用 `--check`，并审查生成的版本化分发。在准备好时，为每个消费者选择新的不可变URL和完整性。保留早期存档以供固定客户端使用。
