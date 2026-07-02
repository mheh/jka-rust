#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_qshared::shared::MAX_QPATH;

/// Raven `dfog_t` — on-disk BSP fog volume.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/../qcommon/qfiles.h:493-497`
#[repr(C)]
pub struct dfog_t {
    pub shader: [c_char; MAX_QPATH],
    pub brushNum: c_int,
    /// the brush side that ray tests need to clip against (-1 == none)
    pub visibleSide: c_int,
}

const _: () = assert!(core::mem::size_of::<dfog_t>() == 72);
const _: () = assert!(core::mem::offset_of!(dfog_t, shader) == 0);
const _: () = assert!(core::mem::offset_of!(dfog_t, brushNum) == 64);
const _: () = assert!(core::mem::offset_of!(dfog_t, visibleSide) == 68);
