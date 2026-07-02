#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `aas_bbox_t` — a bounding box used for a presence type.
///
/// Type definition source: `oracle/oracle/codemp/botlib/aasfile.h:97-102`
#[repr(C)]
pub struct aas_bbox_t {
	pub presencetype: i32,
	pub flags: i32,
	pub mins: vec3_t,
	pub maxs: vec3_t,
}

pub type aas_bbox_s = aas_bbox_t;

const _: () = assert!(core::mem::size_of::<aas_bbox_t>() == 32);
const _: () = assert!(core::mem::offset_of!(aas_bbox_t, presencetype) == 0);
const _: () = assert!(core::mem::offset_of!(aas_bbox_t, flags) == 4);
const _: () = assert!(core::mem::offset_of!(aas_bbox_t, mins) == 8);
const _: () = assert!(core::mem::offset_of!(aas_bbox_t, maxs) == 20);
