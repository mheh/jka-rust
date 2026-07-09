//! `VmSlot` — the engine VM selector for [`EngineHost::vm_call`].
//!
//! [`EngineHost::vm_call`]: crate::engine_host::EngineHost::vm_call

/// Raven's engine VM slots — the `vm_t *` globals passed as `VM_Call`'s first
/// parameter (`vm.cpp:787`). `Gvm` (game) is the live slot under DEDICATED;
/// `Cgvm` (cgame) is NULL there, yet ROFF's `VM_Call( cgvm, … )` sites
/// transcribe 1:1 (`RoffSystem.cpp:837-841,952,983-985`) — a host impl answers
/// for the NULL slot exactly as Raven would (ruling 33b).
///
/// Source: `oracle/codemp/server/server.h:234` (`gvm`),
/// `oracle/codemp/client/client.h:386` (`cgvm`)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmSlot {
    /// Raven `gvm` — the game virtual machine.
    Gvm,
    /// Raven `cgvm` — the cgame virtual machine (NULL under DEDICATED).
    Cgvm,
}
