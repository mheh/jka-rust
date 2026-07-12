#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

use super::deform_t::deform_t;
use super::wave_form_t::waveForm_t;

/// Raven `deformStage_t` — vertex deformation stage.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:310-320`
#[repr(C)]
pub struct deformStage_t {
    /// vertex coordinate modification type
    pub deformation: deform_t,

    pub moveVector: vec3_t,
    pub deformationWave: waveForm_t,
    pub deformationSpread: f32,

    pub bulgeWidth: f32,
    pub bulgeHeight: f32,
    pub bulgeSpeed: f32,
}

const _: () = assert!(core::mem::size_of::<deformStage_t>() == 52);
const _: () = assert!(core::mem::offset_of!(deformStage_t, deformation) == 0);
const _: () = assert!(core::mem::offset_of!(deformStage_t, moveVector) == 4);
const _: () = assert!(core::mem::offset_of!(deformStage_t, deformationWave) == 16);
const _: () = assert!(core::mem::offset_of!(deformStage_t, deformationSpread) == 36);
const _: () = assert!(core::mem::offset_of!(deformStage_t, bulgeWidth) == 40);
const _: () = assert!(core::mem::offset_of!(deformStage_t, bulgeHeight) == 44);
const _: () = assert!(core::mem::offset_of!(deformStage_t, bulgeSpeed) == 48);
