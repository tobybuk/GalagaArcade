# Galaga — Rust source reconstruction

This is a source-level Rust/WebAssembly browser port of **Galaga (Namco rev. B)** from the supplied `galaga.zip`.

Startup, coin/start flow, attract/training/demo, all-stage construction, challenge completion, combat/capture/rescue, two-player handover, results, initials and game recycle are translated. The original sound sequencer, 54xx explosion firmware/circuit, 51xx input firmware, DIP decoder, service screen, PROM wavetable output and 05xx starfield are connected. There is no CPU emulation or replacement gameplay.

This is a functioning game port with original-ROM fidelity checks, **not a claim of 100% hardware equivalence**. Exact instruction/scanline interleaving, refresh/controller phase and original-recording analogue sound equivalence remain unverified. Browser adaptations and these limits are recorded in [PORTING.md](PORTING.md).

## Open the preview

Open **`Galaga preview.html`** in a browser, or double-click **`Run preview.command`** on macOS. The HTML embeds its compiled Rust module and assets and needs no server or installation.

The machine cold-starts automatically through its RAM/ROM result, configuration, sound-check and crosshatch screens. Press **5** or **Add coin**, then **1** for one player or **2** for two players. Arrow keys move, space fires, **P** pauses and **F** enters full screen. **Run/Resume**, **Reset** and **One frame** are browser playback controls. Touch movement/fire controls appear on coarse-pointer devices. Sound starts after a browser interaction. Losing browser focus releases held controls but does not pause board time.

**Cabinet settings** exposes the original DIP-switch choices, including free play, fighters, bonus schedule, difficulty, attract sound, cabinet, freeze and rack test. Galaga did not store these options in CMOS; they were physical switches. The browser remembers their positions in local storage and applies them through the original configuration decoder after a cold reset.

The production browser enters the original cold-start success route before main:02D3, proceeds through its own attract and credit routines, and returns to those routines after game over. `Game::default` is retained only as a native prepared-entry verification fixture; production uses `Game::cold_boot_with_switches`.

## What makes this a port

- The original identified flight tables, formation tables, object ordering and graphics/palette data come from the supplied ROM set.
- The original flight-vector arithmetic and control flow are expressed as Rust functions with explicit 8-bit/16-bit wrapping.
- Formation expansion uses the original rotating bitmaps. Flight paths retain the original integer algorithm, including the shift/add multiply and 17-pass division helpers.
- There is no Z80 emulator, opcode decoder, program counter or program-ROM execution in this project.
- The original main/sub sprite-bank copy routines and tile RAM now feed the renderer. JavaScript handles browser input, playback controls, timing and presenting the Rust framebuffer. It contains no enemy, formation, weapon or movement rules.
- The browser clock uses the board's 16.5 ms video period (approximately 60.606 Hz). An elapsed-time scheduler continues board periods across focus loss and catches up if a background tab is throttled.

Source mapping and remaining work are recorded in [PORTING.md](PORTING.md). Evidence and verification limits are in [evidence/README.md](evidence/README.md).

## Build and test

The Rust crate has **no third-party Rust dependencies**. With Rust installed:

```sh
rustup target add wasm32-unknown-unknown
python3 scripts/build.py
cargo test --release
```

`build.py` creates `dist/galaga.wasm` and the standalone `Galaga preview.html`. The browser module uses a small direct WebAssembly ABI, so no JavaScript bundler or wasm-bindgen CLI is required.

For development with a local web server:

```sh
python3 scripts/preview.py
```

To regenerate assets and identified data tables from the original archive:

```sh
python3 scripts/import_roms.py /path/to/galaga.zip
```

The importer validates the sizes and CRCs of all 16 files. It copies only graphics/palette assets and explicitly identified game-data tables into the project. Complete CPU ROMs and custom-controller firmware are not embedded in the runtime. One explicitly identified page includes instruction encodings that the original randomizer samples as data; Rust never executes those bytes.

During the initial conversion, the Rust toolchain was installed under the chat workspace's `work/toolchain/`, without changing the user's shell settings. The build helper can find that local installation. Build intermediates are kept under `work/build/` in this workspace.

## Current verification

- All 16 supplied ROMs validated; 58 identified data tables extracted.
- 59 assembly regions (16,473 bytes) matched the supplied ROMs. This includes five corrected game routines in `evidence/revision-b.s` plus the corrected sound sequencer/data in `evidence/revision-b-sound.s`. Both complete 1,024-byte controller listings also reassemble exactly.
- The known opening trajectory matches the independent C reference for every completed frame before the original sortie transition (896 frames without input).
- Four isolated dive paths match the independent C aid for 1,024 frames each, including motion state, arithmetic, path pointers and sprite output. Capture, clone and FE handlers missing in that aid are excluded.
- 655,360 common rocket-hit cases match the pinned, unmodified C reference. Its known mistakes are excluded from that golden and covered by separate assembly-based tests.
- Arithmetic matches the reference over 16,777,216 division cases, 65,536 home-angle cases and 65,536 coordinate-step cases.
- Tests also cover duplicate-hit suppression, moving/stationary awards, boss score attributes, explosion state order, score carries, high-score writes, reserve awards, task-table stepping and sprite-copy halves.

The standalone WebAssembly test runs with:

```sh
node tests/browser-module.mjs
```

All 96 native tests pass, covering the cold-start/configuration/crosshatch route, all 1,020 nonzero stage/rank wave combinations, challenge progression, initials, all fifteen attract states, both players through six fighters and recycle, controller filters/coinage, service/editor transitions and sound/circuit checks. A separately compiled hardware renderer matches the framebuffer in 1,024 cases. A separate C sound reference matches 17,408 NMI updates for common state, frequency and waveform output; its incorrect volume code is excluded and checked separately against the supplied assembly.

The actual production Wasm passes startup/coins, service, both players, recycle, combat, filtered input, PCM and framebuffer checks. An automated player using ordinary controls reaches stage six, clears challenge three, earns a bonus fighter, enters RUS in the high-score table and returns to attract. A separate ordinary-input run captures, rescues and joins a second fighter. The player never changes lives, stages or RAM. Reproduce with `node tests/browser-play.mjs`, `node tests/browser-play.mjs 4 --capture` and `node tests/browser-play.mjs 4 --initials`. Current browser visual evidence is `evidence/browser-current.jpg`.

These checks do not prove every possible play-through or cycle-level equivalence. The C reference remains a secondary aid; the supplied ROM instructions and data remain authoritative.
