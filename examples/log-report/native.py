#!/usr/bin/env python3
"""Prepare two ordinary spool projects and an operator-selected Python environment.

This example adapter never grants execution, installs packages or edits a release.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inventory(root):
    files = {}
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise ValueError("Dependency links are unsupported")
        if path.is_file():
            files[path.relative_to(root).as_posix()] = sha(path)
    return hashlib.sha256(json.dumps(files, ensure_ascii=False, sort_keys=True,
                                     separators=(",", ":")).encode()).hexdigest()


def dependencies():
    return dict(line.split("==") for line in (HERE / "requirements.txt").read_text().splitlines()
                if line and not line.startswith("#"))


def select_runtime(python, destination):
    # Explicit operator tooling, outside Loom/host. Inspect only the selected tool.
    inspect = """import importlib.metadata,json,site,sys
names = json.loads(sys.argv[1])
print(json.dumps({'exe':sys.executable,'base':sys.base_prefix,'sites':site.getsitepackages(),
                 'versions':{name:importlib.metadata.version(name) for name in names}}))
"""
    pins = dependencies()
    info = json.loads(subprocess.check_output(
        [str(python), "-B", "-s", "-c", inspect, json.dumps(list(pins))], text=True, timeout=30))
    if info["versions"] != pins or len(info["sites"]) != 1:
        raise ValueError("Use an isolated virtual environment with the pinned example dependencies")
    exe = Path(info["exe"]).resolve(strict=True)
    base = Path(info["base"]).resolve(strict=True)
    site = Path(info["sites"][0]).resolve(strict=True)
    value = {"schema": "an-native-python-runtime-v1", "executable": str(exe),
             "executable_sha256": sha(exe), "stdlib": str(base), "site": str(site),
             "site_sha256": inventory(site),
             "dependencies": sorted(f"{name}-{version}" for name, version in pins.items())}
    with os.fdopen(os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "w") as stream:
        json.dump(value, stream, indent=2)
    return value


def prepare(destination):
    destination.mkdir(mode=0o700)
    pins = sorted(f"{name}-{version}" for name, version in dependencies().items())
    for stage, adapter, output in [("workbook", "workbook", "report.xlsx"),
                                   ("slides", "slides", "report.pptx")]:
        input_name = "source.jsonl" if stage == "workbook" else "source.xlsx"
        root = destination / stage
        root.mkdir()
        (root / "silk").mkdir()
        shutil.copytree(HERE / "kit", root / "kit", ignore=shutil.ignore_patterns("__pycache__"))
        shutil.copytree(HERE / "tests", root / "tests", ignore=shutil.ignore_patterns("__pycache__"))
        for name in ["requirements.txt", "sample.jsonl", "LICENSE"]:
            shutil.copyfile(HERE / name, root / name)
        (root / "README.md").write_text(
            f"# Log report {stage}\n\nHeadless reference spool; domain rules stay in kit/.\n"
            "Run the standalone kit or use an explicitly selected host runtime.\n")
        (root / "silk/render.py").write_text(
            '"""Stable domain entry; runtime selection and custody belong to the host."""\n'
            "import os\nfrom pathlib import Path\n"
            f"from {adapter} import create\n\n"
            f'create(Path(os.environ["AN_SPOOL_INPUT"]), Path("{output}"))\n')
        code = ["silk/render.py", *sorted("kit/" + p.name for p in (root / "kit").glob("*.py"))]
        include = ["loom.toml", "native-spool.toml", "README.md", "LICENSE", "requirements.txt",
                   "sample.jsonl", "tests/test_kit.py", *code]
        (root / "loom.toml").write_text(
            f'schema_version = 1\nname = "log-report-{stage}"\nversion = "0.1.0"\n'
            f'description = "Headless log report {stage}, reusing the standalone document adapters"\n'
            f"include = {json.dumps(include)}\n")
        (root / "native-spool.toml").write_text(
            'schema = "an-native-spool-v1"\nruntime = "macos-native-python-v1"\n'
            'entry = "silk/render.py"\nsilk = ["silk/render.py"]\n'
            f"kit = {json.dumps(code[1:])}\neditable = {json.dumps(code[1:])}\n"
            f'inputs = ["{input_name}"]\noutputs = ["{output}"]\nbinary_outputs = ["{output}"]\n'
            'capabilities = ["read-input", "write-declared-outputs"]\n'
            f"dependencies = {json.dumps(pins)}\n"
            'acceptance = ["python3 -B -m unittest discover -s tests -v"]\n\n[events]\n'
            f'consumes = {json.dumps(["artifact.accepted"] if stage == "slides" else [])}\n'
            'produces = ["artifact.accepted"]\n')
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    project = commands.add_parser("prepare")
    project.add_argument("destination", type=Path)
    runtime = commands.add_parser("runtime")
    runtime.add_argument("--python", type=Path, required=True)
    runtime.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.destination)
    else:
        select_runtime(args.python.absolute(), args.output)


if __name__ == "__main__":
    main()
