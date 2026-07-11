#![allow(non_camel_case_types, non_snake_case)]

use super::sys_event_type_t::sysEventType_t;

/// Raven `sysEvent_t` — a single input/system event pulled from the platform
/// event queue (key, mouse, console, packet, etc.).
///
/// Type definition source: `oracle/code/qcommon/qcommon.h:744-750`
#[repr(C)]
pub struct sysEvent_t {
    pub evTime: i32,
    pub evType: sysEventType_t,
    pub evValue: i32,
    pub evValue2: i32,
    pub evPtrLength: i32, // bytes of data pointed to by evPtr, for journaling
    pub evPtr: *mut ::core::ffi::c_void, // this must be manually freed if not NULL
}

const _: () = assert!(core::mem::size_of::<sysEvent_t>() == 32);
const _: () = assert!(core::mem::offset_of!(sysEvent_t, evTime) == 0);
const _: () = assert!(core::mem::offset_of!(sysEvent_t, evType) == 4);
const _: () = assert!(core::mem::offset_of!(sysEvent_t, evValue) == 8);
const _: () = assert!(core::mem::offset_of!(sysEvent_t, evValue2) == 12);
const _: () = assert!(core::mem::offset_of!(sysEvent_t, evPtrLength) == 16);
const _: () = assert!(core::mem::offset_of!(sysEvent_t, evPtr) == 24);
