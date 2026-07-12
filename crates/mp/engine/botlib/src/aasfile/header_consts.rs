#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_int;

/// Raven `AASID` — magic number identifying an AAS file (`'E'+'A'<<8+'A'<<16+'S'<<24`).
///
/// Source: `oracle/codemp/botlib/aasfile.h:6`
pub const AASID: c_int =
    ('S' as c_int) << 24 | ('A' as c_int) << 16 | ('A' as c_int) << 8 | 'E' as c_int;

/// Raven `AASVERSION_OLD` — previous AAS file format version.
///
/// Source: `oracle/codemp/botlib/aasfile.h:7`
pub const AASVERSION_OLD: c_int = 4;

/// Raven `AASVERSION` — current AAS file format version.
///
/// Source: `oracle/codemp/botlib/aasfile.h:8`
pub const AASVERSION: c_int = 5;
