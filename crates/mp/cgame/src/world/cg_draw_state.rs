//! `CgDrawState` — `cg_draw.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::team_maxoverlay::TEAM_MAXOVERLAY;
use mp_qshared::shared::{vec4_t, ENTITYNUM_NONE};

use crate::cg_draw::LAG_SAMPLES;

/// Raven `lagometer_t` — the lagometer's two ring buffers plus their write
/// counters. Only one instance ever exists (the `lagometer` global below), so
/// it lives here beside it rather than under `local/`.
///
/// Type definition source: `oracle/codemp/cgame/cg_draw.c:4143-4150`
#[derive(Debug, Clone)]
pub struct lagometer_t {
    pub frameSamples: [c_int; LAG_SAMPLES],
    pub frameCount: c_int,
    pub snapshotFlags: [c_int; LAG_SAMPLES],
    pub snapshotSamples: [c_int; LAG_SAMPLES],
    pub snapshotCount: c_int,
}

impl Default for lagometer_t {
    /// Raven's zeroed BSS.
    fn default() -> Self {
        lagometer_t {
            frameSamples: [0; LAG_SAMPLES],
            frameCount: 0,
            snapshotFlags: [0; LAG_SAMPLES],
            snapshotSamples: [0; LAG_SAMPLES],
            snapshotCount: 0,
        }
    }
}

/// `cg_draw.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Fields fold in as the waves transcribe `cg_draw.c`'s file-scope statics
/// (DEC-46.1), so a wave transcriber only ever touches its own TU's two files —
/// the function file and this one — and never `cg_world.rs`. Raven's read-only
/// tables beside them are compiled-in data, not state; they land as `const`s
/// beside the functions that read them (§C8).
///
/// Source: `oracle/codemp/cgame/cg_draw.c:23-40,1791-1792,1940-1941,2196,2425,3167,3172-3174,4152,4738-4740,4799-4803,4847,5325-5326,7317-7338,7351-7354,7481`
#[derive(Debug, Clone)]
pub struct CgDrawState {
    /// Raven `int cg_targVeh` — the vehicle the targeting HUD is locked onto.
    /// Source: `oracle/codemp/cgame/cg_draw.c:1791`
    pub cg_targVeh: c_int,

    /// Raven `int cg_targVehLastTime` — `cg.time` of the last lock; drives the
    /// 3-second fade-out.
    /// Source: `oracle/codemp/cgame/cg_draw.c:1792`
    pub cg_targVehLastTime: c_int,

    /// Raven `lagometer_t lagometer`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:4152`
    pub lagometer: lagometer_t,

    /// Raven `float cg_crosshairPrevPosX` — last frame's crosshair X, the blend
    /// source for [`crate::cg_draw::CG_LerpCrosshairPos`].
    /// Source: `oracle/codemp/cgame/cg_draw.c:4802`
    pub cg_crosshairPrevPosX: f32,

    /// Raven `float cg_crosshairPrevPosY`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:4803`
    pub cg_crosshairPrevPosY: f32,

    /// Raven `int sortedTeamPlayers[TEAM_MAXOVERLAY]` — client numbers the
    /// team overlay shows, filled by the scoreboard sort.
    /// Source: `oracle/codemp/cgame/cg_draw.c:28`
    pub sortedTeamPlayers: [c_int; TEAM_MAXOVERLAY],

    /// Raven `int numSortedTeamPlayers`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:29`
    pub numSortedTeamPlayers: c_int,

    /// Raven `int cg_genericTimerBar` — `cg.time` deadline of the generic HUD
    /// timer bar; 0 when idle.
    /// Source: `oracle/codemp/cgame/cg_draw.c:4738`
    pub cg_genericTimerBar: c_int,

    /// Raven `int cg_genericTimerDur`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:4739`
    pub cg_genericTimerDur: c_int,

    /// Raven `vec4_t cg_genericTimerColor`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:4740`
    pub cg_genericTimerColor: vec4_t,

    /// Raven `int cgSiegeEntityRender` — the siege item entity flagged for an
    /// icon this frame; reset to 0 after each draw.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7481`
    pub cgSiegeEntityRender: c_int,

    /// Raven `int cgSiegeRoundBeganTime` — latched by `CG_ParseSiegeState`
    /// when the round enters pre-round or post-round.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7353`
    pub cgSiegeRoundBeganTime: c_int,

    /// Raven `float *hudTintColor` — the HUD tint `CG_DrawHUD` re-points at
    /// `redhudtint`/`bluehudtint`/`colorTable[CT_WHITE]` each frame. Raven's
    /// pointer starts NULL and `trap_R_SetColor(NULL)` is the renderer's
    /// reset-to-white, so `None` carries that starting state honestly rather
    /// than inventing a colour.
    /// Source: `oracle/codemp/cgame/cg_draw.c:26,1234-1242`
    pub hudTintColor: Option<vec4_t>,

    /// Raven `int cg_vehicleAmmoWarning` — which vehicle weapon (0 = upper,
    /// 1 = lower) the low-ammo flash belongs to.
    /// Source: `oracle/codemp/cgame/cg_draw.c:1940`
    pub cg_vehicleAmmoWarning: c_int,

    /// Raven `int cg_vehicleAmmoWarningTime` — `cg.time` the flash runs until.
    /// Source: `oracle/codemp/cgame/cg_draw.c:1941`
    pub cg_vehicleAmmoWarningTime: c_int,

    /// Raven `qboolean cg_drawLink` — last frame's weapons-linked state; a
    /// change plays the link sound once.
    /// Source: `oracle/codemp/cgame/cg_draw.c:2196`
    pub cg_drawLink: bool,

    /// Raven `float cg_radarRange` — the radar's world-units range, overridden
    /// per map by `CG_ParseEntityFromSpawnVars`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:3167`,
    /// `oracle/codemp/cgame/cg_main.c:3631`
    pub cg_radarRange: f32,

    /// Raven `static int radarLockSoundDebounceTime` — next `cg.time` the
    /// missile-lock alarm may re-fire.
    /// Source: `oracle/codemp/cgame/cg_draw.c:3172`
    pub radarLockSoundDebounceTime: c_int,

    /// Raven `static int impactSoundDebounceTime` — next `cg.time` the
    /// asteroid-impact alarm may re-fire. Doubles as the fade clock for the
    /// asteroid blip's alpha, so it is read as well as debounced.
    /// Source: `oracle/codemp/cgame/cg_draw.c:3173`
    pub impactSoundDebounceTime: c_int,
}

impl Default for CgDrawState {
    /// Raven's initializers — everything is zeroed BSS except `cg_targVeh`
    /// (`ENTITYNUM_NONE`) and `cg_radarRange` (2500).
    fn default() -> Self {
        CgDrawState {
            cg_targVeh: ENTITYNUM_NONE,
            cg_targVehLastTime: 0,
            lagometer: lagometer_t::default(),
            cg_crosshairPrevPosX: 0.0,
            cg_crosshairPrevPosY: 0.0,
            sortedTeamPlayers: [0; TEAM_MAXOVERLAY],
            numSortedTeamPlayers: 0,
            cg_genericTimerBar: 0,
            cg_genericTimerDur: 0,
            cg_genericTimerColor: [0.0; 4],
            cgSiegeEntityRender: 0,
            cgSiegeRoundBeganTime: 0,
            hudTintColor: None,
            cg_vehicleAmmoWarning: 0,
            cg_vehicleAmmoWarningTime: 0,
            cg_drawLink: false,
            cg_radarRange: 2500.0,
            radarLockSoundDebounceTime: 0,
            impactSoundDebounceTime: 0,
        }
    }
}
