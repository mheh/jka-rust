#![allow(non_camel_case_types, non_snake_case)]

use super::aas_lump_t::aas_lump_t;

/// Raven `AAS_LUMPS` — number of data lumps in an AAS file.
///
/// Source: `oracle/oracle/codemp/botlib/aasfile.h:78`
pub const AAS_LUMPS: usize = 14;

/// Raven `aas_header_s` — AAS file header: identity/version stamp, BSP checksum,
/// and the data-lump table.
///
/// Raven: none.
/// Type definition source: `oracle/oracle/codemp/botlib/aasfile.h:217-224`
#[repr(C)]
pub struct aas_header_t {
    pub ident: i32,
    pub version: i32,
    pub bspchecksum: i32,
    //data entries
    pub lumps: [aas_lump_t; AAS_LUMPS],
}

pub type aas_header_s = aas_header_t;

const _: () = assert!(core::mem::size_of::<aas_header_t>() == 124);
const _: () = assert!(core::mem::offset_of!(aas_header_t, ident) == 0);
const _: () = assert!(core::mem::offset_of!(aas_header_t, version) == 4);
const _: () = assert!(core::mem::offset_of!(aas_header_t, bspchecksum) == 8);
const _: () = assert!(core::mem::offset_of!(aas_header_t, lumps) == 12);
