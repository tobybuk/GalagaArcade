#!/usr/bin/env python3
"""Reassemble the research 54xx listing and compare the supplied firmware.

This is a static listing verifier, not an interpreter. It never executes the
firmware and is never linked into the Rust/browser game. Mnemonic encodings
follow the Fujitsu MB88xx disassembler in MAME (Ernesto Corvi, BSD-3-Clause).
"""
import hashlib
import json
from pathlib import Path
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
SINGLE = "NOP OUTO OUTP OUT TAY TATH TATL TAS ICY ICM STIC X ROL L ADC AND DAA DAS INK IN TYA TTHA TTLA TSA DCY DCM STDC XX ROR ST SBC OR SETR SETC RSTR RSTC TSTR TSTI TSTV TSTS TSTC TSTZ STS LS RTS NEG C EOR".split()


def assemble(address, mnemonic, operand=None):
    if mnemonic in SINGLE:
        return bytes([SINGLE.index(mnemonic)])
    if mnemonic == "RTI":
        return b"\x3c"
    number = int(operand, 16) if mnemonic in ["CALL", "JPL", "JMP"] else int(operand, 0)
    if mnemonic in ["JPA", "EN", "DIS"]:
        return bytes([0x3d + ["JPA", "EN", "DIS"].index(mnemonic), number])
    if mnemonic in ["CALL", "JPL"]:
        return bytes([(0x60 if mnemonic == "CALL" else 0x68) | (number >> 8), number & 255])
    if mnemonic == "JMP":
        assert number & ~63 == (address + 1) & ~63
        return bytes([0xc0 | (number & 63)])
    groups = {"SBIT": (0x30,0), "RBIT": (0x34,0), "TBIT": (0x38,0),
              "SETD": (0x40,0), "RSTD": (0x44,0), "TSTD": (0x48,8),
              "TBA": (0x4c,0), "XD": (0x50,0), "XYD": (0x54,4),
              "LXI": (0x58,0), "AI": (0x70,0), "LYI": (0x80,0),
              "LI": (0x90,0), "CYI": (0xa0,0), "CI": (0xb0,0)}
    opcode, bias = groups[mnemonic]
    return bytes([opcode + number - bias])


def verify(archive, chip="54xx"):
    assert chip in ["51xx", "54xx"]
    listing = ROOT / f"evidence/{chip}-firmware.asm"
    reconstructed = bytearray()
    instructions = 0
    for line in listing.read_text().splitlines():
        address, text = line.split(":", 1)
        address = int(address,16)
        encoded, mnemonic, *operand = text.split()
        assert address == len(reconstructed)
        data = assemble(address,mnemonic,operand[0] if operand else None)
        assert data.hex() == encoded, f"Listing encoding mismatch at {address:03x}"
        reconstructed.extend(data)
        instructions += 1
    with zipfile.ZipFile(archive) as z:
        supplied = z.read(f"{chip}.bin")
    assert len(supplied) == 1024
    assert reconstructed == supplied
    report = {"firmware":f"{chip}.bin", "bytes":1024, "listing_entries":instructions,
              "sha256":hashlib.sha256(supplied).hexdigest(), "match":True,
              "listing_sha256":hashlib.sha256(listing.read_bytes()).hexdigest(),
              "scope":"Static reassembly establishes exact firmware mapping, not Rust execution/output equivalence.",
              "runtime":("Direct reconstructed noise/envelope/command functions" if chip=="54xx" else
                         "Direct reconstructed coin/filter/start/control/command functions")+
                        "; no firmware byte array, CPU registers or opcode execution."}
    (ROOT / f"evidence/reference-{chip}.json").write_text(json.dumps(report,indent=2)+"\n")
    print(f"Verified the complete 1,024-byte {chip} firmware listing against the supplied archive.")


if __name__ == "__main__":
    verify(Path(sys.argv[1]),sys.argv[2] if len(sys.argv)>2 else "54xx")
