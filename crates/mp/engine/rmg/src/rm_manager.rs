//! `CRMManager` — the random mission manager LIVE lifecycle (RMG-D1).
//!
//! Per `docs/subsystems/rmg-terrain.md` (roster row, class `CRMManager`) only
//! **six** methods are live under the `DEDICATED` build and survive into the
//! frozen Seam-A `impl RmManager` below: `new`, `set_landscape`,
//! `load_mission`, `automap_symbol_count`, `automap_symbol`, `land`.
//!
//! `CRMManager::SpawnMission` (`RM_Manager.cpp:391`) is **dropped from Seam-A
//! entirely — ruling 38 — and is NOT a stub here.** It is unreachable under
//! DEDICATED (`load_mission` always returns `false`,
//! `oracle/codemp/server/sv_game.cpp:1632-1634`); the Wave-20 `G_RMG_INIT`
//! syscall arm collapses the provably-dead `if load_mission { spawn_mission }`
//! to plain `load_mission(…)` (porting-rules §C10). No `spawn_mission`
//! method/stub exists anywhere (`docs/GOAL-engine.md` no-stub rule).
//!
//! Everything else `CRMManager` declares is §20 zero-caller dead surface
//! (RMG-D4c / ruling 17), not ported:
//!
//! - `mCurObjective` (`RM_Manager.cpp:16`) — static member, zero-init only,
//!   never read/written in codemp.
//! - `SetCurPriority` (`RM_Manager.h:36`), `GetTerrain` (`:38`), `GetMission`
//!   (`:40`), `GetCurPriority` (`:41`), `AddAutomapSymbol` (`:43`), `Preview`
//!   (`:48`), `IsMissionComplete` (`:50`), `HasTimeExpired` (`:51`),
//!   `CompleteObjective` (`:52`), `CompleteMission` (`:53`), `FailedMission`
//!   (`:54`), `UpdateStatisticCvars` (`:23`) — a grep of
//!   `TheRandomMissionManager->` finds no invocation of any of these anywhere
//!   in codemp; `GetMission`/`AddAutomapSymbol` are called only from the
//!   §20-dropped generation path (`RM_Instance*`/`RM_Path.cpp`;
//!   `RM_Manager.cpp:400-410`).
//! - `WriteAutomapSymbols` (`RM_Manager.cpp:424`) — commented-out dead code.
//! - `ProcessAutomapSymbols` (`RM_Manager.cpp:442`) — a `static` client-side
//!   reader, dead under DEDICATED.
//!
//! Source: `oracle/codemp/RMG/RM_Manager.h:9-58`, `oracle/codemp/RMG/RM_Manager.cpp`

use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::terrain_handle::TerrainHandle;
use mp_host_interface::EngineHost;
use mp_qshared::RmAutomapSymbol;

/// `CRMManager` — the one live singleton (`TheRandomMissionManager`,
/// `RM_Manager.cpp:23`; extern `RM_Manager.h:60`), threaded per ruling 12 as a
/// plain direct `Engine.rmg` field (no `Option`/`Box`) rather than a Rust
/// global (porting-rules §B3).
///
/// **Complete field set (frozen).** `CRMManager::CRMManager` null/zero-inits
/// the members this Rust field set mirrors (`mLandScape`/`mTerrain`/`mMission`
/// NULL, `RM_Manager.cpp:34-42`; the only non-default member,
/// `mCurPriority = 1` at `:39`, is §20-dropped), so it is Default-equivalent:
/// `RmManager::default()` and Raven's lazy `new CRMManager` collapse to one
/// construction. `mMission: CRMMission*` (`RM_Manager.h:13`), the cached
/// `CCMLandScape*`/`CRandomTerrain*` (`:14-15`), and the
/// `mAutomapSymbols`/`mAutomapSymbolCount` pair (`:20-21`) are all §20-dropped
/// (RMG-D4b/RMG-D4c) — none is a Rust field; only `initialized` and `land`
/// survive.
///
/// Source: `oracle/codemp/RMG/RM_Manager.h:9-58`
#[derive(Default)]
pub struct RmManager {
    /// The lazy-init flag (ruling 12) — the concrete rendering of Raven's own
    /// `!TheRandomMissionManager` null check at the `G_RMG_INIT` arm
    /// (`oracle/codemp/server/sv_game.cpp:1627-1629`). Default `false`,
    /// flipped to `true` AT that syscall arm, not inside any `RmManager`
    /// method.
    // Flipped/read at the Wave-20 `G_RMG_INIT` syscall arm, which has not
    // landed in this tree yet (doc: "flipped to `true` AT that syscall arm, not
    // inside any `RmManager` method"), so nothing in-crate reads it back yet.
    #[allow(dead_code)]
    initialized: bool,
    /// `CRMManager::mLandScape` cache (`RM_Manager.h:14`) — `None` until
    /// `set_landscape` (matches the NULL-zeroing ctor, `RM_Manager.cpp:34-42`).
    land: Option<TerrainHandle>,
}

