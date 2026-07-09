#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::{qboolean, qhandle_t};

/// Raven `itemInfo_t` — cgame-side per-item model/icon cache entry.
///
/// Type definition source: `oracle/code/cgame/cg_local.h:256-260`
#[repr(C)]
pub struct itemInfo_t {
	pub registered: qboolean,
	pub models: qhandle_t,
	pub icon: qhandle_t,
}

const _: () = assert!(core::mem::size_of::<itemInfo_t>() == 12);
const _: () = assert!(core::mem::offset_of!(itemInfo_t, registered) == 0);
const _: () = assert!(core::mem::offset_of!(itemInfo_t, models) == 4);
const _: () = assert!(core::mem::offset_of!(itemInfo_t, icon) == 8);
