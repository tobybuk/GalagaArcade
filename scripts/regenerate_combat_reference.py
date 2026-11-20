#!/usr/bin/env python3
"""Generate common-hit fixtures from the pinned, unmodified C reference."""
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
    with tempfile.TemporaryDirectory(prefix="galaga-combat-reference-") as temporary:
        binary = Path(temporary) / "combat-reference"
        command = [compiler, "-O2", "-fsigned-char", "-DLSB_FIRST", "-I", str(source),
                   "-Wno-implicit-function-declaration", "-Wno-int-conversion", "-Wno-comment",
                   "-Wno-tautological-constant-out-of-range-compare", "-Wno-constant-logical-operand",
                   str(ROOT / "evidence/combat-oracle.c")]
        command += [str(source / (name + ".c")) for name in files] + ["-o", str(binary)]
        subprocess.run(command, check=True)
        data = subprocess.check_output([str(binary)])
    assert len(data) == 40
    (ROOT / "evidence/reference-combat.bin").write_bytes(data)
    report = {"reference_commit": COMMIT, "cases": 5 * 256 * 512,
              "scenarios": ["stationary moth", "stationary bee", "flying moth", "flying bee", "green boss"],
              "fixture_sha256": hashlib.sha256(data).hexdigest(),
              "harness_sha256": hashlib.sha256((ROOT / "evidence/combat-oracle.c").read_bytes()).hexdigest(),
              "reference_modified": False,
              "limitations": "Only common paths where the C reference agrees with verified assembly; known discrepancies have separate assembly-based tests. Counter comparison includes the low byte only."}
    (ROOT / "evidence/reference-combat.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"Generated independent common-hit fixtures for {report['cases']:,} cases.")
