# Kimi calls a kit through finite host capabilities

This operator acceptance uses Kimi Code 2.3.1 and its existing authenticated
model connection. It qualifies one tool profile, not arbitrary Kimi plugins,
hooks, user agents or a sandbox for a malicious native coding-client executable.
Do not reuse the host's trusted workspace command runner as that sandbox.

The model may read one selected source, submit candidate text and call named kit
handles. No shell or native file tool is exposed. The [Kimi tool profile](kimi-profile.md) is vendor executable configuration, not
a second repository instruction guide. The official client's custom
agent allowlist is enforced at execution; the host independently rejects paths,
commands, ungranted handles and duplicate invocation identities. Prompt wording
is not the security boundary. Review global client configuration before running:
project isolation cannot neutralize a malicious trusted CLI configuration.

## Ownership

- Kimi owns model calls, context and native tape. Authentication stays in its
  original location; this example never copies configuration or links credentials.
- an-agent owns the finite MCP service, native kit sandbox, cancellation,
  termination receipts and artifact acceptance. Its official `rmcp` transport
  reuses the existing file executor and versioned artifact store.
- Loom scaffolds, checks, receives build evidence and packages reviewed source.
  It does not link the host or contain model/runtime supervision code.
- The file-list kit owns sorting and case-folding behavior and domain tests.

## Prerequisites and execution

Use native macOS for this acceptance: the existing isolated Rust development
profile is macOS-only. The underlying file executor/MCP regression has Linux CI;
this does not qualify paid Kimi development inside Linux. Python 3 is an operator
fixture dependency, not a Loom runtime dependency. No GPU is needed.

Build Loom and both `an-agent-host` and `an-agent-tools` from the coordinated host
PR. Prepare the operator-owned Rust toolchain profile described in
[development.md](../../docs/development.md). The driver pins selected executable files, toolchain profile and instructions by
digest and refuses changes between stages; this does not attest the full client
or toolchain dependency graph. Use a **new** output directory outside
package include lists. Preparation explicitly authorizes building only the
committed file-list fixture and its 30-second cancellation variant:

```sh
python3 examples/kimi-kit/qualify.py prepare --output .local/kimi-acceptance \
  --loom target/debug/an-loom --host /selected/an-agent-host \
  --bridge /selected/an-agent-tools --toolchain /selected/toolchain.local.json \
  --kimi /selected/kimi
# Review development-approval.local.json: source, kit releases, store and limits.
python3 examples/kimi-kit/qualify.py run --output .local/kimi-acceptance \
  --approve REVIEWED_DEVELOPMENT_DIGEST
# Review development-session/calls/edit-1/candidate.rs and its returned digest.
python3 examples/kimi-kit/qualify.py accept --output .local/kimi-acceptance \
  --source-sha256 REVIEWED_CANDIDATE_DIGEST
# Review cancellation-approval.local.json separately.
python3 examples/kimi-kit/qualify.py cancel --output .local/kimi-acceptance \
  --approve REVIEWED_CANCELLATION_DIGEST
```

`run` selects `kimi-code/k3`: inspect source, invoke the existing sort kit, request
an ungranted handle and propose a case-folding improvement with a domain test.
Only `accept` can adopt the reviewed digest into a fresh project. It uses the
existing host-isolated build/test, Loom release verification, host registration
and native execution; the original fixture is unchanged.

`cancel` selects `kimi-code/k3-256k`. The driver sends SIGINT only after the actual
kit writes a pending marker inside its sandbox. It then requires a cancelled,
reaped native receipt and no accepted artifact. Failure to reach kit execution
is an inconclusive attempt, not a cancellation pass. Kimi's default configured
reasoning is retained; the profile does not invent provider reasoning flags.

The generated workspace is operator-owned and contains only this reviewed MCP
configuration; `KIMI_CODE_TRUST_WORKSPACE=1` trusts that fixture, not arbitrary
user repositories. An empty skills directory and explicit tool-only agent
disable native model tools. No broad CLI auto-approval flag is supplied. Headless
client behavior still does not replace the finite host grant.

## Evidence and limits

Keep `.local` files private: plans contain machine paths and native client logs
may contain project data. Publish only the closed summary in
[acceptance evidence](../../docs/acceptance/kimi-development.json), never raw tape,
authentication files or process IDs. Failed/uncertain calls are inspected and
never automatically retried; the driver requires fresh destinations.

Cancellation revokes an active local kit, not remote model generation or unknown
remote side effects. A provider CLI's exit alone is insufficient evidence. MCP
notification cancellation and transport EOF are also covered by the host's
credential-free native regression. This is a scoped development route; it does
not add arbitrary code execution authority or a general workflow ABI.

Official client contracts:
[custom agents](https://moonshotai.github.io/kimi-code/en/customization/agents.html),
[MCP configuration](https://moonshotai.github.io/kimi-code/en/customization/mcp.html).
