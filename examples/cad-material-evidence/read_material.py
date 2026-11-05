#!/usr/bin/env python3
"""Select one material from an exact, host-accepted CAD painting revision."""
import argparse
from hashlib import sha256
import json
from pathlib import Path
import re
import subprocess
from tempfile import TemporaryDirectory
from zipfile import ZipFile

MAX_SCENE = 1024 * 1024
MAX_ORA = 32 * 1024 * 1024


def identifier(value):
    return isinstance(value, str) and re.fullmatch(r"[A-Za-z][A-Za-z0-9_-]{0,63}", value)


def digest(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value)


def select_material(scene, document, epoch, revision, part):
    if not isinstance(scene, dict):
        raise ValueError("Invalid CAD scene metadata")
    source = scene.get("source", {})
    expected = {"application": "web-cad", "document": document, "epoch": epoch, "revision": revision}
    if (source != expected or type(source.get("revision")) is not int
            or scene.get("drawing") != document or not digest(scene.get("request_sha256"))):
        raise ValueError("CAD source revision does not match the selected drawing")
    materials = scene.get("materials")
    material = materials.get(part) if isinstance(materials, dict) else None
    if not isinstance(material, dict):
        raise ValueError("The accepted drawing contains no material evidence for this part")
    finish, color = material.get("finish"), material.get("color")
    if (not isinstance(finish, str) or not finish.strip() or len(finish) > 80
            or any(ord(character) < 32 for character in finish)
            or not isinstance(color, str) or not re.fullmatch(r"#[0-9A-Fa-f]{6}", color)
            or not all(type(material.get(k)) in (int, float) and 0 <= material[k] <= 1
                       for k in ("metallic", "roughness"))):
        raise ValueError("The accepted drawing contains an invalid material")
    return {"schema": "cad-material-evidence-v1", "source": expected, "part": part,
            "material": {k: material[k] for k in ("finish", "color", "metallic", "roughness")},
            "request_sha256": scene["request_sha256"]}


def read_scene(archive):
    with ZipFile(archive) as painting:
        matches = [entry for entry in painting.infolist() if entry.filename == "data/scene.json"]
        if len(matches) != 1 or matches[0].file_size > MAX_SCENE:
            raise ValueError("Expected one bounded scene metadata entry")
        with painting.open(matches[0]) as stream:
            raw = stream.read(MAX_SCENE + 1)
    if len(raw) > MAX_SCENE:
        raise ValueError("Scene metadata exceeds the read limit")
    return json.loads(raw)


def host_call(host, *args):
    return json.loads(subprocess.check_output([str(host), *map(str, args)], timeout=30))


def read_accepted(host, instance, run, paint_release, artifact_revision, document, epoch, revision, part):
    if not identifier(document) or not identifier(part) or not digest(paint_release) or not digest(artifact_revision) or revision < 1:
        raise ValueError("Expected a drawing, part, positive revision and exact artifact digest")
    report = host_call(host, "spool-artifacts", instance, run, "--output", "painting.ora")
    output = report["output"]
    binding = output["binding"]
    if (report.get("schema") != "an-native-selected-artifact-v1" or report.get("payload_verified") is not True
            or report["run"] != run or report["release"] != paint_release or output["path"] != "painting.ora"
            or binding["revision"] != artifact_revision or binding["encoding"] != "bytes"
            or not digest(binding["content_sha256"]) or not 0 < binding["bytes"] <= MAX_ORA):
        raise ValueError("The selected accepted ORA does not match the exact reference")
    with TemporaryDirectory(prefix="cad-material-") as directory:
        archive = Path(directory).resolve() / "painting.ora"
        host_call(host, "artifact-checkout", report["catalog"], artifact_revision, archive)
        checksum = sha256()
        with archive.open("rb") as stream:
            for block in iter(lambda: stream.read(65536), b""):
                checksum.update(block)
        if archive.stat().st_size != binding["bytes"] or checksum.hexdigest() != binding["content_sha256"]:
            raise ValueError("Checked-out ORA differs from its accepted artifact")
        evidence = select_material(read_scene(archive), document, epoch, revision, part)
    evidence["artifact"] = {"instance": report["instance"], "release": report["release"],
                            "run": run, "output": "painting.ora", "revision": artifact_revision,
                            "content_sha256": binding["content_sha256"]}
    return evidence


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", type=Path, required=True)
    parser.add_argument("--instance", type=Path, required=True)
    parser.add_argument("--run", default="paint")
    parser.add_argument("--paint-release", required=True)
    parser.add_argument("--artifact-revision", required=True)
    parser.add_argument("--document", required=True)
    parser.add_argument("--epoch", required=True)
    parser.add_argument("--revision", type=int, required=True)
    parser.add_argument("--part", required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(read_accepted(args.host.resolve(strict=True), args.instance.resolve(strict=True),
                                      args.run, args.paint_release, args.artifact_revision, args.document, args.epoch,
                                      args.revision, args.part), ensure_ascii=False, separators=(",", ":")))
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        parser.exit(1, f"Material evidence unavailable: {error}\n")


if __name__ == "__main__":
    main()
