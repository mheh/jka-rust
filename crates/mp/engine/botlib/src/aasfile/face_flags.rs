#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_int;

/// Raven AAS face flags (`FACE_*`). Only `FACE_SOLID`, `FACE_LADDER`, `FACE_GROUND` are
/// engine-referenced; the remaining flags (`FACE_GAP`, `FACE_LIQUID`, `FACE_LIQUIDSURFACE`,
/// `FACE_BRIDGE`) are not on this batch's port list and are left unported.
///
/// Source: `oracle/codemp/botlib/aasfile.h:43-49`
/// just solid at the other side
pub const FACE_SOLID: c_int = 1;
/// ladder
pub const FACE_LADDER: c_int = 2;
/// standing on ground when in this face
pub const FACE_GROUND: c_int = 4;
