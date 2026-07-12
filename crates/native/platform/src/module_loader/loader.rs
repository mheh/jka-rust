//! `sys_load_dll` / `unload_module` — the loading MECHANISM (LOAD-D1).
//!
//! One `libloading`-based loader executes a mode-supplied policy value.
//! `libloading` and all OS types are confined to this crate (LOAD-D4).

use super::loaded_module::LoadedModule;
use super::search_policy::ModuleSearchPolicy;
use super::search_step::SearchStep;
use crate::entrypoints::{RawSyscall, RawVmMain};

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
/// Source: `oracle/codemp/unix/unix_main.c:431-436`.
///
/// Source: `oracle/codemp/win32/win_main.cpp:811-887`
pub fn sys_load_dll(
    policy: &ModuleSearchPolicy,
    name: &str,
    syscall: RawSyscall,
) -> Option<LoadedModule> {
    use crate::entrypoints::RawDllEntry;

    //TODO: Port Sys_UnpackDLL — pure-server pk3 unpack pre-step, deferred (LOAD-D7)
    // Source: oracle/codemp/win32/win_main.cpp:762-800,849-852

    // Filename synthesis (win_main.cpp:826 / unix_main.c:346). A `None` suffix
    // is the LOAD-Q1 unresolved macOS arm — no filename can be synthesized.
    let suffix = policy.naming.suffix?;
    let filename = format!("{name}{suffix}");

    // Win32-only direct probe (win_main.cpp:855); Unix policies set
    // `direct_first: false` (unix_main.c:361-373 `#if 0`).
    let mut lib: Option<libloading::Library> = None;
    if policy.direct_first {
        lib = unsafe { libloading::Library::new(&filename).ok() };
    }
    // Ordered FS_BuildOSPath probes ("<base>/<gamedir>/<file>",
    // files.cpp:479-498); steps walked blindly (caller omitted empty-base
    // steps, LOAD-D9), first hit wins (win_main.cpp:858-869).
    if lib.is_none() {
        for step in &policy.steps {
            let SearchStep::FsPath { base, gamedir } = step;
            let path = base.join(gamedir).join(&filename);
            if let Ok(l) = unsafe { libloading::Library::new(&path) } {
                lib = Some(l);
                break;
            }
        }
    }
    let lib = lib?;

    // Handshake (win_main.cpp:879-887): "dllEntry" + "vmMain" both required;
    // on a miss the library is freed (drop) and None returned.
    let (dll_entry, entry): (RawDllEntry, RawVmMain) = unsafe {
        let de = lib.get::<RawDllEntry>(b"dllEntry\0").ok();
        let vm = lib.get::<RawVmMain>(b"vmMain\0").ok();
        match (de, vm) {
            (Some(d), Some(v)) => (*d, *v),
            _ => {
                // Debug arm: print + None (unix_main.c:433-435). The release
                // (`cfg(not(debug_assertions))`) in-loader receiverless fatal
                // dual of unix_main.c:431-436 is sanctioned-open:
                //TODO: Port NDEBUG in-loader fatal (LOAD-Q13 mechanism)
                // Source: oracle/codemp/unix/unix_main.c:431-436
                eprintln!("Sys_LoadDll({name}) failed dlsym(vmMain/dllEntry)");
                return None;
            }
        }
    };
    // Hand the module the engine syscall trampoline (win_main.cpp:887).
    dll_entry(syscall);
    Some(LoadedModule { lib, entry })
}

/// Faithful to `Sys_UnloadDll` via `VM_Free` (`vm.cpp:605-610`): drop the
/// library, clearing the slot. No global `currentVM`/`lastVM` clobber (LOAD-D5).
///
/// Source: `oracle/codemp/qcommon/vm.cpp:605-610`
pub fn unload_module(module: LoadedModule) {
    // Drop of the `libloading::Library` IS the Sys_UnloadDll/FreeLibrary
    // (vm.cpp:605-610); no global currentVM/lastVM clobber (LOAD-D5).
    drop(module);
}
