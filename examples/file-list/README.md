# File-list kit

A CLI-first file transformer: input.txt to output.txt. Unit tests cover the
pure transform. The host supplies a private read-only input snapshot and permits
only the output file. The kit has no network, child-process or model capability.
The output is opened with create-or-truncate: Linux may supply a host-created
output inode. Do not require an absent output file, use atomic rename, or create
temporary files; the fixed ABI permits only the declared output.
The source and executable are included in the spool package for inspection.

Use the host's approved isolated build before Loom receive-build, declare and
pack. Loom build snapshots package files; it is not a compiler invocation.
Compilation currently uses the macOS host backend; unsupported platforms refuse.
The Linux file runtime may run a separately built, reviewed native program;
this does not add a Linux isolated-build backend or let Loom execute code.
Keep failed build evidence, and do not overwrite an existing accepted binary.
