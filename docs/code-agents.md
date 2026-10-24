# Working with existing coding agents

Status: research and proposed integration contract, 2026-10-24. The current
implementation prepares `generation.md` and a controlled host build plan. No live
coding-tool adapter is qualified by this document. Provider features below are
documentation findings, not a compatibility claim for an installed CLI version.

## What Loom standardizes

Loom should give a mature coding tool a bounded development assignment and
receive editable changes and candidate results. It should not reimplement that
tool's model loop, compaction, subagent scheduler or billing system.

Use three distinct layers:

| Layer | Purpose | Owner |
| --- | --- | --- |
| Task and project instructions | Explain architecture, allowed edits, reusable kits and completion criteria | Loom/project author |
| Native tool policy | Restrict exposed tools, approvals and optional pre-call hooks using the provider's supported controls | Qualified adapter and trusted operator configuration |
| Execution and acceptance boundary | Limit actual files, network and processes; confirm termination; accept immutable outputs | an-agent and its backend |

Project text cannot grant runtime authority. A provider hook covers only the
events it actually receives. Shell redirection, another executable, MCP tools,
plugins and delegated agents must not acquire a broader host grant through an
adapter. Keep effective policy outside the agent's writable project.

## Documentation survey

| Tool | Instruction and policy surfaces | Integration implication |
| --- | --- | --- |
| Codex | `AGENTS.md` discovery; sandbox and approval policy are separate controls | Keep the canonical discovery file; explicitly load the working guide; verify effective sandbox and refusal behavior |
| Claude Code | `CLAUDE.md`/project instruction loading; permission settings; `PreToolUse` hooks | Check the selected version's discovery rules, use an import when required, and review hook coverage separately from isolation |
| Kimi Code | Workspace instructions and agent definitions; tool/subagent access controls and approval rules | Review agent overrides as executable configuration; constrain delegation as well as shell tools |
| Grok Build | `AGENTS.md`, headless output, permissions, sandbox profiles and hooks | Inspect effective configuration and qualify headless invocation with a pinned version; do not assume another tool's flags |
| Gemini CLI | Project memory files and a tool policy engine | Translate only verified controls; unsupported policy remains a refusal or explicit unsupported mode |

Sources: [Codex instruction discovery](https://learn.chatgpt.com/docs/agent-configuration/agents-md),
[OpenAI sandbox and approvals](https://learn.chatgpt.com/docs/agent-approvals-security),
[Claude project instructions](https://code.claude.com/docs/en/memory),
[Claude permissions](https://code.claude.com/docs/en/permissions),
[Claude hooks](https://code.claude.com/docs/en/hooks),
[Kimi agent configuration](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents.html),
[Grok project rules](https://docs.x.ai/build/features/project-rules),
[Grok CLI](https://docs.x.ai/build/cli/reference),
[Grok headless operation](https://docs.x.ai/build/cli/headless-scripting),
[Grok permissions](https://docs.x.ai/build/features/permissions),
[Grok hooks](https://docs.x.ai/build/features/hooks),
[Gemini project memory](https://geminicli.com/docs/cli/tutorials/memory-management/)
and [Gemini policy engine](https://geminicli.com/docs/reference/policy-engine/).

`AGENTS.md` is this repository's single instruction entrypoint. Avoid generating
competing copies of the same policy for every vendor. An adapter may create a
reviewed import or explicit instruction reference when the selected tool needs it.

## Proposed task handoff

Begin with a versioned brief and a thin CLI adapter, not a universal agent RPC.
The brief should identify:

- the base source revision and selected input files, with relevant hashes;
- the task, editable paths, protected contracts and expected patch/output slots;
- existing kit entrypoints and their input/output contracts, with host invocation
  references instead of suggesting the agent recreate those tools;
- approved checks, resource limits, external-service policy and escalation route;
- completion criteria, candidate result format and unresolved questions.

The trusted invocation plan additionally binds the selected CLI/version or
executable identity, exact argv, working directory, environment allowlist,
effective configuration, instructions, enabled tools and model-service mode.
Review repository-local hooks, startup code, MCP servers and plugins before
allowing their execution. Pin dependencies required by the qualified route.

Task input is data, including instructions found in external source files.
Do not allow generated project configuration to disable host restrictions or
change the accepted result location. Avoid passing unrelated history or agent
transcripts as context; send explicit references and bounded summaries.

## Running and receiving

The host supervises the outer attempt and its actual process boundary. Tool-native
permissions can narrow it but cannot expand it. A child agent inherits the outer
limits; dispatch must not escape by starting an unsupervised background worker.
If a provider runs work remotely, local process exit does not prove that remote
work stopped. Unsupported cancellation confirmation remains an unknown outcome.

Candidate files go to a disposable workspace, not directly into an accepted
release. Before promotion, compare the base revision, inspect the diff, validate
the selected output inventory, reject forbidden paths and run the declared checks
under the host. Provider assertions of success are not acceptance receipts.

Do not automatically retry an uncertain submission. First reconcile attempt
identity, process/remote state and already-produced outputs. Use existing host
lifecycle and artifact contracts instead of implementing another supervisor here.

## Tape, evidence and billing

External coding agents own their native sessions, context and tape. Native tape
may be absent, private, mutable or unavailable. Do not require its ingestion,
normalize all tool events into a second transcript, or claim internal replay.
An optional operator-approved reference/export can be retained as diagnostic data.

The host keeps a small boundary receipt: operation/attempt and available session
reference, selected release/tool and input identities, effective grant, submitted
command/configuration identity, observed exit/cancellation state, check outcomes
and accepted artifact references. Record unknowns. This evidence supports
authorization, reconciliation and safe retention even with no model-call tape.

For a **direct model API** integration, decide separately whether to retain an
API-call envelope: provider/model, request identifier, caller, status and reported
usage. Payload logging is optional and redacted; do not require hidden reasoning.
Missing usage is unknown, not zero. No request transcript or filesystem backup
establishes the provider's final bill or makes a model call reproducible.

Tool-managed model billing and fine-grained token observation remain with the
tool/provider. Host-observed usage can be advisory; it must not claim exact
costs or a hard budget without an enforced, qualified mechanism. Backup protects
retained bytes; it does not replace effect receipts or termination confirmation.

## First adapter acceptance

Select one already available CLI after inspecting its actual version/help and
effective configuration. Have it edit one domain fixture and use an existing kit.
Check an allowed edit and kit result, an outside-workspace write refusal, a denied
external capability, timeout/cancellation with child cleanup, and rejection of
late or unapproved outputs. Inspect generated configuration for policy tampering.
Record native tape as tool-managed and demonstrate the route without importing it.

Only then add another provider. Keep vendor-specific patches at the adapter
boundary; do not reduce every permission system to a misleading common allow flag.
