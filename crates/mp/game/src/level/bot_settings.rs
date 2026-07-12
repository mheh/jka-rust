//! MP `bot_settings_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:1490-1496`

#![allow(non_camel_case_types)]

use core::ffi::c_char;

/// Raven `MAX_FILEPATH`. Source: `oracle/codemp/game/g_local.h:1480`
pub const MAX_FILEPATH: usize = 144;

/// Raven `bot_settings_t`.
///
/// Type definition source: `oracle/codemp/game/g_local.h:1490-1496`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct bot_settings_t {
    pub personalityfile: [c_char; MAX_FILEPATH],
    pub skill: f32,
    pub team: [c_char; MAX_FILEPATH],
}
const _: () = assert!(core::mem::size_of::<bot_settings_t>() == 292);
