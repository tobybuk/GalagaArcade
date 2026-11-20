#!/usr/bin/env python3
"""Build the Rust browser module. No JavaScript package dependencies required."""
import base64
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def cargo_environment():
    env = os.environ.copy()
    cargo = shutil.which("cargo")
    if not cargo:
        # Workspace-local toolchain installed during the initial conversion.
        toolchain = ROOT.parents[1] / "work" / "toolchain"
        cargo_path = toolchain / "cargo" / "bin" / "cargo"
        if not cargo_path.exists():
            raise SystemExit("Install Rust and its wasm32-unknown-unknown target, then rerun this script.")
        cargo = str(cargo_path)
        env["RUSTUP_HOME"] = str(toolchain / "rustup")
        env["CARGO_HOME"] = str(toolchain / "cargo")
    return cargo, env


def build():
    cargo, env = cargo_environment()
    target = Path(env.get("CARGO_TARGET_DIR", ROOT / "target"))
    workspace = ROOT.parents[1]
    if "CARGO_TARGET_DIR" not in env and ROOT.parent.name == "outputs" and (workspace / "work").is_dir():
        target = workspace / "work/build/galaga-rust"
        env["CARGO_TARGET_DIR"] = str(target)
    subprocess.run([cargo, "build", "--lib", "--release", "--target", "wasm32-unknown-unknown"],
                   cwd=ROOT, env=env, check=True)
    source = target / "wasm32-unknown-unknown/release/galaga_reconstructed.wasm"
    shutil.copyfile(source, ROOT / "dist/galaga.wasm")
    # Also provide a single file that opens directly in a browser, offline.
    script = (ROOT / "dist/browser.js").read_text()
    script = script.replace(
        "const response = await fetch('./galaga.wasm');\n"
        "  if (!response.ok) throw new Error(`Rust build unavailable (${response.status})`);\n"
        "  const result = await WebAssembly.instantiate(await response.arrayBuffer(), {});",
        "const bytes = Uint8Array.from(atob(\"" + base64.b64encode(source.read_bytes()).decode()
        + "\"), c => c.charCodeAt(0));\n"
        "  const result = await WebAssembly.instantiate(bytes, {});"
    )
    assert "fetch('./galaga.wasm')" not in script, "Standalone Wasm loader replacement failed"
    page = (ROOT / "dist/index.html").read_text().replace(
        '<script type="module" src="./browser.js"></script>',
        '<script type="module">\n' + script + '\n</script>')
    (ROOT / "Galaga preview.html").write_text(page)
    print(f"Browser build ready: {source.stat().st_size:,} bytes")


if __name__ == "__main__":
    build()
