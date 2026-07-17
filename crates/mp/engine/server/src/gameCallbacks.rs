//! `gameCallbacks.cpp` — NAV/nav-mesh callbacks the engine exposes to the game
//! VM via `VM_Call`.
//!
//! Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp`
//!
//! This file and `crate::npcnav::callbacks` both transcribed the same oracle
//! TU; `npcnav::callbacks` is canonical (settled `EngineHost::vm_call` seam,
//! ruling 24/33b), so this file re-exports it rather than keeping two bodies.
pub use crate::npcnav::callbacks::*;
