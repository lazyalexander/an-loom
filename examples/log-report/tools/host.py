"""Optional, operator-owned host adapter. The business kit never imports this."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + "\n")


class Host:
    def __init__(self, executable, python, output):
        self.executable = executable.resolve()
        self.identity = sha(self.executable)
        self.python = python.resolve()
        # Inspect the selected interpreter without relocating its environment.
        info = subprocess.check_output([str(python), "-c", "import sys,site,json; print(json.dumps({'base':sys.base_prefix,'prefix':sys.prefix,'site':site.getsitepackages()}))"], text=True)
        self.runtime = json.loads(info)
        self.output = output

    def command(self, *args):
        if sha(self.executable) != self.identity:
            raise RuntimeError("Selected host executable changed")
        return [str(self.executable), *map(str, args)]

    def call(self, *args, refused=False):
        p = subprocess.run(self.command(*args), text=True, capture_output=True, timeout=90)
        if refused:
            if p.returncode == 0:
                raise RuntimeError("Expected refusal, received success")
            return None
        if p.returncode:
            raise RuntimeError(p.stderr + p.stdout)
        return json.loads(p.stdout)

    def approved(self, *args):
        preview = self.call(*args)
        # This fixed acceptance driver is invoked by the operator. Project text
        # and the kit cannot approve a plan or change this execution authority.
        return self.call(*args, "--approve", preview["approval_sha256"])

    def profile(self, stage, source, output, binding=None, cancel=False):
        roots = [Path(self.runtime["base"]), Path(self.runtime["prefix"]), HERE, source.parent]
        roots += [Path(x) for x in self.runtime["site"]]
        roots = sorted(set(p.resolve() for p in roots))
        roots = [p for p in roots if not any(p != q and p.is_relative_to(q) for q in roots)]
        # pathlib.is_relative_to is available in Python 3.9+.
        script = HERE / "tools/cancel_probe.py" if cancel else HERE / "kit/main.py"
        args = ["-B", "-s", str(script), stage, "--input", str(source), "--output", "{work}/" + output]
        inputs = [source, script, HERE / "requirements.txt", *sorted((HERE / "kit").glob("*.py"))]
        profile = {"schema": "an-workspace-job-v1", "executable": str(self.python),
                   "args": args, "readonly": list(map(str, roots)), "inputs": list(dict.fromkeys(map(str, inputs))),
                   "environment": {"HOME": "{work}", "PYTHONNOUSERSITE": "1", "PYTHONDONTWRITEBYTECODE": "1",
                                   "PYTHONPATH": os.pathsep.join(self.runtime["site"])},
                   "outputs": [output], "binary_outputs": [output], "wall_seconds": 60,
                   "trusted_process_group": True}
        if binding:
            catalog, revision = binding
            profile["artifact_inputs"] = [{"path": str(source), "catalog": str(catalog), "revision": revision}]
        return profile

    def stage(self, name, profile):
        path = self.output / (name + ".local.json")
        save(path, profile)
        root = self.output / name
        receipt = self.approved("workspace-run", path, root)
        if receipt["status"] != "succeeded" or not receipt["termination_confirmed"]:
            raise RuntimeError("Workload did not terminate successfully")
        accepted = self.approved("workspace-accept", root, receipt["attempt"])
        return root, receipt, accepted["outputs"][0]["binding"]

    def checkout(self, catalog, revision, path):
        path.parent.mkdir(parents=True, exist_ok=True)
        return self.call("artifact-checkout", catalog, revision, path)
