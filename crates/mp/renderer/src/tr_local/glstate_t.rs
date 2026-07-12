#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::qboolean;

/// Raven `glstate_t` — cached OpenGL bind/state to avoid redundant GL calls.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1253-1260`
#[repr(C)]
pub struct glstate_t {
    pub currenttextures: [i32; 2],
    pub currenttmu: i32,
    pub finishCalled: qboolean,
    pub texEnv: [i32; 2],
    pub faceCulling: i32,
    pub glStateBits: u64,
}

const _: () = assert!(core::mem::size_of::<glstate_t>() == 40);
const _: () = assert!(core::mem::offset_of!(glstate_t, currenttextures) == 0);
const _: () = assert!(core::mem::offset_of!(glstate_t, currenttmu) == 8);
const _: () = assert!(core::mem::offset_of!(glstate_t, finishCalled) == 12);
const _: () = assert!(core::mem::offset_of!(glstate_t, texEnv) == 16);
const _: () = assert!(core::mem::offset_of!(glstate_t, faceCulling) == 24);
const _: () = assert!(core::mem::offset_of!(glstate_t, glStateBits) == 32);
