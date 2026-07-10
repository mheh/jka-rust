use core::ffi::c_int;

/// Raven `AAS_MAX_PORTALS` — max portals in a single AAS file.
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:31`
pub const AAS_MAX_PORTALS: c_int = 65536;

/// Raven `AAS_MAX_PORTALINDEXSIZE` — max entries in the portal index.
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:32`
pub const AAS_MAX_PORTALINDEXSIZE: c_int = 65536;

/// Raven `AAS_MAX_CLUSTERS` — max clusters in a single AAS file.
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:33`
pub const AAS_MAX_CLUSTERS: c_int = 65536;

/// Raven `MAX_PORTALAREAS` — max areas belonging to a single portal.
/// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:35`
pub const MAX_PORTALAREAS: c_int = 1024;
