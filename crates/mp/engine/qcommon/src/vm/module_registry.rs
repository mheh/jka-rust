//! `ModuleRegistry` — the `vmTable[MAX_VM]` slot registry (LOAD-D8).

use core::ffi::c_void;

use native_platform::entrypoints::RawSyscall;
use native_platform::module_loader::{ModuleSearchPolicy, RestartKind};

use super::engine_slot::SlotSyscall;
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
    /// (`vm.cpp:480-482`) → `com_error(ERR_FATAL, "VM_Create: bad parms")` —
    /// the guard is `name.is_empty()` ONLY (round-5 resolution): Raven's
    /// `!module` and `!systemCalls` disjuncts are structurally unreachable in
    /// Rust (`&str` is non-null; `SlotSyscall` is a non-nullable fn pointer);
    /// reuse
    /// a live slot whose stored name matches case-insensitively
    /// (`vm.cpp:485-489`); else first free slot (`vm.cpp:494`); else
    /// `com_error(ERR_FATAL, …)` when all `MAX_VM` full (`vm.cpp:499-500`). A
    /// fresh slot runs `sys_load_dll` and wraps the result into a
    /// `ModuleSlot { name, module, engine }` — `engine` built from the injected
    /// `system_calls` + `ctx` below.
    ///
    /// Two distinct fn-pointer parameters, both faithful to Raven's call
    /// geometry: `syscall` is the raw trampoline handed to the module's
    /// `dllEntry` (Raven passes `VM_DllSyscall` to `Sys_LoadDll`,
    /// `vm.cpp:515-518`); `system_calls` (+ its opaque `ctx`) is the engine-side
    /// dispatch target that trampoline forwards to — Raven's `VM_Create`
    /// parameter `int (*systemCalls)(int *)` (`vm.cpp:471-472`), stored
    /// `vm->systemCall = systemCalls` (`vm.cpp:506`). Injection keeps this crate
    /// from naming the upstream engine state (skeleton-findings resolution 2,
    /// 2026-07-03).
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
        system_calls: SlotSyscall,
        ctx: *mut c_void,
    ) -> Option<SlotId> {
        let _ = (&self.slots, policy, name, syscall, system_calls, ctx);
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
    /// `DropRecreate`'s reload. The injected `system_calls`/`ctx` are NOT
    /// re-supplied: the slot's stored `EngineSlot` is reused, mirroring Raven's
    /// native arm saving `systemCall`/`name` off the freed `vm_t` before
    /// re-running `VM_Create` (`vm.cpp:399-409`) — the frozen signature is
    /// unchanged by resolution 2.
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
