# CAD painting distribution handoff

The authoritative source and reproduction command are now in
[an-spool](https://github.com/lazyalexander/an-spool/tree/a9bffc3f757f7c82c4dcbca78d063f317d38a27d/spools/cad-paint/DISTRIBUTION.md). The first migration preserves npm name
`@an-loom/cad-paint-spool`, version 0.1.0 and Loom release digest
`1a380dff33ca0fc569d306e6de67af505544bb4185d7db086f600b9bc63ee4a1`.

Consumers pin the immutable an-spool distribution and npm integrity. Loom CI
checks out that exact public commit and reproduces its client using this Loom
binary, preserving the packaging compatibility gate without owning the domain
source. Historical immutable Loom release URLs remain valid.
