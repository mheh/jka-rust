//! `EngineSlot` — the per-module-slot injected engine syscall target (SEAM-D11,
//! amended by skeleton-findings resolution 2, 2026-07-03).
//!
//! SEAM-D11 specced the slot's cell as `Cell<*mut Engine>`, but `Engine` =
//! `mp_engine_core::Engine` and the typed dispatcher lives in
//! `mp_engine_server` — both uphill of this crate. Settled resolution: the slot
//! stores **injected** state — an opaque ctx pointer plus the syscall fn
//! pointer, both passed in at module-load time — mirroring Raven, where
//! `VM_Create` *receives* `systemCalls` as an argument instead of naming the
//! server. No crate-graph change; qcommon still never names `mp_engine_core` or
//! `mp_engine_server`. The trampoline itself lives in `trampoline.rs` +
//! `game_syscall_trampoline.c` (resolution 1).

use core::ffi::c_void;

/// The engine-side syscall dispatch fn injected at module load — Raven's
/// `systemCalls` parameter (`int (*systemCalls)(int *)`, `vm.cpp:471-472`,
/// stored `vm->systemCall = systemCalls`, `vm.cpp:506`), widened with the
/// opaque `ctx` so the injecting caller (which owns the typed engine state) can
/// recover it without this crate naming that state. `extern "C-unwind"` per
/// SEAM-D12.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:471-472,506`
pub type SlotSyscall = extern "C-unwind" fn(ctx: *mut c_void, args: *const isize) -> isize;

/// One per hosted module slot: the injected engine ctx + dispatch target the
/// slot's raw trampoline (`trampoline.rs` / `game_syscall_trampoline.c`)
/// forwards to. Both fields are injected at module-load time through
/// `ModuleRegistry::load_module` (resolution 2, 2026-07-03 — supersedes the
/// per-call `Cell<*mut Engine>` + `EngineSlotGuard` shape). The porting-rules
/// §D11 engine-side seam exemption, the twin of the module shell's
/// `OnceLock<CEngine>` (SEAM-D1), one per slot (STATE-D2).
///
/// Source: `oracle/codemp/qcommon/vm.cpp:471-472,506` (`VM_Create`
/// receiving + storing `systemCalls`).
pub struct EngineSlot {
    /// Opaque engine ctx handed back to `syscall` on every trampoline forward.
    pub(crate) ctx: *mut c_void,
    /// The injected dispatch target — Raven `vm->systemCall` (`vm.cpp:506`).
    pub(crate) syscall: SlotSyscall,
}
