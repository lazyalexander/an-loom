#!/usr/bin/env python3
"""Narrow real-host state handoff for unchanged Loom native v1/v2 exports."""
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
        raise SystemExit("Real macOS native enforcement is required; no fallback")
    root = options.output.resolve()
    root.mkdir(mode=0o700)
    loom, host = options.loom.resolve(strict=True), options.host.resolve(strict=True)
    calls = []

    def call(binary, *args):
        process = subprocess.Popen([str(binary), *map(str, args)],
                                   text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            stdout, stderr = process.communicate(timeout=30)
        except subprocess.TimeoutExpired as error:
            process.send_signal(signal.SIGINT)
            try:
                stdout, stderr = process.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                raise RuntimeError(f"Termination unknown for {process.pid}; retain {root}") from error
            raise RuntimeError(f"Command exceeded deadline: {args}; {stderr}") from error
        calls.append({"args": list(map(str, args)), "code": process.returncode,
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
    call(loom, "check", project)
    archive = call(loom, "pack", project)["archive"]
    call(loom, "verify", archive)
    qualified = []
    # This is protocol qualification, not a new domain-specific Loom state engine.
    for schema, exporter in [("v1", "export-native-spool"), ("v2", "export-native-spool-v2")]:
        delivery = root / f"delivery-{schema}"
        call(loom, exporter, archive, delivery)
        instance = root / f"instance-{schema}"
        call(host, "spool-load", delivery, instance)
        seed = root / f"seed-{schema}.txt"
        seed.write_text("hello\n")
        revision = approved("spool-state-init", instance, "report.txt", seed)["state"]["revision"]
        history = [revision]
        controls = []
        for run, expected in [("first", "Report\nHELLO\n"), ("second", "Report\nREPORT\nHELLO\n")]:
            result = approved("spool-start-state", instance, run, revision, "15")
            assert result["status"] == "succeeded"
            assert result["workspace"]["termination_confirmed"] is True
            approved("spool-accept", instance, run)
            approved("spool-reclaim", instance, run)
            assert not (instance / "runs" / run / "job/work").exists()
            revision = approved("spool-state-commit", instance, revision, run)["revision"]
            history.append(revision)
            checkpoint = call(host, "spool-state-show", instance, revision)["selected"]
            saved = root / f"{schema}-{run}.txt"
            call(host, "artifact-checkout", checkpoint["catalog"], revision, saved)
            assert saved.read_text() == expected
            if run == "first":
                paused = approved("spool-pause", instance)["round_control"]
                assert paused["mode"] == "paused"
                resumed = approved("spool-resume", instance)["round_control"]
                assert resumed["mode"] == "active"
                assert not (instance / "runs/second/job").exists()
                controls.extend([paused, resumed])
        suspended = approved("spool-suspend", instance, revision)["round_control"]
        assert suspended["mode"] == "suspended"
        assert suspended["checkpoint"]["binding"]["revision"] == revision
        assert call(host, "spool-state-show", instance)["head"] == revision
        resumed = approved("spool-resume", instance)["round_control"]
        assert resumed["mode"] == "active"
        controls.extend([suspended, resumed])
        approved("spool-unload", instance)
        assert call(host, "spool-state-show", instance)["head"] == revision
        qualified.append({"format": schema, "release": result["release"], "revisions": history,
                          "control_modes": [value["mode"] for value in controls],
                          "control_generations": [value["generation"] for value in controls]})
    summary = {"schema": "loom-native-state-qualification-v1", "passed": True,
               "host_revision": options.host_revision,
               "host_sha256": hashlib.sha256(host.read_bytes()).hexdigest(),
               "loom_sha256": hashlib.sha256(loom.read_bytes()).hexdigest(),
               "platform": platform.platform(), "qualified": qualified,
               "model_invoked": False, "execution_retried": False}
    (root / "acceptance.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
