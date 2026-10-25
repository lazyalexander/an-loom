#!/usr/bin/env python3
"""Staged native-editor acceptance. UI control stays in the replaceable editor adapter."""
import argparse
import hashlib
import json
from pathlib import Path
import signal
import subprocess
import sys
import time
import zipfile
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools.host import Host, save, sha
from desktop.monitor import event, watch
import psutil

HERE = Path(__file__).resolve().parent
STATE = "session.private.json"


class ObservedHost(Host):
    def call(self, *args, refused=False):
        event(self.output, "host.call.begin", command=str(args[0]))
        p = subprocess.Popen(self.command(*args), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        watch(self.output, p.pid, "host")
        try:
            stdout, stderr = p.communicate(timeout=90)
        except subprocess.TimeoutExpired:
            p.send_signal(signal.SIGINT)
            p.communicate(timeout=15)
            event(self.output, "host.call.uncertain", command=str(args[0]))
            raise
        event(self.output, "host.call.end", command=str(args[0]), code=p.returncode)
        if refused:
            if p.returncode == 0:
                raise RuntimeError("Expected refusal")
            return None
        if p.returncode:
            raise RuntimeError(stderr + stdout)
        return json.loads(stdout)


def load(root):
    return json.loads((root / STATE).read_text())


def host(root, state):
    return ObservedHost(Path(state["host"]), Path(state["python"]), root)


def prepare(a, root):
    root.mkdir(mode=0o700)
    editor = a.editor_executable.resolve()
    # Refuse to characterize an already-running shared app as our fresh launch.
    before = subprocess.run(["/usr/bin/pgrep", "-x", editor.name], capture_output=True, text=True)
    if any(Path(psutil.Process(int(pid)).exe()).resolve() == editor for pid in before.stdout.splitlines()):
        raise RuntimeError("Selected editor already running; use a dedicated editor session")
    observer = subprocess.Popen([str(a.observer_python.absolute()), str(HERE / "monitor.py"),
                                "--root", str(root), "--editor-executable", str(editor)],
                               stdout=(root / "observer.stdout").open("w"), stderr=(root / "observer.stderr").open("w"),
                               start_new_session=True)
    deadline = time.monotonic() + 10
    while not (root / "origin.private.json").exists():
        if observer.poll() is not None or time.monotonic() > deadline:
            raise RuntimeError("Observer failed to start")
        time.sleep(.02)
    state = dict(host=str(a.host.resolve()), python=str(a.python.absolute()), host_revision=a.host_revision,
                 editor=str(editor), observer_pid=observer.pid, revisions={})
    try:
        h = host(root, state)
        event(root, "workbook.begin")
        producer, receipt, binding = h.stage("workbook", h.profile("workbook", Path(__file__).resolve().parents[1] / "sample.jsonl", "report.xlsx"))
        state.update(producer=str(producer), producer_attempt=receipt["attempt"])
        state["revisions"]["v1"] = binding
        working = (a.working_directory.resolve() if a.working_directory else root / "editable") / "report.xlsx"
        h.checkout(producer / "artifacts", binding["revision"], working)
        state["working"] = str(working)
        save(root / STATE, state)
        event(root, "workbook.accepted", revision=binding["revision"])
        event(root, "editor.launch.requested")
        print(json.dumps({"working_copy":str(working),"next":"Open this file in the real editor. Edit E8/F8, save XLSX, then commit its reviewed SHA-256."}))
    except BaseException:
        (root / "stop").write_text("prepare failed")
        observer.wait(timeout=10)
        raise



def commit(a, root, state):
    if a.label not in ["v2", "v3"] or a.label in state["revisions"]:
        raise RuntimeError("Use a new v2/v3 label; no replay")
    parent = "v1" if a.label == "v2" else "v2"
    file = Path(state["working"])
    first = file.read_bytes()
    time.sleep(.3)
    if first != file.read_bytes() or hashlib.sha256(first).hexdigest() != a.approve:
        raise RuntimeError("Saved document is unstable or approval is stale")
    # Read-only domain validation; this stage never writes workbook content.
    code = "import sys,json;sys.path.insert(0,sys.argv[1]);from workbook import read;print(json.dumps(read(sys.argv[2])))"
    rows = json.loads(subprocess.check_output([state["python"], "-B", "-c", code, str(HERE.parent / "kit"), str(file)], text=True))
    if rows[0]["note"] != a.expected_note:
        raise RuntimeError("Saved native-editor note does not match the observed UI edit")
    with zipfile.ZipFile(file) as z:
        producer = z.read("docProps/app.xml").decode()
    h = host(root, state)
    event(root, "revision.commit.begin", label=a.label)
    revision = h.call("artifact-edit", Path(state["producer"]) / "artifacts", state["revisions"][parent]["revision"], file, a.approve)
    snapshot = root / "saved" / (a.label + ".xlsx")
    h.checkout(Path(state["producer"]) / "artifacts", revision["revision"], snapshot)
    state["revisions"][a.label] = revision
    state.setdefault("notes", {})[a.label] = a.expected_note
    state.setdefault("editor_evidence", {})[a.label] = dict(method="native GUI edit/save observed by operator", producer_hint="LibreOffice" if "LibreOffice" in producer else "other")
    save(root / STATE, state)
    event(root, "revision.committed", label=a.label, revision=revision["revision"])
    print(json.dumps(revision))


def finish(a, root, state):
    if "v3" not in state["revisions"]:
        raise RuntimeError("Both native saved revisions are required")
    events = [json.loads(line) for line in (root / "events.jsonl").read_text().splitlines()]
    editor_samples = [json.loads(line) for line in (root / "samples.jsonl").read_text().splitlines() if json.loads(line)["role"] == "editor"]
    closed = [e["t"] for e in events if e["name"] == "document.closed"]
    if ((root / "done").exists() or not editor_samples or editor_samples[-1]["unknown"] or editor_samples[-1]["alive"] != 0
            or not closed or not any(e["name"] == "editor.process.absent" and e["t"] >= closed[-1] for e in events)):
        raise RuntimeError("A live capture and observed editor exit are required")
    h = host(root, state)
    chosen = state["revisions"]["v2"]["revision"]
    selected = root / "inputs/v2.xlsx"
    h.checkout(Path(state["producer"]) / "artifacts", chosen, selected)
    event(root, "downstream.begin", selected_revision=chosen)
    output, receipt, ppt = h.stage("slides-pinned-v2", h.profile("slides", selected, "report.pptx", (Path(state["producer"]) / "artifacts", chosen)))
    delivered = root / "documents/report.pptx"
    checked = h.checkout(output / "artifacts", ppt["revision"], delivered)
    code = "from pptx import Presentation;import sys,json;d=Presentation(sys.argv[1]);print(json.dumps({'slides':len(d.slides),'text':'\\n'.join(s.text for p in d.slides for s in p.shapes if s.has_text_frame)}))"
    actual = json.loads(subprocess.check_output([state["python"], "-c", code, str(delivered)], text=True))
    assert state["notes"]["v2"] in actual["text"] and state["notes"]["v3"] not in actual["text"]
    assert checked["binding"]["source"]["input_revisions"][0]["revision"] == chosen
    event(root, "downstream.accepted", revision=ppt["revision"])
    event(root, "workspace.reclaim.begin")
    h.approved("workspace-reclaim", output, receipt["attempt"])
    h.approved("workspace-reclaim", Path(state["producer"]), state["producer_attempt"])
    assert not (output / "work").exists() and not (Path(state["producer"]) / "work").exists()
    for label, binding in state["revisions"].items():
        h.checkout(Path(state["producer"]) / "artifacts", binding["revision"], root / "retained" / (label + ".xlsx"))
    event(root, "workspace.reclaimed", artifacts="retained")
    summary = dict(schema="an-native-office-acceptance-v1", host_revision=state["host_revision"],
                   revisions=state["revisions"], selected_input_revision=chosen, ppt=ppt,
                   editor_evidence=state["editor_evidence"], native_slide_count=actual["slides"], model_calls=0,
                   assertions=["real desktop GUI saves create child revisions", "v3 exists while downstream consumes v2", "PPT contains v2 note, not v3", "input provenance binds v2", "formal revisions survive workspace collection"],
                   scope="native editor is trusted; operator-observed UI actions are not authenticated human authorship")
    save(root / "acceptance.json", summary)
    # Leave a small measured idle interval before stopping the observer.
    time.sleep(1)
    (root / "stop").write_text("stop")
    deadline = time.monotonic() + 10
    while not (root / "done").exists():
        if time.monotonic() > deadline:
            raise RuntimeError("Observer termination not confirmed")
        time.sleep(.05)
    print("PASS: native saves, pinned downstream and retained binary artifacts")


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("stage", choices=["prepare", "mark", "commit", "finish"])
    p.add_argument("--root", type=Path, required=True)
    for name in ["host", "python", "observer-python", "editor-executable"]:
        p.add_argument("--" + name, type=Path)
    p.add_argument("--working-directory", type=Path, help="Optional visible GUI working directory; report.xlsx must not exist")
    p.add_argument("--host-revision")
    p.add_argument("--event")
    p.add_argument("--label")
    p.add_argument("--approve")
    p.add_argument("--expected-note")
    a = p.parse_args(); root = a.root.resolve()
    if a.stage == "prepare":
        prepare(a, root)
    elif a.stage == "mark":
        event(root, a.event)
    elif a.stage == "commit":
        commit(a, root, load(root))
    else:
        finish(a, root, load(root))
