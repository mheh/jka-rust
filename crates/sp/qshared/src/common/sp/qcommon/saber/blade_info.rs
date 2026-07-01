//! SP `bladeInfo_t` and `MAX_BLADES`.
//!
//! Type definition source: `oracle/oracle/code/game/q_shared.h:1634-1658`

#![allow(non_camel_case_types)]

use crate::shared::{qboolean, vec3_t};

use super::saber_colors::saber_colors_t;
use super::saber_trail::saberTrail_t;

/// Raven SP `bladeInfo_t` — one blade of a saber.
///
/// Diverges from MP: SP lacks `desiredLength` and the `hitWallDebounceTime`/
/// `storageTime`/`extendDebounce` timing ints MP added.
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1634-1657`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct bladeInfo_t {
    pub active: qboolean,
    pub color: saber_colors_t,
    pub radius: f32,
    pub length: f32,
    pub lengthMax: f32,
    pub lengthOld: f32,
    pub muzzlePoint: vec3_t,
    pub muzzlePointOld: vec3_t,
    pub muzzleDir: vec3_t,
    pub muzzleDirOld: vec3_t,
    pub trail: saberTrail_t,
}
const _: () = assert!(core::mem::size_of::<bladeInfo_t>() == 164);

/// Raven SP `MAX_BLADES`.
///
/// Source: `oracle/oracle/code/game/q_shared.h:1658`
pub const MAX_BLADES: usize = 8;
