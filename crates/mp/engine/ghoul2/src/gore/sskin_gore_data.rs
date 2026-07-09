#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int};

use mp_qshared::shared::{qboolean, vec3_t};

/// Raven `SSkinGoreData` — one gore-mark (splotch) request applied to a Ghoul2 skin.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:3112-3144`
/// (dup: `oracle/codemp/ghoul2/ghoul2_shared.h:202`)
#[repr(C)]
pub struct SSkinGoreData {
    pub angles: vec3_t,
    pub position: vec3_t,
    pub currentTime: c_int,
    pub entNum: c_int,
    /// in world space
    pub rayDirection: vec3_t,
    /// in world space
    pub hitLocation: vec3_t,
    pub scale: vec3_t,
    /// size of splotch in the S texture direction in world units
    pub SSize: c_float,
    /// size of splotch in the T texture direction in world units
    pub TSize: c_float,
    /// angle to rotate the splotch
    pub theta: c_float,

    // growing stuff
    /// time over which we want this to scale up, set to -1 for no scaling
    pub growDuration: c_int,
    /// fraction of the final size at which we want the gore to initially appear
    pub goreScaleStartFraction: c_float,

    pub frontFaces: qboolean,
    pub backFaces: qboolean,
    pub baseModelOnly: qboolean,
    /// effect expires after this amount of time
    pub lifeTime: c_int,
    /// duration of fading, counted back from lifeTime
    pub fadeOutTime: c_int,
    /// unimplemented
    pub shrinkOutTime: c_int,
    /// unimplemented
    pub alphaModulate: c_float,
    /// unimplemented
    pub tint: vec3_t,
    /// unimplemented
    pub impactStrength: c_float,

    /// shader handle
    pub shader: c_int,

    /// used internally
    pub myIndex: c_int,

    /// specify fade method to modify RGB (by default, the alpha is set instead)
    pub fadeRGB: qboolean,
}

const _: () = assert!(core::mem::size_of::<SSkinGoreData>() == 144);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, angles) == 0);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, currentTime) == 24);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, entNum) == 28);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, rayDirection) == 32);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, hitLocation) == 44);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, scale) == 56);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, SSize) == 68);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, TSize) == 72);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, theta) == 76);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, growDuration) == 80);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, goreScaleStartFraction) == 84);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, frontFaces) == 88);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, backFaces) == 92);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, baseModelOnly) == 96);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, lifeTime) == 100);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, fadeOutTime) == 104);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, shrinkOutTime) == 108);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, alphaModulate) == 112);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, tint) == 116);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, impactStrength) == 128);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, shader) == 132);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, myIndex) == 136);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, fadeRGB) == 140);
