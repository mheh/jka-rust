//! `sys_load_dll` / `unload_module` — the loading MECHANISM (LOAD-D1).
//!
//! One `libloading`-based loader executes a mode-supplied policy value.
//! `libloading` and all OS types are confined to this crate (LOAD-D4).

use super::loaded_module::LoadedModule;
use super::search_policy::ModuleSearchPolicy;
use crate::entrypoints::RawSyscall;

/// Faithful to `Sys_LoadDll` (`win_main.cpp:811-887`) MINUS the pure-server
/// `Sys_UnpackDLL` pre-step (`:849-852`), which is IN SCOPE but DEFERRED to a
/// later MP-server slice (LOAD-D7). Apply naming, then walk `policy.steps` in
/// order, blindly (the caller has already omitted any empty-base step,
/// LOAD-D9 round-3), first hit wins. At a hit, resolve `"dllEntry"`+`"vmMain"`
/// (both required) and call `dllEntry(syscall)`, returning
/// `LoadedModule { lib, entry }`. `None` = not found (Raven's QVM fallback is
/// out of scope, DEC-05.4; the caller decides fatal-vs-skip per mode).
///
/// **Missing-export handshake arm — two build-mode arms** (round-6 item-18
/// resolution, reproducing Raven's Unix `#ifdef NDEBUG` split faithfully,
/// porting-rules §20): debug builds print (`Com_Printf`) and return `None`;
/// release builds (`cfg(not(debug_assertions))`) raise the in-loader
/// receiverless `com_error(ERR_FATAL, "Sys_LoadDll(%s) failed dlsym(vmMain): …")`.
/// Source: `oracle/oracle/codemp/unix/unix_main.c:431-436`.
///
/// Source: `oracle/oracle/codemp/win32/win_main.cpp:811-887`
pub fn sys_load_dll(
    _policy: &ModuleSearchPolicy,
    _name: &str,
    _syscall: RawSyscall,
) -> Option<LoadedModule> {
    todo!("Port Sys_LoadDll — oracle/oracle/codemp/win32/win_main.cpp:811-887")
    //TODO: Port Sys_UnpackDLL — oracle/oracle/codemp/win32/win_main.cpp:762-800 (LOAD-D7)
}

/// Faithful to `Sys_UnloadDll` via `VM_Free` (`vm.cpp:605-610`): drop the
/// library, clearing the slot. No global `currentVM`/`lastVM` clobber (LOAD-D5).
///
/// Source: `oracle/oracle/codemp/qcommon/vm.cpp:605-610`
pub fn unload_module(_module: LoadedModule) {
    // Drop of `LoadedModule` releases the `libloading::Library` handle.
    //TODO: Port Sys_UnloadDll semantics beyond the Drop — oracle/oracle/codemp/qcommon/vm.cpp:605-610
}
