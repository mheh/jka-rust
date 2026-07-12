//! SP `gitem_t` copied from Raven `code/game/bg_public.h`.
//!
//! Source: `oracle/code/game/bg_public.h:622-658`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use crate::shared::vec3_t;

/// Raven SP `itemType_t`.
///
/// Type definition source: `oracle/code/game/bg_public.h:622-634`
pub type itemType_t = c_int;

/// Raven SP `gitem_t`.
///
/// Type definition source: `oracle/code/game/bg_public.h:638-658`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct gitem_t {
    /// Spawning name.
    pub classname: *mut c_char,
    pub pickup_sound: *mut c_char,
    pub world_model: *mut c_char,
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
    /// Bbox.
    pub mins: vec3_t,
    /// Bbox.
    pub maxs: vec3_t,
    // Raven guards the two fields below with `#ifdef _IMMERSION`; the SP game
    // vcproj defines _IMMERSION in every configuration, so they are part of
    // the real shipped layout (104 B, not 88).
    pub pickup_force: *mut c_char,
    pub forces: *mut c_char,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<gitem_t>() == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gitem_t, classname) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gitem_t, icon) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gitem_t, quantity) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gitem_t, giTag) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gitem_t, precaches) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gitem_t, maxs) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gitem_t, pickup_force) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gitem_t, forces) == 96);
