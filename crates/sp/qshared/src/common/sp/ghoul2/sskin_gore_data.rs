#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int};

use crate::shared::vec3_t;

/// Raven SP `SSkinGoreData` — one gore-mark (splotch) request applied to a Ghoul2 skin.
///
/// Diverges from MP: SP adds `uaxis`/`depthStart`/`depthEnd`/`useTheta`/
/// `firstModel`, drops `baseModelOnly`/`shrinkOutTime`/`alphaModulate`/`tint`/
/// `impactStrength`, and writes the four flags as C++ `bool` (1 byte), not the
/// int-wide `qboolean` MP uses.
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2530-2568`
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
    /// mark direction
    pub uaxis: vec3_t,
    /// limit marks begin depth
    pub depthStart: c_float,
    /// depth to stop making marks
    pub depthEnd: c_float,

    pub useTheta: bool,
    pub frontFaces: bool,
    pub backFaces: bool,
    /// specify fade method to modify RGB (by default, the alpha is set instead)
    pub fadeRGB: bool,

    // growing stuff
    /// time over which we want this to scale up, set to -1 for no scaling
    pub growDuration: c_int,
    /// fraction of the final size at which we want the gore to initially appear
    pub goreScaleStartFraction: c_float,

    /// effect expires after this amount of time
    pub lifeTime: c_int,
    /// which model to start the gore on (can skip the first)
    pub firstModel: c_int,
    /// duration of fading, counted back from lifeTime
    pub fadeOutTime: c_int,

    /// shader handle
    pub shader: c_int,

    /// used internally
    pub myIndex: c_int,
}

const _: () = assert!(core::mem::size_of::<SSkinGoreData>() == 132);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, angles) == 0);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, currentTime) == 24);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, entNum) == 28);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, rayDirection) == 32);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, hitLocation) == 44);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, scale) == 56);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, SSize) == 68);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, TSize) == 72);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, theta) == 76);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, uaxis) == 80);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, depthStart) == 92);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, depthEnd) == 96);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, useTheta) == 100);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, frontFaces) == 101);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, backFaces) == 102);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, fadeRGB) == 103);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, growDuration) == 104);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, goreScaleStartFraction) == 108);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, lifeTime) == 112);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, firstModel) == 116);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, fadeOutTime) == 120);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, shader) == 124);
const _: () = assert!(core::mem::offset_of!(SSkinGoreData, myIndex) == 128);
