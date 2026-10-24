# an-loom contributor contract

- Rust 2024; deterministic plans and validation form the functional core. Keep
  filesystem and host CLI effects in explicit adapters. Payload languages are open.
- English only for repository documentation, comments and examples.
- Main is stable. Submit feature branches as pull requests to main. The maintainer
  reviews and merges; never merge on their behalf or force push.
- Loom owns development and packaging; an-agent owns execution authority, runtime
  model/context services, instance state, artifact custody and audit. No runtime
  dependency from an-agent to Loom; communicate through explicit public contracts.
- Domain decisions, algorithms, state schemas and domain checks belong to spool
  projects. Reuse existing coding tools; do not implement another coding agent.
- Never execute project commands implicitly. Controlled build requests must expose
  exact commands, selected inputs, output boundaries and approvals. Host grants
  cannot come from source, models, package metadata or analysis findings.
- Preserve allowlists, path containment, secret checks, deterministic identities,
  exclusive destinations and archive verification when migrating code.
- Keep credentials, machine profiles, absolute user paths and private records out
  of public files. Record portable evidence separately from local operational data.
- Preserve upstream licenses and provenance. Keep old an-waver available until
  replacement acceptance is reviewed and migration is complete.
- Before committing: cargo fmt --all -- --check; cargo test --locked;
  cargo clippy --all-targets --locked -- -D warnings. Real host integration checks
  are required before claiming controlled build or execution acceptance.
