#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `bsp_entdata_t` — BSP entity data (origin/angles/bounds/solid/model).
///
/// Type definition source: `oracle/oracle/codemp/botlib/be_aas_def.h:67-75`
#[repr(C)]
pub struct bsp_entdata_t {
	pub origin: vec3_t,
	pub angles: vec3_t,
	pub absmins: vec3_t,
	pub absmaxs: vec3_t,
	pub solid: i32,
	pub modelnum: i32,
}

pub type bsp_entdata_s = bsp_entdata_t;

const _: () = assert!(core::mem::size_of::<bsp_entdata_t>() == 56);
const _: () = assert!(core::mem::offset_of!(bsp_entdata_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(bsp_entdata_t, angles) == 12);
const _: () = assert!(core::mem::offset_of!(bsp_entdata_t, absmins) == 24);
const _: () = assert!(core::mem::offset_of!(bsp_entdata_t, absmaxs) == 36);
const _: () = assert!(core::mem::offset_of!(bsp_entdata_t, solid) == 48);
const _: () = assert!(core::mem::offset_of!(bsp_entdata_t, modelnum) == 52);
