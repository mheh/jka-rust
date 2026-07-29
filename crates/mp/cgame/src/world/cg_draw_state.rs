//! `CgDrawState` — `cg_draw.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::team_maxoverlay::TEAM_MAXOVERLAY;
use mp_qshared::shared::{vec3_t, vec4_t, ENTITYNUM_NONE};

use crate::cg_draw::{FPS_FRAMES, LAG_SAMPLES};

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
/// Source: `oracle/codemp/cgame/cg_draw.c:23-40,1791-1792,1940-1941,2196,2425,3167,3172-3174,4152,4738-4740,4799-4803,4847,5325-5326,7317-7335,7351-7354,7481`
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

    /// Raven `int lastvalidlockdif` — the last rocket-lock wedge count taken
    /// while the lock was still live; replayed once the lock time goes to -1.
    /// Source: `oracle/codemp/cgame/cg_draw.c:31`
    pub lastvalidlockdif: c_int,

    /// Raven's `static qboolean flip = qtrue` inside `CG_DrawZoomMask` — which
    /// way the binocular mask's top triangle currently points.
    /// Source: `oracle/codemp/cgame/cg_draw.c:224`
    pub flip: bool,

    /// Raven `int cg_beatingSiegeTime` — the "beat this time" siege target in
    /// msec, off `CS_SIEGE_TIMEOVERRIDE`; `cg_main.c`/`cg_servercmds.c` write it.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7351`
    pub cg_beatingSiegeTime: c_int,

    /// Raven: The time at which you died and the time it will take for you to
    /// rejoin game. Written by `EV_SIEGESPEC`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:40`
    pub cg_siegeDeathTime: c_int,

    /// Raven `int cgSiegeRoundCountTime` — the last round-begin countdown value
    /// (1/2/3) the announcer already spoke; guards the count sound to once each.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7354`
    pub cgSiegeRoundCountTime: c_int,

    /// Raven `vec3_t gCGFallVector` — the origin latched the frame the local
    /// client started falling to his death; the death-cam looks back at it.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7338`
    pub gCGFallVector: vec3_t,

    /// Raven `qboolean gCGHasFallVector` — whether `gCGFallVector` currently
    /// holds a live fall origin.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7337`
    pub gCGHasFallVector: bool,

    /// Raven's `static int oldDif = 0` inside `CG_DrawRocketLocking` — last
    /// frame's wedge count; a change fires the tick/lock sound.
    /// Source: `oracle/codemp/cgame/cg_draw.c:5752`
    pub oldDif: c_int,

    /// Raven `vec3_t cg_crosshairPos` — the world point the crosshair was last
    /// painted at, latched by `CG_DrawCrosshair` for the zoom/lock overlays.
    /// Source: `oracle/codemp/cgame/cg_draw.c:4847`
    pub cg_crosshairPos: vec3_t,

    /// Raven `int cg_saberFlashTime` — `cg.time` of the last saber clash; the
    /// flare lives for 150ms after it.
    /// Source: `oracle/codemp/cgame/cg_draw.c:5325`
    pub cg_saberFlashTime: c_int,

    /// Raven `vec3_t cg_saberFlashPos` — where that clash happened.
    /// Source: `oracle/codemp/cgame/cg_draw.c:5326`
    pub cg_saberFlashPos: vec3_t,

    /// Raven `int cgRageTime` — `cg.time` the rage screen tint started.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7317`
    pub cgRageTime: c_int,

    /// Raven `int cgRageFadeTime`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7318`
    pub cgRageFadeTime: c_int,

    /// Raven `float cgRageFadeVal`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7319`
    pub cgRageFadeVal: f32,

    /// Raven `int cgRageRecTime` — same clock for the rage-recovery grey.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7321`
    pub cgRageRecTime: c_int,

    /// Raven `int cgRageRecFadeTime`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7322`
    pub cgRageRecFadeTime: c_int,

    /// Raven `float cgRageRecFadeVal`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7323`
    pub cgRageRecFadeVal: f32,

    /// Raven `int cgAbsorbTime` — `cg.time` the absorb tint started.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7325`
    pub cgAbsorbTime: c_int,

    /// Raven `int cgAbsorbFadeTime`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7326`
    pub cgAbsorbFadeTime: c_int,

    /// Raven `float cgAbsorbFadeVal`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7327`
    pub cgAbsorbFadeVal: f32,

    /// Raven `int cgProtectTime` — `cg.time` the protect tint started.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7329`
    pub cgProtectTime: c_int,

    /// Raven `int cgProtectFadeTime`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7330`
    pub cgProtectFadeTime: c_int,

    /// Raven `float cgProtectFadeVal`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7331`
    pub cgProtectFadeVal: f32,

    /// Raven `int cgYsalTime` — `cg.time` the ysalamiri tint started.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7333`
    pub cgYsalTime: c_int,

    /// Raven `int cgYsalFadeTime`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7334`
    pub cgYsalFadeTime: c_int,

    /// Raven `float cgYsalFadeVal`.
    /// Source: `oracle/codemp/cgame/cg_draw.c:7335`
    pub cgYsalFadeVal: f32,

    /// Raven's `static unsigned short previousTimes[FPS_FRAMES]` inside
    /// `CG_DrawFPS` — the ring buffer of recent frame times the fps counter
    /// averages. `fps`-prefixed here since its three siblings have names too
    /// generic to fold bare into the shared draw struct.
    /// Source: `oracle/codemp/cgame/cg_draw.c:3074`
    pub fpsPreviousTimes: [u16; FPS_FRAMES],

    /// Raven's `static unsigned short index` inside `CG_DrawFPS` — the ring
    /// write cursor (wrapped by `% FPS_FRAMES`).
    /// Source: `oracle/codemp/cgame/cg_draw.c:3075`
    pub fpsIndex: u16,

    /// Raven's `static int previous` inside `CG_DrawFPS` — last frame's
    /// `trap_Milliseconds`, so this frame's time is the delta.
    /// Source: `oracle/codemp/cgame/cg_draw.c:3076`
    pub fpsPrevious: c_int,

    /// Raven's `static int lastupdate` inside `CG_DrawFPS` — the last time we
    /// wrote a sample; caps sampling at 20Hz.
    /// Source: `oracle/codemp/cgame/cg_draw.c:3076`
    pub fpsLastupdate: c_int,
}

