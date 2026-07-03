//! `ModuleRegistry` — the `vmTable[MAX_VM]` slot registry (LOAD-D8).

use native_platform::entrypoints::RawSyscall;
use native_platform::module_loader::{ModuleSearchPolicy, RestartKind};

use super::module_slot::ModuleSlot;
use super::slot_id::SlotId;

/// `#define MAX_VM 3` (`vm.cpp:28-29`), beside `ModuleRegistry` (mechanical
/// placement mirroring oracle).
pub const MAX_VM: usize = 3;

/// The `vmTable[MAX_VM]` replacement — per-slot only, no current-module global
/// (LOAD-D5). Home crate mirrors `vm.cpp`'s subsystem.
///
/// Source: `oracle/oracle/codemp/qcommon/vm.cpp:28-29`
pub struct ModuleRegistry {
    slots: [Option<ModuleSlot>; MAX_VM],
}

impl ModuleRegistry {
    /// `VM_Create` slot semantics (`vm.cpp:471` region): bad-parms guard
    /// (`vm.cpp:480-482`) → `com_error(ERR_FATAL, "VM_Create: bad parms")`; reuse
    /// a live slot whose stored name matches case-insensitively
    /// (`vm.cpp:485-489`); else first free slot (`vm.cpp:494`); else
    /// `com_error(ERR_FATAL, …)` when all `MAX_VM` full (`vm.cpp:499-500`). A
    /// fresh slot runs `sys_load_dll` and wraps the result into a `ModuleSlot`.
    ///
    /// Returns `Option<SlotId>` (LOAD-Q10 RESOLVED, round-4): `None` = artifact
    /// not found on any policy step (mirrors `sys_load_dll`'s own contract). The
    /// CALLER owns the fatal disposition, exactly as the oracle
    /// (`if (!gvm) Com_Error(ERR_FATAL, "VM_Create on game failed")`,
    /// `sv_game.cpp:1750-1752`). The slot-full/bad-parms `ERR_FATAL`s stay INSIDE
    /// via the receiverless `com_error` (LOAD-D11).
    ///
    /// Source: `oracle/oracle/codemp/qcommon/vm.cpp:471-524`
    pub fn load_module(
        &mut self,
        policy: &ModuleSearchPolicy,
        name: &str,
        syscall: RawSyscall,
    ) -> Option<SlotId> {
        let _ = (&self.slots, policy, name, syscall);
        todo!("Port VM_Create slot semantics — oracle/oracle/codemp/qcommon/vm.cpp:471-524")
    }

    /// `VM_Free` (`vm.cpp:605-610`): `unload_module` the slot's module, clearing
    /// it. No global `currentVM`/`lastVM` clobber (LOAD-D5).
    ///
    /// Source: `oracle/oracle/codemp/qcommon/vm.cpp:605-610`
    pub fn unload(&mut self, slot: SlotId) {
        let _ = (&self.slots, slot);
        todo!("Port VM_Free — oracle/oracle/codemp/qcommon/vm.cpp:605-610")
    }

    /// Native `VM_Restart` = drop+recreate in place (`vm.cpp:398-409`). `kind` is
    /// caller-supplied (LOAD-D12b); `policy`/`name`/`syscall` are needed for
    /// `DropRecreate`'s reload.
    ///
    /// Source: `oracle/oracle/codemp/qcommon/vm.cpp:391-458`
    pub fn restart(
        &mut self,
        slot: SlotId,
        kind: RestartKind,
        policy: &ModuleSearchPolicy,
        name: &str,
        syscall: RawSyscall,
    ) {
        let _ = (&self.slots, slot, kind, policy, name, syscall);
        todo!("Port VM_Restart — oracle/oracle/codemp/qcommon/vm.cpp:391-458")
    }
}
