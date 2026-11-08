# an-loom

English · [简体中文](docs/zh-CN/README.md)

A development and packaging workbench for reusable domain spools.

Loom prepares bounded coding tasks, validates selected source and evidence, and
packages reviewed bytes. Existing coding tools own editing and model loops.
Runtime permissions, scheduling, lifecycle and artifact custody belong to an-agent.
Mature domain implementations live in an-spool; Shuttle connects applications.

## Start

```sh
cargo install --path . --locked
an-loom init-native ./my-spool
an-loom check ./my-spool
an-loom pack ./my-spool
an-loom verify ARCHIVE
```

Replace `ARCHIVE` with the path returned by `pack`. Packaging creates no execution
grant. See [the native starter](examples/native-spool/README.md),
[development walkthrough](docs/development.md), [native handoff](docs/develop-native.md)
and [coding-tool contract](docs/code-agents.md).

Every release baseline file is **silk**; authorized mutable copies and newly
created work are **kit**. Instructions remain protected. Runtime authority is
checked by the host, not inferred from source spellings or these labels.
[Content contract](docs/spool-code.md) · [Repository boundaries](docs/spool-boundary.md)

## Example: CAD → Krita

Loom packages a domain spool that turns saved CAD geometry into a layered painting.
The mature source is maintained in an-spool; the application integration is in Shuttle.

![CAD painting](https://raw.githubusercontent.com/lazyalexander/an-shuttle/0dfd646/examples/cad-paint/sample/vise.png)

[Package handoff](examples/cad-paint/DISTRIBUTION.md) ·
[Application setup and acceptance](https://github.com/lazyalexander/an-shuttle) ·
[Domain source](https://github.com/lazyalexander/an-spool)

## Contracts and development

Explicit include lists, deterministic inventories and bounded archives preserve
selected package identities. Source inspection supplies diagnostics; qualified
host profiles enforce operations. Hashes are not publisher signatures, and
partial or uncertain execution is not automatically retried.

See [architecture](docs/architecture.md), [platform support](docs/platforms.md),
[acceptance](docs/acceptance.md), [quality gates](docs/quality.md) and
[contribution requirements](CONTRIBUTING.md). Run formatting, locked tests,
strict clippy, `cargo quality --base origin/dev-advance` and quality-tool tests.
Real host acceptance is a separate required check for execution claims.

Other references: [text spool](examples/native-spool/README.md),
[log → workbook → slides](docs/log-report-workflow.md),
[finite MCP](examples/finite-mcp/README.md), [Nextflow](docs/nextflow-observation.md).
These are examples, not business capabilities built into Loom.
