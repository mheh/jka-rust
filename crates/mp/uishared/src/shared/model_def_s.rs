//! `ModelDef` — Raven `modelDef_s`/`modelDef_t`.

use core::ffi::c_int;

use mp_qshared::shared::vec3_t;

/// Raven `modelDef_s` (typedef `modelDef_t`) — a 3D model preview definition,
/// one of the `itemDef_t::typeData` payloads.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:208-224`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[doc(alias = "modelDef_s")]
#[doc(alias = "modelDef_t")]
#[allow(non_snake_case)]
pub struct ModelDef {
    pub angle: c_int,
    pub origin: vec3_t,
    pub fov_x: f32,
    pub fov_y: f32,
    pub rotationSpeed: c_int,

    /// required
    pub g2mins: vec3_t,
    /// required
    pub g2maxs: vec3_t,
    /// optional
    pub g2scale: vec3_t,
    /// optional
    pub g2skin: c_int,
    /// optional
    pub g2anim: c_int,
    // JLF
    // Transition extras
    pub g2mins2: vec3_t,
    pub g2maxs2: vec3_t,
    pub g2minsEffect: vec3_t,
    pub g2maxsEffect: vec3_t,
    pub fov_x2: f32,
    pub fov_y2: f32,
    pub fov_Effectx: f32,
    pub fov_Effecty: f32,
}
