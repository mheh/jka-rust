//! `CgSagaState` — `cg_saga.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::fmt;

use mp_qshared::shared::MAX_CLIENTS;

use crate::local::siege_extended_s::siegeExtended_t;

/// `siegeExtended_t` (`crate::local::siege_extended_s`) has no derives of its
/// own (it's a plain `#[repr(C)]` ABI type, not owned by this file); these
/// impls give `CgSagaState` the `Debug`/`Clone`/`Default` it needs without
/// touching that file, since same-crate impls aren't subject to the orphan
/// rule.
impl fmt::Debug for siegeExtended_t {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("siegeExtended_t")
            .field("health", &self.health)
            .field("maxhealth", &self.maxhealth)
            .field("ammo", &self.ammo)
            .field("weapon", &self.weapon)
            .field("lastUpdated", &self.lastUpdated)
            .finish()
    }
}

impl Clone for siegeExtended_t {
    fn clone(&self) -> Self {
        siegeExtended_t {
            health: self.health,
            maxhealth: self.maxhealth,
            ammo: self.ammo,
            weapon: self.weapon,
            lastUpdated: self.lastUpdated,
        }
    }
}

impl Copy for siegeExtended_t {}

impl Default for siegeExtended_t {
    /// Raven's zeroed BSS.
    fn default() -> Self {
        siegeExtended_t {
            health: 0,
            maxhealth: 0,
            ammo: 0,
            weapon: 0,
            lastUpdated: 0,
        }
    }
}

/// `cg_saga.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Fields fold in as the waves transcribe `cg_saga.c`'s file-scope statics
/// (DEC-46.1), so a wave transcriber only ever touches its own TU's two files —
/// the function file and this one — and never `cg_world.rs`. Raven's read-only
/// tables beside them are compiled-in data, not state; they land as `const`s
/// beside the functions that read them (§C8).
///
/// `cgParseObjectives` (`static char cgParseObjectives[MAX_SIEGE_INFO_SIZE]`)
/// is not a field here: every reader in this wave writes it via
/// `BG_SiegeGetValueGroup` and reads it straight back in the same call, so
/// it's call-scoped scratch under the `Option<String>`-returning bg signature —
/// never cross-call state — and stays a local.
///
/// Source: `oracle/codemp/cgame/cg_saga.c:15-27,984`
#[derive(Debug, Clone, Default)]
pub struct CgSagaState {
    /// Raven `int cgSiegeRoundState` — set by `CG_ParseSiegeState`.
    /// Source: `oracle/codemp/cgame/cg_saga.c:15`
    pub cgSiegeRoundState: i32,

    /// Raven `int cgSiegeRoundTime` — set by `CG_ParseSiegeState`.
    /// Source: `oracle/codemp/cgame/cg_saga.c:16`
    pub cgSiegeRoundTime: i32,

    /// Raven `static char team1[512]` — side-1 siege team name, set by the
    /// `.siege` file's `Teams` group parse (a later wave) and read by the
    /// objective lookups here.
    /// Source: `oracle/codemp/cgame/cg_saga.c:18`
    pub team1: String,

    /// Raven `static char team2[512]` — side-2 siege team name.
    /// Source: `oracle/codemp/cgame/cg_saga.c:19`
    pub team2: String,

    /// Raven `int team1Timed` — side-1's round time limit in msec, 0 when the
    /// team has no `Timed` entry.
    /// Source: `oracle/codemp/cgame/cg_saga.c:21`
    pub team1Timed: i32,

    /// Raven `int team2Timed` — side-2's round time limit in msec.
    /// Source: `oracle/codemp/cgame/cg_saga.c:22`
    pub team2Timed: i32,

    /// Raven `int cgSiegeTeam1PlShader` — friendly-player HUD shader for side 1.
    /// Raven types it `int` even though it holds a `qhandle_t`; kept as `i32`.
    /// Source: `oracle/codemp/cgame/cg_saga.c:24`
    pub cgSiegeTeam1PlShader: i32,

    /// Raven `int cgSiegeTeam2PlShader` — friendly-player HUD shader for side 2.
    /// Source: `oracle/codemp/cgame/cg_saga.c:25`
    pub cgSiegeTeam2PlShader: i32,

    /// Raven `siegeExtended_t cg_siegeExtendedData[MAX_CLIENTS]` — per-client
    /// cached health/ammo/weapon HUD extras for siege mode, refreshed from the
    /// server's extended-data string.
    /// Source: `oracle/codemp/cgame/cg_saga.c:984`,
    /// `oracle/codemp/cgame/cg_local.h:1621`
    pub cg_siegeExtendedData: [siegeExtended_t; MAX_CLIENTS],
}
