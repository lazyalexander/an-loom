#!/usr/bin/env python3
"""Package three headless releases and qualify registered event fan-out on the paired host."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time

from host_qualification import require_native_workspace

ACCUMULATOR = Path(__file__).resolve().parents[2] / "examples/log-accumulator"


def write(path, value):
    path.write_text(json.dumps(value))
    return path


def footprint(paths):
    files = [p.stat() for root in paths for p in root.rglob("*") if p.is_file()]
    return {"files": len(files), "logical_bytes": sum(f.st_size for f in files),
            "allocated_bytes": sum(f.st_blocks * 512 for f in files)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ("loom", "host", "output"):
        parser.add_argument("--" + key, type=Path, required=True)
    parser.add_argument("--host-revision", required=True)
    args = parser.parse_args()
    require_native_workspace()
    loom, host, root = args.loom.resolve(strict=True), args.host.resolve(strict=True), args.output.resolve()
    root.mkdir(mode=0o700)
    journal = root / "commands.local.jsonl"

    def finish(proc, ok=True, error=None):
        try:
            out, err = proc.communicate(timeout=40)
        except subprocess.TimeoutExpired as exc:
            proc.send_signal(signal.SIGINT)
            try:
                proc.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                raise RuntimeError("Termination unconfirmed; retain the run") from exc
            raise RuntimeError("Client deadline exceeded; inspect without replay") from exc
        with journal.open("a") as log:
            log.write(json.dumps({"argv": proc.args, "code": proc.returncode, "out": out, "err": err}) + "\n")
        assert (proc.returncode == 0) == ok and (error is None or error in err), (proc.args, out, err)
        return json.loads(out) if out else None

    def spawn(*cmd):
        return subprocess.Popen(list(map(str, cmd)), text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def call(*cmd, **opts):
        return finish(spawn(*cmd), **opts)

    def approve(*argv):
        return call(host, *argv, "--approve", call(host, *argv)["approval_sha256"])

    def export(project, name):
        call(loom, "check", project)
        archive = call(loom, "pack", project)["archive"]
        call(loom, "verify", archive)
        release = root / (name + "-release")
        digest = call(loom, "export-native-spool-v3", archive, release)["release"]
        target = root / name
        assert call(host, "spool-load", release, target)["release"] == digest
        return target, digest

    def scaffold(name, script, consumes, extra_output=False):
        project = root / (name + "-project")
        call(loom, "init-native", project)
        (project / "silk/main.sh").write_text("set -eu\n" + script)
        descriptor = project / "native-spool.toml"
        body = descriptor.read_text()
        if extra_output:
            body = body.replace('outputs = ["report.txt"]', 'outputs = ["report.txt", "brief.log"]')
        descriptor.write_text(body + '\n[events]\nconsumes=' + json.dumps(consumes) +
                              '\nproduces=["artifact.accepted"]\n')
        return export(project, name)

    producer, src = scaffold("source", 'cat "$AN_SPOOL_INPUT" > report.txt\n'
                             'head -n 1 "$AN_SPOOL_INPUT" > brief.log\n', [], True)
    audit, aud = scaffold("audit", 'read -r level mode < "$AN_SPOOL_INPUT"\n'
                          'if [ "$mode" = hold ] || [ "$mode" = interrupt ]; then\n'
                          '  printf ready > ready.marker\n  if [ "$mode" = hold ]; then sleep 60; else n=0; while [ ! -f exit.marker ] && [ "$n" -lt 200 ]; do sleep .05; n=$((n+1)); done; fi\nfi\n'
                          'cat "$AN_SPOOL_INPUT" > report.txt\n', ["artifact.accepted"])
    project = root / "accumulator-project"
    shutil.copytree(ACCUMULATOR, project, ignore=shutil.ignore_patterns("dist", "__pycache__"))
    descriptor = project / "native-spool.toml"
    descriptor.write_text(descriptor.read_text().replace('outputs = ["state.txt", "report.txt"]',
                          'outputs = ["state.txt", "report.txt", "checkpoint-input-policy.json"]'))
    policy = {"schema": "an-native-checkpoint-managed-input-policy-v1", "state_output": "state.txt",
              "data": "checkpoint-pin", "parent_state": "not-required"}
    with (project / "silk/main.sh").open("a") as silk:
        silk.write("echo '" + json.dumps(policy) + "' > checkpoint-input-policy.json\n")
    counter, acc = export(project, "counter")
    ws = root / "events"
    call(host, "spool-events-init", ws)
    parent = approve("spool-state-init", counter, "state.txt", project / "initial-state.txt", "--registered")["state"]["revision"]
    for name, target, output in (("count", counter, "brief.log"), ("observe", audit, "brief.log"),
                                 ("raw", audit, "report.txt")):
        route = write(root / (name + ".json"), {"schema": "an-native-artifact-route-v1",
            "source": str(producer), "output": output, "target": str(target), "result": "report.txt", "wall_seconds": 60 if name == "raw" else 15})
        approve("spool-event-subscribe", ws, name, route)

    def produce(name, text):
        path = root / "batch.log"
        path.write_text(text)
        approve("spool-start", producer, name, path, 15)
        approve("spool-accept", producer, name, "--registered")
        approve("spool-reclaim", producer, name)
        approve("spool-reclaim-snapshots", producer, name)
        return {o: approve("spool-event-publish", ws, producer, name, o)["event"] for o in ("brief.log", "report.txt")}

    def managed(action, event):
        data = event["event"]
        return ["spool-managed", action, producer, data["run"], data["output"], data["artifact"]["binding"]["revision"]]

    def reclaim(instance, run):
        approve("spool-reclaim", instance, run)
        approve("spool-reclaim-snapshots", instance, run)

    def count(name, event, parent):
        argv = ["spool-event-deliver", ws, event["id"], "count", "--state", parent]
        if name == "second":
            selection = write(root / "dispatch.json", {"schema": "an-native-event-dispatch-request-v1",
                "name": "count-second", "entries": [{"event": event["id"], "subscription": "count", "state_revision": parent}]})
            batch = ["spool-events-dispatch", ws, selection]
            preview = call(host, *batch)
            run = preview["plan"]["entries"][0]["plan"]["context"]["delivery"]
            assert call(host, *batch, "--approve", preview["approval_sha256"])["status"] == "succeeded"
        else:
            delivery = approve(*argv)
            assert delivery["result"]["workspace"]["termination_confirmed"] is True
            run = delivery["context"]["delivery"]
        approve("spool-accept", counter, run)
        checkpoint = approve("spool-state-commit", counter, parent, run)["revision"]
        approve("spool-event-complete", ws, event["id"], "count")
        refs = [c["reference"] for c in call(host, "spool-input-references", counter, run)["claims"]]
        revision = event["event"]["artifact"]["binding"]["revision"]
        assert [r["binding"]["revision"] for r in refs] == [revision, parent]
        pins = write(root / "pins.json", {"schema": "an-native-checkpoint-dependencies-v1",
            "artifacts": [{"instance": str(producer), "run": name, "output": "brief.log", "revision": revision}]})
        declaration = approve("spool-state-dependencies", counter, checkpoint, pins)["dependencies"]
        coverage = write(root / "coverage.json", {"schema": "an-checkpoint-exhaustive-input-coverage-v1",
            "checkpoint": checkpoint, "declaration_sha256": declaration["approval_sha256"], "inputs": [
                {"catalog": r["catalog"], "revision": r["binding"]["revision"], "disposition": use}
                for r, use in zip(refs, ("checkpoint-pin", "not-required"))]})
        reclaim(counter, run)
        approve("spool-input-release", counter, run, "--delivery", ws, "count", "--coverage", coverage)
        assert call(host, *argv)["disposition"] == "already-consumed"
        return checkpoint

    def started(event):
        argv = ["spool-event-deliver", ws, event["id"], "raw"]
        preview = call(host, *argv)
        run = preview["plan"]["context"]["delivery"]
        proc = spawn(host, *argv, "--approve", preview["approval_sha256"])
        deadline = time.monotonic() + 12
        while not (audit / "runs" / run / "job/work/ready.marker").is_file():
            if time.monotonic() > deadline or proc.poll() is not None:
                raise RuntimeError("Readiness unconfirmed; retain evidence without retry")
            time.sleep(.02)
        return argv, run, proc

    sizes = {"loaded": footprint([producer, counter, audit, ws])}
    first = produce("first", "INFO initial\n")
    bulk = "WARN hold\n" + "INFO stable bytes\n" * 3000
    second = produce("second", bulk)
    # Both consumers must still get the first version after a newer batch exists.
    old = first["brief.log"]
    parent = first_checkpoint = count("first", old, parent)
    observer = approve("spool-event-deliver", ws, old["id"], "observe")["context"]["delivery"]
    approve("spool-accept", audit, observer)
    accepted = call(host, "spool-artifacts", audit, observer)
    output = root / "first-observation.txt"
    call(host, "artifact-checkout", accepted["catalog"], accepted["outputs"][0]["binding"]["revision"], output)
    assert output.read_text() == "INFO initial\n"
    assert call(host, "spool-input-references", audit, observer)["claims"][0]["reference"]["binding"] == old["event"]["artifact"]["binding"]
    approve("spool-event-complete", ws, old["id"], "observe")
    reclaim(audit, observer)
    approve("spool-input-release", audit, observer, "--delivery", ws, "observe")
    assert len(call(host, *managed("show", old))["readers"]) == 3
    approve(*managed("close", old))
    call(host, *managed("delete", old), ok=False, error="checkpoint")

    raw = second["report.txt"]
    argv, run, proc = started(raw)
    proc.kill()
    finish(proc, ok=False)
    try:
        approve(*managed("close", raw))
        call(host, *managed("delete", raw), ok=False, error="release is unconfirmed")
        parent = count("second", second["brief.log"], parent)
    finally:
        approve("spool-supervisor", "cancel", audit, run)
    deadline = time.monotonic() + 15
    while not (audit / "runs" / run / "supervisor/completion.json").exists():
        if time.monotonic() > deadline:
            raise RuntimeError("No supervisor completion; retain original attempt")
        time.sleep(.025)
    cancelled = call(host, "spool-status", audit, run)
    assert cancelled["execution_status"] == "cancelled" and cancelled["termination_confirmed"] is True
    reclaim(audit, run)
    approve("spool-input-release", audit, run)
    token = call(host, *managed("delete", raw))["approval_sha256"]
    # In a disposable instance remove just the final consumer receipt. A source marker survives.
    marker = audit / "runs" / run / "input-released.json"
    saved = marker.with_suffix(".saved")
    marker.rename(saved)
    call(host, *managed("delete", raw), "--approve", token, ok=False)
    assert call(host, *managed("show", raw))["intent"] is None
    saved.rename(marker)
    data = raw["event"]["artifact"]
    db = Path(data["catalog"]) / "revisions" / data["binding"]["revision"] / "registered-custody.sqlite3"
    before = db.stat().st_size
    receipt = call(host, *managed("delete", raw), "--approve", token)["receipt"]
    after = db.stat().st_size
    assert receipt["logical_bytes"] == len(bulk.encode()) and after < before
    assert call(host, *managed("show", raw))["phase"] == "deleted"
    assert call(host, "spool-artifacts", producer, "second", "--output", "brief.log")["payload_verified"]
    sizes["after_collection"] = footprint([producer, counter, audit, ws])

    # An author selects one declared output. The larger sibling stays with the producer.
    audience = call(host, "spool-show", audit)["instance"]
    selected = "one approved observation"
    packet = json.dumps({"schema": "an-context-v1", "source": "domain:brief", "audience": audience,
                         "text": selected, "sha256": hashlib.sha256(selected.encode()).hexdigest()}) + "\n"
    private_input = root / "context-source.txt"
    private_input.write_text(packet + "unrelated private note\n")
    approve("spool-start", producer, "context", private_input, 15)
    approve("spool-accept", producer, "context")
    approve("spool-reclaim", producer, "context")
    offer = write(root / "context-peer.json", {"schema": "an-native-peer-context-request-v1",
        "source_run": "context", "source_output": "brief.log", "target": str(audit),
        "reply_output": "report.txt", "wall_seconds": 15})
    facts = (ws / "facts/log.jsonl").read_bytes()
    approve("spool-peer", "request", producer, "brief", offer)
    received = approve("spool-peer", "run", producer, "brief", "request")
    approve("spool-accept", audit, received["run"])
    result = call(host, "spool-artifacts", audit, received["run"])
    copy = root / "received-context.txt"
    call(host, "artifact-checkout", result["catalog"], result["outputs"][0]["binding"]["revision"], copy)
    assert copy.read_bytes() == packet.encode() and (ws / "facts/log.jsonl").read_bytes() == facts

    third = produce("third", "ERROR interrupt\n")
    raw = third["report.txt"]
    argv, run, proc = started(raw)
    owner_dir = audit / "runs" / run / "supervisor"
    owner = json.loads((owner_dir / "owner.json").read_text())
    assert owner["request"] == json.loads((owner_dir / "request.json").read_text())["approval_sha256"]
    proc.kill()
    finish(proc, ok=False)
    os.kill(owner["pid"], signal.SIGKILL)  # Only the verified owner of this disposable attempt.
    (audit / "runs" / run / "job/work/exit.marker").touch(exist_ok=False)
    assert not call(host, "spool-status", audit, run)["termination_confirmed"]
    parent = count("third", third["brief.log"], parent)
    approve(*managed("close", raw))
    call(host, *managed("delete", raw), ok=False, error="release is unconfirmed")
    call(host, "spool-reclaim", audit, run, ok=False)
    observed = call(host, *argv, ok=False)
    assert observed["status"] == "uncertain" and observed["automatic_retries"] == 0 and observed["executed"] is False
    assert len(list((audit / "runs" / run / "job/attempts").iterdir())) == 1
    assert (audit / "runs" / run / "job/work").is_dir()
    state = call(host, "spool-state-show", counter, parent)
    final = root / "final-state.txt"
    # The accepted state output and the enrolled checkpoint bind the same bytes.
    committed = state["selected"]["binding"]
    assert committed["reader_profile"] == "registered-copy-v1"
    source = committed["source"]
    accepted = call(host, "spool-artifacts", counter, source["run"], "--output", "state.txt")
    assert accepted["output"]["binding"]["revision"] == source["artifact"]["revision"]
    call(host, "artifact-checkout", accepted["catalog"], source["artifact"]["revision"], final)
    assert hashlib.sha256(final.read_bytes()).hexdigest() == committed["content_sha256"]
    assert final.read_text() == "total=3\nINFO=1\nWARN=1\nERROR=1\n"
    assert call(host, *managed("show", old))["phase"] == "closed"
    approve("spool-event-subscribe", ws, "retiring", root / "count.json")
    stale = ["spool-event-deliver", ws, third["brief.log"]["id"], "retiring", "--state", first_checkpoint]
    earlier = call(host, *stale)
    approve("spool-state-retire", counter, first_checkpoint)
    call(host, *stale, "--approve", earlier["approval_sha256"], ok=False, error="admission is closed")
    assert not (ws / "deliveries" / earlier["plan"]["context"]["delivery"]).exists()
    proof = approve("spool-state-release", counter, first_checkpoint)["receipt"]
    assert len(proof["released_references"]) == 1
    assert call(host, "spool-state-show", counter, first_checkpoint)["dependencies"]["receipt"] == proof
    assert call(host, *managed("delete", old))["executed"] is False
    assert call(host, *managed("show", old))["phase"] == "closed"
    assert call(host, "spool-state-show", counter)["head"] == parent
    sizes["three_rounds"] = footprint([producer, counter, audit, ws])
    summary = {"passed": True, "host_revision": args.host_revision, "host_sha256": hashlib.sha256(host.read_bytes()).hexdigest(),
               "loom_sha256": hashlib.sha256(loom.read_bytes()).hexdigest(), "releases": [src, acc, aud],
               "producer_rounds": 3, "consumers": 2, "final_state": final.read_text().splitlines(),
               "first_revision": old["event"]["artifact"]["binding"]["revision"], "deleted_revision": receipt["revision"],
               "logical_bytes_removed": receipt["logical_bytes"], "database_bytes": [before, after], "footprint": sizes,
               "checkpoint_release": proof["approval_sha256"], "confirmed_cancellation": True, "missing_consumer_receipt_blocked": True,
               "cancel_after_client_loss": True, "lost_supervisor_attempts": 1, "lost_cli_attempts": 2,
               "unknown_disposition": observed["disposition"], "automatic_retries": 0}
    # A fresh context request cannot take over the still-held receiver.
    call(host, "spool-peer", "request", producer, "after-interruption", offer, ok=False)
    assert not (producer / "peers/after-interruption").exists()
    assert len(list((audit / "runs" / run / "job/attempts").iterdir())) == 1
    summary["bounded_peer_packet_bytes"] = len(packet.encode())
    write(root / "acceptance.json", summary)
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
