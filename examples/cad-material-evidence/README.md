# CAD material evidence

English · [简体中文](../../docs/zh-CN/examples/cad-material-evidence/README.md)

Use the local adapter when the operator supplies an exact accepted painting
reference. It selects one part from `data/scene.json` in the host's accepted ORA,
checks the artifact digest and drawing identity, and prints the material with
its provenance. It reads the accepted version through the host catalog.

```sh
python3 -B examples/cad-material-evidence/read_material.py \
  --host /path/to/an-agent-host --instance /path/to/accepted/instance \
  --paint-release "$APPROVED_PAINT_RELEASE" \
  --artifact-revision "$ACCEPTED_ORA_REVISION" --document handwheel \
  --epoch "$CAD_EPOCH" --revision "$CAD_REVISION" --part grip
```

The paired local worker qualification uses the same evidence as a bounded
peer context, with an exact worker release and a separate admission. The
worker receives the material and its provenance. The drawing mesh, ORA,
other parts, and instance paths remain with the operator. A verified worker
reply is evidence for the primary agent, which decides whether to reuse the
finish.

`tests/cli/cad_material_read.py` packs the worker with Loom and verifies its
accepted result on the native host. The test uses an accepted synthetic CAD
metadata fixture. The CLI can independently inspect prior accepted paintings
with the managed CAD metadata format.

## Compare with a hosted harness

`remote_compare.py` is an opt-in experiment for `public-grip.json`. Its local
input is `verified.json` from the native qualification. It compares the grip
material and drawing revision from that proof with the public fixture, then
projects a common citation from the public fixture bytes. Host instance IDs,
runtime paths, ORA bytes and local artifact revision hashes stay local.

Run the script once to preview the exact request and permitted tool result.

```sh
python3 -B examples/cad-material-evidence/remote_compare.py \
  --local-proof .local/cad-material-read/verified.json \
  --model gpt-6-astra --output .local/cad-remote-read
```

For a live run, install the reviewed OpenAI Python SDK interface with
`python3 -m pip install openai==3.46.0`. Use
an Agents API key in the environment, and pass the preview's SHA-256 as
`--approve` to the same command. The live run opens one turn with
`environment.type = none`. The preview includes a five-cent
provider session spending limit. Its only function can return the fixed public
record once. The output directory holds the intent, session and turn IDs,
disclosure intent, and verified result with elapsed time and available usage.
A lost stream leaves evidence for inspection. Each output directory can be
used once. The tool-result budget is local to the probe. Usage is measured
from the provider's completion event. A missing usage value is recorded as null.
The report separates request-to-action from tool-submission-to-completion
latency. These intervals use the host's clock, including stream delivery.

The native qualification writes `local-measurement.json` alongside the verified
proof. It measures from the first host admission request through proof checkout
and validation, excluding fixture creation and spool loading. This measurement
and the remote report cover the same public fixture and neither includes CAD
rendering.

[OpenAI Agents API function tools](https://developers.openai.com/api/docs/guides/agents-api/tools/functions)
