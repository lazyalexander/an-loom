# CAD painting release

This directory owns the CAD painting agent policy, Python silks, editable appearance kit and domain checks. The web CAD and Krita integrations live in an-shuttle. The source moved from [an-shuttle PR 4](https://github.com/lazyalexander/an-shuttle/pull/4) at `e6a4a882730a2305cf7097887f923785a55c04cf`. The v0.1.0 Loom bundle retains release digest `1a380dff33ca0fc569d306e6de67af505544bb4185d7db086f600b9bc63ee4a1`.

[Version 0.1.0 client distribution](releases/an-loom-cad-paint-spool-0.1.0.tgz) contains the verified Loom archive and a compiled ES module for TypeScript clients. Native code and the requirements list are taken from the same verified bundle. Install from the URL of a fixed Git commit and commit the npm lockfile with its integrity hash. Installation runs no package hooks. The host can pass the file exported as `@an-loom/cad-paint-spool/archive` to `an-loom verify` and `export-native-spool-v3`.

To check the reviewed client from this directory, use Python 3.12+, npm, and a built Loom CLI:

```sh
npm ci --ignore-scripts --no-audit --no-fund
python3 pack-client.py --loom ../../target/debug/an-loom --tsc node_modules/.bin/tsc --output releases --check
```

The check compares the distributed files and embedded archive with the independently rebuilt package. For a new release, set both package versions, run the same command without `--check`, and review the resulting versioned distribution. Select the new immutable URL and integrity in each consumer when ready. Retain earlier archives for pinned clients.
