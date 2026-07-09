#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_reversedreachability_t` — reversed reachability links for one area.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_def.h:171-175`
#[repr(C)]
pub struct aas_reversedreachability_t {
	pub numlinks: i32,
	pub first: *mut super::aas_reversedlink_s::aas_reversedlink_t,
}

/// Raven's C tag is `aas_reversedreachability_s`; the typedef name
/// `aas_reversedreachability_t` is house style for the struct itself.
pub type aas_reversedreachability_s = aas_reversedreachability_t;

const _: () = assert!(core::mem::size_of::<aas_reversedreachability_t>() == 16);
const _: () = assert!(core::mem::offset_of!(aas_reversedreachability_t, numlinks) == 0);
const _: () = assert!(core::mem::offset_of!(aas_reversedreachability_t, first) == 8);
