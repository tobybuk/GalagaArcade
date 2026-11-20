# Sources and acknowledgements

Original game: **Galaga**, Namco, 1981. The game graphics, palettes and recovered game data originate from the user-supplied Namco rev. B ROM archive.

Research reference: Glenn Neidermeier and contributors, [neiderm/arcade](https://github.com/neiderm/arcade), pinned at `248be0d9d80e04cf24ec5eaa1a2b50aa2f90795f`. Its annotated assembly and C translation helped identify routines and provided an independent verification reference. The repository does not state a project-wide licence; this reconstruction does not assign a new blanket licence to original or third-party material.

Hardware/video reference: Nicola Salmoria and the MAME contributors, [MAME Galaga driver](https://github.com/mamedev/mame/blob/master/src/mame/namco/galaga.cpp), and Nicola Salmoria / Aaron Giles, [MAME graphics decoder](https://github.com/mamedev/mame/blob/master/src/emu/drawgfx.cpp). The renderer follows the documented bit-plane, colour lookup, resistor-network and sprite-coordinate layouts.

Additional hardware references: Nicola Salmoria / Aaron Giles, [Namco WSG](https://github.com/mamedev/mame/blob/master/src/devices/sound/namco.cpp), and Robert Hildinger, [05xx starfield circuit](https://github.com/mamedev/mame/blob/master/src/mame/namco/starfield_05xx.cpp). These document the circuit adaptation; they are not a gameplay reference or a CPU execution engine.

MAME reference files identify their code as BSD-3-Clause. The applicable notice is reproduced below for attribution of derived video-decoding work.

Controller/circuit research also uses Mike Harris / Aaron Giles' [51xx wiring](https://github.com/mamedev/mame/blob/master/src/mame/namco/namco51.cpp) and [06xx interface](https://github.com/mamedev/mame/blob/master/src/mame/namco/namco06.cpp), Ernesto Corvi's [MB88xx static disassembler](https://github.com/mamedev/mame/blob/master/src/devices/cpu/mb88xx/mb88dasm.cpp), Derrick Renaud's [Galaga analogue circuit](https://github.com/mamedev/mame/blob/master/src/mame/namco/galaga_a.cpp), and K. Wilkins / Derrick Renaud's [filter](https://github.com/mamedev/mame/blob/master/src/devices/sound/disc_flt.hxx) and [mixer](https://github.com/mamedev/mame/blob/master/src/devices/sound/disc_mth.hxx) documentation. The controller firmware is reconstructed as direct Rust functions; no CPU interpreter from these references is executed or linked.

Copyright (c) Nicola Salmoria, Aaron Giles, Robert Hildinger, Mike Harris, Ernesto Corvi, Derrick Renaud, K. Wilkins and respective MAME contributors.

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
