#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::shared::{qboolean, vec3_t};

/// Raven `trRefEntity_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:94-106`
#[repr(C)]
pub struct trRefEntity_t {
    pub e: refEntity_t,

    /// compensate for non-normalized axis
    pub axisLength: f32,

    /// true for bmodels that touch a dlight
    pub needDlights: qboolean,
    pub lightingCalculated: qboolean,
    /// normalized direction towards light
    pub lightDir: vec3_t,
    /// color normalized to 0-255
    pub ambientLight: vec3_t,
    /// 32 bit rgba packed
    pub ambientLightInt: i32,
    pub directedLight: vec3_t,
    pub dlightBits: i32,
}

const _: () = assert!(core::mem::offset_of!(trRefEntity_t, e) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<trRefEntity_t>() == 272);
    assert!(core::mem::offset_of!(trRefEntity_t, axisLength) == 216);
    assert!(core::mem::offset_of!(trRefEntity_t, needDlights) == 220);
    assert!(core::mem::offset_of!(trRefEntity_t, lightingCalculated) == 224);
    assert!(core::mem::offset_of!(trRefEntity_t, lightDir) == 228);
    assert!(core::mem::offset_of!(trRefEntity_t, ambientLight) == 240);
    assert!(core::mem::offset_of!(trRefEntity_t, ambientLightInt) == 252);
    assert!(core::mem::offset_of!(trRefEntity_t, directedLight) == 256);
    assert!(core::mem::offset_of!(trRefEntity_t, dlightBits) == 268);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<trRefEntity_t>() == 268);
    assert!(core::mem::offset_of!(trRefEntity_t, axisLength) == 212);
    assert!(core::mem::offset_of!(trRefEntity_t, needDlights) == 216);
    assert!(core::mem::offset_of!(trRefEntity_t, lightingCalculated) == 220);
    assert!(core::mem::offset_of!(trRefEntity_t, lightDir) == 224);
    assert!(core::mem::offset_of!(trRefEntity_t, ambientLight) == 236);
    assert!(core::mem::offset_of!(trRefEntity_t, ambientLightInt) == 248);
    assert!(core::mem::offset_of!(trRefEntity_t, directedLight) == 252);
    assert!(core::mem::offset_of!(trRefEntity_t, dlightBits) == 264);
};
