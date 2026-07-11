#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_long, c_void};

use crate::shared::{fileHandle_t, fsMode_t, vec3_t};

use super::bsp_trace_s::bsp_trace_t;

/// Raven `botlib_import_t` — engine services imported by the bot library.
///
/// Type definition source: `oracle/codemp/game/botlib.h:157-193`
// `Copy`/`Clone`: the engine assigns the whole table by value
// (`bot.botimport = *import` in `GetBotLibAPI`), matching Raven's C struct copy.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct botlib_import_t {
    /// print messages from the bot library
    pub Print: Option<unsafe extern "C" fn(r#type: c_int, fmt: *mut c_char, ...)>,
    /// trace a bbox through the world
    pub Trace: Option<
        unsafe extern "C" fn(
            trace: *mut bsp_trace_t,
            start: *mut vec3_t,
            mins: *mut vec3_t,
            maxs: *mut vec3_t,
            end: *mut vec3_t,
            passent: c_int,
            contentmask: c_int,
        ),
    >,
    /// trace a bbox against a specific entity
    pub EntityTrace: Option<
        unsafe extern "C" fn(
            trace: *mut bsp_trace_t,
            start: *mut vec3_t,
            mins: *mut vec3_t,
            maxs: *mut vec3_t,
            end: *mut vec3_t,
            entnum: c_int,
            contentmask: c_int,
        ),
    >,
    /// retrieve the contents at the given point
    pub PointContents: Option<unsafe extern "C" fn(point: *mut vec3_t) -> c_int>,
    /// check if the point is in potential visible sight
    pub inPVS: Option<unsafe extern "C" fn(p1: *mut vec3_t, p2: *mut vec3_t) -> c_int>,
    /// retrieve the BSP entity data lump
    pub BSPEntityData: Option<unsafe extern "C" fn() -> *mut c_char>,
    pub BSPModelMinsMaxsOrigin: Option<
        unsafe extern "C" fn(
            modelnum: c_int,
            angles: *mut vec3_t,
            mins: *mut vec3_t,
            maxs: *mut vec3_t,
            origin: *mut vec3_t,
        ),
    >,
    /// send a bot client command
    pub BotClientCommand: Option<unsafe extern "C" fn(client: c_int, command: *mut c_char)>,
    /// memory allocation
    /// allocate from Zone
    pub GetMemory: Option<unsafe extern "C" fn(size: c_int) -> *mut c_void>,
    /// free memory from Zone
    pub FreeMemory: Option<unsafe extern "C" fn(ptr: *mut c_void)>,
    /// available Zone memory
    pub AvailableMemory: Option<unsafe extern "C" fn() -> c_int>,
    /// allocate from hunk
    pub HunkAlloc: Option<unsafe extern "C" fn(size: c_int) -> *mut c_void>,
    /// file system access
    pub FS_FOpenFile: Option<
        unsafe extern "C" fn(
            qpath: *const c_char,
            file: *mut fileHandle_t,
            mode: fsMode_t,
        ) -> c_int,
    >,
    pub FS_Read:
        Option<unsafe extern "C" fn(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int>,
    pub FS_Write:
        Option<unsafe extern "C" fn(buffer: *const c_void, len: c_int, f: fileHandle_t) -> c_int>,
    pub FS_FCloseFile: Option<unsafe extern "C" fn(f: fileHandle_t)>,
    pub FS_Seek:
        Option<unsafe extern "C" fn(f: fileHandle_t, offset: c_long, origin: c_int) -> c_int>,
    /// debug visualisation stuff
    pub DebugLineCreate: Option<unsafe extern "C" fn() -> c_int>,
    pub DebugLineDelete: Option<unsafe extern "C" fn(line: c_int)>,
    pub DebugLineShow: Option<
        unsafe extern "C" fn(line: c_int, start: *mut vec3_t, end: *mut vec3_t, color: c_int),
    >,
    pub DebugPolygonCreate:
        Option<unsafe extern "C" fn(color: c_int, numPoints: c_int, points: *mut vec3_t) -> c_int>,
    pub DebugPolygonDelete: Option<unsafe extern "C" fn(id: c_int)>,
}

pub type botlib_import_s = botlib_import_t;

const _: () = assert!(core::mem::size_of::<botlib_import_t>() == 176);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, Print) == 0);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, Trace) == 8);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, EntityTrace) == 16);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, PointContents) == 24);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, inPVS) == 32);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, BSPEntityData) == 40);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, BSPModelMinsMaxsOrigin) == 48);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, BotClientCommand) == 56);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, GetMemory) == 64);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, FreeMemory) == 72);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, AvailableMemory) == 80);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, HunkAlloc) == 88);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, FS_FOpenFile) == 96);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, FS_Read) == 104);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, FS_Write) == 112);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, FS_FCloseFile) == 120);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, FS_Seek) == 128);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, DebugLineCreate) == 136);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, DebugLineDelete) == 144);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, DebugLineShow) == 152);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, DebugPolygonCreate) == 160);
const _: () = assert!(core::mem::offset_of!(botlib_import_t, DebugPolygonDelete) == 168);
