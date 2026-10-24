#!/usr/bin/env python3
"""Fixed operator acceptance: saved edits, pinned consumption and binary custody."""
import argparse
import hashlib
import json
from pathlib import Path
import signal
import subprocess
import sys
import time
from tools.host import Host, HERE, save, sha


def inspect(python, script):
    return json.loads(subprocess.check_output([str(python), "-c", script], text=True))


def modify(python, path, note):
    # This simulates an operator-saved edit, not an actual human GUI interaction.
    subprocess.run([str(python), "-c", "import openpyxl,sys; p=sys.argv[1]; w=openpyxl.load_workbook(p); w['Report']['E8']='Operations'; w['Report']['F8']=sys.argv[2]; w.save(p)", str(path), note], check=True)


def main(a):
    out = a.output.resolve()
    out.mkdir(mode=0o700)
    host = Host(a.host, a.python, out)
    source = HERE / "sample.jsonl"
    first_root, _, first = host.stage("workbook", host.profile("workbook", source, "report.xlsx"))
    catalog = first_root / "artifacts"
    v1 = first["revision"]
    original = out / "documents/report-v1.xlsx"
    host.checkout(catalog, v1, original)
    pinned_v1 = out / "inputs/v1/report.xlsx"
    host.checkout(catalog, v1, pinned_v1)
    _, _, ppt1 = host.stage("slides-v1", host.profile("slides", pinned_v1, "report.pptx", (catalog, v1)))
    host.checkout(out / "slides-v1/artifacts", ppt1["revision"], out / "documents/report-v1.pptx")
    edited = out / "editable/report.xlsx"
    host.checkout(catalog, v1, edited)
    before = sha(edited)
    note = "Prioritize API timeout investigation"
    modify(a.python, edited, note)
    host.call("artifact-edit", catalog, v1, edited, before, refused=True)
    second = host.call("artifact-edit", catalog, v1, edited, sha(edited))
    assert second["parent"] == v1 and second["encoding"] == "bytes"
    v2 = second["revision"]
    host.checkout(catalog, v2, out / "documents/report-v2.xlsx")
    pinned = out / "inputs/v2/report.xlsx"
    host.checkout(catalog, v2, pinned)
    # A later saved version must not redirect a previously selected consumer.
    modify(a.python, edited, "Later edit must not appear in the pinned report")
    third = host.call("artifact-edit", catalog, v2, edited, sha(edited))
    assert third["parent"] == v2
    profile = host.profile("slides", pinned, "report.pptx", (catalog, v2))
    path = out / "stale.local.json"
    save(path, profile)
    preview = host.call("workspace-run", path, out / "stale-attempt")
    snapshot = pinned.read_bytes()
    pinned.write_bytes(edited.read_bytes())
    host.call("workspace-run", path, out / "stale-attempt", "--approve", preview["approval_sha256"], refused=True)
    assert not (out / "stale-attempt").exists()
    pinned.write_bytes(snapshot)
    second_root, second_receipt, ppt2 = host.stage("slides-v2", profile)
    delivered = out / "documents/report-v2.pptx"
    checked = host.checkout(second_root / "artifacts", ppt2["revision"], delivered)
    assert checked["binding"]["source"]["input_revisions"][0]["revision"] == v2
    content = inspect(a.python, "from pptx import Presentation; import json; d=Presentation(" + repr(str(delivered)) + "); print(json.dumps({'slides':len(d.slides),'tables':sum(s.has_table for p in d.slides for s in p.shapes),'text':'\\n'.join(s.text for p in d.slides for s in p.shapes if s.has_text_frame)}))")
    assert content["slides"] == 3 and content["tables"] == 1
    assert note in content["text"] and "Later edit" not in content["text"]
    assert sha(original) == first["content_sha256"]
    # The same ZIP bytes are not accepted by an undeclared text output profile.
    text_profile = host.profile("slides", pinned, "report.pptx", (catalog, v2))
    del text_profile["binary_outputs"]
    path = out / "text-only.local.json"
    save(path, text_profile)
    text_root = out / "text-only"
    receipt = host.approved("workspace-run", path, text_root)
    assert receipt["status"] == "succeeded"
    host.call("workspace-accept", text_root, receipt["attempt"], refused=True)
    assert not (text_root / "artifacts").exists()
    # Cancellation after a complete candidate exists still prohibits publication.
    path = out / "cancel.local.json"
    save(path, host.profile("slides", pinned, "report.pptx", (catalog, v2), cancel=True))
    cancel_root = out / "cancelled"
    preview = host.call("workspace-run", path, cancel_root)
    with (out / "cancel.stdout").open("w") as stdout, (out / "cancel.stderr").open("w") as stderr:
        child = subprocess.Popen(host.command("workspace-run", path, cancel_root, "--approve", preview["approval_sha256"]), stdout=stdout, stderr=stderr, text=True)
        try:
            deadline = time.monotonic() + 40
            while not (cancel_root / "work/ready").exists():
                if child.poll() is not None or time.monotonic() > deadline:
                    raise RuntimeError("No active binary-producing operation; cancellation is not qualified")
                time.sleep(.02)
            attempt = next((cancel_root / "attempts").iterdir()).name
            host.call("workspace-accept", cancel_root, attempt, refused=True)
            child.send_signal(signal.SIGINT)
            child.wait(timeout=15)
        finally:
            if child.poll() is None:
                child.send_signal(signal.SIGINT)
                child.wait(timeout=15)
    cancelled = json.loads((out / "cancel.stdout").read_text())
    assert cancelled["status"] == "cancelled" and cancelled["termination_confirmed"] and cancelled["reaped"]
    host.call("workspace-accept", cancel_root, attempt, refused=True)
    (cancel_root / "work/report.pptx").write_bytes(delivered.read_bytes())
    host.call("workspace-accept", cancel_root, attempt, refused=True)
    assert not (cancel_root / "artifacts").exists()
    # Reclaim scratch only after explicit settlement; formal versions survive.
    host.approved("workspace-reclaim", second_root, second_receipt["attempt"])
    host.checkout(second_root / "artifacts", ppt2["revision"], out / "documents/after-reclaim.pptx")
    assert sha(out / "documents/after-reclaim.pptx") == ppt2["content_sha256"]
    # Optional packaging; no host or document SDK becomes a Loom dependency.
    if a.loom:
        for command in ["check", "build", "pack"]:
            subprocess.run([str(a.loom.resolve()), command, str(HERE)], check=True, stdout=subprocess.DEVNULL)
    versions = inspect(a.python, "import importlib.metadata as m,json; print(json.dumps({n:m.version(n) for n in ['openpyxl','python-pptx','defusedxml']}))")
    record = {"schema":"an-log-report-acceptance-v1", "platform":sys.platform,
              "host_revision":a.host_revision, "host_binary_sha256":host.identity,
              "dependencies":versions, "model_calls":0, "gpu_required":False,
              "edit_method":"operator fixture simulates a saved edit; no human GUI interaction claimed",
              "xlsx":{"original":first,"edited":second,"later":third}, "pptx":ppt2,
              "selected_input_revision":v2, "editable_slides":content["slides"], "native_tables":content["tables"],
              "assertions":["standalone domain kit has no host or Loom imports", "actual binary outputs accepted only with explicit declaration",
                            "saved edit creates immutable child revision", "stale edit digest rejected", "later edit does not redirect consumer",
                            "substituted pinned snapshot rejected before execution", "downstream provenance binds selected revision",
                            "edited note reaches editable PPT", "active and cancelled outputs refused", "late output refused",
                            "termination confirmed after cancellation", "formal artifact survives scratch reclamation"]}
    save(out / "acceptance.json", record)
    print("PASS: editable documents, saved revisions, pinned consumption, binary acceptance and cancellation")


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--host", type=Path, required=True)
    p.add_argument("--host-revision", required=True)
    p.add_argument("--python", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--loom", type=Path)
    main(p.parse_args())
