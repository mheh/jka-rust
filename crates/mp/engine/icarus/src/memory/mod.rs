//! MP ICARUS `Memory.cpp` — **no code** (§20 zero-caller drop).
//!
//! ICARUS-D3 (ruling 20) drops the ICARUS arena entirely: every `TAG_ICARUS5`
//! user is an owned buffer (`BlockMember::m_data` and `Pscript::buffer` are owned
//! `Vec<u8>`; the `Save`/`Load` `bData` scratch folds to a function-local). So
//! `ICARUS_Malloc` (`Memory.cpp:8-14`, `icarus.h:29`) and `ICARUS_Free`
//! (`Memory.cpp:16-20`, `icarus.h:30`) have **zero live callers** under the
//! owned-buffer shape and are **§20-dropped, not ported** (porting-rules §20).
//! There is no `IcarusArena` type and no `Icarus.arena` field.
