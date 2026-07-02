#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::renderer::color4ub_t::color4ub_t;
use sp_qshared::shared::MAX_QPATH;

/// Raven `clightstyle_t` — a compiled cgame lightstyle (base value + per-frame map).
///
/// Type definition source: `oracle/oracle/code/cgame/cg_lights.h:5-10`
#[repr(C)]
pub struct clightstyle_t {
    pub length: i32,
    pub value: color4ub_t,
    pub map: [color4ub_t; MAX_QPATH as usize],
}

const _: () = assert!(core::mem::size_of::<clightstyle_t>() == 264);
const _: () = assert!(core::mem::offset_of!(clightstyle_t, length) == 0);
const _: () = assert!(core::mem::offset_of!(clightstyle_t, value) == 4);
const _: () = assert!(core::mem::offset_of!(clightstyle_t, map) == 8);
