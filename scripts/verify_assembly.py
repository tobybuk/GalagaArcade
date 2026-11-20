#!/usr/bin/env python3
"""Check selected annotated assembly routines against the supplied ROMs.

Pass the rom0 directory after building neiderm/arcade's pinned assembly with
ASxxxx 5.10. Explicit relocations account for known revision address changes;
no other differences are accepted. This does not execute any arcade code.
"""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile


def read_hex(path):
    memory = {}
    base = 0
    for line in path.read_text().splitlines():
        record = bytes.fromhex(line[1:])
        assert sum(record) % 256 == 0, "Bad Intel HEX checksum"
        length, kind = record[0], record[3]
        offset = int.from_bytes(record[1:3], "big")
        if kind == 4:
            base = int.from_bytes(record[4:4 + length], "big") << 16
        elif kind == 0:
            for i, byte in enumerate(record[4:4 + length]):
                address = base + offset + i
                assert address not in memory, "Overlapping assembly output"
                memory[address] = byte
    return memory


# name, bank, annotation start/end, actual revision delta, explicitly verified
# 16-bit relocation operands: annotation address -> (old target, new target).
ROUTINES = [
    ("service executive, crosshatch and hidden-message data", "main", 0x35B5, 0x37F4, 10,
     {0x35B6:(0x3B88,0x3B95),0x35BC:(0x37F4,0x37FE),0x35CE:(0x35A5,0x35AF),
      0x35E0:(0x37EC,0x37F6),0x35F1:(0x35A9,0x35B3),0x3603:(0x37EC,0x37F6),
      0x3613:(0x39F2,0x39FC),0x363D:(0x36BA,0x36C4),0x365B:(0x39D6,0x39E0),
      0x3660:(0x37F4,0x37FE),0x3670:(0x39BB,0x39C5),0x367D:(0x3620,0x362A),
      0x3685:(0x3620,0x362A),0x3689:(0x3782,0x378C),0x3697:(0x3620,0x362A),
      0x369F:(0x3620,0x362A),0x36A2:(0x3958,0x3962),0x36A5:(0x3972,0x397C),
      0x36A8:(0x3798,0x37A2),0x36B0:(0x3766,0x3770),0x36CA:(0x3620,0x362A),
      0x36CD:(0x3972,0x397C),0x371D:(0x37EC,0x37F6),0x373D:(0x37EC,0x37F6),
      0x374F:(0x37EC,0x37F6),0x3767:(0x3774,0x377E),0x376A:(0x3774,0x377E),
      0x376D:(0x3774,0x377E)}),
    ("service totals, input and sound-test helpers", "main", 0x3958, 0x3A61, 10,
     {0x3988:(0x3997,0x39A1),0x398D:(0x3997,0x39A1),0x3992:(0x3997,0x39A1),
      0x3998:(0x39AA,0x39B4),0x399B:(0x39A0,0x39AA),0x39A8:(0x39AE,0x39B8),
      0x39CB:(0x397D,0x3987),0x3A19:(0x3A47,0x3A51),0x3A2E:(0x3A3C,0x3A46),
      0x3A31:(0x3A4F,0x3A59)}),
    ("sub-board reset and default tasks", "sub", 0x057C, 0x05BE, 0, {}),
    ("unused switch and optional external memory editor", "sub", 0x0ECA, 0x0FDA, 0, {}),
    ("service DIP decoder and display", "main", 0x37F4, 0x3958, 10,
     {0x37FF:(0x3ACC,0x3AD6),0x3803:(0x3A61,0x3A6B),0x3824:(0x3A68,0x3A72),
      0x382D:(0x3AE4,0x3AEE),0x3846:(0x3AEB,0x3AF5),0x384C:(0x3AC4,0x3ACE),
      0x386C:(0x3A6C,0x3A76),0x3891:(0x3AF3,0x3B00),0x38A6:(0x3B04,0x3B11),
      0x38BB:(0x392D,0x3937),0x38C7:(0x3AA4,0x3AAE),0x38D5:(0x38DA,0x38E4),
      0x38DD:(0x393B,0x3945),0x38E3:(0x3B1A,0x3B27),0x3900:(0x391E,0x3928),
      0x390A:(0x3949,0x3953),0x3910:(0x391E,0x3928),0x3914:(0x3B4D,0x3B5A),
      0x392E:(0x3B64,0x3B71)}),
    ("cabinet wrapper and configuration tables", "main", 0x3A61, 0x3AEB, 10,
     {0x3ACC:(0x3AD0,0x3ADA),0x3ACE:(0x3ADA,0x3AE4)}),
    ("power-on, attract wait and game start", "main", 0x02D3, 0x045E, 0,
     {0x0345: (0x1230,0x1242),0x0374: (0x1230,0x1242),0x0394: (0x1230,0x1242),
      0x040A: (0x1269,0x127B),0x042A: (0x1398,0x13B3),0x044C: (0x1398,0x13B3),
      0x044F: (0x128C,0x129E)}),
    ("credit, game time and accumulated machine score", "main", 0x0977, 0x0A53, 0, {}),
    ("player state, enemy data and task exchange", "main", 0x110C, 0x117F, 0, {}),
    ("task table initialisation", "main", 0x1230, 0x1269, 18, {0x1231: (0x1249,0x125B)}),
    ("static sprite layout", "main", 0x128C, 0x12C3, 18, {}),
    ("attract and scripted fighter control", "main", 0x1700, 0x19B2, 0,
     {0x172E:(0x1F0F,0x1F19),0x1756:(0x1F92,0x1F9C),0x1764:(0x1F0F,0x1F19),
      0x1809:(0x133A,0x134C),0x1842:(0x1230,0x1242),0x1870:(0x133A,0x134C),
      0x18DC:(0x128C,0x129E),0x18E8:(0x133A,0x134C),0x1923:(0x1269,0x127B),
      0x19A4:(0x128C,0x129E)}),
    ("formation retreat and star-control acceleration", "main", 0x1D32, 0x1DB3, 6, {}),
    ("high-score initials executive and text", "main", 0x3000, 0x336C, 0, {}),
    ("stage splash and environment", "main", 0x0185, 0x0237, 0,
     {0x01D7: (0x12C3, 0x12D5), 0x0232: (0x1398, 0x13B3)}),
    ("challenge-stage completion", "main", 0x064A, 0x0728, 0,
     {0x064B: (0x1398, 0x13B3), 0x0661: (0x131F, 0x1331),
      0x0667: (0x131F, 0x1331), 0x0673: (0x131F, 0x1331),
      0x0680: (0x131F, 0x1331), 0x06C3: (0x131F, 0x1331),
      0x06C6: (0x131F, 0x1331), 0x06CE: (0x1398, 0x13B3),
      0x06D6: (0x1398, 0x13B3)}),
    ("refresh-based random lookup", "main", 0x1000, 0x1012, 0, {}),
    ("bonus-bee selection and launch", "main", 0x1A80, 0x1B65, 6,
     {0x1AE5: (0x1B54, 0x1B5A), 0x1B17: (0x1B59, 0x1B5F), 0x1B2E: (0x1B5F, 0x1B65)}),
    ("capture beam, spin and rescue", "main", 0x2000, 0x23DD, 0, {}),
    ("background restart, termination and respawn flow", "main", 0x049E, 0x064A, 0,
     {0x04FB: (0x1398, 0x13B3), 0x0501: (0x131F, 0x1331),
      0x0504: (0x131F, 0x1331), 0x0539: (0x1398, 0x13B3),
      0x05C6: (0x136C, 0x137E), 0x05E2: (0x12C3, 0x12D5),
      0x061C: (0x1398, 0x13B3), 0x061F: (0x132B, 0x133D),
      0x0630: (0x131F, 0x1331), 0x0643: (0x1398, 0x13B3)}),
    ("delay and fighter onscreen helpers", "main", 0x131F, 0x136C, 18,
     {0x1341: (0x136C, 0x137E)}),
    ("original string writer", "main", 0x1398, 0x13D6, 27,
     {0x139E: (0x13D4, 0x13EF)}),
    ("playfield clear", "main", 0x0160, 0x0185, 0, {}),
    ("decimal output and hit ratio", "main", 0x0A53, 0x0B0F, 0, {}),
    ("main multiply and divide helpers", "main", 0x104E, 0x1079, 0, {}),
    ("attacker motion-slot setup", "main", 0x1079, 0x110C, 0, {}),
    ("enemy sortie manager and escort selection", "main", 0x1B65, 0x1D32, 6,
     {0x1BC9: (0x1BD1, 0x1BD7), 0x1BD1: (0x1BD7, 0x1BDD),
      0x1BD3: (0x1BF7, 0x1BFD), 0x1BD5: (0x1C01, 0x1C07),
      0x1C2E: (0x1CAE, 0x1CB4), 0x1C31: (0x1D2C, 0x1D32),
      0x1C59: (0x1C8D, 0x1C93), 0x1C69: (0x1C8D, 0x1C93),
      0x1C86: (0x1D25, 0x1D2B), 0x1CC8: (0x1CFD, 0x1D03),
      0x1CDE: (0x1D03, 0x1D09), 0x1CE1: (0x1D03, 0x1D09),
      0x1D10: (0x1D2C, 0x1D32)}),
    ("main interrupt task kernel", "main", 0x0237, 0x02B0, 0, {}),
    ("score supervisor and decimal tile increment", "main", 0x0728, 0x07FB, 0,
     {0x07C6: (0x136C, 0x137E)}),
    ("hit notification and game timers", "main", 0x1DB3, 0x1DE6, 6, {}),
    ("main sprite copy and bomber timer setup", "main", 0x0828, 0x0935, 0, {}),
    ("player label blink", "main", 0x0935, 0x0977, 0, {}),
    ("missile object initialisation", "main", 0x1269, 0x128C, 18, {}),
    ("initial score header writes", "main", 0x00D6, 0x00EB, 0, {}),
    ("sub sprite copy", "sub", 0x05BF, 0x05EE, 0, {}),
    ("fighter collision and termination", "sub", 0x05EE, 0x06F5, 0, {}),
    ("fighter control", "main", 0x1F85, 0x1FDD, 10, {}),
    ("fire input and rocket initialisation", "main", 0x1F04, 0x1F85, 10, {}),
    ("formation coordinate initialisation", "main", 0x12C3, 0x130F, 18,
     {0x12CD: (0x130F, 0x1321), 0x12DE: (0x130F, 0x1321)}),
    ("formation expansion", "main", 0x1DE6, 0x1E64, 6,
     {0x1E14: (0x1E64, 0x1E6A), 0x1E3D: (0x1E43, 0x1E49)}),
    ("object dispatcher", "main", 0x23DD, 0x25A0, 0, {}),
    ("wave sequence builder", "main", 0x25A2, 0x26A8, 0, {}),
    ("enemy class initialisation", "main", 0x2896, 0x2900, 0, {}),
    ("entrance launcher and formation slide", "main", 0x2916, 0x2AED, 0, {}),
    ("stage parameter unpacker", "main", 0x2C00, 0x2C65, 0, {}),
    ("rocket movement and detection", "sub", 0x06F5, 0x08D3, 0, {}),
    ("flight dispatcher", "sub", 0x08D3, 0x0C05, 0,
     {0x08E9: (0x0DFB, 0x0DFF), 0x0900: (0x0E49, 0x0E4D),
      0x0920: (0x0E49, 0x0E4D), 0x0A3C: (0x0EAA, 0x0EAE),
      0x0A82: (0x0E5B, 0x0E5F), 0x0AFB: (0x0E5B, 0x0E5F),
      0x0B36: (0x0EAA, 0x0EAE), 0x0B96: (0x0DFB, 0x0DFF)}),
    ("home detection and sprite orientation", "sub", 0x0C05, 0x0CA7, 0,
     {0x0C22: (0x0E08, 0x0E0C), 0x0C2B: (0x0E08, 0x0E0C)}),
    ("fixed point coordinate step", "sub", 0x0CA7, 0x0D03, 0,
     {0x0CE8: (0x0E97, 0x0E9B)}),
    ("home arrival", "sub", 0x0E08, 0x0E49, 4, {}),
    ("home angle", "sub", 0x0E5B, 0x0E97, 4, {0x0E88: (0x0EAA, 0x0EAE)}),
    ("8 bit multiplication", "sub", 0x0E97, 0x0EAA, 4, {}),
    ("16 by 8 bit division", "sub", 0x0EAA, 0x0EC2, 4,
     {0x0EC0: (0x0EB7, 0x0EBB)}),
]


