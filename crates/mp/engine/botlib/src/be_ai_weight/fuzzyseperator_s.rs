#![allow(non_camel_case_types, non_snake_case)]

/// Raven `fuzzyseperator_t` — fuzzy logic weight tree separator node.
///
/// Type definition source: `oracle/oracle/codemp/botlib/be_ai_weight.h:19-29`
#[repr(C)]
pub struct fuzzyseperator_t {
	pub index: i32,
	pub value: i32,
	pub r#type: i32,
	pub weight: f32,
	pub minweight: f32,
	pub maxweight: f32,
	pub child: *mut fuzzyseperator_t,
	pub next: *mut fuzzyseperator_t,
}

pub type fuzzyseperator_s = fuzzyseperator_t;

const _: () = assert!(core::mem::size_of::<fuzzyseperator_t>() == 40);
const _: () = assert!(core::mem::offset_of!(fuzzyseperator_t, index) == 0);
const _: () = assert!(core::mem::offset_of!(fuzzyseperator_t, value) == 4);
const _: () = assert!(core::mem::offset_of!(fuzzyseperator_t, r#type) == 8);
const _: () = assert!(core::mem::offset_of!(fuzzyseperator_t, weight) == 12);
const _: () = assert!(core::mem::offset_of!(fuzzyseperator_t, minweight) == 16);
const _: () = assert!(core::mem::offset_of!(fuzzyseperator_t, maxweight) == 20);
const _: () = assert!(core::mem::offset_of!(fuzzyseperator_t, child) == 24);
const _: () = assert!(core::mem::offset_of!(fuzzyseperator_t, next) == 32);
