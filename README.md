# an-loom

A Rust development workbench for reusable domain spools.

Loom helps developers turn domain knowledge, existing CLI programs, skills and
application integrations into inspectable releases. It owns authoring, assembly,
validation orchestration and packaging. Domain behavior belongs to spool projects;
an-agent owns runtime authority, model/context services, state and artifact
custody. Execution backends enforce isolation; shuttles connect applications.

This repository starts with the architecture and contributor contract. The first
implementation will migrate identified packaging and validation capabilities from
an-waver and qualify a real lightweight CLI kit through the independent host.

See [the architecture](docs/architecture.md). This project is CLI-first and follows
a functional core / imperative shell design.
