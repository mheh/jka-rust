#![allow(non_camel_case_types, non_snake_case)]

/// Raven `md3Triangle_t` — MD3 model triangle indices.
///
/// Type definition source: `oracle/codemp/qcommon/../qcommon/qfiles.h:156-158`
#[repr(C)]
pub struct md3Triangle_t {
    pub indexes: [i32; 3],
}

const _: () = assert!(core::mem::size_of::<md3Triangle_t>() == 12);
const _: () = assert!(core::mem::offset_of!(md3Triangle_t, indexes) == 0);
