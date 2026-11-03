#!/usr/bin/env python3
"""Prepare finite MCP fixtures and check host evidence; never launch a model client."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def call(executable, *args):
    p = subprocess.run([str(executable), *map(str, args)], capture_output=True, text=True)
    if p.returncode:
        raise RuntimeError(p.stderr + p.stdout)
    # The host run command emits a plan followed by its result. Other commands
    # have one JSON record. Do not interpret mixed human diagnostics as success.
    records, rest, decoder = [], p.stdout, json.JSONDecoder()
    while rest.strip():
        value, n = decoder.raw_decode(rest.lstrip())
        records.append(value)
        rest = rest.lstrip()[n:]
    expected = 2 if args and args[0] == "run" else 1
    if len(records) != expected:
        raise RuntimeError("Unexpected CLI result sequence")
    return records[-1]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def selected(root):
    settings = read(root / "settings.local.json")
    keys = {"loom", "host", "bridge", "toolchain"}
    if set(settings) != keys | {"sha256"} or set(settings["sha256"]) != keys:
        raise RuntimeError("Unsupported fixture settings; use the original adapter for legacy client-bound directories")
    for key, expected in settings["sha256"].items():
        if digest(settings[key]) != expected:
            raise RuntimeError("Selected tool or toolchain profile changed: " + key)
    return settings


def build(root, name, source=None):
    settings = selected(root)
    loom, host = settings["loom"], settings["host"]
    project = root / name
    call(loom, "init", project)
    config = project / "loom.toml"
    config.write_text(config.read_text().replace('name = "file-list"', f'name = "{name}"'))
    if source is not None:
        (project / "kit/main.rs").write_text(source)
    args = ("develop", project, host, settings["toolchain"], root / (name + "-build"))
    plan = call(loom, *args)
    write(root / (name + "-plan.local.json"), plan)
    # prepare authorizes only committed fixtures; accept requires an explicitly
    # reviewed candidate digest. The agent cannot call this operator function.
    call(loom, *args, "--approve", plan["approval_sha256"])
    release = root / (name + "-release")
    call(loom, "release", project, release)
    call(loom, "verify-release", release)
    return call(host, "register", release / "deployment", root / "store")["registered"]


def configure(root, name, handles):
    settings = selected(root)
    grant = root / (name + "-grant.local.json")
    write(grant, {"schema": "an-agent-tools-v1", "store": str(root / "store"),
                  "session": str(root / (name + "-session")),
                  "source": str(root / "sort-kit/kit/main.rs"), "kits": handles, "max_calls": 16})
    plan = call(settings["bridge"], grant)
    write(root / (name + "-approval.local.json"), plan)
    return plan


def prepare(a):
    root = a.output.resolve()
    root.mkdir(mode=0o700)
    settings = {k: str(getattr(a, k).resolve())
                for k in ("loom", "host", "bridge", "toolchain")}
    settings["sha256"] = {k: digest(v) for k, v in settings.items()}
    write(root / "settings.local.json", settings)
    sort = build(root, "sort-kit")
    source = (root / "sort-kit/kit/main.rs").read_text()
    wait = source.replace("let result = transform(&input);", '''if input == "wait\\n" {
        fs::write("output.txt", "pending")?;
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
    let result = transform(&input);''')
    slow = build(root, "wait-probe", wait)
    handles = {"sort": sort, "wait": slow}
    write(root / "handles.json", handles)
    configure(root, "development", handles)
    configure(root, "cancellation", handles)
    print("Reviewed fixtures built; inspect both approval files before configuring your native MCP client.")


def check(a):
    root = a.output.resolve()
    selected(root)
    session = root / "development-session/calls"
    inspected = read(session / "inspect-1/result.json")
    result = read(session / "sample-1/result.json")
    denied = read(session / "denied-1/result.json")
    proposal = read(session / "edit-1/result.json")
    if not (inspected["sha256"] == digest(root / "sort-kit/kit/main.rs")
            and result["status"] == "succeeded" and result["reaped"] is True
            and result["publication"]["status"] == "committed" and result.get("artifact")
            and result["text"] == "alpha\nbeta\n"
            and denied["status"] == "refused-or-uncertain" and denied["error"] == "kit capability is not granted"
            and proposal["executed"] is False and proposal["accepted"] is False
            and digest(session / "edit-1/candidate.rs") == proposal["sha256"]):
        raise RuntimeError("Incomplete or inconsistent finite-MCP results")
    print("Review candidate before accept; sha256=" + proposal["sha256"])


def accept(a):
    check(a)
    root = a.output.resolve()
    settings = selected(root)
    candidate = root / "development-session/calls/edit-1/candidate.rs"
    data = candidate.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if a.source_sha256 != digest:
        raise RuntimeError("Reviewed source digest mismatch")
    release = build(root, "case-fold-kit", data.decode())
    inp = root / "case-input.txt"
    inp.write_text("Beta\nALPHA\nbeta\n")
    result = call(settings["host"], "run", root / "store", release, inp, "--execute")
    receipt = read(Path(result["receipt"]))
    assert receipt["status"] == "succeeded"
    assert Path(result["receipt"]).parent.joinpath("artifact.txt").read_bytes() == b"alpha\nbeta\n"
    write(root / "development-summary.json", {"model": None, "source_reviewed": True,
          "proposal_sha256": digest, "release": release, "tests": "passed",
          "candidate_was_not_executed_by_agent": True, "ungranted_kit_refused": True,
          "result_sha256": receipt["artifact"]["sha256"], "native_tape": "tool-managed"})
    print("Reviewed candidate built, tested, packaged, registered and executed successfully.")


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("stage", choices=["prepare", "check", "accept"])
    p.add_argument("--output", type=Path, required=True)
    for name in ["loom", "host", "bridge", "toolchain"]:
        p.add_argument("--" + name, type=Path)
    p.add_argument("--source-sha256")
    a = p.parse_args()
    if a.stage == "prepare":
        if any(getattr(a, k) is None for k in ["loom", "host", "bridge", "toolchain"]):
            p.error("prepare requires Loom, host, bridge and the selected toolchain profile")
        prepare(a)
    elif a.stage == "check":
        check(a)
    elif a.stage == "accept":
        accept(a)


if __name__ == "__main__":
    main()
