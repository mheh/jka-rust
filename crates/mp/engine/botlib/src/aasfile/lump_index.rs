#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_int;

/// Raven AAS file header lump indices (`AASLUMP_*`) — index into `aas_header_t.lumps`.
///
/// Source: `oracle/codemp/botlib/aasfile.h:79-92`
pub const AASLUMP_BBOXES: c_int = 0;
pub const AASLUMP_VERTEXES: c_int = 1;
pub const AASLUMP_PLANES: c_int = 2;
pub const AASLUMP_EDGES: c_int = 3;
pub const AASLUMP_EDGEINDEX: c_int = 4;
pub const AASLUMP_FACES: c_int = 5;
pub const AASLUMP_FACEINDEX: c_int = 6;
pub const AASLUMP_AREAS: c_int = 7;
pub const AASLUMP_AREASETTINGS: c_int = 8;
pub const AASLUMP_REACHABILITY: c_int = 9;
pub const AASLUMP_NODES: c_int = 10;
pub const AASLUMP_PORTALS: c_int = 11;
pub const AASLUMP_PORTALINDEX: c_int = 12;
pub const AASLUMP_CLUSTERS: c_int = 13;
