# Verification evidence

`rom-map.json` records the supplied archive hash, checksums for all 16 ROMs, and the exact address/length/hash of each of the 58 extracted data tables. Graphics and palette assets are decoded from the supplied set. Complete CPU ROMs are not bundled into the Rust module.

`reference-assembly.json` records 59 assembled regions (16,473 bytes) matched against the supplied ROMs, with every permitted absolute-address relocation listed. Five revision-specific routines are reconstructed in `revision-b.s`: reserve drawing, bomb position/release, bomb creation/control, delayed captured-fighter transfer and stage tokens. `revision-b-sound.s` corrects the supplied sound revision, matching its complete sequencer and data at 00BB..0F5E. A byte match establishes the research reference; it is not a whole-game Rust execution oracle.

`51xx-firmware.asm` and `54xx-firmware.asm` statically map the complete supplied 1,024-byte controller images. `reference-51xx.json` and `reference-54xx.json` record exact reassembly checks. Runtime functions reconstruct the original command/filter/coin/start and noise/envelope logic directly; neither firmware image nor a CPU execution engine is embedded. MCU/06xx phase and mid-section interrupt ordering remain validation boundaries.

`video-oracle.c` is a separately compiled hardware-reference renderer, derived from the cited board-layout documentation and supplied graphics/PROMs. `reference-video.bin` contains 1,024 full-frame RGBA hashes; `reference-video.json` records provenance/hashes. Cases cover coordinate boundaries, wrapping double-height sprites, control bits, size/flips, all graphics/colour selections, tile addressing/priority and star colours. This checks the display circuit model, not original gameplay or writes within a scanline.

`reference-entrance.bin` retains 4,096 little-endian 64-bit FNV-1a hashes, one per frame, from the first research fixture. Each covers all 64 objects' state, sprite code, colour, X register, Y low byte, Y high bit and sprite flags. The fixture comes from an independently compiled copy of the pinned upstream C translation using `reference-harness.c`, with a reduced task set and prepared stage-one entry. It was not generated from Rust. **The prepared-entry reference test compares only the 896 completed frames before the original sortie transition at main:1B6B.** Later fixture frames use a reduced task set and are not coverage of the complete production flow, which executes the original startup and full task set.

`reference-arithmetic.bin` contains three 64-bit hashes of independently computed results:

1. Original 16/8-bit division for every numerator `0..65535` and denominator `0..255`, including the original zero-divisor result.
2. Original home-angle routine for every X/Y byte pair against target `(0x80,0x40)`.
3. Original coordinate routine for every 10-bit angle, every displacement nibble and four boundary/wraparound seeds.

`reference-fixtures.json` records the reference commit, compiler, harness/input hashes and these fixture hashes.

`reference-combat.bin` contains five 64-bit hashes from `combat-oracle.c`, compiled with the pinned, unmodified upstream C translation. It exercises every X byte and 9-bit Y coordinate for stationary moth/bee, flying moth/bee and green-boss hits: 655,360 cases. `reference-combat.json` records hashes and limits. Only common paths where the C aid agrees with verified assembly enter this golden. Known differences (pending-notification suppression, flying blue-boss attributes and byte-only countdown decrement) have separate assembly-derived tests. The countdown comparison in this C golden deliberately includes only the low byte.

`reference-flight.bin` contains 4,096 per-frame hashes from `flight-oracle.c`: 1,024 frames each for a bee, moth and left/right boss sortie. It checks sprite output and motion coordinates/angle, path, duration, rotation, active flag, home targets, displacements and bomb timers. Formation is fixed and bombing/object-state tasks are disabled to isolate the common flight routines. FE, clone and capture handlers missing in the C aid are excluded. Only the active flag is compared at termination, because C incorrectly clears additional flag bits. `reference-flight.json` records hashes and these limits.

`reference-sound.bin` contains 17,408 per-NMI hashes from `sound-oracle.c`, compiled with the pinned unmodified C sound source. It covers 17 common isolated requests: status, data indexes, requests/active flags and masked frequency/wave registers. The C aid has proven volume errors (stale accumulator in CPL, missing SUB 10 before NEG, unaliased voice byte at completion) and omits request 12h. Its three volume registers and missing/revision-specific handlers are excluded from that golden. `reference-sound.json` records these exact limits. Rust volume checks instead follow the supplied instructions for all 256 counters of three envelope kinds.

The C harnesses are test infrastructure only and are never linked into the Rust game or WebAssembly module. They call translated game functions, not a CPU emulator. There is no opcode execution engine in the deliverable.

Additional assembly-derived tests cover stage/rank wave tables, challenge awards, icon waits, initials, attract, BCD credits/totals, player exchange/recycle, sound/54xx sequencing, controller history/coin/start rules, DIP selections, service menus, hidden message, crosshatch and full sprite controls. Independent analogue/audio coverage remains to be extended. Current local HTTP browser visual evidence is `browser-current.jpg`; historical `../preview.jpg` belongs to an earlier build.

The production Wasm ordinary-input player reaches stage six, including challenge three, a bonus fighter and eventual game recycle. A separate ordinary-input run completes beam capture, replacement launch, rescue and joined fighters; another enters RUS in the high-score table and returns to attract. The player uses read-only object inspection and physical keys, never RAM/life/stage modifications.

Other tests exercise ROM-derived task stepping and split sprite copies, collision bounds, explosion state order, score digits/carries, high-score changes, reserve awards and the supplied revision's reserve/bomb routines. Those tests are based on the assembly; they must not be represented as an independent whole-game oracle.

These checks do not prove complete game equivalence, original power-on phase alignment, exact scanline presentation, full processor interleaving, input-controller timing, or complete original-revision visual/audio behaviour. The supplied ROM bytes remain authoritative over the C aid.

## Reproduce

Check out [neiderm/arcade](https://github.com/neiderm/arcade) at `248be0d9d80e04cf24ec5eaa1a2b50aa2f90795f`. Then, with clang available:

```sh
python3 scripts/regenerate_reference.py /path/to/arcade
python3 scripts/regenerate_combat_reference.py /path/to/arcade
python3 scripts/regenerate_flight_reference.py /path/to/arcade
python3 scripts/regenerate_video_reference.py
python3 scripts/regenerate_sound_reference.py /path/to/arcade
cargo test --release
python3 scripts/build.py
node tests/browser-module.mjs
node tests/browser-play.mjs
node tests/browser-play.mjs 4 --capture
node tests/browser-play.mjs 4 --initials
```

To repeat the byte comparisons, build the reference's `galag/galagao_ASxxx/rom0` assembly with ASxxxx 5.10, as described in that repository. Assemble and link `revision-b.s` with the same assembler, retaining its `.i86` output. Assemble/link `revision-b-sound.s` separately as well. No emulator is required. Then run:

```sh
python3 scripts/verify_assembly.py /path/to/galaga.zip /path/to/arcade/galag/galagao_ASxxx/rom0 --revision-b /path/to/revision-b.i86 --revision-b-sound /path/to/revision-b-sound.i86
python3 scripts/verify_54xx_listing.py /path/to/galaga.zip 51xx
python3 scripts/verify_54xx_listing.py /path/to/galaga.zip 54xx
```

The initial reference checkout, assembler source/build and compiler are retained in the chat workspace's `work/` directory. They are not part of the deliverable archive.
