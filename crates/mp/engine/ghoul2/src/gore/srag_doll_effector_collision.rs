#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::vec3_t;

/// Raven `SRagDollEffectorCollision` — per-effector ragdoll collision query/result.
///
/// Raven: constructor takes `(const vec3_t effectorPos, const trace_t &t)`, copies
/// `effectorPos` into `effectorPosition`, binds the `tr` reference, and defaults
/// `useTracePlane` to false. The `tr` reference is modeled here as a raw pointer
/// since Rust has no ABI-layout reference member.
/// Type definition source: `oracle/codemp/ghoul2/G2_gore.h:81-92`
#[repr(C)]
pub struct SRagDollEffectorCollision {
    pub effectorPosition: vec3_t,
    pub tr: *const trace_t,
    pub useTracePlane: bool,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<SRagDollEffectorCollision>() == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(SRagDollEffectorCollision, effectorPosition) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(SRagDollEffectorCollision, tr) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(SRagDollEffectorCollision, useTracePlane) == 24);
