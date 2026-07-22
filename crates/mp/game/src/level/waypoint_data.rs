//! MP `waypointData_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:810-818`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `waypointData_t` — cleared as each map is entered.
///
/// The five name fields hold `Q_strncpyz`-bounded (`MAX_QPATH`) copies of an
/// entity's targetname/target chain; they are owned `String`s (the byte bound
/// is applied at the `NAV_StoreWaypoint` write sites). Game-internal storage
/// (`tempWaypointList`), so layout is free.
///
/// Type definition source: `oracle/codemp/game/g_local.h:810-818`
#[derive(Clone, Default)]
pub struct waypointData_t {
    pub targetname: String,
    pub target: String,
    pub target2: String,
    pub target3: String,
    pub target4: String,
    pub nodeID: c_int,
}
