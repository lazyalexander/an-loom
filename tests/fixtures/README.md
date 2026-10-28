# Historical native release

`native-spool-85c832a.tar.gz` was produced by the unmodified Loom implementation
at commit `85c832a` using its `examples/native-spool` project and `pack` command.
It retains the six-field native v1 contract. Tests verify and export these actual
historical bytes without adding authoring metadata or changing release identity.

Release digest prefix: `631ebec19015`. The fixture contains only the public
example, never local host receipts, credentials or session data. Do not regenerate
it with the current packager to make a compatibility test pass.
