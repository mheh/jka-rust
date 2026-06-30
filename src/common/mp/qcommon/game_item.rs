//! MP `gitem_t` copied from Raven `codemp/game/bg_public.h`.
//!
//! Source: `oracle/oracle/codemp/game/bg_public.h:1105-1138`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

pub const MAX_ITEM_MODELS: usize = 4;

/// Raven MP `itemType_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:1105-1118`
pub type itemType_t = c_int;

/// Raven MP `gitem_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:1122-1138`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct gitem_t {
    /// Spawning name.
    pub classname: *mut c_char,
    pub pickup_sound: *mut c_char,
    pub world_model: [*mut c_char; MAX_ITEM_MODELS],
    pub view_model: *mut c_char,
    pub icon: *mut c_char,
    /// For ammo how much, or duration of powerup.
    pub quantity: c_int,
    /// IT_* flags.
    pub giType: itemType_t,
    pub giTag: c_int,
    /// String of all models and images this item will use.
    pub precaches: *mut c_char,
    /// String of all sounds this item will use.
    pub sounds: *mut c_char,
    pub description: *mut c_char,
}