impl RmManager {
    /// `CRMManager::CRMManager` — null/zero-inits the members the kept Rust
    /// field set mirrors (`mLandScape`/`mTerrain`/`mMission` NULL). The only
    /// non-default member, `mCurPriority = 1` (`:39`), is §20-dropped
    /// (RMG-D4c). Default-equivalent to `RmManager::default()`.
    ///
    /// Source: `oracle/codemp/RMG/RM_Manager.cpp:34-42`
    pub fn new() -> Self {
        Self::default()
    }

    /// `CRMManager::SetLandScape` — stores the handle into `self.land` as
    /// `Some(land)`; `mTerrain = GetRandomTerrain()` is always `None` under
    /// DEDICATED (RMG-D1).
    ///
    /// Source: `oracle/codemp/RMG/RM_Manager.cpp:79`
    pub fn set_landscape(&mut self, land: TerrainHandle) {
        self.land = Some(land);
    }

    /// `CRMManager::LoadMission` — prints the RMG banner (guarded
    /// `#ifndef FINAL_BUILD`, `:105-108`) then early-outs `false`: `mTerrain`
    /// is always NULL under DEDICATED (RMG-D1, `:110-113`) — never constructs
    /// a mission.
    ///
    /// **Seam deviation (not a design change).** Raven's `LoadMission` takes
    /// only `qboolean IsServer` and reaches the landscape through the
    /// `RmManager` members `mLandScape`/`mTerrain` — all past the
    /// `if (!mTerrain) return false` early-out. Per §B (no hidden globals)
    /// `RmManager` owns only `land: Option<TerrainHandle>`; the `CCMLandScape`
    /// data lives in `CollisionWorld` (STATE-D2), so `cm` threads it as the
    /// §B4 substitute for those dropped members. On the live DEDICATED path
    /// both `cm` and `is_server` are unused — the body is just the
    /// `#ifndef FINAL_BUILD` banner print + `false`. **KEEPS its full
    /// faithful signature — RESOLVED by RMG-D8/ruling 47:** `load_mission` is
    /// a live call transcribed 1:1 at the Wave-20 arm; ruling 38 collapsed the
    /// dead `spawn_mission` *call*, not this signature.
    ///
    /// Source: `oracle/codemp/RMG/RM_Manager.cpp:96-113`
    pub fn load_mission(
        &mut self,
        cm: &mut CollisionWorld,
        host: &mut impl EngineHost,
        is_server: bool,
    ) -> bool {
        let _ = (cm, is_server);
        // `#ifndef FINAL_BUILD` banner (`:105-108`) — the harness/port both
        // compile the non-FINAL_BUILD TU, so this always prints (Verification
        // strategy).
        host.print("--------- Random Mission Manager ---------\n\n");
        host.print("RMG version : 1.01\n\n");

        // `if (!mTerrain) return false` (`:110-113`) — mTerrain is always NULL
        // under DEDICATED (RMG-D1: GetRandomTerrain() == 0), so RmManager
        // (owning only `land`, not the dropped `mTerrain` member) always
        // early-outs here; the mission-construction body past this point
        // (`:115-135`+) is unreachable and not ported (§20).
        false
    }

