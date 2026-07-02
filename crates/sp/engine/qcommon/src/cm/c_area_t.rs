#![allow(non_camel_case_types, non_snake_case)]

/// Raven `cArea_t` — flood-fill area tracking for connectivity queries.
///
/// Type definition source: `oracle/oracle/code/qcommon/cm_local.h:95-98`
#[repr(C)]
pub struct cArea_t {
    pub floodnum: i32,
    pub floodvalid: i32,
}

const _: () = assert!(core::mem::size_of::<cArea_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cArea_t, floodnum) == 0);
const _: () = assert!(core::mem::offset_of!(cArea_t, floodvalid) == 4);
