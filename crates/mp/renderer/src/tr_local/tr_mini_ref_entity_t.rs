#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::mini_ref_entity_s::miniRefEntity_t;

/// Raven `trMiniRefEntity_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:87-90`
#[repr(C)]
pub struct trMiniRefEntity_t {
    pub e: miniRefEntity_t,
}

const _: () = assert!(core::mem::size_of::<trMiniRefEntity_t>() == 108);
const _: () = assert!(core::mem::offset_of!(trMiniRefEntity_t, e) == 0);
