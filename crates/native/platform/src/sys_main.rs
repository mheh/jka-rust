//! `Sys_*` process/OS entrypoints transcribed from Raven's unix main layer.
//!
//! The deliberately-native twin of `oracle/codemp/unix/unix_main.c`: the memory
//! probe, the dylib unloader, and the (no-async) streamed-file wrappers.
//!
//! Source: `oracle/codemp/unix/unix_main.c:56-61, 300-311, 744-764`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use native_types::{fileHandle_t, qboolean, qfalse, qtrue};

/// `Sys_LowPhysicalMemory` (unix): the Win32 `MEMORYSTATUS` probe is stubbed to
/// `qfalse` in Raven's unix build.
///
/// Source: `oracle/codemp/unix/unix_main.c:56-61`
pub fn Sys_LowPhysicalMemory() -> qboolean {
    qfalse
}

/// `Sys_CheckCD` (unix): always reports the disc present.
///
/// Source: `oracle/codemp/unix/unix_main.c:1056-1058`
pub fn Sys_CheckCD() -> qboolean {
    qtrue
}

/// `Sys_UnloadDll` (unix): `dlclose` the handle (no-op on NULL). The verbose
/// `dlerror` reporting Raven does through `Com_Printf` is dropped — that print
/// seam is not reachable from this base-tier crate.
///
/// Source: `oracle/codemp/unix/unix_main.c:300-311`
pub fn Sys_UnloadDll(dllHandle: *mut c_void) {
    if dllHandle.is_null() {
        return;
    }
    unsafe {
        libc::dlclose(dllHandle);
    }
}

/// `Sys_BeginStreamedFile` (unix): a no-op in the non-async (`#if 1`) build.
///
/// Source: `oracle/codemp/unix/unix_main.c:752-753`
pub fn Sys_BeginStreamedFile(_f: fileHandle_t, _readAhead: c_int) {}

/// `Sys_EndStreamedFile` (unix): a no-op in the non-async (`#if 1`) build.
///
/// Source: `oracle/codemp/unix/unix_main.c:755-756`
pub fn Sys_EndStreamedFile(_f: fileHandle_t) {}

/// Raven unix `Sys_Quit`'s process tail: restore blocking stdin
/// (`fcntl(0, F_SETFL, … & ~FNDELAY)`) and `Sys_Exit` — the NDEBUG `_exit`
/// branch, Raven's regular behavior. The `CL_Shutdown()` head stays with the
/// caller (a hook the platform tier cannot reach).
///
/// Source: `oracle/codemp/unix/unix_main.c:140-158`
pub fn Sys_Exit_restore_stdin(ex: c_int) -> ! {
    unsafe {
        let fl = libc::fcntl(0, libc::F_GETFL, 0);
        libc::fcntl(0, libc::F_SETFL, fl & !libc::O_NDELAY);
        libc::_exit(ex);
    }
}

/// Raven `Sys_GetClipboardData` (unix): the oracle's unix build never wires up
/// an X11 clipboard, so the body always returns `NULL`.
///
/// Source: `oracle/codemp/unix/unix_main.c:1063-1066`
pub fn Sys_GetClipboardData() -> *mut c_char {
    core::ptr::null_mut()
}

/// Raven `Sys_MonkeyShouldBeSpanked` (unix): the "monkey test" packet-fuzzer
/// gate always reports off.
///
/// Source: `oracle/codemp/unix/unix_main.c:83-87`
pub fn Sys_MonkeyShouldBeSpanked() -> c_int {
    0
}

/// Raven unix `Sys_GetCurrentUser` — `getpwuid(getuid())->pw_name`, or
/// `"player"` when there is no passwd entry.
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:342-350`
pub fn Sys_GetCurrentUser() -> String {
    unsafe {
        let p = libc::getpwuid(libc::getuid());
        if p.is_null() {
            return "player".to_string();
        }
        core::ffi::CStr::from_ptr((*p).pw_name)
            .to_string_lossy()
            .into_owned()
    }
}