def verify(archive, assembled, destination, revision_b=None, revision_b_sound=None):
    with zipfile.ZipFile(archive) as z:
        original = {
            "main": b"".join(z.read(n) for n in ["gg1_1b.3p", "gg1_2b.3m", "gg1_3.2m", "gg1_4b.2l"]),
            "sub": z.read("gg1_5b.3f"),
            "sound": z.read("gg1_7b.2c"),
        }
    images = {"main": read_hex(assembled / "ga0.i86"),
              "sub": read_hex(assembled / "ga0_sub.i86")}
    results = []
    for name, bank, start, end, delta, relocations in ROUTINES:
        code = bytearray(images[bank][i] for i in range(start, end))
        for address, (old, new) in relocations.items():
            i = address - start
            assert int.from_bytes(code[i:i + 2], "little") == old, (name, address)
            code[i:i + 2] = new.to_bytes(2, "little")
        assert code == original[bank][start + delta:end + delta], name
        results.append({
            "routine": name, "bank": bank, "bytes": len(code),
            "annotation_start": f"0x{start:04x}", "rom_start": f"0x{start + delta:04x}",
            "rom_end_exclusive": f"0x{end + delta:04x}",
            "relocations": [{"annotation_operand": f"0x{a:04x}", "old_target": f"0x{o:04x}",
                             "rom_target": f"0x{n:04x}"} for a, (o, n) in relocations.items()],
            "sha256": hashlib.sha256(code).hexdigest(), "match": True,
        })
    if revision_b is not None:
        corrected = read_hex(revision_b)
        for name, bank, start, end in [("rev. B reserve icons", "main", 0x137E, 0x13B3),
                                  ("rev. B bomb updater", "main", 0x1EAA, 0x1F0E),
                                  ("rev. B sprite output and bomb generation", "sub", 0x0D03, 0x0E0C),
                                  ("rev. B captured-fighter transfer", "main", 0x19B2, 0x1A86),
                                  ("rev. B stage tokens", "main", 0x117F, 0x1242)]:
            code = bytes(corrected[i] for i in range(start, end))
            assert code == original[bank][start:end], name
            results.append({"routine": name, "bank": bank, "bytes": len(code),
                            "rom_start": f"0x{start:04x}", "rom_end_exclusive": f"0x{end:04x}",
                            "source": "evidence/revision-b.s", "relocations": [],
                            "sha256": hashlib.sha256(code).hexdigest(), "match": True})
    if revision_b_sound is not None:
        sound = read_hex(revision_b_sound)
        start, end = 0x00BB, 0x0F5F
        code = bytes(sound[i] for i in range(start,end))
        assert code == original["sound"][start:end], "rev. B sound sequencing and data"
        results.append({"routine":"rev. B sound sequencing and data", "bank":"sound", "bytes":len(code),
                        "rom_start":f"0x{start:04x}", "rom_end_exclusive":f"0x{end:04x}",
                        "source":"evidence/revision-b-sound.s", "relocations":[],
                        "sha256":hashlib.sha256(code).hexdigest(), "match":True})
    destination.write_text(json.dumps({
        "reference": "https://github.com/neiderm/arcade",
        "commit": "248be0d9d80e04cf24ec5eaa1a2b50aa2f90795f",
        "assembler": "ASxxxx 5.10", "routines": results,
        "limitation": "Matching assembly establishes the reference mapping; it is not proof that every Rust routine or every game feature has been ported.",
    }, indent=2) + "\n")
    print(f"Verified {len(results)} assembly regions ({sum(r['bytes'] for r in results):,} bytes) against the supplied ROMs.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("assembled", type=Path)
    parser.add_argument("--revision-b", type=Path, help="Assembled evidence/revision-b.s Intel HEX output")
    parser.add_argument("--revision-b-sound", type=Path, help="Assembled evidence/revision-b-sound.s Intel HEX output")
    args = parser.parse_args()
    verify(args.archive, args.assembled,
           Path(__file__).resolve().parents[1] / "evidence/reference-assembly.json", args.revision_b, args.revision_b_sound)
