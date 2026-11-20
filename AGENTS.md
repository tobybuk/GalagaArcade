# User's reconstruction requirement

This project is a faithful source-level Rust conversion of the supplied Galaga
Namco rev. B ROMs. Preserve the original functions, branch decisions, task tables,
state transitions and program flow. Make only necessary environment adaptations.

- The supplied ROM instructions and data are authoritative. Annotated assembly
  and the pinned C translation are aids; both differ from this ROM revision.
- Check the relevant original bytes/assembly before porting a routine. Retain
  original routine names/addresses in the mapping and record revision changes.
- Do not introduce replacement flight paths, combat rules, scoring, timing,
  health, difficulty or invented missing behaviour. Preserve original quirks.
- Retain original arithmetic subroutines and helper-call structure where it is
  practical. Mathematical equivalence alone is not a reason to redesign them.
- Missing original routines must remain explicit execution stops at the
  corresponding call. Do not silently skip enabled tasks or keep a reduced
  gameplay loop running past an untranslated transition.
- No CPU emulator or opcode execution engine. Task-vector lookup maps original
  function pointers to translated Rust functions.
- Keep browser input collection, video pacing and canvas presentation separate
  from original game logic. Document every remaining environment assumption.
- Do not describe secondary C-reference checks as proof of whole-game fidelity.
  Keep independent-reference coverage and assembly-derived tests distinct.
- Rebuild the standalone preview and browser module after source changes, run
  relevant checks, and update PORTING.md and evidence before packaging.

See PORTING.md for implemented routine ranges, known C/annotation errors,
validation limits and the native prepared-entry verification fixture.
