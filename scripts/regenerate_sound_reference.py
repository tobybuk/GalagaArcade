#!/usr/bin/env python3
"""Generate isolated common-sound evidence from the pinned unmodified C aid."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
ROOT=Path(__file__).resolve().parents[1]
COMMIT='248be0d9d80e04cf24ec5eaa1a2b50aa2f90795f'
checkout=Path(sys.argv[1]).resolve()
assert subprocess.check_output(['git','-C',str(checkout),'rev-parse','HEAD'],text=True).strip()==COMMIT
source=checkout/'galag/c_code/src'
files=['game_ctrl','gg1-2','gg1-2_1700','gg1-3','gg1-4','task_man','pe_string','new_stage']
sounds=[1,2,3,4,8,9,10,11,12,13,15,16,17,19,20,21,22]
with tempfile.TemporaryDirectory(prefix='galaga-sound-reference-') as temporary:
    executable=Path(temporary)/'sound-reference'
    command=[shutil.which('clang'),'-O2','-fsigned-char','-DLSB_FIRST','-I',str(source),
             '-Wno-implicit-function-declaration','-Wno-int-conversion','-Wno-comment',
             '-Wno-tautological-constant-out-of-range-compare','-Wno-constant-logical-operand',
             str(ROOT/'evidence/sound-oracle.c')]
    command += [str(source/(name+'.c')) for name in files]+['-o',str(executable)]
    subprocess.run(command,check=True);data=subprocess.check_output([str(executable)])
assert len(data)==len(sounds)*1024*8
(ROOT/'evidence/reference-sound.bin').write_bytes(data)
report={'reference_commit':COMMIT,'reference_modified':False,'sounds':sounds,'nmis_per_sound':1024,
        'fixture_sha256':hashlib.sha256(data).hexdigest(),
        'harness_sha256':hashlib.sha256((ROOT/'evidence/sound-oracle.c').read_bytes()).hexdigest(),
        'scope':'Isolated common sequencer status, indexes, requests/active flags and masked WSG frequency/wave registers; no analogue PCM or interprocessor timing. Revision-specific beam/ship-hit/challenge tails excluded.',
        'excluded_registers':[21,26,31],
        'omitted_c_handler':'The C NMI omits request 0x12 entirely (supplied sound:0208..0215); it is covered by assembly-derived Rust tests, not this golden.',
        'c_volume_errors':'Type-2 CPL uses stale A instead of the note counter; decay omits SUB 10 before NEG and differs at >=10; completion reads unaliased b_9A70[7] rather than the selected voice. Supplied sound:061E..065E/06BC..06D9 and separate Rust assembly-derived tests remain authoritative.'}
(ROOT/'evidence/reference-sound.json').write_text(json.dumps(report,indent=2)+'\n')
print(f'Generated {len(sounds)*1024:,} independent common-sound NMI hashes.')
