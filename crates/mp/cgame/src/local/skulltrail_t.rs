#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `MAX_SKULLTRAIL`.
///
/// Source: `oracle/codemp/cgame/cg_local.h:728`
pub const MAX_SKULLTRAIL: usize = 10;

/// Raven `skulltrail_t`.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:730-733`
#[repr(C)]
pub struct skulltrail_t {
    pub positions: [vec3_t; MAX_SKULLTRAIL],
    pub numpositions: i32,
}

const _: () = assert!(core::mem::size_of::<skulltrail_t>() == 124);
const _: () = assert!(core::mem::offset_of!(skulltrail_t, positions) == 0);
const _: () = assert!(core::mem::offset_of!(skulltrail_t, numpositions) == 120);
