# Use the existing finite MCP service from a native client

Codex connects directly to `an-agent-tools` over its native stdio MCP support.
The official `rmcp` SDK already supplies the server transport. No Codex SDK,
proxy, agent loop, reply parser or new host tool is needed. The existing file-list
kit owns the domain code and tests; the shared prompt selects the task. The host
owns granted kit execution, termination and formal artifact publication.

## Prepare and inspect the existing fixture

Use native macOS, the built Loom/host/tools binaries and the selected Rust
toolchain profile described in [development](../../docs/development.md).
The existing preparation builds only the public sort kit and cancellation
variant. No model client is selected or invoked by this step. The fixture exposes
only `prepare`, `check` and `accept`; it never starts, waits for or signals a
coding client. This leaves no per-provider launch command to maintain.

```sh
python3 -B examples/finite-mcp/qualify.py prepare --output /new/mcp-fixture \
  --loom /selected/an-loom --host /selected/an-agent-host \
  --bridge /selected/an-agent-tools --toolchain /selected/toolchain.local.json
```

Inspect `development-approval.local.json` and the public
[development prompt](development-prompt.txt). Select `REVIEW` as that new absolute
fixture directory, `BRIDGE` as the same tools binary, `APPROVAL` as the exact
reviewed digest, and `CODEX_MODEL` as your chosen model. Protect the fixture from
concurrent edits. Keep native authentication in its original client store.

## Invoke the native client once

The qualified command uses Codex 0.166.0. These are native configuration flags,
not a Loom provider API. For a different client/version, inspect its actual
configuration and requalify rather than assuming matching flag semantics.
The [historical Kimi route](../kimi-kit/README.md) retains its fixed adapter and
evidence; its client-bound directories are not migrated by this fixture.

```sh
mkdir "$REVIEW/development-client" || exit 1
# Exclusive submission marker: do not remove it to replay an uncertain request.
mkdir "$REVIEW/development-submitted" || exit 1
codex exec --ignore-user-config --strict-config --sandbox read-only \
  --skip-git-repo-check --ephemeral --model "$CODEX_MODEL" \
  --cd "$REVIEW/development-client" --json \
  --disable shell_tool --disable multi_agent --disable plugins --disable apps \
  --disable hooks --disable browser_use --disable computer_use \
  --disable image_generation --disable view_image --disable memories \
  --disable skill_search --disable skill_mcp_dependency_install --disable goals \
  -c 'web_search="disabled"' -c 'project_doc_max_bytes=0' \
  -c 'skills.include_instructions=false' \
  -c "mcp_servers.host.command=\"$BRIDGE\"" \
  -c "mcp_servers.host.args=[\"$REVIEW/development-grant.local.json\",\"--approve\",\"$APPROVAL\"]" \
  -c 'mcp_servers.host.enabled_tools=["read_source","run_kit","propose_source"]' \
  -c 'mcp_servers.host.required=true' -c 'mcp_servers.host.tool_timeout_sec=45' \
  -c 'mcp_servers.host.default_tools_approval_mode="prompt"' \
  -c 'mcp_servers.host.tools.read_source.approval_mode="approve"' \
  -c 'mcp_servers.host.tools.run_kit.approval_mode="approve"' \
  -c 'mcp_servers.host.tools.propose_source.approval_mode="approve"' \
  --output-last-message "$REVIEW/development-last.txt" - \
  < examples/finite-mcp/development-prompt.txt \
  > "$REVIEW/development-events.jsonl" 2> "$REVIEW/development-stderr.txt" || exit 1
python3 -B examples/finite-mcp/qualify.py check --output "$REVIEW"
```

Inspect the complete `development-session/calls/edit-1/candidate.rs` and its
digest before adopting it:

```sh
python3 -B examples/finite-mcp/qualify.py accept --output "$REVIEW" \
  --source-sha256 REVIEWED_CANDIDATE_DIGEST
```

Only the three named MCP tools receive client approval. The server still checks
the exact host grant, handles, request fields and invocation identities. A client
approval cannot compile the submitted candidate or enlarge those capabilities.
The `check` stage reads durable host results, not the model's final message.
`accept` separately builds/tests, packages, registers and executes the reviewed
source through the existing host. Its model field is unknown for externally
invoked clients; selecting a native client does not attest candidate authorship.

## Evidence and limits

The unchanged [portable acceptance](../../docs/acceptance/codex-mcp.json) records actual
Codex calls through the existing host, an ungranted-handle refusal, independently
built candidate output and fixed-version checkout. A second native turn was
interrupted after observed kit activity; the host confirmed cancellation and
reaping without publication. The earlier accepted artifact remained readable.
Its verifier digest identifies the [qualified script](https://github.com/lazyalexander/an-loom/blob/5068ef8442795b42d99573052ec1476c79cef0ac/examples/kimi-kit/qualify.py)
before the shared fixture moved here; retirement does not rewrite that historical
record or claim another model invocation.

This is interoperability with a trusted native client, not containment of that
client. Feature flags narrow incidental tools; read-only shell policy and an MCP
allowlist do not prove that all other native tools are absent. Public source and
host tool results enter the selected model connection. Native logs, plans and
machine paths remain private. No native session store is read or copied.

The cancellation fixture uses the separately reviewed cancellation grant and a
new native invocation. Call `run_kit` once with invocation `cancel-1`, handle
`wait` and `wait` followed by a newline. Send SIGINT only after the actual native
run writes its `pending` marker. Require the durable tool result and native
receipt to report cancellation, reaping and no accepted artifact. A CLI exit,
timeout or missing response alone is not success; never resubmit uncertainty.
This verifies local kit cancellation, not remote model termination or rollback.

Official client references: [MCP configuration](https://learn.chatgpt.com/docs/extend/mcp)
and [feature/configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference).
