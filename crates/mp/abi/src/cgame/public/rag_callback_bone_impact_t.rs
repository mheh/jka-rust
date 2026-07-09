#![allow(non_camel_case_types, non_snake_case)]

/// Raven `ragCallbackBoneImpact_t` — ragdoll bone-impact callback payload.
///
/// Type definition source: `oracle/codemp/cgame/cg_public.h:567-571`
#[repr(C)]
pub struct ragCallbackBoneImpact_t {
	pub boneName: [i8; 128], //name of the bone in question
	pub entNum: i32,         //index of entity who owns the bone in question
}

const _: () = assert!(core::mem::size_of::<ragCallbackBoneImpact_t>() == 132);
const _: () = assert!(core::mem::offset_of!(ragCallbackBoneImpact_t, boneName) == 0);
const _: () = assert!(core::mem::offset_of!(ragCallbackBoneImpact_t, entNum) == 128);
