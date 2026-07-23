#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use crate::shared::q_string::chars_str;

/// `MAX_STRINGFIELD`.
///
/// Source: `oracle/codemp/botlib/l_struct.h:16`
const MAX_STRINGFIELD: usize = 80;

/// Raven `projectileinfo_t` — bot AI weapon projectile info.
///
/// Type definition source: `oracle/codemp/game/be_ai_weap.h:27-43`
#[repr(C)]
pub struct projectileinfo_t {
    pub name: [c_char; MAX_STRINGFIELD],
    pub model: [c_char; MAX_STRINGFIELD],
    pub flags: i32,
    pub gravity: f32,
    pub damage: i32,
    pub radius: f32,
    pub visdamage: i32,
    pub damagetype: i32,
    pub healthinc: i32,
    pub push: f32,
    pub detonation: f32,
    pub bounce: f32,
    pub bouncefric: f32,
    pub bouncestop: f32,
}

impl projectileinfo_t {
    /// `name` as `&str` — decodes the live NUL-terminated array each call
    /// (gentity-`_str()` convention; array shape ABI-frozen per DEC-33). A
    /// missing NUL or non-UTF-8 bytes decode as `""`.
    pub fn name_str(&self) -> &str {
        chars_str(&self.name)
    }

    /// `model` as `&str` (see [`Self::name_str`]).
    pub fn model_str(&self) -> &str {
        chars_str(&self.model)
    }
}

pub type projectileinfo_s = projectileinfo_t;

const _: () = assert!(core::mem::size_of::<projectileinfo_t>() == 208);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, model) == 80);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, flags) == 160);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, gravity) == 164);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, damage) == 168);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, radius) == 172);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, visdamage) == 176);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, damagetype) == 180);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, healthinc) == 184);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, push) == 188);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, detonation) == 192);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, bounce) == 196);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, bouncefric) == 200);
const _: () = assert!(core::mem::offset_of!(projectileinfo_t, bouncestop) == 204);
