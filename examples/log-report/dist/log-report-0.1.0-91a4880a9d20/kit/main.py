#!/usr/bin/env python3
"""Standalone CLI kit. Runtime authority and publication are outside this program."""
import argparse
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=["workbook", "slides"])
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("Output must be new")
    if args.stage == "workbook":
        from workbook import create
    else:
        from slides import create
    create(args.input, args.output)


if __name__ == "__main__":
    main()
