//! `gameCallbacks.cpp` — NAV/nav-mesh callbacks the engine exposes to the game
//! VM via `VM_Call`.
//!
//! Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp`
//!
//! PORT-NOTE(duplicate-reconciliation): this file and
//! `crate::npcnav::callbacks` both transcribed the same oracle TU. The
//! `npcnav::callbacks` copy is the canonical one — it calls through the
//! settled `EngineHost::vm_call` seam (ruling 24/33b, `VmSlot::Gvm`) instead
//! of this file's stale guessed `mp_engine_qcommon::vm::VM_Call` free
//! function, which was never landed. Re-exporting rather than keeping two
//! divergent bodies for the same Raven functions.
pub use crate::npcnav::callbacks::*;
