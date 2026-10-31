#!/usr/bin/env python3
"""Package an ordinary log spool and qualify independently bound data/state."""
import argparse
import hashlib
import json
from pathlib import Path
from host_qualification import require_native_workspace
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
    require_native_workspace()
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
    references = call(host, "spool-input-references", b, "first")
    assert references["collection_authorized"] is False
    assert references["plan"]["owner"]["release"] == exported
    assert [c["reference"]["binding"]["revision"] for c in references["claims"]] == [data, initial]
    assert all(c["release"] is None for c in references["claims"])
    for material in ["work", "snapshots"]:
        selection = root / ("collect-" + material + ".local.json")
        selection.write_text(json.dumps({"schema": "an-native-private-collection-request-v1",
            "material": material, "runs": ["first", "stale"]}))
        preview = call(host, "spool-storage-collect", b, selection)
        assert preview["executed"] is False and preview["plan"]["runs"] == ["first", "stale"]
        collected = call(host, "spool-storage-collect", b, selection, "--approve", preview["approval_sha256"])
        assert len(collected["completed"]) == 2 and collected["automatic_retries"] == 0
        call(host, "spool-storage-collect", b, selection, success=False)
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
    # A Loom-packaged stateful release also supports finite host dispatch.
    next_event = approved("spool-event-publish", ws, a, "newer", "report.txt")["event"]
    dispatch = root / "dispatch.local.json"
    dispatch.write_text(json.dumps({"schema": "an-native-event-dispatch-request-v1", "name": "next-log",
        "entries": [{"event": next_event["id"], "subscription": "logs", "state_revision": head}]}))
    preview = call(host, "spool-events-dispatch", ws, dispatch)
    next_run = preview["plan"]["entries"][0]["plan"]["context"]["delivery"]
    assert not (b / "runs" / next_run / "job").exists()
    assert preview["plan"]["entries"][0]["plan"]["start"]["state_input"]["binding"]["revision"] == head
    approved("spool-event-pause", ws, "logs")
    call(host, "spool-events-dispatch", ws, dispatch, "--approve", preview["approval_sha256"], success=False)
    approved("spool-event-resume", ws, "logs")
    call(host, "spool-events-dispatch", ws, dispatch, "--approve", preview["approval_sha256"], success=False)
    assert not (b / "runs" / next_run / "job").exists()
    fresh = call(host, "spool-events-dispatch", ws, dispatch)
    assert fresh["plan"]["entries"][0]["plan"]["context"]["delivery"] == next_run
    assert fresh["approval_sha256"] != preview["approval_sha256"]
    result = call(host, "spool-events-dispatch", ws, dispatch, "--approve", fresh["approval_sha256"])
    assert result["reason"] == "completed" and result["consumed"] is False
    approved("spool-accept", b, next_run)
    next_products = call(host, "spool-artifacts", b, next_run)
    next_report = next(p for p in next_products["outputs"] if p["path"] == "report.txt")
    next_saved = root / "next-report.txt"
    call(host, "artifact-checkout", next_products["catalog"], next_report["binding"]["revision"], next_saved)
    assert next_saved.read_text() == "total=7\nINFO=5\nWARN=0\nERROR=2\n"
    approved("spool-event-pause", ws, "logs")  # Pause must not block already-admitted custody.
    head = approved("spool-state-commit", b, head, next_run)["revision"]
    approved("spool-event-complete", ws, next_event["id"], "logs")
    approved("spool-reclaim", b, next_run)
    approved("spool-reclaim-snapshots", b, next_run)
    assert call(host, "spool-events-dispatch", ws, dispatch)["executed"] is False
    assert len(list((b / "runs" / next_run / "job/attempts").iterdir())) == 1
    assert saved.read_text() == "total=6\nINFO=4\nWARN=0\nERROR=2\n"
    summary_path = ws / "dispatches/next-log/result.json"
    summary_path.unlink()  # Known completed execution; lose only its batch projection.
    call(host, "spool-events-dispatch", ws, dispatch, success=False)
    restored = approved("spool-events-dispatch-reconcile", ws, "next-log")
    assert restored["executed"] is False and restored["summary_reconstructed"] is True
    assert len(list((b / "runs" / next_run / "job/attempts").iterdir())) == 1
    # The host can cut over to another instance of the same Loom-built release.
    c = root / "successor-instance"
    assert call(host, "spool-load", release, c)["release"] == exported
    committed = call(host, "spool-state-show", b, head)["selected"]
    transferred = root / "checkpoint-for-successor.txt"
    call(host, "artifact-checkout", committed["catalog"], head, transferred)
    successor_state = approved("spool-state-init", c, "state.txt", transferred)["state"]["revision"]
    route.write_text(json.dumps({"schema": "an-native-artifact-route-v1", "source": str(a),
        "output": "report.txt", "target": str(c), "result": "report.txt", "wall_seconds": 15}))
    replaced = approved("spool-event-replace", ws, "logs", "logs-next", route)
    assert replaced["execution_granted"] is False and replaced["historical_events_replayed"] is False
    call(host, "spool-event-deliver", ws, event["id"], "logs-next", "--state", successor_state, success=False)
    assert list((c / "runs").iterdir()) == []
    batch.write_text("WARN successor\n")
    approved("spool-start", a, "third", batch, 15)
    approved("spool-accept", a, "third")
    third_event = approved("spool-event-publish", ws, a, "third", "report.txt")["event"]
    next_dispatch = root / "successor-dispatch.local.json"
    next_dispatch.write_text(json.dumps({"schema": "an-native-event-dispatch-request-v1", "name": "successor-log",
        "entries": [{"event": third_event["id"], "subscription": "logs-next", "state_revision": successor_state}]}))
    candidate = call(host, "spool-events-dispatch", ws, next_dispatch)
    successor_run = candidate["plan"]["entries"][0]["plan"]["context"]["delivery"]
    call(host, "spool-events-dispatch", ws, next_dispatch, "--approve", candidate["approval_sha256"])
    approved("spool-accept", c, successor_run)
    successor_head = approved("spool-state-commit", c, successor_state, successor_run)["revision"]
    approved("spool-event-complete", ws, third_event["id"], "logs-next")
    products = call(host, "spool-artifacts", c, successor_run)
    product = next(p for p in products["outputs"] if p["path"] == "report.txt")
    output = root / "successor-report.txt"
    call(host, "artifact-checkout", products["catalog"], product["binding"]["revision"], output)
    assert output.read_text() == "total=8\nINFO=5\nWARN=1\nERROR=2\n"
    approved("spool-reclaim", c, successor_run)
    approved("spool-reclaim-snapshots", c, successor_run)
    assert call(host, "spool-events-dispatch", ws, next_dispatch)["executed"] is False
    assert call(host, "spool-state-show", b)["head"] == head
    assert call(host, "spool-state-show", c)["head"] == successor_head
    # Runtime retirement is a host decision; packaged bytes and existing custody stay fixed.
    closed = approved("spool-artifact-retire", a, "first", "report.txt")
    assert closed["payload_deleted"] is False
    retired = call(host, "spool-artifacts", a, "first")
    assert retired["admission"]["report.txt"]["mode"] == "retired"
    assert retired["outputs"] == versions[0]["outputs"]
    call(host, "artifact-checkout", versions[0]["catalog"], data, root / "retired-copy.txt", success=False)
    retirement_selection = root / "retired-input.local.json"
    retirement_selection.write_text(json.dumps({"schema": "an-native-data-state-v1",
        "data": {"catalog": versions[0]["catalog"], "revision": data}, "state_revision": successor_head}))
    call(host, "spool-start-data-state", c, "retired-input", retirement_selection, 15, success=False)
    assert not (c / "runs/retired-input").exists()
    assert call(host, *argv)["disposition"] == "already-consumed"
    assert output.read_text() == "total=8\nINFO=5\nWARN=1\nERROR=2\n"
    approved("spool-unload", c)
    approved("spool-unload", b)
    state = call(host, "spool-state-show", b)
    assert state["head"] == head
    # Successful execution still needs historical input custody after collection,
    # state commit and unload. Packaging cannot release runtime references.
    assert call(host, "spool-input-references", b, "first") == references
    summary = {"schema": "loom-data-state-qualification-v1", "passed": True,
        "host_revision": args.host_revision, "host_sha256": hashlib.sha256(host.read_bytes()).hexdigest(),
        "loom_sha256": hashlib.sha256(loom.read_bytes()).hexdigest(), "release": exported,
        "data_revision": data, "initial_state": initial, "committed_state": head,
        "stale_commit_refused": True, "outputs_survive_collection": True, "finite_dispatch_qualified": True, "subscription_control_qualified": True, "receiver_cutover_qualified": True, "private_collection_qualified": True,
        "artifact_retirement_qualified": True, "input_references_qualified": True,
        "model_invoked": False, "automatic_retries": 0}
    (root / "acceptance.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
