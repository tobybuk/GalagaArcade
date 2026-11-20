#!/usr/bin/env python3
"""Regenerate independent fixtures from the pinned C reference (test-only).

Usage: python3 scripts/regenerate_reference.py /path/to/neiderm/arcade
Requires clang and a checkout at the recorded commit. Does not download files.
"""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
COMMIT = "248be0d9d80e04cf24ec5eaa1a2b50aa2f90795f"


def generate(checkout):
    revision = subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True).strip()
    if revision != COMMIT:
        raise SystemExit(f"Reference revision must be {COMMIT}; got {revision}.")
    source = checkout / "galag/c_code/src"
    files = ["game_ctrl", "gg1-2", "gg1-2_1700", "gg1-3", "gg1-4", "gg1-7", "task_man", "pe_string", "new_stage"]
    compiler = shutil.which("clang")
    if not compiler:
        raise SystemExit("clang is needed to regenerate the independent reference fixtures.")
    with tempfile.TemporaryDirectory(prefix="galaga-reference-") as temporary:
        binary = Path(temporary) / "reference"
        command = [compiler, "-O2", "-fsigned-char", "-DLSB_FIRST", "-I", str(source),
                   "-Wno-implicit-function-declaration", "-Wno-int-conversion", "-Wno-comment",
                   "-Wno-tautological-constant-out-of-range-compare", "-Wno-constant-logical-operand",
                   str(ROOT / "evidence/reference-harness.c")]
        command += [str(source / (name + ".c")) for name in files]
        command += ["-o", str(binary)]
        subprocess.run(command, check=True)
        entrance = subprocess.check_output([str(binary)])
        arithmetic = subprocess.check_output([str(binary), "arithmetic"])
    assert len(entrance) == 4096 * 8 and len(arithmetic) == 24
    evidence = ROOT / "evidence"
    (evidence / "reference-entrance.bin").write_bytes(entrance)
    (evidence / "reference-arithmetic.bin").write_bytes(arithmetic)
    metadata = {
        "reference": "https://github.com/neiderm/arcade", "commit": COMMIT,
        "compiler": subprocess.check_output([compiler, "--version"], text=True).splitlines()[0],
        "harness_sha256": hashlib.sha256((evidence / "reference-harness.c").read_bytes()).hexdigest(),
        "inputs": [{"file": p.name, "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
                   for p in sorted(source.glob("*.[ch]"))],
        "fixtures": [{"file": name, "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
                     for name, data in [("reference-entrance.bin", entrance), ("reference-arithmetic.bin", arithmetic)]],
        "entrance_frames": 4096,
        "division_cases": 65536 * 256,
        "home_angle_cases": 256 * 256,
        "coordinate_step_cases": 1024 * 16 * 4,
        "limitations": "Reference comparison covers the supported stage-one slice and arithmetic, not a complete hardware/gameplay comparison.",
    }
    (evidence / "reference-fixtures.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print("Regenerated the independent entrance and arithmetic fixtures.")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    generate(Path(sys.argv[1]).resolve())
