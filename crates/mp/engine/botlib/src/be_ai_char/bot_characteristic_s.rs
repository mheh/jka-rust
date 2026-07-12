#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use super::cvalue::cvalue;

/// Raven `bot_characteristic_t` — a single bot characteristic (typed value).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_char.cpp:46-50`
#[repr(C)]
pub struct bot_characteristic_t {
    /// characteristic type
    pub r#type: c_char,
    /// characteristic value
    pub value: cvalue,
}

pub type bot_characteristic_s = bot_characteristic_t;

const _: () = assert!(core::mem::size_of::<bot_characteristic_t>() == 16);
const _: () = assert!(core::mem::offset_of!(bot_characteristic_t, r#type) == 0);
const _: () = assert!(core::mem::offset_of!(bot_characteristic_t, value) == 8);
