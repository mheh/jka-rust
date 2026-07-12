#![allow(non_camel_case_types, non_snake_case)]

use crate::be_ai_weight::weightconfig_s::weightconfig_t;

/// Raven `bot_weaponstate_t` — the weapon state of a single bot.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_weap.cpp:105-109`
#[repr(C)]
pub struct bot_weaponstate_t {
    /// weapon weight configuration
    pub weaponweightconfig: *mut weightconfig_t,
    /// weapon weight index
    pub weaponweightindex: *mut i32,
}

pub type bot_weaponstate_s = bot_weaponstate_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_weaponstate_t>() == 16);
    assert!(core::mem::offset_of!(bot_weaponstate_t, weaponweightconfig) == 0);
    assert!(core::mem::offset_of!(bot_weaponstate_t, weaponweightindex) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_weaponstate_t>() == 8);
    assert!(core::mem::offset_of!(bot_weaponstate_t, weaponweightconfig) == 0);
    assert!(core::mem::offset_of!(bot_weaponstate_t, weaponweightindex) == 4);
};