impl Default for CgDrawState {
    /// Raven's initializers — everything is zeroed BSS except `cg_targVeh`
    /// (`ENTITYNUM_NONE`), `cg_radarRange` (2500) and `flip` (`qtrue`).
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
            lastvalidlockdif: 0,
            flip: true,
            oldDif: 0,
            cg_crosshairPos: [0.0; 3],
            cg_saberFlashTime: 0,
            cg_saberFlashPos: [0.0; 3],
            cgRageTime: 0,
            cgRageFadeTime: 0,
            cgRageFadeVal: 0.0,
            cgRageRecTime: 0,
            cgRageRecFadeTime: 0,
            cgRageRecFadeVal: 0.0,
            cgAbsorbTime: 0,
            cgAbsorbFadeTime: 0,
            cgAbsorbFadeVal: 0.0,
            cgProtectTime: 0,
            cgProtectFadeTime: 0,
            cgProtectFadeVal: 0.0,
            cgYsalTime: 0,
            cgYsalFadeTime: 0,
            cgYsalFadeVal: 0.0,
            cg_beatingSiegeTime: 0,
            cg_siegeDeathTime: 0,
            cgSiegeRoundCountTime: 0,
            gCGFallVector: [0.0; 3],
            gCGHasFallVector: false,
            fpsPreviousTimes: [0; FPS_FRAMES],
            fpsIndex: 0,
            fpsPrevious: 0,
            fpsLastupdate: 0,
        }
    }
}
