//! `ModuleRegistry` — the `vmTable[MAX_VM]` slot registry (LOAD-D8).

use crate::common::error::{com_error, ErrorLevel};
use core::ffi::c_void;

use native_platform::entrypoints::RawSyscall;
use native_platform::module_loader::{
    sys_load_dll, unload_module, ModuleSearchPolicy, RestartKind,
};

use super::engine_slot::SlotSyscall;
use super::module_slot::ModuleSlot;
use super::slot_id::SlotId;

/// `#define MAX_VM 3` (`vm.cpp:28-29`), beside `ModuleRegistry` (mechanical
/// placement mirroring oracle).
pub const MAX_VM: usize = 3;

/// The `vmTable[MAX_VM]` replacement — per-slot only, no current-module global
/// (LOAD-D5). Home crate mirrors `vm.cpp`'s subsystem.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:28-29`
pub struct ModuleRegistry {
    slots: [Option<ModuleSlot>; MAX_VM],
}

/// The `Com_Init` step-30 `VM_Init` empty build (lifecycle.md: "a
/// `ModuleRegistry::default()`-shaped empty build"; Raven zeroes `vmTable`,
/// `vm.cpp:50-61`).
impl Default for ModuleRegistry {
    fn default() -> Self {
        ModuleRegistry {
            slots: [None, None, None],
        }
    }
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
    /// Source: `oracle/codemp/qcommon/vm.cpp:471-524`
    pub fn load_module(
        &mut self,
        policy: &ModuleSearchPolicy,
        name: &str,
        syscall: RawSyscall,
        system_calls: SlotSyscall,
        ctx: *mut c_void,
    ) -> Option<SlotId> {
        // Bad-parms guard (vm.cpp:480-482); `name.is_empty()` only — the
        // `!module`/`!systemCalls` disjuncts are structurally unreachable
        // (round-5 resolution).
        if name.is_empty() {
            com_error(ErrorLevel::ERR_FATAL, "VM_Create: bad parms".into());
        }
        // Reuse a live slot whose stored name matches case-insensitively
        // (Q_stricmp, vm.cpp:485-489) — returned as-is, NO reload.
        for (i, slot) in self.slots.iter().enumerate() {
            if let Some(s) = slot {
                if s.name.eq_ignore_ascii_case(name) {
                    return Some(SlotId(i as u32));
                }
            }
        }
        // First free slot (vm.cpp:493-494), else fatal when all MAX_VM are
        // full (vm.cpp:499-500).
        let Some(free) = self.slots.iter().position(|s| s.is_none()) else {
            com_error(ErrorLevel::ERR_FATAL, "VM_Create: no free vm_t".into());
        };
        // Fresh slot: run the loader; None = artifact not found — surfaced to
        // the caller, which owns the fatal disposition (LOAD-D11/LOAD-Q10;
        // sv_game.cpp:1750-1752).
        let module = native_platform::module_loader::sys_load_dll(policy, name, syscall)?;
        self.slots[free] = Some(ModuleSlot {
            name: name.to_string(),
            module,
            engine: super::engine_slot::EngineSlot {
                ctx,
                syscall: system_calls,
            },
        });
        Some(SlotId(free as u32))
    }

    /// `VM_Free` (`vm.cpp:605-610`): `unload_module` the slot's module, clearing
    /// it. No global `currentVM`/`lastVM` clobber (LOAD-D5).
    ///
    /// Source: `oracle/codemp/qcommon/vm.cpp:605-610`
    pub fn unload(&mut self, slot: SlotId) {
        if let Some(s) = self.slots[slot.0 as usize].take() {
            native_platform::module_loader::unload_module(s.module);
        }
    }

    /// Engine→module call into a loaded slot — Raven `VM_Call( vm_t *vm,
    /// int callnum, ... )` (`vm.cpp:787-819`): the native arm packs `int
    /// args[16]` and forwards to `vm->entryPoint`; the callee's fixed
    /// 12-word parameter list silently drops the extras, so this typed dual
    /// forwards `command` + exactly 12 words, extras zero-filled by the
    /// caller.
    ///
    /// PROVISIONAL SIGNATURE (checkpoint-7 finding): no frozen doc pins a
    /// `VM_Call` dual; minimal faithful shape pending its doc home.
    /// Source: `oracle/codemp/qcommon/vm.cpp:787-819`
    pub fn vm_call(&self, slot: &SlotId, command: core::ffi::c_int, args: [isize; 12]) -> isize {
        let s = self.slots[slot.0 as usize]
            .as_ref()
            .expect("vm_call on an empty slot");
        let entry = s.module.entry();
        entry(
            command, args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
            args[8], args[9], args[10], args[11],
        )
    }

    /// Native `VM_Restart` = drop+recreate in place (`vm.cpp:398-409`). `kind` is
    /// caller-supplied (LOAD-D12b); `policy`/`name`/`syscall` are needed for
    /// `DropRecreate`'s reload. The injected `system_calls`/`ctx` are NOT
    /// re-supplied: the slot's stored `EngineSlot` is reused, mirroring Raven's
    /// native arm saving `systemCall`/`name` off the freed `vm_t` before
    /// re-running `VM_Create` (`vm.cpp:399-409`) — the frozen signature is
    /// unchanged by resolution 2.
    ///
    /// Source: `oracle/codemp/qcommon/vm.cpp:391-458`
    pub fn restart(
        &mut self,
        slot: SlotId,
        kind: RestartKind,
        policy: &ModuleSearchPolicy,
        name: &str,
        syscall: RawSyscall,
    ) {
        match kind {
            // DLL's can't be restarted in place (vm.cpp:398): the native arm
            // saves `systemCall`/`name` off the vm, `VM_Free`s it, then re-runs
            // `VM_Create` (vm.cpp:399-409). The QVM in-place reload arm
            // (vm.cpp:412-457) is out of scope (native-only). Here the slot's
            // stored `EngineSlot` stands in for the saved `systemCall`, and the
            // recreate lands back in the same slot (drop+recreate in place).
            RestartKind::DropRecreate => {
                let Some(old) = self.slots[slot.0 as usize].take() else {
                    return;
                };
                // Save the injected engine seam before freeing (vm.cpp:403).
                let engine = old.engine;
                // VM_Free (vm.cpp:406): unload the old artifact.
                unload_module(old.module);
                // VM_Create (vm.cpp:408): reload, reinjecting the saved seam.
                // None = artifact not found (mirrors `sys_load_dll`'s contract);
                // the slot is left empty for the caller's fatal disposition.
                self.slots[slot.0 as usize] =
                    sys_load_dll(policy, name, syscall).map(|module| ModuleSlot {
                        name: name.to_string(),
                        module,
                        engine,
                    });
            }
        }
    }
}
