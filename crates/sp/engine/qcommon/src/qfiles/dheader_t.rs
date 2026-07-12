#![allow(non_camel_case_types, non_snake_case)]

use super::lump_t::lump_t;

/// Number of lumps in a `dheader_t`.
///
/// Source: `oracle/code/qcommon/../qcommon/qfiles.h:442`
pub const HEADER_LUMPS: usize = 18;

/// Raven `dheader_t` — BSP file header.
///
/// Type definition source: `oracle/code/qcommon/../qcommon/qfiles.h:444-449`
#[repr(C)]
pub struct dheader_t {
    pub ident: i32,
    pub version: i32,

    pub lumps: [lump_t; HEADER_LUMPS],
}

const _: () = assert!(core::mem::size_of::<dheader_t>() == 152);
const _: () = assert!(core::mem::offset_of!(dheader_t, ident) == 0);
const _: () = assert!(core::mem::offset_of!(dheader_t, version) == 4);
const _: () = assert!(core::mem::offset_of!(dheader_t, lumps) == 8);
