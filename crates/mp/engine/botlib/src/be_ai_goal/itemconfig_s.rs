#![allow(non_camel_case_types, non_snake_case)]

use super::iteminfo_s::iteminfo_t;

/// Raven `itemconfig_t` — the loaded item configuration.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_goal.cpp:141-145`
#[repr(C)]
pub struct itemconfig_t {
    pub numiteminfo: i32,
    pub iteminfo: *mut iteminfo_t,
}

pub type itemconfig_s = itemconfig_t;

const _: () = assert!(core::mem::size_of::<itemconfig_t>() == 16);
const _: () = assert!(core::mem::offset_of!(itemconfig_t, numiteminfo) == 0);
const _: () = assert!(core::mem::offset_of!(itemconfig_t, iteminfo) == 8);
