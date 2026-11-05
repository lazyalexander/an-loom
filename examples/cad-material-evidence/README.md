# CAD material evidence

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
