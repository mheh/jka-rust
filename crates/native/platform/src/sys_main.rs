//! `Sys_*` process/OS entrypoints transcribed from Raven's unix main layer.
//!
//! The deliberately-native twin of `oracle/codemp/unix/unix_main.c`: the memory
//! probe, the dylib unloader, and the (no-async) streamed-file wrappers.
//!
//! Source: `oracle/codemp/unix/unix_main.c:56-61, 300-311, 744-764`

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

use native_types::{fileHandle_t, qboolean, qfalse};

/// `Sys_LowPhysicalMemory` (unix): the Win32 `MEMORYSTATUS` probe is stubbed to
/// `qfalse` in Raven's unix build.
///
/// Source: `oracle/codemp/unix/unix_main.c:56-61`
pub fn Sys_LowPhysicalMemory() -> qboolean {
    qfalse
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
