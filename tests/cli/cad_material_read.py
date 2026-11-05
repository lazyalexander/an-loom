#!/usr/bin/env python3
"""Exercise an accepted CAD ORA reference through the bounded native worker."""
import argparse
from base64 import b64encode
from hashlib import sha256
import importlib.util
import json
from pathlib import Path
import time
from zipfile import ZipFile

from host_qualification import require_native_workspace

EXAMPLE = Path(__file__).resolve().parents[2] / "examples/cad-material-evidence"
loader = importlib.util.spec_from_file_location("read_material", EXAMPLE / "read_material.py")
reader = importlib.util.module_from_spec(loader)
loader.loader.exec_module(reader)


def call(binary, *args):
    return reader.host_call(binary, *args)


def approve(host, *args):
    preview = call(host, *args)
    return call(host, *args, "--approve", preview["approval_sha256"])


def release(loom, root, name, output, code, declaration=None, binary=False):
    project = root / name
    silk = project / "silk"
    silk.mkdir(parents=True)
    (silk / "main.sh").write_text(code)
    if declaration:
        (silk / "peer-read.json").write_text(json.dumps(declaration))
    files = ["silk/main.sh"] + (["silk/peer-read.json"] if declaration else [])
    (project / "loom.toml").write_text(f'schema_version = 1\nname = "{name}"\nversion = "0.1.0"\n'
        'description = "A bounded CAD material evidence qualification"\n'
        'include = ["loom.toml", "native-spool.toml", "silk"]\n')
    metadata = f'silk = {json.dumps(files)}\noutputs = ["{output}"]\n'
    if binary:
        metadata += f'binary_outputs = ["{output}"]\n[events]\nconsumes = []\nproduces = []\n'
    (project / "native-spool.toml").write_text('schema = "an-native-spool-v1"\n'
        'runtime = "macos-native-process-v1"\nentry = "silk/main.sh"\nkit = []\neditable = []\n'
        'inputs = ["input.txt"]\ncapabilities = ["read-input", "write-declared-outputs"]\n'
        'dependencies = []\nacceptance = ["bash -n silk/main.sh"]\n' + metadata)
    archive = call(loom, "pack", project)["archive"]
    delivery = root / (name + "-delivery")
    exported = call(loom, "export-native-spool-v3" if binary else "export-native-spool-v2", archive, delivery)
    return delivery, exported["release"]


def produce(host, instance, run, data, output):
    approve(host, "spool-supervisor", "start", instance, run, data, 15)
    approve(host, "spool-accept", instance, run)
    return call(host, "spool-artifacts", instance, run, "--output", output)


def qualify(loom, host, root):
    # Build a small fixture matching the current CAD scene contract. The native
    # host runs and accepts it as binary output before the exact-read adapter sees it.
    fixture = json.loads((EXAMPLE / "public-grip.json").read_text())
    source = {"application": "web-cad", "document": fixture["document"], "epoch": "fixture-epoch",
              "revision": fixture["revision"]}
    material = fixture["material"]
    ora = root / "fixture.ora"
    with ZipFile(ora, "w") as archive:
        archive.writestr("data/scene.json", json.dumps({"source": source, "drawing": "handwheel",
            "request_sha256": "a" * 64, "materials": {"grip": material}, "solids": ["private geometry fixture"]}))
    input_file = root / "encoded.txt"
    input_file.write_bytes(b64encode(ora.read_bytes()))
    painter, paint_release = release(loom, root, "fixture-paint", "painting.ora",
                         '/usr/bin/base64 -D < "$AN_SPOOL_INPUT" > painting.ora\n', binary=True)
    origin = root / "paint-instance"
    call(host, "spool-load", painter, origin)
    report = produce(host, origin, "paint", input_file, "painting.ora")
    revision = report["output"]["binding"]["revision"]
    evidence = reader.read_accepted(host, origin, "paint", paint_release, revision, "handwheel", source["epoch"], 7, "grip")
    assert evidence["material"] == material and evidence["source"] == source
    assert "solids" not in evidence and evidence["artifact"]["revision"] == revision

    worker, worker_release = release(loom, root, "cad-grip-reader", "proof.json",
                                     (EXAMPLE / "silk/verify-grip.sh").read_text())
    target = root / "reader-instance"
    audience = call(host, "spool-load", worker, target)["instance"]
    publication, _ = release(loom, root, "cad-evidence-publisher", "material.txt",
        '/bin/cat "$AN_SPOOL_INPUT" > material.txt\n', declaration={
            "schema": "an-native-peer-read-v1", "task": "verify-grip-finish",
            "worker_release": worker_release, "material_output": "material.txt",
            "reply_output": "proof.json", "max_text_bytes": 2048, "max_wall_seconds": 15})
    sender = root / "publication-instance"
    call(host, "spool-load", publication, sender)
    text = json.dumps(evidence, ensure_ascii=False, separators=(",", ":"))
    packet = root / "packet.json"
    packet.write_text(json.dumps({"schema": "an-context-v1", "audience": audience,
        "source": f"cad:ora:{revision}", "text": text, "sha256": sha256(text.encode()).hexdigest()}) + "\n")
    accepted = produce(host, sender, "offer", packet, "material.txt")
    spec = root / "request.json"
    spec.write_text(json.dumps({"schema": "an-native-peer-context-request-v1", "source_run": "offer",
        "source_output": "material.txt", "target": str(target), "reply_output": "proof.json", "wall_seconds": 15,
        "read": {"task": "verify-grip-finish", "material_revision": accepted["output"]["binding"]["revision"]}}))
    args = ["spool-peer", "request", sender, "grip", spec]
    started = time.monotonic_ns()
    pending = call(host, *args)
    assert pending["decision"] == "needs_authorization" and not (sender / "peers/grip").exists()
    assert call(host, *args, "--approve", pending["approval_sha256"])["decision"] == "admitted"
    result = approve(host, "spool-peer", "run", sender, "grip", "request")
    assert result["workspace"]["termination_confirmed"] and result["status"] == "succeeded"
    approve(host, "spool-accept", target, result["run"])
    approve(host, "spool-peer", "reply", sender, "grip")
    receipt = call(host, "spool-artifacts", target, result["run"], "--output", "proof.json")
    checked = root / "verified.json"
    call(host, "artifact-checkout", receipt["catalog"], receipt["output"]["binding"]["revision"], checked)
    assert json.loads(checked.read_text()) == evidence
    measurement = {"document": "handwheel", "part": "grip", "drawing_revision": 7,
                   "elapsed_ms": round((time.monotonic_ns() - started) / 1_000_000), "model_calls": 0}
    (root / "local-measurement.json").write_text(json.dumps(measurement) + "\n")
    print(json.dumps({**measurement,
                      "ora_revision": revision, "worker_release": worker_release,
                      "proof_revision": receipt["output"]["binding"]["revision"]}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ("loom", "host", "output"):
        parser.add_argument(f"--{option}", type=Path, required=True)
    args = parser.parse_args()
    require_native_workspace()
    output = args.output.resolve()
    output.mkdir(mode=0o700)
    qualify(args.loom.resolve(strict=True), args.host.resolve(strict=True), output)
