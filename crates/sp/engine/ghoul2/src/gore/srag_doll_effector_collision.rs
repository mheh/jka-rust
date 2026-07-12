#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::trace_t::trace_t;
use sp_qshared::shared::vec3_t;

/// Raven `SRagDollEffectorCollision` — ragdoll effector-vs-trace collision result.
///
/// Raven: constructor copies `effectorPos` into `effectorPosition` and binds the
/// `const trace_t&` reference; not modeled at the ABI-layout level here (`tr` is
/// a non-owning pointer to the referenced `trace_t`).
/// Type definition source: `oracle/code/ghoul2/ghoul2_gore.h:69-80`
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
