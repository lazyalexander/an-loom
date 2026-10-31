#!/usr/bin/env python3
"""Qualify actual v3 packages through one host-owned workspace event exchange."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
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
        calls.append({"binary": str(binary), "args": list(map(str, args)), "code": process.returncode,
                      "stdout": stdout, "stderr": stderr})
        (root / "commands.local.json").write_text(json.dumps(calls, indent=2))
        if process.returncode:
            raise RuntimeError(stderr or stdout)
        return json.loads(stdout)

    def approved(*args):
        preview = call(host, *args)
        return call(host, *args, "--approve", preview["approval_sha256"])

    project = root / "project"
    call(loom, "init-native", project)
    config = project / "native-spool.toml"
    with config.open("a") as stream:
        stream.write('\n[events]\nconsumes = ["artifact.accepted"]\nproduces = ["artifact.accepted"]\n')
    call(loom, "check", project)
    archive = call(loom, "pack", project)["archive"]
    call(loom, "verify", archive)
    delivery = root / "delivery"
    exported = call(loom, "export-native-spool-v3", archive, delivery)
    a, b, workspace = root / "a", root / "b", root / "workspace"
    for instance in [a, b]:
        loaded = call(host, "spool-load", delivery, instance)
        assert loaded["release"] == exported["release"]
        assert loaded["execution_granted"] is False
        assert loaded["event_interface"]["consumes"] == ["artifact.accepted"]
    call(host, "spool-events-init", workspace)
    route = root / "route.local.json"
    route.write_text(json.dumps({"schema": "an-native-artifact-route-v1", "source": str(a),
                                 "output": "report.txt", "target": str(b), "result": "report.txt",
                                 "wall_seconds": 15}))
    registered = approved("spool-event-subscribe", workspace, "reports", route)
    assert registered["execution_granted"] is False
    seed = root / "input.txt"
    seed.write_text("hello\n")
    approved("spool-start", a, "produce", seed, "15")
    approved("spool-accept", a, "produce")
    event = approved("spool-event-publish", workspace, a, "produce", "report.txt")["event"]
    inventory = call(host, "spool-events-pending", workspace)
    assert inventory["execution_granted"] is False
    assert inventory["deliveries"][0]["phase"] == "awaiting-authorization"
    assert list((b / "runs").iterdir()) == []
    delivered = approved("spool-event-deliver", workspace, event["id"], "reports")
    assert delivered["result"]["workspace"]["termination_confirmed"] is True
    assert delivered["consumed"] is False
    run = delivered["context"]["delivery"]
    assert call(host, "spool-events-pending", workspace)["deliveries"][0]["phase"] == "awaiting-acceptance"
    approved("spool-accept", b, run)
    assert call(host, "spool-events-pending", workspace)["deliveries"][0]["phase"] == "awaiting-confirmation"
    complete = approved("spool-event-complete", workspace, event["id"], "reports")
    assert call(host, "spool-events-pending", workspace)["deliveries"][0]["phase"] == "consumed"
    receipt = complete["receipt"]
    assert receipt["context"]["event"] == event["id"]
    assert receipt["result_event"]["event"]["cause"] == receipt["context"]
    outputs = call(host, "spool-artifacts", b, run)
    report = outputs["outputs"][0]["binding"]["revision"]
    saved = root / "report.txt"
    call(host, "artifact-checkout", outputs["catalog"], report, saved)
    assert saved.read_text() == "Report\nREPORT\nHELLO\n"
    for instance, name in [(a, "produce"), (b, run)]:
        approved("spool-reclaim", instance, name)
        approved("spool-reclaim-snapshots", instance, name)
    duplicate = call(host, "spool-event-deliver", workspace, event["id"], "reports")
    assert duplicate["executed"] is False and duplicate["disposition"] == "already-consumed"
    assert len(list((b / "runs" / run / "job/attempts").iterdir())) == 1
    summary = {"schema": "loom-native-event-qualification-v1", "passed": True,
               "host_revision": options.host_revision, "host_sha256": hashlib.sha256(host.read_bytes()).hexdigest(),
               "loom_sha256": hashlib.sha256(loom.read_bytes()).hexdigest(), "platform": platform.platform(),
               "release": exported["release"], "event": event["id"], "result_revision": report,
               "model_invoked": False, "execution_retried": False, "background_dispatch": False,
               "host_delivery_inventory_qualified": True}
    (root / "acceptance.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
