#!/usr/bin/env python3
"""Validate the supplied Namco rev. B set; extract assets and identified DATA.

No program ROM is embedded in the runtime. No instruction decoder is generated.
Only these explicitly identified data tables are copied out of program ROMs.
"""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile
import zlib

EXPECTED = {
    "51xx.bin": (1024, "c2f57ef8"), "54xx.bin": (1024, "ee7357e0"),
    "gg1_1b.3p": (4096, "ab036c9f"), "gg1_2b.3m": (4096, "d9232240"),
    "gg1_3.2m": (4096, "753ce503"), "gg1_4b.2l": (4096, "499fcc76"),
    "gg1_5b.3f": (4096, "bb5caae3"), "gg1_7b.2c": (4096, "d016686b"),
    "gg1_9.4l": (4096, "58b2f47c"), "gg1_10.4f": (4096, "dd6f1afc"),
    "gg1_11.4d": (4096, "ad447c80"), "prom-1.1d": (256, "7a2815b4"),
    "prom-2.5c": (256, "77245b66"), "prom-3.1c": (256, "4a04bb6b"),
    "prom-4.2n": (256, "59b6edab"), "prom-5.5n": (32, "54603c6b"),
}


def import_set(archive: Path, root: Path):
    roms = {}
    with zipfile.ZipFile(archive) as z:
        for name, (size, crc) in EXPECTED.items():
            data = z.read(name)
            if len(data) != size or f"{zlib.crc32(data):08x}" != crc:
                raise ValueError(f"Wrong ROM revision or damaged data: {name}")
            roms[name] = data
    main = b"".join(roms[n] for n in
                    ["gg1_1b.3p", "gg1_2b.3m", "gg1_3.2m", "gg1_4b.2l"])
    sub = roms["gg1_5b.3f"]
    sound = roms["gg1_7b.2c"]
    tables = []

    def table(name, cpu, start, length, signature):
        image = {"main":main,"sub":sub,"sound":sound}[cpu]
        signature = bytes.fromhex(signature)
        assert image[start:start + len(signature)] == signature, name
        data = image[start:start + length]
        assert len(data) == length
        tables.append((name, cpu, start, data))

    # Addresses are from this archive, not blindly copied from older annotations.
    table("FORMATION_ORIGIN", "main", 0x1321, 16, "31415161718191a1b1c1928a827c7670")
    table("FORMATION_ORDINALS", "sub", 0x0100, 96, "1406140c1408140a1c001c121e001e12")
    table("EXPANSION_BITMAPS", "main", 0x1E6A, 64, "ff7755141010145577ff0010145577ff")
    table("STAGE_PARAMETERS", "main", 0x2C6D, 520, "000022c600001123c700")
    # Original stage-counter wrap can read beyond a nominal table. Preserve
    # those bytes as data instead of sanitising the original behaviour.
    table("STAGE_PARAMETER_WRAP_TAIL", "main", 0x2E75, 129, "ffffffffffffffff")
    # First-stage entrance patterns: data bytes include Galaga's flight-language
    # control tokens, interpreted by a direct port of the original flight routine.
    table("ENTRANCE_TOP", "sub", 0x001D, 23, "230616230019f74b0023f002f05e00")
    table("ENTRANCE_SIDE", "sub", 0x0067, 29, "23080823031b23080f231615f78400")
    # Identified flight-language data, ending exactly before the sub executive.
    # Includes later entrances, challenge patterns, dives and capture routes.
    table("FLIGHT_TRANSIENT_TOP", "sub", 0x004B, 27, "23f026231413fe")
    table("FLIGHT_EARLY", "sub", 0x0084, 116, "231601fe0d0c0a08")
    table("FLIGHT_PATTERNS", "sub", 0x0160, 947, "441606fe0c0b0a0806040201")
    table("BOSS_SORTIE_ATTRIBUTES", "main", 0x1D03, 6, "0dba05b701b5")
    table("BOSS_ESCORT_IDS", "main", 0x1D32, 6, "4a525a585048")
    table("TEXT_DATA", "main", 0x13F1, 539, "2f14441451145c1466147214")
    table("TOP5_TEXT_LAYOUT", "main", 0x327F, 110, "248315040e171d0e1b24")
    table("HEROES_TEXT", "main", 0x3345, 39, "258313021d110e24100a15")
    table("TOP5_HIGHLIGHT_ROWS", "main", 0x3197, 10, "7487768778877a877c87")
    table("TOP5_NAME_SHIFT_LENGTHS", "main", 0x31A1, 5, "0c09060300")
    table("TOP5_SCORE_POINTERS", "main", 0x31ED, 10, "258a2b8a318a378a3d8a")
    table("TOP5_INITIAL_NAME_CHARS", "main", 0x02BF, 5, "170a160c18")
    table("BONUS_SHIP_TILES", "main", 0x0452, 12, "00811956028119620481196e")
    table("CREDIT_TEXT", "main", 0x09CA, 15, "1d120d0e1b0c220a1519240e0e1b0f")
    table("DEMO_POST_CAPTURE", "main", 0x181F, 33, "08188a0888068128")
    table("DEMO_COMBAT", "main", 0x1887, 37, "028a048207aa2888")
    table("DEMO_TRAINING", "main", 0x1928, 24, "081b813d810a4219")
    table("ATTRACT_SPRITES", "main", 0x195C, 40, "081b443a0a124442")
    table("SOUND_DATA", "sound", 0x06DA, 0x0885, "508100892691c899eca29dac")
    table("NOISE_INITIAL_PARAMETERS", "main", 0x35B3, 10, "30400002df40303003df")
    table("NOISE_TRIGGER_COMMANDS", "main", 0x0092, 4, "10102020")
    table("CONFIGURATION_DATA", "main", 0x3A72, 0x012D, "0b0c0d0a04010401041c0124")
    table("SERVICE_SOUND_DATA", "main", 0x3A51, 26, "ee82051c181e170d")
    table("SERVICE_EASTER_EGG_DATA", "main", 0x378C, 106, "0202020202080808080808")
    table("TRACTOR_BEAM_TILES", "main", 0x23A1, 60, "244e4f505124245253545524")
    table("COMBAT_STAGE_INDEX", "main", 0x26A8, 68, "0012243600486c5a486c007e")
    table("CHALLENGE_STAGE_INDEX", "main", 0x26EC, 8, "00122436485a6c7e")
    table("COMBAT_STAGE_DATA", "main", 0x26F4, 234, "14000000c0000101004141")
    table("CHALLENGE_STAGE_DATA", "main", 0x27DE, 144, "ff000006c6000707004747")
    table("CHALLENGE_HIT_ATTRIBUTES", "main", 0x2900, 8, "0ab80fb914bc1ebd")
    table("CHALLENGE_CLASSES", "main", 0x290E, 8, "3624d4bae4cca8f4")
    table("ENTRANCE_VECTORS", "main", 0x2A3C, 48, "1d0067209f40d420")
    table("FLIGHT_LATE", "sub", 0x0FDA, 35, "23001b23f040230009")
    table("BONUS_BEE_ATTRIBUTES", "main", 0x1B5F, 6, "1ebd0ab814bc")
    table("BONUS_BEE_VECTORS", "main", 0x1B65, 6, "ea047304ab04")
    # c_1000 intentionally reads arbitrary bytes in this ROM page as DATA.
    # The page contains both formation data and instruction encodings; no
    # byte from it is decoded or executed by the Rust runtime.
    table("REFRESH_SAMPLE_PAGE", "main", 0x0100, 256, "1406140c1408140a")

    def locate_table(name, length, signature):
        sig = bytes.fromhex(signature)
        starts = [i for i in range(len(main)) if main.startswith(sig, i)]
        assert len(starts) == 1, (name, starts)
        table(name, "main", starts[0], length, signature)

    locate_table("STAGE_ONE_WAVES", 18,
                 "14000000c0000101004141004040000000ff")
    locate_table("WAVE_OBJECT_IDS", 40,
                 "585a5c5e282a2c2e3034363250525456")
    locate_table("ENTRANCE_INITIALS", 36,
                 "9b34039b44032300002378029b2c039b4c03")
    table("MAIN_TASK_VECTORS", "main", 0x0096, 64, "27082808b2170017861a570827082708")
    table("SUB_TASK_VECTORS", "sub", 0x003B, 16, "be05bf05d308be05f506ee05be05ca0e")
    table("MAIN_TASK_DEFAULTS", "main", 0x125B, 32, "1f010000000100000000000001010001")
    table("SCORE_FACTORS", "main", 0x07FB, 16, "10000000000000005008080805081500")
    table("SCORE_INITIAL", "main", 0x0495, 9, "000024242424242424")
    locate_table("HEADER_TILES", 17, "0e1b180c1c241110121124242424191e01")
    locate_table("HIGH_SCORE_INITIAL", 6, "000000000224")
    table("DRONE_LAUNCH_TIMERS", "main", 0x08CD, 30, "090705080604070504060403")
    table("BOSS_LAUNCH_TIMERS", "main", 0x08EB, 30, "060504050403050303040302")
    table("BOMB_ENABLE_TIMERS", "main", 0x0909, 44, "030301010303030107030301")
    table("BOMBER_TIMER_READS", "main", 0x08CD, 160, "090705080604070504060403")
    table("BOMB_CLASS_MASKS", "main", 0x2908, 6, "a55aa90f0a50")

    assets = root / "assets"
    assets.mkdir(parents=True, exist_ok=True)
    for dst, data in {
        "characters.bin": roms["gg1_9.4l"],
        "sprites.bin": roms["gg1_11.4d"] + roms["gg1_10.4f"],
        "palette.bin": roms["prom-5.5n"],
        "character-colours.bin": roms["prom-4.2n"],
        "sprite-colours.bin": roms["prom-3.1c"],
        "sound-waves.bin": roms["prom-1.1d"],
    }.items():
        (assets / dst).write_bytes(data)
    src = root / "src"
    src.mkdir(exist_ok=True)
    out = ["// Generated from validated user-supplied ROM data. See evidence/rom-map.json.\n"]
    for name, cpu, start, data in tables:
        out.append(f"// {cpu}:0x{start:04X}..0x{start + len(data):04X} (end exclusive)\n")
        out.append(f"pub const {name}: [u8; {len(data)}] = [\n")
        for i in range(0, len(data), 16):
            out.append("    " + ", ".join(f"0x{x:02x}" for x in data[i:i + 16]) + ",\n")
        out.append("];\n")
    (src / "rom_data.rs").write_text("".join(out))
    evidence = root / "evidence"
    evidence.mkdir(exist_ok=True)
    report = {
        "set": "Galaga (Namco rev. B)",
        "archive_sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
        "roms": [{"name": n, "bytes": len(d), "crc32": f"{zlib.crc32(d):08x}",
                  "sha256": hashlib.sha256(d).hexdigest()} for n, d in roms.items()],
        "extracted_data": [{"name": n, "cpu": c, "start": f"0x{s:04x}",
                            "end_exclusive": f"0x{s + len(d):04x}", "bytes": len(d),
                            "sha256": hashlib.sha256(d).hexdigest()} for n, c, s, d in tables],
        "runtime_program_roms": [],
    }
    (evidence / "rom-map.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"Verified {len(roms)} ROMs; extracted {len(tables)} identified data tables.")


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("archive", type=Path)
    args = p.parse_args()
    import_set(args.archive, Path(__file__).resolve().parents[1])
