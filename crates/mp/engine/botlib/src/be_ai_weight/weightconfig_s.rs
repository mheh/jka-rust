#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::MAX_QPATH;

use super::weight_s::weight_t;

/// `MAX_WEIGHTS`.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.h:16`
pub const MAX_WEIGHTS: usize = 128;

/// Raven `weightconfig_t` — a set of named fuzzy weights loaded from a file.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_weight.h:39-44`
#[repr(C)]
pub struct weightconfig_t {
	pub numweights: i32,
	pub weights: [weight_t; MAX_WEIGHTS],
	pub filename: [c_char; MAX_QPATH as usize],
}

pub type weightconfig_s = weightconfig_t;

const _: () = assert!(core::mem::size_of::<weightconfig_t>() == 2120);
const _: () = assert!(core::mem::offset_of!(weightconfig_t, numweights) == 0);
const _: () = assert!(core::mem::offset_of!(weightconfig_t, weights) == 8);
const _: () = assert!(core::mem::offset_of!(weightconfig_t, filename) == 2056);
