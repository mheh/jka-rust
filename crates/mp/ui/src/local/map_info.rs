//! `MapInfo` — Raven `mapInfo`.

use core::ffi::c_int;

use mp_qshared::shared::qhandle_t;

/// Raven `#define MAX_GAMETYPES 16`.
///
/// Source: `oracle/codemp/ui/ui_local.h:566`
pub const MAX_GAMETYPES: usize = 16;

/// Raven `mapInfo` — one map row of the arena list the map/browser menus draw.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:629-640`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "mapInfo")]
#[allow(non_snake_case)]
pub struct MapInfo {
    pub mapName: String,
    pub mapLoadName: String,
    pub imageName: String,
    pub opponentName: String,
    pub teamMembers: c_int,
    pub typeBits: c_int,
    pub cinematic: c_int,
    pub timeToBeat: [c_int; MAX_GAMETYPES],
    pub levelShot: qhandle_t,
    pub active: bool,
}
