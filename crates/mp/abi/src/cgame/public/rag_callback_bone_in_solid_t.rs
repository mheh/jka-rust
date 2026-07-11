#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `ragCallbackBoneInSolid_t` — ragdoll bone-in-solid callback payload.
///
/// Type definition source: `oracle/codemp/cgame/cg_public.h:574-579`
#[repr(C)]
pub struct ragCallbackBoneInSolid_t {
    pub bonePos: vec3_t, //world coordinate position of the bone
    pub entNum: i32,     //index of entity who owns the bone in question
    pub solidCount: i32, //higher the count, the longer we've been in solid (the worse off we are)
}

const _: () = assert!(core::mem::size_of::<ragCallbackBoneInSolid_t>() == 20);
const _: () = assert!(core::mem::offset_of!(ragCallbackBoneInSolid_t, bonePos) == 0);
const _: () = assert!(core::mem::offset_of!(ragCallbackBoneInSolid_t, entNum) == 12);
const _: () = assert!(core::mem::offset_of!(ragCallbackBoneInSolid_t, solidCount) == 16);
