//! `TerrainHandle` — the ABI-crossing terrain id (Raven `thandle_t`).
//!
//! §F idiomatic reimplementation (porting-rules §17-21); own file, beside
//! `collision_world.rs`, per **ruling 39d** (`docs/subsystems/rmg-terrain.md`
//! RMG-D2(d)): the settled `mp_engine_rmg → mp_engine_qcommon` dependency
//! direction never runs in reverse, so the handle both `register_terrain`
//! (Seam-B, `cm_terrain.rs`) constructs and `RmManager::set_landscape`
//! (`mp_engine_rmg`) consumes must live here (or lower), not in `mp_engine_rmg`.
//!
//! Layout-free (§F): nothing here crosses the module ABI as a struct — the
//! `G_CM_REGISTER_TERRAIN` syscall marshals the raw `thandle_t` scalar, not
//! this newtype.
//!
//! Source: `oracle/codemp/qcommon/cm_landscape.h:139,220`

/// Raven `thandle_t` — `typedef int thandle_t` (rosetta:
/// `crates/native/types/src/lib.rs:65`). `mp_engine_qcommon` has no
/// `native_types` dependency today; the numeric alias is repeated locally
/// rather than adding a crate edge for one `c_int`.
///
/// Source: `oracle/codemp/qcommon/cm_local.h` (`thandle_t` typedef)
type ThandleT = core::ffi::c_int;

/// `TerrainHandle` — a newtype over `thandle_t`, the id `CM_RegisterTerrain`
/// returns across the `G_CM_REGISTER_TERRAIN` syscall arm
/// (`CCMLandScape::GetTerrainId`/`mTerrainHandle`, `cm_landscape.h:139,220`;
/// `sv_game.cpp:1640-1641`).
///
/// §B5 idiomatic handle. `register_terrain` (Seam-B, `cm_terrain.rs`)
/// constructs it (folding `CM_InitTerrain`'s `SetTerrainId` call,
/// `cm_terrain.cpp:1618-1626`); `RmManager::set_landscape` (`mp_engine_rmg`)
/// consumes it; `RmManager::land()` returns it (ruling 28/RMG-D1).
///
/// Source: `oracle/codemp/qcommon/cm_landscape.h:139,220`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainHandle(pub ThandleT);

impl TerrainHandle {
    /// The raw `thandle_t` value — `GetTerrainId`'s return, and the value the
    /// `G_CM_REGISTER_TERRAIN` syscall arm marshals back across the vmCall
    /// boundary (`sv_game.cpp:1641`).
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:220`
    #[inline]
    pub const fn raw(self) -> ThandleT {
        self.0
    }
}
