#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_int;

/// Raven AAS area contents flags (`AREACONTENTS_*`). `AREACONTENTS_TELEPORTAL`,
/// `AREACONTENTS_MODELNUMSHIFT`, `AREACONTENTS_MAXMODELNUM`, `AREACONTENTS_MODELNUM` are not on
/// this batch's port list and are left unported.
///
/// Source: `oracle/codemp/botlib/aasfile.h:52-64`
pub const AREACONTENTS_WATER: c_int = 1;
pub const AREACONTENTS_LAVA: c_int = 2;
pub const AREACONTENTS_SLIME: c_int = 4;
pub const AREACONTENTS_CLUSTERPORTAL: c_int = 8;
pub const AREACONTENTS_ROUTEPORTAL: c_int = 32;
pub const AREACONTENTS_TELEPORTER: c_int = 64;
pub const AREACONTENTS_JUMPPAD: c_int = 128;
pub const AREACONTENTS_DONOTENTER: c_int = 256;
pub const AREACONTENTS_VIEWPORTAL: c_int = 512;
pub const AREACONTENTS_MOVER: c_int = 1024;
pub const AREACONTENTS_NOTTEAM1: c_int = 2048;
pub const AREACONTENTS_NOTTEAM2: c_int = 4096;
