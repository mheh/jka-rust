#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_int;

/// Raven AAS area flags (`AREA_*`).
///
/// Source: `oracle/codemp/botlib/aasfile.h:71-75`
/// bot can stand on the ground
pub const AREA_GROUNDED: c_int = 1;
/// area contains one or more ladder faces
pub const AREA_LADDER: c_int = 2;
/// area contains a liquid
pub const AREA_LIQUID: c_int = 4;
/// area is disabled for routing when set
pub const AREA_DISABLED: c_int = 8;
/// area ontop of a bridge
pub const AREA_BRIDGE: c_int = 16;
