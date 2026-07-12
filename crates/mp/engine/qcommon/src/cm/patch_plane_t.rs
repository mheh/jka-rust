#![allow(non_camel_case_types, non_snake_case)]

/// Raven `patchPlane_t` — a surface or edge plane used by patch collision.
///
/// Type definition source: `oracle/codemp/qcommon/cm_patch.h:45-48`
#[repr(C)]
pub struct patchPlane_t {
    pub plane: [f32; 4],
    /// signx + (signy<<1) + (signz<<2), used as lookup during collision
    pub signbits: i32,
}

const _: () = assert!(core::mem::size_of::<patchPlane_t>() == 20);
const _: () = assert!(core::mem::offset_of!(patchPlane_t, plane) == 0);
const _: () = assert!(core::mem::offset_of!(patchPlane_t, signbits) == 16);
