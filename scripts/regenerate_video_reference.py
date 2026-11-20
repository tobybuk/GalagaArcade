#!/usr/bin/env python3
"""Compile the separate hardware renderer and retain its pixel hashes."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[1]
source=ROOT/'evidence/video-oracle.c'
with tempfile.TemporaryDirectory() as directory:
    executable=Path(directory)/'video-oracle'
    subprocess.run(['clang','-O2',str(source),'-lm','-o',str(executable)],check=True)
    data=subprocess.check_output([str(executable),str(ROOT/'assets')])
assert len(data)==1024*8
(ROOT/'evidence/reference-video.bin').write_bytes(data)
report={'cases':1024,'pixels_per_case':224*288,'channels':'RGBA',
        'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
        'reference_sha256':hashlib.sha256(data).hexdigest(),
        'authority':'MAME Galaga video/palette hardware documentation, BSD-3-Clause; supplied graphics and PROMs',
        'scope':'Independent C hardware renderer; coordinates, size/flips, clipping, PROM colours, tile addressing/layers and star colours. Not original game execution or scanline/interprocessor timing.'}
(ROOT/'evidence/reference-video.json').write_text(json.dumps(report,indent=2)+'\n')
print('Generated 1,024 independent hardware-reference RGBA hashes.')
