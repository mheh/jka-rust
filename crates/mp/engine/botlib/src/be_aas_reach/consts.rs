use core::ffi::c_int;

/// Raven `AAS_MAX_REACHABILITYSIZE` — max size in bytes of the reachability data
/// blob loaded per AAS file.
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:37`
pub const AAS_MAX_REACHABILITYSIZE: c_int = 65536;

/// Raven `REACHABILITYAREASPERCYCLE` — number of areas processed per reachability
/// calculation cycle.
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:39`
pub const REACHABILITYAREASPERCYCLE: c_int = 15;

/// Raven `INSIDEUNITS` — distance (in units) a reachability start/end point is
/// moved inside the area boundary.
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:41`
pub const INSIDEUNITS: c_int = 2;

/// Raven `INSIDEUNITS_WALKEND` — inside-units offset for the end of a walk
/// reachability.
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:42`
pub const INSIDEUNITS_WALKEND: c_int = 5;

/// Raven `INSIDEUNITS_WALKSTART` — inside-units offset for the start of a walk
/// reachability.
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:43`
pub const INSIDEUNITS_WALKSTART: f32 = 0.1;

/// Raven `INSIDEUNITS_WATERJUMP` — inside-units offset for a water-jump
/// reachability.
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:44`
pub const INSIDEUNITS_WATERJUMP: c_int = 15;

/// Raven `AREA_WEAPONJUMP` — area content flag marking a valid weapon-jump
/// destination area.
/// Source: `oracle/codemp/botlib/be_aas_reach.cpp:46`
pub const AREA_WEAPONJUMP: c_int = 8192;
