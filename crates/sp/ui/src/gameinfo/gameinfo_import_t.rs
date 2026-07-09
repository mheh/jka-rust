#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use sp_qshared::shared::{fileHandle_t, fsMode_t};

/// Raven `gameinfo_import_t` — engine import table for the SP gameinfo module.
///
/// Type definition source: `oracle/code/ui/gameinfo.h:9-19`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct gameinfo_import_t {
    pub FS_FOpenFile: Option<
        unsafe extern "C" fn(qpath: *const c_char, file: *mut fileHandle_t, mode: fsMode_t) -> c_int,
    >,
    pub FS_Read: Option<unsafe extern "C" fn(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int>,
    pub FS_FCloseFile: Option<unsafe extern "C" fn(f: fileHandle_t)>,
    pub Cvar_Set: Option<unsafe extern "C" fn(name: *const c_char, value: *const c_char)>,
    pub Cvar_VariableStringBuffer: Option<
        unsafe extern "C" fn(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int),
    >,
    pub Cvar_Create:
        Option<unsafe extern "C" fn(var_name: *const c_char, var_value: *const c_char, flags: c_int)>,
    pub FS_ReadFile: Option<unsafe extern "C" fn(name: *const c_char, buf: *mut *mut c_void) -> c_int>,
    pub FS_FreeFile: Option<unsafe extern "C" fn(buf: *mut c_void)>,
    //TODO: Port Printf variadic args
    // Source: oracle/code/ui/gameinfo.h:18
    pub Printf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
}

const _: () = assert!(core::mem::size_of::<gameinfo_import_t>() == 72);
const _: () = assert!(core::mem::offset_of!(gameinfo_import_t, FS_FOpenFile) == 0);
const _: () = assert!(core::mem::offset_of!(gameinfo_import_t, FS_Read) == 8);
const _: () = assert!(core::mem::offset_of!(gameinfo_import_t, FS_FCloseFile) == 16);
const _: () = assert!(core::mem::offset_of!(gameinfo_import_t, Cvar_Set) == 24);
const _: () = assert!(core::mem::offset_of!(gameinfo_import_t, Cvar_VariableStringBuffer) == 32);
const _: () = assert!(core::mem::offset_of!(gameinfo_import_t, Cvar_Create) == 40);
const _: () = assert!(core::mem::offset_of!(gameinfo_import_t, FS_ReadFile) == 48);
const _: () = assert!(core::mem::offset_of!(gameinfo_import_t, FS_FreeFile) == 56);
const _: () = assert!(core::mem::offset_of!(gameinfo_import_t, Printf) == 64);
