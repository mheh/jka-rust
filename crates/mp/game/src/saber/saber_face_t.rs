#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `saberFace_s` (typedef `saberFace_t`) is one triangular collision face of the saber blade, built per-frame around the blade's radius.
///
/// This type is file-local to `G_BuildSaberFaces`/`G_SaberFaceCollisionCheck`.
/// It is not ABI-crossing, so no `#[repr(C)]`/layout asserts are required.
/// Type definition source: `oracle/codemp/game/w_saber.c:2446-2451`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct saberFace_t {
    pub v1: vec3_t,
    pub v2: vec3_t,
    pub v3: vec3_t,
}
