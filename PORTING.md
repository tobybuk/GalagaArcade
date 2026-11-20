# Reconstruction ledger

## Authority and reference

The user-supplied Namco rev. B ROM bytes are the authority. The annotated assembly and older C translation in [neiderm/arcade](https://github.com/neiderm/arcade/tree/248be0d9d80e04cf24ec5eaa1a2b50aa2f90795f) are research aids, pinned at commit `248be0d9d80e04cf24ec5eaa1a2b50aa2f90795f`.

The reference does **not** have uniformly identical layout to this ROM revision. It was rebuilt with ASxxxx 5.10, then selected routines were compared byte for byte with the supplied ROMs. Known absolute-address changes are recorded individually in `evidence/reference-assembly.json`. Unexplained differences are not silently masked.

## Translated routines

Addresses below refer to the supplied ROM set. Older annotation names are retained where helpful.

| Rust area | Original routine/data | Current scope |
|---|---|---|
| `rom_data.rs` | 58 explicitly identified ROM-data tables | Exact extraction with checksums, offsets and lengths |
| `formation.rs` | main `0x12D5` (`c_12C3`) | Original ten column/six row setup, offset argument and flipped screen |
| `formation.rs` | main `0x2A90` | Original entrance-time horizontal movement and transition to pulsing |
| `formation.rs` | main `0x1DEC` (`f_1DE6`) | Bitmap-driven expansion/contraction, normal screen |
| `formation.rs` | main `0x2C00` | All 256 stage values, including original adjacent-data reads on counter wrap |
| `stage.rs` | main `0x2896`, `0x28E9` | Normal/challenge classes, hit-eight awards and bomb attributes |
| `stage.rs` | main `0x25A2`, `0x1000` | All stage/rank wave tables, original transient rejection/merge, sampled refresh input |
| `stage.rs` | main `0x2916`, `0x1A86` | All entrance vectors, transient remap, challenge timing, bonus-bee selection/launch |
| `lib.rs` | main `0x23DD` | Entry, home-return, standby and missile object states |
| `tasks.rs`, `lib.rs` | sub `0x0513`, `0x08D3` | Original task executive, counters, delayed flying count and movement traversal |
| `motion.rs` | sub `0x090E`, `0x0AA0`, `0x0C05` | Original entrance token handling, home targeting and arrival |
| `motion.rs` | sub `0x0C46`, `0x0CA7`, `0x0D03` | Original sprite orientation, fixed-point motion, coordinate output |
| `motion.rs` | sub `0x0E5F`, `0x0EAE` | Home-angle computation and integer division |
| `weapons.rs` | main `0x1F9C` (`c_1F92`) | Normal single-fighter movement and original limits |
| `weapons.rs` | main `0x1F19` (`c_1F0F`) | Two rocket slots and orientation attributes |
| `weapons.rs` | sub `0x06F5` | Rocket movement, original exclusive upper limit and hit-dispatch call |
| `tasks.rs` | main `0x0237`, imported main/sub task vectors | Saved enable-byte steps, original dispatch order, explicit untranslated-call stops |
| `tasks.rs` | main `0x0828`, sub `0x05BF` | Original separate sprite-copy halves and their position in each executive |
| `tasks.rs` | main `0x0857`, `0x1DD8`, `0x1EAA` | Bomber-parameter tables, game timers and rev. B bomb position/release behaviour |
| `combat.rs` | sub `0x076A`..`0x08D2` | Rocket rectangle, hit count, boss colour/score attributes, original notification and sound-request writes |
| `combat.rs` | main `0x1DB9`, `0x24B2`, `0x2535`, `0x243C` | Hit notification, enemy explosion, bonus bitmap and rogue-fighter explosion states |
| `score.rs` | main `0x0728`, `0x07D8` | Original score digits/carries in tile RAM, high-score comparison, bonus reserve/sound/service writes |
| `score.rs` | main `0x137E`..`0x13B2` | Supplied rev. B reserve-drawing routine, eight slots and stage-symbol protection |
| `score.rs` | main `0x0935` | Original player-label blink writes |
| `attacks.rs` | main `0x1B6B..0x1D37`, `0x1079..0x110B` | Sortie timers, boss/escort/captured-fighter queues, capture toggle, original launch helpers |
| `capture.rs` | main `0x2000..0x23A0`, supplied rev. B `0x19B2..0x1A85` | Spin, beam tile/colour writes, approach/capture, delayed object transfer, rescue, joined fighters |
| `executive.rs` | main `0x049E..0x0649`, `0x1331..0x137D` | Restart waits, reserve decrement, player handover, respawn, results and challenge completion |
| `text.rs` | main `0x13B3..0x13F0`, `0x0A53..0x0B0E`, `0x104E..0x1078` | Original strings, decimal output, scaled hit ratio and arithmetic helpers |
| `video.rs` | Graphics, lookup PROMs and board tilemap mapper | Original 2-bit graphics, buffered sprites, tile/colour RAM and portrait rotation |

| `stage.rs` | main `0x0185..0x0236`, `0x117F..0x1241` | Stage splash/environment and rev. B timed icons/reserve protection |
| `highscore.rs` | main `0x3000..0x336B` | Ranking, five-score/name table, initials, original text formats and timeouts |
| `machine.rs`, `attract.rs` | main `0x02D3..0x045D`, `0x0977..0x0A52`, `0x1700..0x19B1` | Startup after self-test, credits, opening, attract/training/demo, recycle and BCD totals |
| `players.rs` | main `0x110C..0x117E`, `0x058E..0x0603`, `0x1D38..0x1D7B` | Player records, packed enemy/task exchange, formation retreat/return |
| `sound.rs` | sound `0x00BB..0x06D9` | Original priority/requests, note/envelope/duration routines and register writes |
| `starfield.rs`, `video.rs`, `sound.rs` | 05xx/WSG circuits | Hardware adaptation: original star generator and PROM wavetable output |
| `noise54.rs`, `noise_circuit.rs` | 54xx `0000..03FF`, board DAC/filter/mixer | Original noise/envelope/command functions and component-value circuit output |
| `input51.rs` | 51xx `0000..03FF` | Original commands, coin slots, filters, start/free play, panels, read cursor, lamps/meters/lockout |
| `configuration.rs` | main `37FE..3961`, `3A6B` | Original DIP wiring, cabinet/rank/lives/bonus/coinage decoding and display |
| `service.rs` | main `35BF..37F5`, `3962..3A6A` | Service controls, sound test, machine totals, hidden message, crosshatch and return |
| `tasks.rs` | sub `057C..05BD`, `0ECA..0FD9` | Reset/task defaults, unused-switch branch and optional external editor/hex helper |

The flight dispatcher translates every control-token case `EF..FF`, including clone/capture setup. Identified flight data also includes the supplied revision's `0FDA..0FFC` paths. Instruction encodings sampled by `c_1000` are treated solely as data; there is no opcode interpretation or complete CPU ROM in the runtime.

## Findings preserved in the translation

1. **Fighter step size:** the original code toggles a flag, giving successive movements of 1, 2, 1, 2 pixels. Some annotation prose describes this incorrectly as one initial pixel followed by constant two-pixel steps. Rust follows the instructions.
2. **Stage reduction boundary:** the `CP 0x1B / JR C` sequence reduces stages **at or above 27**. The older C version uses `> 27`, which would index beyond the intended final row at stage 27. Rust follows the assembly.
3. **Clone-attack threshold:** the assembly's OR/increment test produces zero on challenge stages and ten on normal stages from stage four onward. The C expression does not preserve this correctly. Rust follows the assembly.
4. **Home arrival:** the assembly clears the fractional coordinate bytes before snapping to the home coordinates. Rust preserves that step.
5. **Revision addresses:** for example, the fighter-control entry annotated `0x1F85` is at `0x1F8F` in the supplied set, while its helper is at `0x1F9C`. References are mapped before relying on addresses.

## Further assembly findings and corrections in v0.2.0

6. **Task stepping:** both original interrupt kernels save the enable/advance byte before calling the function. Rust now does this. The C main scheduler re-reads that byte after the call.
7. **Rocket upper limit:** `CP 0x9C / JR NC` removes a rocket at scaled Y 156, so physical Y 312 is already out of range. The C aid accepted 312.
8. **Pending-hit flags:** the original tests bit 7 of `state | notification`, excluding values such as `0x81`. The C rocket detector compared against `0x80`, which can let a second rocket hit an already fatally hit enemy.
9. **Flying-hit countdown:** the assembly uses `DEC (HL)`, affecting one byte. The C variable is declared as a word and its decrement borrows into the high byte. Rust retains the one-byte flow.
10. **Flying blue boss:** the assembly loads its two score/bitmap attribute bytes. Those loads are commented out in the C aid, leaving a motion-slot index used as points. Rust performs the original loads.
11. **Reserve icons:** the supplied rev. B routine has eight slots and protects existing tile codes `0x36..0x49`. The older annotation has a different five-slot loop. The corrected assembly is in `evidence/revision-b.s` and reassembles byte for byte.
12. **Bomb release:** rev. B adds a write of `0x80` to object state when the bomb code is invalid or its X register is zero. This is absent from the older annotation. The corrected routine also reassembles byte for byte.
13. **Formation carry/sign:** the original retains the ninth coordinate bit on carry/borrow and sign-extends formation-relative offsets. The earlier Rust slice inherited simplifications from the C aid; those are corrected.
14. **Sprite presentation:** the main routine copies the upper 64-byte half of each sprite bank; the sub routine copies the lower half. The C aid copied everything in the main task and left the sub task empty. Rust now preserves the two calls and their order.
15. **Arithmetic helper flow:** the first milestone replaced the original multiplication/division subroutines with equivalent native arithmetic. Rust now retains the original shift/add multiplication loop and 17-pass division loop as separate functions.

## Further reconstruction and revision findings in v0.3.0

16. **Bomb creation:** rev. B adds an explicit sprite-control write of `1` before copying the bomber's position. `revision-b.s` now includes this exact reassembly. The original numerator's low byte is the bomb register address; discarding it changes some horizontal rates.
17. **Fighter hit scan:** its common hit tail jumps into the rocket detector's next-object label. Rust retains that transition, including its subsequent hit-counter increments and the even impactor offset that clears fighter X. Bomb notification reads at `9268..9276` use the original unused/service-RAM aliases, retained as shared bytes.
18. **Captured-fighter transfer:** the supplied rev. B shows the captured text first, colours the active fighter red and delays the transfer to its formation object until the timer expires. It also copies both sprite-control bytes with LDD/LDI. The older annotation transfers immediately and omits one control byte. The corrected routine reassembles exactly.
19. **Background flow:** original busy-waits are retained as named Rust continuations that yield to the next video period. They keep the call/return sequence, timer tests and stores; an unported continuation remains explicit. No host-time life-loss or respawn timer has been introduced.
20. **C-reference boundaries:** the C aid omits FE, clone, capture and other handlers. Four common isolated sorties are checked for 4,096 frames; that result is not evidence for its missing handlers. Capture tests instead exercise ROM/assembly-derived transitions and the corrected revision source.

## Further reconstruction findings

21. **Stage builder:** `CP 17h` reduces stages at or above 23, subtracting four. The full sixteen-byte rejection/merge buffer and fixed-ID placement are retained. C leaves transient placement incomplete.
22. **Challenge classes:** original attributes use a two-byte index; original mask tests are bitwise. C's indexing and logical-AND differ. Rust follows the verified assembly.
23. **Stage-counter wrap:** `DEC 0` produces FF, and rotating twice then adding gives FE. The adjacent original ROM bytes are retained as data for the resulting reads.
24. **Rev. B stage icons:** clearing protects reserve tiles >=4A, and the routine tail-jumps to redraw reserves. Eight-frame icon waits remain tied to the original frame byte.
25. **Player exchange:** only 48 packed enemy bytes are suspended. The code stores resting-enemy presence/code/colour, then exchanges both 64-byte player records and 32-byte task tables. Coordinates are rebuilt through the original retreat/return calls.
26. **Rev. B sound:** ship-hit music skips later themes, capture beam changes voice zero's wave, challenge end writes volumes even when its request finishes, and sound-seven completion has additional exact clearing ranges. The corrected sound source reassembles to the supplied bytes.
27. **51xx filtering/transfer:** two inactive samples followed by two active samples qualify a press. Fire retains separate edge and held bits for both panels. The suppressed first 06xx NMI and final unused MCU read preserve the four-byte cursor over each three-byte transfer; main tasks use the preceding interrupt's input.
28. **51xx starts/arithmetic:** start one wins simultaneous starts; a new coin award inhibits starts for that sample. Starts stay disabled until the original command. Free play publishes the 99/98 debit before restoring A0. The nibble DAA carry quirk is retained.
29. **Full sprite control:** capture and sprite copies retain both control bytes. Motion/bombs modify only odd bit zero; firing tests odd bit two. The renderer retains odd bit one's coordinate contribution. A zero position byte does not disable a wrapped double-height sprite.
30. **Attract wait:** fighter setup resumes the same interrupt task after the sub board finishes, preserving preceding sprite calls and the remaining task order.
31. **Service revision:** FIGHTERS replaces SHIPS, shifting later text by three additional bytes. Explicit relocations are recorded in the assembly evidence. The hidden-message key sequence and bitmap drawing use original data/helpers.
32. **Sound C-reference errors:** its type-two envelope complements a stale accumulator, its decay omits SUB 10 before NEG, and its completion helper reads an unaliased voice byte. It also omits request 12h entirely. Rust follows supplied `061E..065E`, `06BC..06D9` and `0208..0215`. The independent golden compares common status/frequency/wave output; separate assembly-derived checks cover all three volume branches.

## Environment adaptations and remaining fidelity work

- Production browser entry is `Game::cold_boot_with_switches`: it follows the original successful cold-start display and waits through RAM/ROM OK, DIP configuration, sound check, crosshatch, controller setup and main:02D3. Native `Game::default` remains a prepared stage-one **verification fixture**, and `Game::power_on` a post-test fixture, to retain focused independent-reference comparisons; the browser uses neither.
- Startup, credits, opening, all attract states, later/challenge stages, both players, results, initials and recycle are translated. Assembly-derived tests exercise transitions; these are not proof of whole-game equivalence.
- Factory settings select three fighters, easy rank, 20K/70K/every 70K and upright cabinet through the original decoder. Native checks cover every DIP selection. Galaga used persistent physical DIP positions, not CMOS configuration; the shell stores those positions locally and passes the two original bytes at cold reset. Physical destructive RAM/checksum loops are replaced by Rust allocation and importer validation, while their original successful visible route and waits are retained.
- Keys/touch supply physical switches to the translated 51xx firmware. Its filters, coinage, free play, commands, panel bytes and read cursor are retained. One browser panel serves either player; physical-port entry permits separate cocktail inputs, coin slots and service switches. Exact MCU/06xx/host phase remains unverified; boot primes idle history at the post-self-test boundary.
- `c_1000` retains its ROM lookup, additions and within-routine refresh sample spacing. The host provides a provisional refresh phase from its monotonic clock; exact whole-board phase and repeatability against the arcade remain unverified.
- Original background waits use named continuations, currently resumed once per video period. Main/sub tasks retain table order and saved stepping. Concurrent hardware handshakes are sequential calls; exact instruction/scanline interleaving remains unverified.
- WSG sequencing and PROM synthesis are translated. Two sound NMIs run at original raster lines 64/192 relative to IRQ224. Original 54xx noise/envelope/command functions drive the board DAC/resistor/filter/mixer circuit. Normalisation/resampling is an output adaptation. Named firmware sections retain cycle/output offsets without instruction execution; exact mid-section interrupt/store ordering and global 06xx phase need independent trace validation. Original-recording audio/analogue equivalence remains unverified.
- The 05xx starfield uses the documented original circuit and original star-port speed/blink writes, advancing on board frames independently of redraw. Initial mid-frame star-enable phase and full visual comparison remain unverified.
- Rendering uses copied sprite banks, original graphics/PROMs, full control bytes and tile/colour RAM. A separate C hardware renderer matches 1,024 full-frame cases. The documented one-scanline coordinate delay is included; within-frame writes and broader original-game raster comparison remain unverified.
- The unused switch at sub:0ECA takes the supplied-board reset branch: two reads from the same static location beyond the populated ROM XOR to zero. The optional `0EDE..0FD4` editor is also translated/tested, including FIFO, nibble/selection order, target accesses and hex helper. Its external device at 10DF/10FF is absent from the supplied board configuration; the browser does not invent a peripheral or expose a new game control for it.
- Browser run/pause/reset/frame/fullscreen controls, touch, persistent DIP positions and focus-loss clock handling are shell controls. Focus loss clears held inputs without pausing; an elapsed-time scheduler advances missed board periods after browser throttling. None implement gameplay.

## Validation coverage and limits

Ordinary production-Wasm inputs reach stage six, clear challenge three, earn the 20K reserve award, enter initials and recycle. A separate run completes beam capture, replacement launch, rescue and joined fighters. Both players also complete a full game/recycle. Native table checks span all stages/ranks; these runs do not exercise every possible later-stage play-through.

The separate C sound reference covers 17 common requests over 17,408 NMIs, excluding its proven volume bugs and missing/revision-specific handlers. The hardware renderer checks 1,024 framebuffer cases. These support fidelity of the reconstructed routines and circuit interfaces, without proving original-recording analogue equivalence, within-frame writes, exact instruction interleaving or a whole-original-game execution trace. Those unresolved limits are explicit above; no replacement gameplay conceals them.

## Architecture boundary

Keep gameplay functions in Rust with fixed-width arithmetic, original helper boundaries and original task order. Keep browser input, pacing and presentation separate. Named continuations retain background waits; they do not execute CPU instructions. Untranslated original routines remain the next implementation work, not a reason to substitute gameplay or stop the Goal.
