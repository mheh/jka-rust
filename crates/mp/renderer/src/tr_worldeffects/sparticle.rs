#![allow(non_camel_case_types, non_snake_case)]
use mp_qshared::shared::vec3_t;

/// Raven `SParticle` — a single world-effects particle.
///
/// Type definition source: `oracle/codemp/renderer/tr_WorldEffects.h:13-18`
#[repr(C)]
pub struct SParticle {
    pub pos: vec3_t,
    pub velocity: vec3_t,
    pub flags: u32,
}
const _: () = assert!(core::mem::size_of::<SParticle>() == 28);
const _: () = assert!(core::mem::offset_of!(SParticle, pos) == 0);
const _: () = assert!(core::mem::offset_of!(SParticle, velocity) == 12);
const _: () = assert!(core::mem::offset_of!(SParticle, flags) == 24);
