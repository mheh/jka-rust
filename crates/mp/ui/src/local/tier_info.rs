#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::qhandle_t;

// Raven's `#define MAPS_PER_TIER 3`.
// Source: oracle/codemp/ui/ui_local.h:588
pub const MAPS_PER_TIER: usize = 3;

/// Raven `tierInfo` — a tier's map rotation entry (name, maps, gametypes, level shots).
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:642-647`
#[repr(C)]
pub struct tierInfo {
    pub tierName: *const c_char,
    pub maps: [*const c_char; MAPS_PER_TIER],
    pub gameTypes: [i32; MAPS_PER_TIER],
    pub mapHandles: [qhandle_t; MAPS_PER_TIER],
}

const _: () = assert!(core::mem::size_of::<tierInfo>() == 56);
const _: () = assert!(core::mem::offset_of!(tierInfo, tierName) == 0);
const _: () = assert!(core::mem::offset_of!(tierInfo, maps) == 8);
const _: () = assert!(core::mem::offset_of!(tierInfo, gameTypes) == 32);
const _: () = assert!(core::mem::offset_of!(tierInfo, mapHandles) == 44);
