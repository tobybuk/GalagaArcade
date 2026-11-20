# Completed port and validation checkpoint

The faithful Galaga Namco rev. B Rust/browser source port is implemented and playable. Production follows the original successful cold-start display and timing route through RAM/ROM OK, cabinet configuration, sound check and crosshatch before the game executive. Game::default is only the native prepared-entry reference fixture. No CPU emulator or replacement gameplay is included.

All 96 release native tests pass. The production Wasm/standalone payload, cold startup/service, both players through game recycle, combat, input filtering, PCM and raster checks pass. Ordinary-input play reaches stage six, clears challenge three, earns a bonus fighter, enters RUS in the high-score table and returns to attract. A separate run completes beam/pull/capture/replacement/rescue/joined-fighter flow. See evidence/browser-runs.json and evidence/native-tests.txt.

All 16 ROMs are validated; 58 identified data tables are extracted. The research assembly matches 59 regions/16,473 bytes. Both complete controller listings reassemble exactly. Independent common C references cover opening, sorties, arithmetic, collisions and 17,408 sound NMIs. A separately compiled hardware renderer matches 1,024 framebuffer cases.

PORTING.md documents every browser/hardware boundary and unresolved fidelity limit. Exact CPU/scanline interleaving, refresh/06xx/MCU phase and original-recording analogue sound equivalence are not claimed. Physical destructive RAM/checksum work is replaced by allocation/importer checks, while the authentic successful display route remains. The optional external editor is translated and tested; absent peripheral hardware is not invented.

The standalone HTML embeds all game assets and the compiled module. Historical milestone ZIPs and preview.jpg remain historical; the current screenshot is evidence/browser-current.jpg. The local verification server has been stopped.

Build: python3 scripts/build.py
Tests: cargo test --release; node tests/browser-module.mjs; node tests/browser-play.mjs; node tests/browser-play.mjs 4 --capture; node tests/browser-play.mjs 4 --initials.
