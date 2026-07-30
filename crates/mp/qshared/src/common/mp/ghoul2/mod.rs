//! MP ghoul2 seam constants and structs shared by `bg`, `game`, `cgame`, `ui`,
//! and the engine's ghoul2 crate, mirroring `oracle/codemp/ghoul2/G2.h` and the
//! ghoul2 block of `oracle/codemp/game/q_shared.h`.

pub mod bone_flags;
pub mod perf_flags;
pub mod sskin_gore_data;

pub use bone_flags::*;
pub use perf_flags::*;
