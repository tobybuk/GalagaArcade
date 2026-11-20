#!/usr/bin/env python3
"""Generate common-sortie fixtures from the pinned, unmodified C reference."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
COMMIT = "248be0d9d80e04cf24ec5eaa1a2b50aa2f90795f"

if __name__ == "__main__":
    checkout = Path(sys.argv[1]).resolve()
    revision = subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True).strip()
    assert revision == COMMIT
    source = checkout / "galag/c_code/src"
    files = ["game_ctrl", "gg1-2", "gg1-2_1700", "gg1-3", "gg1-4", "gg1-7", "task_man", "pe_string", "new_stage"]
    compiler = shutil.which("clang")
    with tempfile.TemporaryDirectory(prefix="galaga-flight-reference-") as temporary:
        binary = Path(temporary) / "flight-reference"
        command = [compiler, "-O2", "-fsigned-char", "-DLSB_FIRST", "-I", str(source),
                   "-Wno-implicit-function-declaration", "-Wno-int-conversion", "-Wno-comment",
                   "-Wno-tautological-constant-out-of-range-compare", "-Wno-constant-logical-operand",
                   str(ROOT / "evidence/flight-oracle.c")]
        command += [str(source / (name + ".c")) for name in files] + ["-o", str(binary)]
        subprocess.run(command, check=True)
        data = subprocess.check_output([str(binary)])
    assert len(data) == 4 * 1024 * 8
    (ROOT / "evidence/reference-flight.bin").write_bytes(data)
    report = {"reference_commit": COMMIT, "frames_per_path": 1024,
              "scenarios": ["bee", "moth", "boss left", "boss right"],
              "fixture_sha256": hashlib.sha256(data).hexdigest(),
              "harness_sha256": hashlib.sha256((ROOT / "evidence/flight-oracle.c").read_bytes()).hexdigest(),
              "reference_modified": False,
              "limitations": "Isolated fixed-formation sorties, no bombing or object-state task. Capture, clone and FE paths are not covered. Only the active bit of motion.flags is compared because C incorrectly clears other bits at FF termination."}
    (ROOT / "evidence/reference-flight.json").write_text(json.dumps(report, indent=2) + "\n")
    print("Generated 4,096 independent common-sortie frames.")
