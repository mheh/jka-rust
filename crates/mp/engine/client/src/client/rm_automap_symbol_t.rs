#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `rmAutomapSymbol_t` — an automap symbol marker.
///
/// Type definition source: `oracle/oracle/codemp/client/client.h:143-149`
#[repr(C)]
pub struct rmAutomapSymbol_t {
    pub mType: i32,
    pub mSide: i32,
    pub mOrigin: vec3_t,
}

const _: () = assert!(core::mem::size_of::<rmAutomapSymbol_t>() == 20);
const _: () = assert!(core::mem::offset_of!(rmAutomapSymbol_t, mType) == 0);
const _: () = assert!(core::mem::offset_of!(rmAutomapSymbol_t, mSide) == 4);
const _: () = assert!(core::mem::offset_of!(rmAutomapSymbol_t, mOrigin) == 8);
