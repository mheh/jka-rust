//! `TierInfo` — Raven `tierInfo`.

use core::ffi::c_int;

use mp_qshared::shared::qhandle_t;

/// Raven `#define MAPS_PER_TIER 3`.
///
/// Source: `oracle/codemp/ui/ui_local.h:588`
pub const MAPS_PER_TIER: usize = 3;

/// Raven `tierInfo` — a tier's map rotation entry (name, maps, gametypes,
/// level shots).
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:642-647`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "tierInfo")]
#[allow(non_snake_case)]
pub struct TierInfo {
    pub tierName: String,
    pub maps: [String; MAPS_PER_TIER],
    pub gameTypes: [c_int; MAPS_PER_TIER],
    pub mapHandles: [qhandle_t; MAPS_PER_TIER],
}
