#![allow(non_camel_case_types)]

use core::ffi::c_char;

use crate::vehicles::veh_field_type_t::vehFieldType_t;

/// Raven `vehField_t` — one `.veh`/`.vwp` key/value parse-table entry
/// (`name` -> byte offset into `vehicleInfo_t`/`vehWeaponInfo_t` + value kind).
///
/// Type definition source: `oracle/codemp/game/bg_vehicleLoad.c:131-135`
#[derive(Clone, Copy)]
pub struct vehField_t {
    pub name: *const c_char,
    pub ofs: i32,
    pub r#type: vehFieldType_t,
}

// Internal-only parse-table type (never crosses the ABI seam), so no
// #[repr(C)]/layout asserts are required (porting-rules §12). Table instances
// are `const`, not `static`, so the raw `name` pointer needs no `Sync` impl.
