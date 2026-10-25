"""Opt-in operator process observations, never execution authority or an audit tape."""
import argparse
import json
from pathlib import Path
import subprocess
import time
import psutil


def append(path, value):
    with path.open("a") as f:
        f.write(json.dumps(value, separators=(",", ":")) + "\n")


def event(root, name, **fields):
    origin = json.loads((root / "origin.private.json").read_text())["monotonic"]
    append(root / "events.jsonl", dict(t=time.monotonic() - origin, name=name, **fields))


def watch(root, pid, role):
    append(root / "watch.private.jsonl", {"pid": pid, "role": role})


def run(root, executable, interval):
    origin = time.monotonic()
    (root / "origin.private.json").write_text(json.dumps({"monotonic": origin, "observer_pid": psutil.Process().pid}))
    tracked, lines = {}, 0
    missing_editor = True

    def add(pid, role):
        try:
            p = psutil.Process(pid)
            key = (pid, p.create_time())
            if key not in tracked:
                tracked[key] = dict(process=p, role=role, cpu=0)
        except psutil.NoSuchProcess:
            pass

    add(psutil.Process().pid, "observer")
    event(root, "measurement.started")
    while not (root / "stop").exists() and time.monotonic() - origin < 1800:
        control = root / "watch.private.jsonl"
        requests = control.read_text().splitlines() if control.exists() else []
        for row in requests[lines:]:
            request = json.loads(row)
            add(request["pid"], request["role"])
        lines = len(requests)
        # Discovery inspects only exact selected app candidates, never command
        # arguments, environment variables or unrelated process resource records.
        probe = subprocess.run(["/usr/bin/pgrep", "-x", executable.name], capture_output=True, text=True)
        for raw in probe.stdout.splitlines():
            try:
                p = psutil.Process(int(raw))
                if Path(p.exe()).resolve() == executable and p.create_time() >= time.time() - (time.monotonic() - origin) - 1:
                    add(p.pid, "editor")
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
        for entry in list(tracked.values()):
            if entry["role"] == "observer":
                continue
            try:
                if not entry["process"].is_running():
                    continue
                for child in entry["process"].children(recursive=True):
                    add(child.pid, "editor" if entry["role"] == "editor" else "kit")
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
        aggregate = {role: dict(rss_bytes=0, cpu_seconds=0, alive=0, unknown=False)
                     for role in ["editor", "host", "kit", "observer"]}
        for key, entry in tracked.items():
            role = aggregate[entry["role"]]
            p = entry["process"]
            try:
                if not p.is_running() or p.status() == psutil.STATUS_ZOMBIE:
                    raise psutil.NoSuchProcess(p.pid)
                with p.oneshot():
                    cpu = p.cpu_times()
                    entry["cpu"] = cpu.user + cpu.system
                    role["rss_bytes"] += p.memory_info().rss
                    role["alive"] += 1
            except psutil.NoSuchProcess:
                pass
            except psutil.AccessDenied:
                role["unknown"] = True
            role["cpu_seconds"] += entry["cpu"]
        alive = aggregate["editor"]["alive"] > 0
        if alive and missing_editor:
            event(root, "editor.process.observed", state="running")
        if not alive and not missing_editor and not aggregate["editor"]["unknown"]:
            event(root, "editor.process.absent", state="exited")
        if not aggregate["editor"]["unknown"]:
            missing_editor = not alive
        for role, values in aggregate.items():
            if values["unknown"]:
                values["rss_bytes"] = None
            append(root / "samples.jsonl", dict(t=time.monotonic() - origin, role=role, **values))
        time.sleep(interval)
    event(root, "measurement.stopped", reason="requested" if (root / "stop").exists() else "duration-limit")
    (root / "done").write_text("observer stopped")


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--root", type=Path, required=True)
    p.add_argument("--editor-executable", type=Path, required=True)
    p.add_argument("--interval", type=float, default=.25)
    a = p.parse_args()
    if not .1 <= a.interval <= 2:
        p.error("interval must be between 0.1 and 2 seconds")
    run(a.root.resolve(), a.editor_executable.resolve(), a.interval)
