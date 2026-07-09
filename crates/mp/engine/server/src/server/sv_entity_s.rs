#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::qcommon::entity_state::entityState_t;

use super::world_sector_s::worldSector_t;

/// Raven `MAX_ENT_CLUSTERS`.
///
/// Type definition source: `oracle/codemp/server/server.h:25`
pub const MAX_ENT_CLUSTERS: usize = 16;

/// Raven `svEntity_t` — server-side per-entity bookkeeping (world linkage,
/// baseline for delta compression, PVS cluster caching).
///
/// Raven: non-Xbox variant (`_XBOX` undefined) is the one this codebase ports.
/// Type definition source: `oracle/codemp/server/server.h:27-45`
#[repr(C)]
pub struct svEntity_t {
    pub worldSector: *mut worldSector_t,
    pub nextEntityInWorldSector: *mut svEntity_t,

    /// for delta compression of initial sighting
    pub baseline: entityState_t,

    /// if -1, use headnode instead
    pub numClusters: i32,
    pub clusternums: [i32; MAX_ENT_CLUSTERS],
    /// if all the clusters don't fit in clusternums
    pub lastCluster: i32,
    pub areanum: i32,
    pub areanum2: i32,
    /// used to prevent double adding from portal views
    pub snapshotCounter: i32,
}

const _: () = assert!(core::mem::size_of::<svEntity_t>() == 632);
const _: () = assert!(core::mem::offset_of!(svEntity_t, worldSector) == 0);
const _: () = assert!(core::mem::offset_of!(svEntity_t, nextEntityInWorldSector) == 8);
const _: () = assert!(core::mem::offset_of!(svEntity_t, baseline) == 16);
const _: () = assert!(core::mem::offset_of!(svEntity_t, numClusters) == 548);
const _: () = assert!(core::mem::offset_of!(svEntity_t, clusternums) == 552);
const _: () = assert!(core::mem::offset_of!(svEntity_t, lastCluster) == 616);
const _: () = assert!(core::mem::offset_of!(svEntity_t, areanum) == 620);
const _: () = assert!(core::mem::offset_of!(svEntity_t, areanum2) == 624);
const _: () = assert!(core::mem::offset_of!(svEntity_t, snapshotCounter) == 628);

/// C tag `svEntity_s` is the same type as the `svEntity_t` typedef.
pub type svEntity_s = svEntity_t;
