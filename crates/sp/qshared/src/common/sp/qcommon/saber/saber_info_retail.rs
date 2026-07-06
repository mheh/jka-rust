//! SP `saberInfoRetail_t`.
//!
//! Type definition source: `oracle/oracle/code/game/q_shared.h:1947-2062`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

use crate::shared::qboolean;

use super::blade_info::{bladeInfo_t, MAX_BLADES};
use super::saber_styles::saber_styles_t;
use super::saber_type::saberType_t;

/// Raven SP `saberInfoRetail_t` — the *retail* `saberInfo_t` layout.
///
/// Raven: "ONLY used for loading retail-version savegames (we load the savegame
/// into this smaller structure, then copy each field into the appropriate field
/// in the new structure — see `SG_ConvertRetailSaberinfoToNewSaberinfo()`)".
/// Its C++ inline methods (`Activate`, `Deactivate`, `BladeActivate`, `Active`,
/// `SetLength`, `Length`, `LengthMax`, `ActivateTrail`, `DeactivateTrail`) are
/// behavior, not layout, and are deferred to the SP savegame system.
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1947-2062`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct saberInfoRetail_t {
    pub name: *mut c_char,     // entry in sabers.cfg, if any
    pub fullName: *mut c_char, // the "Proper Name" of the saber, shown in the UI
    pub r#type: saberType_t,   // none, single or staff
    pub model: *mut c_char,    // hilt model
    pub skin: *mut c_char,     // hilt custom skin
    pub soundOn: c_int,
    pub soundLoop: c_int,
    pub soundOff: c_int,
    pub numBlades: c_int,
    pub blade: [bladeInfo_t; MAX_BLADES],
    pub style: saber_styles_t, // locked style to use, if any
    pub maxChain: c_int,
    pub lockable: qboolean,
    pub throwable: qboolean,
    pub disarmable: qboolean,
    pub activeBlocking: qboolean,
    pub twoHanded: qboolean,
    pub forceRestrictions: c_int,
    pub lockBonus: c_int,
    pub parryBonus: c_int,
    pub breakParryBonus: c_int,
    pub disarmBonus: c_int,
    pub singleBladeStyle: saber_styles_t,
    pub singleBladeThrowable: qboolean,
    pub brokenSaber1: *mut c_char, // replacement saber for right hand when cut in half/broken
    pub brokenSaber2: *mut c_char, // replacement saber for left hand when cut in half/broken
    pub returnDamage: qboolean,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberInfoRetail_t, model) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberInfoRetail_t, blade) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberInfoRetail_t, singleBladeStyle) == 1416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberInfoRetail_t, brokenSaber1) == 1424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(saberInfoRetail_t, returnDamage) == 1440);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<saberInfoRetail_t>() == 1448);
