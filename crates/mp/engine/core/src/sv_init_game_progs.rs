//! The `SV_InitGameProgs`-equivalent load call site (Slice-0 provisional).
//!
//! PROVISIONAL CRATE + SIGNATURE — **LOAD-Q12 is open** (module-loading.md:
//! the equiv's own crate/signature is pinned by neither doc; the settled
//! *trigger* is map spawn, `SV_SpawnServer → SV_InitGameProgs`, post-Slice-0).
//! Placed here on the same reasoning that puts `sv_frame` in the facade
//! (state-ownership § entry points: it needs `Engine` + the registry + the
//! loader, which no leaf crate can reach together); the Slice-0 acceptance
//! driver calls it directly from `main()` pending the map-spawn wiring.
//! Reported as needsSession — do not treat this placement as frozen.

use core::ffi::c_void;

use mp_engine_qcommon::common::error::{com_error, ErrorLevel};
use mp_engine_qcommon::vm::{arm_game_slot, game_syscall_trampoline, SlotId};
use mp_engine_server::game_system_calls_shim;
use native_platform::entrypoints::RawSyscall;
use native_platform::module_loader::ModuleSearchPolicy;

use crate::engine::Engine;

/// Raven MP `SV_InitGameProgs` (`sv_game.cpp:1734-1753`), Slice-0 subset:
/// `gvm = VM_Create("jampgame", SV_GameSystemCalls, …)` → the frozen
/// `load_module(policy, "jampgame", syscall, system_calls, ctx)`; the `None`
/// (not-found) disposition is the CALLER's, reproduced exactly:
/// `if (!gvm) Com_Error(ERR_FATAL, "VM_Create on game failed")`
/// (`sv_game.cpp:1750-1752`, the LOAD-D11/LOAD-Q10 ground truth). The
/// GAME_INIT round-trip itself is the caller's next step (`SV_InitGameVM` →
/// `VM_Call(gvm, GAME_INIT, …)`, `sv_game.cpp:1690`).
///
/// `syscall` handed to the module handshake is the SEAM-D11 C trampoline
/// (`game_syscall_trampoline`, our `VM_DllSyscall` dual, `vm.cpp:515-518`);
/// the injected `(system_calls, ctx)` pair is `VM_Create`'s `systemCalls`
/// argument widened per the checkpoint-2 resolution. `ctx` is null pending the
/// `ServerGame` reborrow wiring (its concrete shape is unpinned — finding).
///
/// Source: `oracle/codemp/server/sv_game.cpp:1734-1753`
pub fn sv_init_game_progs(engine: &mut Engine, policy: &ModuleSearchPolicy) -> SlotId {
    // The raw C trampoline address (VM_DllSyscall dual) for dllEntry's hand-off.
    let syscall: RawSyscall =
        game_syscall_trampoline as unsafe extern "C-unwind" fn(isize, ...) -> isize as usize
            as *const c_void;
    //TODO: Port SV_InitGameProgs ctx injection (&mut Engine.sv into the game slot)
    // Source: docs/architecture/engine-seam.md § Engine-side dispatchers
    let ctx: *mut c_void = core::ptr::null_mut();

    // Arm the game slot's trampoline cell with the same injected pair
    // (checkpoint-7 provisional bridge — see vm/trampoline.rs).
    arm_game_slot(ctx, game_system_calls_shim);

    let slot =
        engine
            .common
            .modules
            .load_module(policy, "jampgame", syscall, game_system_calls_shim, ctx);
    // Caller-side fatal disposition (sv_game.cpp:1750-1752).
    let Some(slot) = slot else {
        com_error(ErrorLevel::ERR_FATAL, "VM_Create on game failed".into());
    };
    slot
}
