#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

/// Raven `winding_t` — a variable-sized polygon (up to 4 points).
///
/// Raven: `p` is `// variable sized`.
/// Type definition source: `oracle/oracle/code/qcommon/cm_polylib.h:7-11`
#[repr(C)]
pub struct winding_t {
    pub numpoints: i32,
    pub p: [vec3_t; 4],
}

const _: () = assert!(core::mem::size_of::<winding_t>() == 52);
const _: () = assert!(core::mem::offset_of!(winding_t, numpoints) == 0);
const _: () = assert!(core::mem::offset_of!(winding_t, p) == 4);
