"""Acceptance-only pause after a real binary is written, not a business feature."""
import argparse
from pathlib import Path
import sys
import time
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "kit"))
from slides import create
parser = argparse.ArgumentParser()
parser.add_argument("stage", choices=["slides"])
parser.add_argument("--input", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
args = parser.parse_args()
create(args.input, args.output)
Path("ready").write_text("binary candidate exists; process still active")
time.sleep(30)