    /// `CRMManager::GetAutomapSymbolCount` — **no backing storage; hardcoded
    /// `0`.** Raven's `mAutomapSymbols[MAX_AUTOMAP_SYMBOLS]` /
    /// `mAutomapSymbolCount` pair (`RM_Manager.h:20-21`) is NOT mirrored as an
    /// `RmManager` field: its sole writer `AddAutomapSymbol`
    /// (`RM_Manager.cpp:400`, incremented only at `:410`) is §20-dropped
    /// (never reached under DEDICATED), so the count stays at its ctor value
    /// `0` (`:41`) for the process lifetime.
    ///
    /// Source: `oracle/codemp/RMG/RM_Manager.cpp:413`
    pub fn automap_symbol_count(&self) -> i32 {
        0
    }

    /// `CRMManager::GetAutomapSymbol` — **returns `None` unconditionally.**
    /// With no backing array (above) and `automap_symbol_count` always `0`, a
    /// well-behaved caller (`SV_WriteRMGAutomapSymbols` walks `0..count`,
    /// `sv_client.cpp:668-684`) never calls this. Raven's body is the
    /// unchecked `&mAutomapSymbols[index]` (`:420`, UB on any out-of-range/
    /// negative `index`). **§19 (RMG-D8/ruling 47):** the unchecked C index
    /// becomes a `.get()` — since there is no backing array (§20) every
    /// `index` yields `None`; no bounds/sign check on real storage occurs.
    ///
    /// Source: `oracle/codemp/RMG/RM_Manager.cpp:418-420`
    pub fn automap_symbol(&self, _index: i32) -> Option<&RmAutomapSymbol> {
        // No backing array (§20) — every index yields `None` (§19, RMG-D8 /
        // ruling 47: the ruled rendering of Raven's unchecked
        // `&mAutomapSymbols[index]`).
        None
    }

    /// `CRMManager::GetLandScape` — returns the stored handle (`self.land`);
    /// the snapshot read (Seam-C) resolves it against the owning
    /// `CollisionWorld`. `None` before the `G_RMG_INIT` arm's
    /// `set_landscape` (ruling 28/RMG-D1).
    ///
    /// Source: `oracle/codemp/RMG/RM_Manager.h:39`
    pub fn land(&self) -> Option<TerrainHandle> {
        self.land
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `new()`/`Default` collapse to one Default-equivalent construction
    /// (`RM_Manager.cpp:34-42`): no landscape until `set_landscape`.
    #[test]
    fn new_starts_with_no_landscape() {
        let rm = RmManager::new();
        assert_eq!(rm.land(), None);
    }

    /// `SetLandScape` (`RM_Manager.cpp:79-83`) stores the handle; `mTerrain`
    /// (always `None` under DEDICATED, RMG-D1) has no Rust field to observe.
    #[test]
    fn set_landscape_round_trips_through_land() {
        let mut rm = RmManager::new();
        let handle = TerrainHandle(7);
        rm.set_landscape(handle);
        assert_eq!(rm.land(), Some(handle));
    }

    /// `GetAutomapSymbolCount`/`GetAutomapSymbol` (`RM_Manager.cpp:413-420`) —
    /// with no backing array (§20, `AddAutomapSymbol` unreachable under
    /// DEDICATED), the count is frozen `0` and every index reads `None`,
    /// including negative/large indices that would be UB on Raven's raw
    /// array (§19, RMG-D8/ruling 47).
    #[test]
    fn automap_symbols_are_always_empty() {
        let rm = RmManager::new();
        assert_eq!(rm.automap_symbol_count(), 0);
        assert!(rm.automap_symbol(0).is_none());
        assert!(rm.automap_symbol(-1).is_none());
        assert!(rm.automap_symbol(511).is_none());
    }

    // `load_mission` (RM_Manager.cpp:96-113) is not exercised here: it needs a
    // live `&mut CollisionWorld`, which `mp_engine_qcommon` exposes with no
    // public constructor/`Default` impl (a private field blocks construction
    // from outside that crate) — reported under `problems`, not invented
    // around. Its banner-print + always-`false` behavior is covered by the
    // `tools/rmg-oracle/` differential harness (Verification strategy).
}
