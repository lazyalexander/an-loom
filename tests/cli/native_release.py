#!/usr/bin/env python3
"""Package two real revisions and compare/continue them through the native host."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import signal
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loom", type=Path, required=True)
    parser.add_argument("--host", type=Path, required=True)
    parser.add_argument("--host-revision", required=True)
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args()
    if platform.system() != "Darwin":
        raise SystemExit("Real macOS native enforcement required; no fallback")
    root = options.output.resolve()
    root.mkdir(mode=0o700)
    loom, host = options.loom.resolve(strict=True), options.host.resolve(strict=True)
    calls = []

    def call(binary, *args):
        process = subprocess.Popen([str(binary), *map(str, args)], text=True,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            stdout, stderr = process.communicate(timeout=30)
        except subprocess.TimeoutExpired as error:
            process.send_signal(signal.SIGINT)
            try:
                stdout, stderr = process.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                raise RuntimeError(f"Termination unknown for {process.pid}; retain {root}") from error
            raise RuntimeError(f"Command deadline exceeded: {args}; no retry") from error
        calls.append({"binary": str(binary), "args": list(map(str, args)),
                      "code": process.returncode, "stdout": stdout, "stderr": stderr})
        (root / "commands.local.json").write_text(json.dumps(calls, indent=2))
        if process.returncode:
            raise RuntimeError(stderr or stdout)
        return json.loads(stdout)

    def approved(*args):
        preview = call(host, *args)
        return call(host, *args, "--approve", preview["approval_sha256"])

    previous = root / "previous-project"
    candidate = root / "candidate-project"
    call(loom, "init-native", previous)
    shutil.copytree(previous, candidate)
    # This changed glue behavior belongs to the project, not generic Loom code.
    (candidate / "kit/transform.sh").write_text('set -eu\nprintf "v2:"\ncat "$1"\n')
    archives = []
    for project in [previous, candidate]:
        call(loom, "check", project)
        archive = call(loom, "pack", project)["archive"]
        call(loom, "verify", archive)
        archives.append(archive)
    qualified = []
    for version, exporter in [("v1", "export-native-spool"), ("v2", "export-native-spool-v2")]:
        deliveries, exports = [], []
        for name, archive in zip(["previous", "candidate"], archives):
            delivery = root / f"{version}-{name}-delivery"
            exports.append(call(loom, exporter, archive, delivery))
            deliveries.append(delivery)
        report = call(host, "spool-release-diff", *deliveries)
        assert report["reused_coordinates"] is True and report["same_release"] is False
        assert report["previous"]["release"] == exports[0]["release"]
        assert report["candidate"]["release"] == exports[1]["release"]
        assert report["changes"]["code"][0]["path"] == "kit/transform.sh"
        assert len(report["changes"]["code"]) == 1
        assert report["changes"]["requirements"] == []
        assert report["execution_granted"] is False and report["successor_authorized"] is False
        assert report["candidate"]["runtime_supported"] is True
        assert report["candidate"]["legacy_implicit_kit_editability"] == (version == "v1")
        old, new = root / f"{version}-old", root / f"{version}-new"
        old_instance = call(host, "spool-load", deliveries[0], old)
        seed = root / f"{version}-seed.txt"
        seed.write_text("hello\n")
        first = approved("spool-state-init", old, "report.txt", seed)["state"]["revision"]
        approved("spool-start-state", old, "only", first, "15")
        approved("spool-accept", old, "only")
        old_head = approved("spool-state-commit", old, first, "only")["revision"]
        state = call(host, "spool-state-show", old)["selected"]
        fixed = root / f"{version}-fixed.txt"
        call(host, "artifact-checkout", state["catalog"], old_head, fixed)
        assert fixed.read_text() == "Report\nHELLO\n"
        new_instance = call(host, "spool-load", deliveries[1], new)
        assert new_instance["instance"] != old_instance["instance"]
        assert not (new / "checkpoints").exists()
        selected = approved("spool-state-init", new, "report.txt", fixed)["state"]["revision"]
        approved("spool-start-state", new, "only", selected, "15")
        approved("spool-accept", new, "only")
        new_head = approved("spool-state-commit", new, selected, "only")["revision"]
        final = call(host, "spool-state-show", new)
        assert Path(final["selected"]["path"]).read_text() == "Report\nv2:Report\nHELLO\n"
        assert call(host, "spool-state-show", old)["head"] == old_head
        assert call(host, "spool-show", old)["release"] == exports[0]["release"]
        assert fixed.read_text() == "Report\nHELLO\n"
        qualified.append({"format": version, "previous_release": exports[0]["release"],
                          "candidate_release": exports[1]["release"],
                          "previous_checkpoint": old_head, "candidate_checkpoint": new_head})
    summary = {"schema": "loom-native-release-qualification-v1", "passed": True,
               "host_revision": options.host_revision, "platform": platform.platform(),
               "host_sha256": hashlib.sha256(host.read_bytes()).hexdigest(),
               "loom_sha256": hashlib.sha256(loom.read_bytes()).hexdigest(),
               "qualified": qualified, "model_invoked": False, "execution_retried": False,
               "in_place_upgrade": False, "successor_authorized": False}
    (root / "acceptance.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
