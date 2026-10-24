#!/usr/bin/env python3
"""Plots measured operator observations; never synthesize missing resource samples."""
import argparse
import html
import json
from pathlib import Path
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

COLORS = {"editor":"#2463A6", "host":"#D9822B", "kit":"#2A8B70", "observer":"#8B6BA8"}
NAMES = {"editor":"Editor", "host":"Host CLI", "kit":"Kit processes", "observer":"Observer"}
ACTIONS = [
    ("workbook.accepted","Workbook generated and accepted"),
    ("editor.launch.requested","Editor launch requested"),
    ("editor.window.ready","Editor window ready (GUI observed)"),
    ("editor.process.observed","Editor process detected (launch/restart)"),
    ("document.opened","Workbook opened in editor"),
    ("document.saved.v2","Native GUI saved revision v2 bytes"),
    ("revision.committed","Host committed saved revision"),
    ("document.saved.v3","Native GUI saved later v3 bytes"),
    ("document.closed","Document closed (GUI observed)"),
    ("editor.process.absent","Editor process exit observed"),
    ("downstream.accepted","PPT accepted from pinned v2"),
    ("workspace.reclaimed","Scratch reclaimed; artifacts retained")]


def records(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def run(root, output):
    events = sorted(records(root / "events.jsonl"), key=lambda x:x["t"])
    samples = records(root / "samples.jsonl")
    acceptance = json.loads((root / "acceptance.json").read_text())
    stopped = [e for e in events if e["name"] == "measurement.stopped"]
    if not stopped or stopped[-1].get("reason") != "requested":
        raise ValueError("Measurement expired or termination was not confirmed")
    if acceptance.get("schema") != "an-native-office-acceptance-v1":
        raise ValueError("Native acceptance is required before a success report")
    first = lambda name: next((e["t"] for e in events if e["name"] == name), None)
    roles = {role: [s for s in samples if s["role"] == role] for role in COLORS}
    if not roles["editor"] or max(s["alive"] for s in roles["editor"]) == 0:
        raise ValueError("No actual selected editor process was observed")
    if roles["editor"][-1]["alive"] != 0 or first("editor.process.absent") is None:
        raise ValueError("Editor unload not confirmed; do not label retained processes as unloaded")
    output.mkdir(parents=True)
    labels = dict(ACTIONS)
    selected = [e for e in events if e["name"] in labels]
    plt.rcParams.update({"svg.fonttype":"none", "font.family":"DejaVu Sans", "font.size":11, "axes.spines.top":False, "axes.spines.right":False})
    fig, ax = plt.subplots(figsize=(11, max(4.5,len(selected)*.42)))
    for i,e in enumerate(selected):
        y=len(selected)-i
        ax.scatter([0],[y],s=48,color=COLORS["editor"])
        ax.text(.04,y,labels[e["name"]]+(" ("+e["label"]+")" if "label" in e else ""),va="center")
        ax.text(.99,y,f'{e["t"]:.2f} s',ha="right",va="center",family="monospace")
    ax.plot([0,0],[1,len(selected)],color="#BBC6D1",zorder=0)
    ax.set(xlim=(-.02,1.03),ylim=(.4,len(selected)+.8),title="Observed action sequence — elapsed seconds, ordered steps")
    ax.axis("off");fig.tight_layout();fig.savefig(output/"actions.png",dpi=150);fig.savefig(output/"actions.svg");plt.close(fig)
    fig, axes=plt.subplots(6,1,figsize=(12,8),sharex=True)
    for ax,role in zip(axes[:3],["editor","host","kit"]):
        rows=roles[role]
        ax.step([r["t"] for r in rows],[r["alive"] for r in rows],where="post",color=COLORS[role])
        ax.set_ylabel(NAMES[role]+"\ncount");ax.grid(axis="x",alpha=.18);ax.set_ylim(bottom=-.1)
    end=max(e["t"] for e in events)
    for ax,label,start,stop,color in [
        (axes[3],"Document\nopen",first("document.opened"),first("document.closed"),COLORS["editor"]),
        (axes[4],"Scratch\nretained",first("workbook.begin"),first("workspace.reclaimed"),COLORS["host"]),
        (axes[5],"Artifacts\nretained",first("workbook.accepted"),end,COLORS["kit"])]:
        if start is None or stop is None: raise ValueError("Missing actual lifecycle observation")
        ax.broken_barh([(start,stop-start)],(.2,.6),facecolors=color)
        ax.set(ylim=(0,1),yticks=[]);ax.set_ylabel(label);ax.grid(axis="x",alpha=.18)
    axes[0].set_title("Lifecycle monitor — process samples, GUI observations and host settlement")
    axes[-1].set_xlabel("Elapsed time (s)")
    fig.tight_layout();fig.savefig(output/"lifecycle.png",dpi=150);fig.savefig(output/"lifecycle.svg");plt.close(fig)
    fig,axes=plt.subplots(2,1,figsize=(12,7),sharex=True)
    stats={}
    for role,rows in roles.items():
        x=[r["t"] for r in rows]
        rss=[float("nan") if r["rss_bytes"] is None else r["rss_bytes"]/1048576 for r in rows]
        cpu=[float("nan")]
        for prev,row in zip(rows,rows[1:]):
            cpu.append(float("nan") if row["unknown"] or prev["unknown"] else max(0,row["cpu_seconds"]-prev["cpu_seconds"])/(row["t"]-prev["t"])*100)
        axes[0].plot(x,rss,label=NAMES[role],color=COLORS[role],lw=1.2)
        axes[1].plot(x,cpu,label=NAMES[role],color=COLORS[role],lw=1.1)
        known=[v for v in rss if v==v]
        stats[role]={"sampled_peak_rss_mib":max(known) if known else None,
                     "observed_cpu_seconds":rows[-1]["cpu_seconds"],"final_alive":rows[-1]["alive"]}
    axes[0].set(ylabel="Summed process RSS (MiB)",title="Load/unload resource observations — one native desktop run")
    axes[0].legend(ncol=4,loc="upper right")
    axes[1].set(ylabel="Sampled CPU (core %, 100 = one core)",xlabel="Elapsed time (s)")
    for ax in axes:ax.grid(alpha=.18);ax.set_ylim(bottom=0)
    fig.tight_layout();fig.savefig(output/"resources.png",dpi=150);fig.savefig(output/"resources.svg");plt.close(fig)
    phase_names=[("App launch","editor.launch.requested","editor.window.ready"),
                 ("Document open","document.open.requested","document.opened"),
                 ("Save v2","document.save.v2.requested","document.saved.v2"),
                 ("Save v3","document.save.v3.requested","document.saved.v3"),
                 ("Document close","document.close.requested","document.closed"),
                 ("App quit","editor.quit.requested","editor.process.absent")]
    phases=[]
    for label,a,b in phase_names:
        start = first(a)
        stop = next((e["t"] for e in events if e["name"] == b and start is not None and e["t"] >= start), None)
        if start is None or stop is None or stop<start:continue
        editor=roles["editor"]
        before=[r for r in editor if r["t"]<=start]
        during=[r for r in editor if start<=r["t"]<=stop]
        after=[r for r in editor if r["t"]<=stop]
        known=[r["rss_bytes"] for r in during if r["rss_bytes"] is not None]
        phases.append(dict(phase=label,observed_wall_seconds=stop-start,
                           sampled_peak_editor_rss_mib=max(known)/1048576 if known else None,
                           observed_editor_cpu_seconds=max(0,after[-1]["cpu_seconds"]-(before[-1]["cpu_seconds"] if before else 0)) if after else None))
    limits=["Process samples are observations, not lifecycle authority.",
            "Summed RSS can double-count shared pages; it is not total physical memory.",
            "Short-lived workers and final CPU bursts may be missed at 250 ms sampling.",
            "Access-denied measurements remain unknown rather than zero.",
            "GUI action windows include operator/automation waits; this is not an isolated application benchmark.",
            "No VM/spool hot-load manager or new backend is measured; GPU and system-wide resource use are not measured."]
    notes_path = root / "capture-notes.json"
    notes = json.loads(notes_path.read_text()) if notes_path.exists() else []
    limits.extend(notes)
    summary=dict(schema="an-native-office-observations-v1",sample_interval_seconds=.25,roles=stats,phases=phases,limits=limits)
    (output/"observations.json").write_text(json.dumps(summary,indent=2)+"\n")
    (output/"events.json").write_text(json.dumps(events,indent=2)+"\n")
    # These records deliberately contain no PIDs, argv, environment or machine paths.
    (output/"samples.jsonl").write_text("\n".join(json.dumps(s) for s in samples)+"\n")
    rows="".join("<tr><td>"+html.escape(p["phase"])+"</td><td>"+f'{p["observed_wall_seconds"]:.2f}'+"</td><td>"+(f'{p["sampled_peak_editor_rss_mib"]:.1f}' if p["sampled_peak_editor_rss_mib"] is not None else "unknown")+"</td></tr>" for p in phases)
    page='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Native office roundtrip</title><style>body{font:16px system-ui;max-width:1100px;margin:32px auto;padding:0 20px;color:#183447}img{width:100%;height:auto}table{border-collapse:collapse;width:100%}th,td{text-align:left;padding:8px;border-bottom:1px solid #ccd6df}small{color:#526675}section{margin-top:36px}</style><h1>Native office roundtrip</h1><p>Actual GUI saves → immutable revisions → downstream fixed to v2.</p><section><h2>Action sequence</h2><img src="actions.png" alt="Ordered observed UI and host actions"></section><section><h2>Load/unload monitor</h2><img src="lifecycle.png" alt="Measured process counts and observed document, scratch and artifact states"></section><section><h2>Resource consumption</h2><img src="resources.png" alt="Measured process RSS and sampled CPU over elapsed time"></section><table><tr><th>Observed operation window</th><th>Wall seconds</th><th>Sampled peak editor RSS MiB</th></tr>'''+rows+"</table><h2>Measurement limits</h2><ul>"+"".join("<li>"+html.escape(s)+"</li>" for s in limits)+"</ul></html>"
    (output/"index.html").write_text(page)
    for svg in output.glob("*.svg"):
        svg.write_text("\n".join(line.rstrip() for line in svg.read_text().splitlines()) + "\n")
    print(json.dumps(summary))


if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("--root",type=Path,required=True);p.add_argument("--output",type=Path,required=True)
    a=p.parse_args();run(a.root,a.output)
