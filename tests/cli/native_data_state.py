#!/usr/bin/env python3
"""Package an ordinary log spool and qualify independently bound data/state."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import signal
import subprocess

EXAMPLE = Path(__file__).resolve().parents[2] / "examples/log-accumulator"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loom", type=Path, required=True)
    parser.add_argument("--host", type=Path, required=True)
    parser.add_argument("--host-revision", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != "Darwin":
        raise SystemExit("Real macOS native enforcement required; no fallback")
    root, loom, host = args.output.resolve(), args.loom.resolve(strict=True), args.host.resolve(strict=True)
    root.mkdir(mode=0o700)
    calls = []

    def call(binary, *argv, success=True):
        process = subprocess.Popen([str(binary), *map(str, argv)], text=True,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            out, err = process.communicate(timeout=45)
        except subprocess.TimeoutExpired as error:
            process.send_signal(signal.SIGINT)
            try:
                process.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                raise RuntimeError(f"Termination unknown for {process.pid}; retain {root}") from error
            raise RuntimeError("Command exceeded deadline; no retry") from error
        calls.append({"binary": str(binary), "args": list(map(str, argv)), "code": process.returncode,
                      "stdout": out, "stderr": err})
        (root / "commands.local.json").write_text(json.dumps(calls, indent=2))
        assert (process.returncode == 0) == success, (argv, out, err)
        return json.loads(out) if out else None

    def approved(*argv):
        preview = call(host, *argv)
        return call(host, *argv, "--approve", preview["approval_sha256"])

    def package(project, output):
        call(loom, "check", project)
        archive = call(loom, "pack", project)["archive"]
        call(loom, "verify", archive)
        return call(loom, "export-native-spool-v3", archive, output)["release"]

    producer = root / "producer-project"
    call(loom, "init-native", producer)
    (producer / "silk/main.sh").write_text('set -eu\ncat "$AN_SPOOL_INPUT" > report.txt\n')
    with (producer / "native-spool.toml").open("a") as stream:
        stream.write('\n[events]\nconsumes=[]\nproduces=["artifact.accepted"]\n')
    source_release, release = root / "source-release", root / "release"
    package(producer, source_release)
    exported = package(EXAMPLE, release)
    manifest = json.loads((release / "native-spool.json").read_text())
    assert manifest["requirements"]["state_input"] == "checkpoint.txt"
    a, b, ws = root / "a", root / "b", root / "workspace"
    call(host, "spool-load", source_release, a)
    call(host, "spool-load", release, b)
    call(host, "spool-events-init", ws)
    initial = approved("spool-state-init", b, "state.txt", EXAMPLE / "initial-state.txt")["state"]["revision"]
    batch = root / "batch.txt"
    versions = []
    for name, data in [("first", (EXAMPLE / "sample.txt").read_text()), ("newer", "INFO newer\n")]:
        batch.write_text(data)
        approved("spool-start", a, name, batch, 15)
        approved("spool-accept", a, name)
        versions.append(call(host, "spool-artifacts", a, name))
    data = versions[0]["outputs"][0]["binding"]["revision"]
    event = approved("spool-event-publish", ws, a, "first", "report.txt")["event"]
    request = root / "selection.local.json"
    for name, version in [("first", versions[0]), ("stale", versions[1])]:
        request.write_text(json.dumps({"schema": "an-native-data-state-v1", "state_revision": initial,
            "data": {"catalog": version["catalog"], "revision": version["outputs"][0]["binding"]["revision"]}}))
        started = approved("spool-start-data-state", b, name, request, 15)
        assert started["workspace"]["termination_confirmed"] is True
        approved("spool-accept", b, name)
        approved("spool-reclaim", b, name)
        approved("spool-reclaim-snapshots", b, name)
    current = approved("spool-state-commit", b, initial, "first")["revision"]
    conflict = call(host, "spool-state-commit", b, initial, "stale", success=False)
    assert conflict["status"] == "conflict" and conflict["automatic_retries"] == 0
    route = root / "route.local.json"
    route.write_text(json.dumps({"schema": "an-native-artifact-route-v1", "source": str(a),
        "output": "report.txt", "target": str(b), "result": "report.txt", "wall_seconds": 15}))
    approved("spool-event-subscribe", ws, "logs", route)
    argv = ["spool-event-deliver", ws, event["id"], "logs", "--state", current]
    delivered = approved(*argv)
    assert delivered["status"] == "succeeded" and delivered["consumed"] is False
    run = delivered["context"]["delivery"]
    approved("spool-accept", b, run)
    head = approved("spool-state-commit", b, current, run)["revision"]
    approved("spool-event-complete", ws, event["id"], "logs")
    approved("spool-reclaim", b, run)
    approved("spool-reclaim-snapshots", b, run)
    assert call(host, *argv)["disposition"] == "already-consumed"
    artifacts = call(host, "spool-artifacts", b, run)
    report = next(p for p in artifacts["outputs"] if p["path"] == "report.txt")
    saved = root / "report.txt"
    call(host, "artifact-checkout", artifacts["catalog"], report["binding"]["revision"], saved)
    assert saved.read_text() == "total=6\nINFO=4\nWARN=0\nERROR=2\n"
    version = json.loads((Path(artifacts["catalog"]) / "revisions" / report["binding"]["revision"] / "revision.json").read_text())
    assert [p["revision"] for p in version["source"]["input_revisions"]] == [data, current]
    approved("spool-unload", b)
    state = call(host, "spool-state-show", b)
    assert state["head"] == head
    summary = {"schema": "loom-data-state-qualification-v1", "passed": True,
        "host_revision": args.host_revision, "host_sha256": hashlib.sha256(host.read_bytes()).hexdigest(),
        "loom_sha256": hashlib.sha256(loom.read_bytes()).hexdigest(), "release": exported,
        "data_revision": data, "initial_state": initial, "committed_state": head,
        "stale_commit_refused": True, "outputs_survive_collection": True,
        "model_invoked": False, "automatic_retries": 0}
    (root / "acceptance.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
