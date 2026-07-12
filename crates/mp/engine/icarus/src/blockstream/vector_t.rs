#![allow(non_camel_case_types, non_snake_case)]

use ::core::ffi::c_float;

/// Raven `vector_t` — 3-element float vector.
///
/// Type definition source: `oracle/codemp/icarus/blockstream.h:24-24`
pub type vector_t = [c_float; 3];
