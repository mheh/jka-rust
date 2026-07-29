//! Port of `oracle/codemp/cgame/cg_view.c` — view/camera placement, fov, and the per-frame scene build. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case, non_upper_case_globals)]

use core::f64::consts::PI;
use core::ffi::c_int;

use mp_bg::bg_misc::BG_EmplacedView;
use mp_bg::bg_panimate::{BG_InGrappleMove, BG_SaberInSpecial, PM_InKnockDown};
use mp_bg::bg_pmove::BG_UnrestrainedPitchRoll;
use mp_bg::public::configstring::{CS_GLOBAL_AMBIENT_SET, CS_SKYBOXORG};
use mp_bg::public::dm_flags::DF_FIXED_FOV;
use mp_bg::public::entity_effects::EF2_HELD_BY_MONSTER;
use mp_bg::public::entity_flags::{EF_NODRAW, EF_SOUNDTRACKER};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::force_hand_anims::forceHandAnims_t::HANDEXTEND_KNOCKDOWN;
use mp_bg::public::gametype::{GT_SIEGE, GT_TEAM};
use mp_bg::public::hyperspace::HYPERSPACE_TIME;
use mp_bg::public::pers_enum::persEnum_t::PERS_TEAM;
use mp_bg::public::pmtype::pmtype_t;
use mp_bg::public::stat_index::statIndex_t::{STAT_DEAD_YAW, STAT_HEALTH};
use mp_bg::public::team::TEAM_SPECTATOR;
use mp_bg::public::viewheight::{DEFAULT_MAXS_2, DEFAULT_MINS_2};
use mp_bg::weapons::weapon_t::{WP_EMPLACED_GUN, WP_MELEE, WP_SABER};
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::refdef_t::{
    refdef_t, MAX_MAP_AREA_BYTES, MAX_RENDER_STRINGS, MAX_RENDER_STRING_LENGTH,
};
use mp_qshared::common::mp::cgame::stereo_frame_t::{stereoFrame_t, STEREO_RIGHT};
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::common::mp::qcommon::player_state::{playerState_t, MAX_POWERUPS};
use mp_qshared::common::mp::qcommon::pm_flags::{PMF_DUCKED, PMF_FOLLOW};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::com_parse::{COM_ParseExt, QSharedScratch};
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin,
    vectoangles, AngleNormalize180, AngleVectors, AnglesToAxis, Q_fabs, VectorLength,
    VectorNormalize, VectorSet, PITCH, ROLL, YAW,
};
use mp_qshared::shared::sound_channel::{CHAN_ANNOUNCER, CHAN_LOCAL};
use mp_qshared::shared::surface_flags::{
    CONTENTS_LAVA, CONTENTS_PLAYERCLIP, CONTENTS_SLIME, CONTENTS_WATER, MASK_SOLID, SOLID_BMODEL,
};
use mp_qshared::shared::{
    qboolean, qfalse, qtrue, sfxHandle_t, trType_t, vec3_t, ENTITYNUM_NONE, ENTITYNUM_WORLD,
    MAX_QPATH, SNAPFLAG_NOT_ACTIVE,
};
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;
use native_string::{atof, atoi, buf_to_string, Q_strncpyz};

use crate::cg_draw::{CG_AddLagometerFrameInfo, CG_DrawActive};
use crate::cg_drawtools::CG_DrawPic;
use crate::cg_ents::{CG_AddPacketEntities, CG_CalcEntityLerpPositions, CG_S_UpdateLoopingSounds};
use crate::cg_info::CG_DrawInformation;
use crate::cg_light::CG_RunLightStyles;
use crate::cg_localents::CG_AddLocalEntities;
use crate::cg_main::{
    CG_Argv, CG_ConfigString, CG_DrawMiscEnts, CG_Error, CG_Printf, CG_UpdateCvars,
};
use crate::cg_marks::{CG_AddMarks, CG_AddParticles};
use crate::cg_players::CG_ActualLoadDeferredPlayers;
use crate::cg_predict::{CG_PointContents, CG_PredictPlayerState, CG_Trace};
use crate::cg_snapshot::CG_ProcessSnapshots;
use crate::cg_weapons::{CG_AddViewWeapon, LAND_DEFLECT_TIME, LAND_RETURN_TIME};
use crate::local::cg_t::MAX_SOUNDBUFFER;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

// ---------------------------------------------------------------------------
// File-scope constants
// Source: `oracle/codemp/cgame/cg_view.c:13-14,219-222,808,1034,1187-1188,2006-2007,2282-2283`
// ---------------------------------------------------------------------------

/// Raven `MASK_CAMERACLIP` — what the third-person camera trace collides with.
/// Source: `oracle/codemp/cgame/cg_view.c:13`
pub const MASK_CAMERACLIP: c_int = MASK_SOLID | CONTENTS_PLAYERCLIP;

/// Raven `CAMERA_SIZE` — half-extent of the camera's collision box.
/// Source: `oracle/codemp/cgame/cg_view.c:14`
pub const CAMERA_SIZE: f32 = 4.0;

/// Raven `CAMERA_DAMP_INTERVAL` — msec between camera damping updates.
/// Source: `oracle/codemp/cgame/cg_view.c:219`
pub const CAMERA_DAMP_INTERVAL: c_int = 50;

/// Raven `static vec3_t cameramins` — the camera trace's mins.
/// Source: `oracle/codemp/cgame/cg_view.c:221`
pub const cameramins: vec3_t = [-CAMERA_SIZE, -CAMERA_SIZE, -CAMERA_SIZE];

/// Raven `static vec3_t cameramaxs` — the camera trace's maxs.
/// Source: `oracle/codemp/cgame/cg_view.c:222`
pub const cameramaxs: vec3_t = [CAMERA_SIZE, CAMERA_SIZE, CAMERA_SIZE];

/// Raven `FOCUS_DISTANCE` — how far ahead the first-person view focuses.
/// Source: `oracle/codemp/cgame/cg_view.c:808`
pub const FOCUS_DISTANCE: f32 = 512.0;

/// Raven `NECK_LENGTH` — the offset the view drops when the player dies.
/// Source: `oracle/codemp/cgame/cg_view.c:1034`
pub const NECK_LENGTH: f32 = 8.0;

/// Raven `WAVE_AMPLITUDE` — underwater fov warp amplitude. Only the `#if 0`
/// arm of [`CG_CalcFOVFromX`] ever read it.
/// Source: `oracle/codemp/cgame/cg_view.c:1187`
pub const WAVE_AMPLITUDE: f32 = 1.0;

/// Raven `WAVE_FREQUENCY` — underwater fov warp frequency. Same dead arm as
/// [`WAVE_AMPLITUDE`].
/// Source: `oracle/codemp/cgame/cg_view.c:1188`
pub const WAVE_FREQUENCY: f64 = 0.4;

/// Raven `CAMERA_DEFAULT_FOV` — the fov the shake intensity scale normalizes
/// against.
/// Source: `oracle/codemp/cgame/cg_view.c:2006`
pub const CAMERA_DEFAULT_FOV: f32 = 90.0;

/// Raven `MAX_SHAKE_INTENSITY` — [`CGCam_Shake`] clamps to this.
/// Source: `oracle/codemp/cgame/cg_view.c:2007`
pub const MAX_SHAKE_INTENSITY: f32 = 16.0;

/// Raven `SIDEFRAME_WIDTH` — the letterbox side frame's width.
/// Source: `oracle/codemp/cgame/cg_view.c:2282`
pub const SIDEFRAME_WIDTH: c_int = 16;

/// Raven `SIDEFRAME_HEIGHT` — the letterbox side frame's height.
/// Source: `oracle/codemp/cgame/cg_view.c:2283`
pub const SIDEFRAME_HEIGHT: c_int = 32;

// ---------------------------------------------------------------------------
// `cg_local.h` constants this TU reads. The header has no ported cross-crate
// home yet, so they land here beside their readers (§C8).
// ---------------------------------------------------------------------------

/// Raven `POWERUP_BLINKS` — how many times a powerup icon blinks before it
/// runs out.
/// Source: `oracle/codemp/cgame/cg_local.h:23`
const POWERUP_BLINKS: c_int = 5;

/// Raven `POWERUP_BLINK_TIME` — msec per blink.
/// Source: `oracle/codemp/cgame/cg_local.h:25`
const POWERUP_BLINK_TIME: c_int = 1000;

/// Raven `DAMAGE_DEFLECT_TIME` — how long the damage view kick winds out, in
/// msec.
/// Source: `oracle/codemp/cgame/cg_local.h:28`
const DAMAGE_DEFLECT_TIME: c_int = 100;

/// Raven `DAMAGE_RETURN_TIME` — the recovery tail after the deflect, in msec.
/// Source: `oracle/codemp/cgame/cg_local.h:29`
const DAMAGE_RETURN_TIME: c_int = 400;

/// Raven `DAMAGE_TIME` — how long the damage blend blob lives, in msec.
/// Source: `oracle/codemp/cgame/cg_local.h:30`
pub(crate) const DAMAGE_TIME: c_int = 500;

/// Raven `DUCK_TIME` — how long the crouch view drop smooths over, in msec.
/// Source: `oracle/codemp/cgame/cg_local.h:34`
const DUCK_TIME: c_int = 100;

/// Raven `MAX_ZOOM_FOV` — the tightest the disruptor scope zooms in to.
/// Source: `oracle/codemp/cgame/cg_local.h:41`
const MAX_ZOOM_FOV: f32 = 3.0;

/// Raven `ZOOM_OUT_TIME` — msec the fov takes to blend back out of a zoom.
/// Source: `oracle/codemp/cgame/cg_local.h:43`
const ZOOM_OUT_TIME: f32 = 100.0;

/// Raven `STEP_TIME` — how long the stair-climb view smoothing lasts, in msec.
/// Source: `oracle/codemp/cgame/cg_local.h:33`
const STEP_TIME: c_int = 200;

/// Raven `RF_FIRST_PERSON` — only draw through eyes (view weapon, damage blood
/// blob). `tr_types.h`'s renderfx bits have no ported cross-crate home yet, so
/// the one this TU sets lands here beside its reader (§C8).
/// Source: `oracle/codemp/cgame/tr_types.h:20`
const RF_FIRST_PERSON: c_int = 0x00004;

/// Raven `CG_CalcVrect` — sets the coordinates of the rendered window from
/// `cg_viewsize`, clamping the cvar back into 30..100 as a side effect.
///
/// Source: `oracle/codemp/cgame/cg_view.c:174-201`
pub fn CG_CalcVrect(ctx: &mut CgContext) {
    let size: c_int;

    // the intermission should allways be full screen
    // Raven derefs `cg.snap` unguarded; a null there is UB, so the port reads
    // it as "not the intermission" and falls through to the cvar (§F19).
    let intermission = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.pm_type == pmtype_t::PM_INTERMISSION as c_int)
        .unwrap_or(false);

    if intermission {
        size = 100;
    } else {
        // bound normal viewsize
        if ctx.world.cvars.cg_viewsize.integer < 30 {
            trap::Cvar_Set(ctx.engine, "cg_viewsize", "30");
            size = 30;
        } else if ctx.world.cvars.cg_viewsize.integer > 100 {
            trap::Cvar_Set(ctx.engine, "cg_viewsize", "100");
            size = 100;
        } else {
            size = ctx.world.cvars.cg_viewsize.integer;
        }
    }

    let vidWidth = ctx.world.cgs.glconfig.vidWidth;
    let vidHeight = ctx.world.cgs.glconfig.vidHeight;
    let refdef = &mut ctx.world.cg.refdef;

    refdef.width = vidWidth * size / 100;
    refdef.width &= !1;

    refdef.height = vidHeight * size / 100;
    refdef.height &= !1;

    refdef.x = (vidWidth - refdef.width) / 2;
    refdef.y = (vidHeight - refdef.height) / 2;
}

/// Raven `CG_StepOffset` — smooth out stair climbing.
///
/// Source: `oracle/codemp/cgame/cg_view.c:208-217`
pub fn CG_StepOffset(world: &mut CgWorld) {
    // smooth out stair climbing
    let timeDelta = world.cg.time - world.cg.stepTime;
    if timeDelta < STEP_TIME {
        world.cg.refdef.vieworg[2] -=
            world.cg.stepChange * (STEP_TIME - timeDelta) as f32 / STEP_TIME as f32;
    }
}

/// Raven `CG_CalcIdealThirdPersonViewTarget` — where the third-person camera
/// would look with no damping: the eye point, raised by the view height and
/// the vertical offset.
///
/// Source: `oracle/codemp/cgame/cg_view.c:257-325`
pub fn CG_CalcIdealThirdPersonViewTarget(world: &mut CgWorld) {
    // Initialize IdealTarget
    //
    // DEFERRED: `gCGHasFallVector` / `gCGFallVector` — the two globals are
    // `cg_draw.c`'s (`oracle/codemp/cgame/cg_draw.c:7337-7338`), so they belong
    // to `CgDrawState`, and the wave that transcribes their only writer
    // (`cg_draw.c:8308-8319`) folds them in. Until then only the else arm is
    // reachable — the one that runs whenever the local client isn't falling to
    // his death.
    // Source: `oracle/codemp/cgame/cg_view.c:260-267`
    let mut cameraFocusLoc = world.cg.refdef.vieworg;

    // Add in the new viewheight
    // Raven derefs `cg.snap` unguarded here; a null there is UB, so the port
    // adds no view height rather than diverging further (§F19).
    let viewheight = world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.viewheight)
        .unwrap_or(0);
    cameraFocusLoc[2] += viewheight as f32;

    // Add in a vertical offset from the viewpoint, which puts the actual target above the head, regardless of angle.
    let mut cameraIdealTarget = cameraFocusLoc;

    {
        // not `mut` only because every arm that would reassign it is deferred
        // below.
        let vertOffset = world.cvars.cg_thirdPersonVertOffset.value;

        let m_iVehicleNum = world
            .cg
            .snap_ref()
            .map(|snap| snap.ps.m_iVehicleNum)
            .unwrap_or(0);
        if m_iVehicleNum != 0 && world.entity(m_iVehicleNum as usize).m_pVehicle.is_some() {
            // DEFERRED: `Vehicle_t::m_pVehicleInfo` — `cameraOverride`,
            // `cameraPitchDependantVertOffset`, `cameraVertOffset` and
            // `type == VH_ANIMAL` all hang off the `Vehicle_t` referent pool
            // behind `centity_t.m_pVehicle`, which lands with
            // `oracle/codemp/cgame/cg_players.c:7014-7042` (DEC-46.2). Only the
            // presence test is reachable, so `vertOffset` keeps the cvar value
            // — that matches Raven for any vehicle that isn't `cameraOverride`
            // and isn't `VH_ANIMAL` (Raven's else-if zeroes it for animals);
            // the animal case is a known divergence until the referent lands.
            // Source: `oracle/codemp/cgame/cg_view.c:284-320`
        }

        cameraIdealTarget[2] += vertOffset;
    }

    world.view.cameraFocusLoc = cameraFocusLoc;
    world.view.cameraIdealTarget = cameraIdealTarget;
}

/// Raven `CG_CalcIdealThirdPersonViewLocation` — backs the camera off the
/// ideal target along `camerafwd` by the third-person range.
///
/// Source: `oracle/codemp/cgame/cg_view.c:335-363`
pub fn CG_CalcIdealThirdPersonViewLocation(world: &mut CgWorld) {
    let mut thirdPersonRange = world.cvars.cg_thirdPersonRange.value;

    let snapPs = world.cg.snap_ref().map(|snap| {
        (
            snap.ps.m_iVehicleNum,
            snap.ps.eFlags2,
            snap.ps.hasLookTarget,
            snap.ps.lookTarget,
        )
    });

    if let Some((m_iVehicleNum, eFlags2, hasLookTarget, lookTarget)) = snapPs {
        if m_iVehicleNum != 0 && world.entity(m_iVehicleNum as usize).m_pVehicle.is_some() {
            // DEFERRED: `Vehicle_t::m_pVehicleInfo->cameraOverride` /
            // `cameraRange`, and `veh->playerState->hackingTime` — same missing
            // `Vehicle_t` referent pool as
            // `CG_CalcIdealThirdPersonViewTarget` above
            // (`oracle/codemp/cgame/cg_players.c:7014-7042`, DEC-46.2), so the
            // range keeps the cvar value.
            // Source: `oracle/codemp/cgame/cg_view.c:342-350`
        }

        // only possibility for now, may add Wampa and sand creature later
        if (eFlags2 & EF2_HELD_BY_MONSTER) != 0
            && hasLookTarget != qfalse
            && world.entity(lookTarget as usize).currentState.NPC_class
                == class_t::CLASS_RANCOR as c_int
        {
            // stay back
            thirdPersonRange = 120.0;
        }
    }

    let cameraIdealTarget = world.view.cameraIdealTarget;
    let camerafwd = world.view.camerafwd;
    _VectorMA(
        cameraIdealTarget,
        -thirdPersonRange,
        camerafwd,
        &mut world.view.cameraIdealLoc,
    );
}

/// Raven `CG_GetVehicleCamPos` — hands out wherever the view ended up this
/// frame. Raven's `vec3_t camPos` out-param is the return value (§C7).
///
/// Source: `oracle/codemp/cgame/cg_view.c:797-800`
pub fn CG_GetVehicleCamPos(world: &CgWorld) -> vec3_t {
    world.cg.refdef.vieworg
}

/// Raven `CG_ZoomDown_f` — the `+zoom` console command; latches the zoom start
/// time. Re-pressing while already zoomed does nothing.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1105-1111`
pub fn CG_ZoomDown_f(world: &mut CgWorld) {
    if world.cg.zoomed != qfalse {
        return;
    }
    world.cg.zoomed = qtrue;
    world.cg.zoomTime = world.cg.time;
}

/// Raven `CG_ZoomUp_f` — the `-zoom` console command; latches the zoom-out
/// start time.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1113-1119`
pub fn CG_ZoomUp_f(world: &mut CgWorld) {
    if world.cg.zoomed == qfalse {
        return;
    }
    world.cg.zoomed = qfalse;
    world.cg.zoomTime = world.cg.time;
}

/// Raven `CG_CalcFOVFromX` — derives `fov_y` from the given `fov_x` and the
/// render window's aspect, and stores both on the refdef.
///
/// Raven's underwater fov warp is `#if 0`'d out (his comment: the leafbrush
/// test ignores entity brushes, so a moving water door warped the view the
/// whole time), which is why the return is always false.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1130-1178`
pub fn CG_CalcFOVFromX(world: &mut CgWorld, fov_x: f32) -> bool {
    // Raven's tan/atan2 are the double libm calls and M_PI is math.h's double,
    // but `fov_x / 360` is a float divide (fov_x is float, 360 promotes to
    // float) that only widens to double for the `* M_PI` after it.
    let x = (world.cg.refdef.width as f64 / ((fov_x / 360.0) as f64 * PI).tan()) as f32;
    let mut fov_y = (world.cg.refdef.height as f64).atan2(x as f64) as f32;
    fov_y = ((fov_y * 360.0) as f64 / PI) as f32;

    let inwater = false;

    // set it
    world.cg.refdef.fov_x = fov_x;
    world.cg.refdef.fov_y = fov_y;

    inwater
}

/// Raven `CG_DamageBlendBlob` — the red/blue pain sprite in front of the eye,
/// shaded by which pool the damage came out of and faded over `DAMAGE_TIME`.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1343-1394`
pub fn CG_DamageBlendBlob(ctx: &mut CgContext) {
    if ctx.world.cg.damageValue == 0.0 {
        return;
    }

    let maxTime = DAMAGE_TIME;
    // `cg.damageTime` is Raven's float; the subtraction lands in an int `t`, so
    // it truncates toward zero.
    let t = (ctx.world.cg.time as f32 - ctx.world.cg.damageTime) as c_int;
    if t <= 0 || t >= maxTime {
        return;
    }

    let mut ent = refEntity_t::zeroed();
    ent.reType = refEntityType_t::RT_SPRITE;
    ent.renderfx = RF_FIRST_PERSON;

    let vieworg = ctx.world.cg.refdef.vieworg;
    let viewaxis = ctx.world.cg.refdef.viewaxis;
    _VectorMA(vieworg, 8.0, viewaxis[0], &mut ent.origin);
    let origin = ent.origin;
    _VectorMA(
        origin,
        ctx.world.cg.damageX * -8.0,
        viewaxis[1],
        &mut ent.origin,
    );
    let origin = ent.origin;
    _VectorMA(
        origin,
        ctx.world.cg.damageY * 8.0,
        viewaxis[2],
        &mut ent.origin,
    );

    // `1.0 - ((float)t / maxTime)` is Raven's double, so every scaled channel
    // below is computed in f64 and truncated into the byte.
    let fade = 1.0 - (t as f32 / maxTime as f32) as f64;
    ent.radius = ((ctx.world.cg.damageValue * 3.0) as f64 * fade) as f32;

    // Raven derefs `cg.snap` unguarded; a null there is UB, so the port takes
    // the pure-health arm, which is the `damageType == 0` zeroed state (§F19).
    let damageType = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.damageType)
        .unwrap_or(0);

    if damageType == 0 {
        // pure health
        ent.customShader = ctx.world.cgs.media.viewPainShader;
        ent.shaderRGBA[0] = (180.0 * fade) as u8;
        ent.shaderRGBA[1] = (50.0 * fade) as u8;
        ent.shaderRGBA[2] = (50.0 * fade) as u8;
        ent.shaderRGBA[3] = 255;
    } else if damageType == 1 {
        // pure shields
        ent.customShader = ctx.world.cgs.media.viewPainShader_Shields;
        ent.shaderRGBA[0] = (50.0 * fade) as u8;
        ent.shaderRGBA[1] = (180.0 * fade) as u8;
        ent.shaderRGBA[2] = (50.0 * fade) as u8;
        ent.shaderRGBA[3] = 255;
    } else {
        // shields and health
        ent.customShader = ctx.world.cgs.media.viewPainShader_ShieldsAndHealth;
        ent.shaderRGBA[0] = (180.0 * fade) as u8;
        ent.shaderRGBA[1] = (180.0 * fade) as u8;
        ent.shaderRGBA[2] = (50.0 * fade) as u8;
        ent.shaderRGBA[3] = 255;
    }

    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_CheckPassengerTurretView` — if we're a passenger manning one of
/// the vehicle's turrets, snap the view to the passenger's own origin/angles.
/// True when it took the view over.
///
/// Raven: "Ah, crap, just look around freely" — the bolt-matrix version of this
/// is commented out in the oracle and never ran.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1477-1555`
pub fn CG_CheckPassengerTurretView(world: &mut CgWorld) -> bool {
    // in a vehicle, as a passenger
    if world.cg.predictedPlayerState.m_iVehicleNum != 0
        && world.cg.predictedPlayerState.generic1 != 0
    {
        // passenger in a vehicle
        let vehNum = world.cg.predictedPlayerState.m_iVehicleNum as usize;
        if world.entity(vehNum).m_pVehicle.is_some() {
            // DEFERRED: `Vehicle_t::m_pVehicleInfo->maxPassengers` and the
            // `turret[MAX_VEHICLE_TURRETS]` table (`iAmmoMax`, `passengerNum`)
            // — the `Vehicle_t` referent pool behind `centity_t.m_pVehicle`
            // lands with `oracle/codemp/cgame/cg_players.c:7014-7042`
            // (DEC-46.2), so the turret scan can't run and the fn answers
            // Raven's no-turret-matched `qfalse`. The receiver stays `&mut`
            // because the matched arm writes `cg.refdef.vieworg`/`viewangles`.
            // Source: `oracle/codemp/cgame/cg_view.c:1483-1552`
        }
    }

    false
}

/// Raven `CG_PowerupTimerSounds` — walks the powerup timers looking for one
/// crossing a blink boundary.
///
/// The wear-off sound it would play is commented out in the oracle, so the
/// whole walk is a no-op; it is kept because Raven kept it.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1720-1737`
pub fn CG_PowerupTimerSounds(world: &CgWorld) {
    // powerup timers going away
    // Raven derefs `cg.snap` unguarded; a null there is UB, so the port walks
    // nothing (§F19) — which, the sound being commented out, is what the loop
    // does anyway.
    let Some(snap) = world.cg.snap_ref() else {
        return;
    };

    for i in 0..MAX_POWERUPS {
        let t = snap.ps.powerups[i];
        if t <= world.cg.time {
            continue;
        }
        if t - world.cg.time >= POWERUP_BLINKS * POWERUP_BLINK_TIME {
            continue;
        }
        if (t - world.cg.time) / POWERUP_BLINK_TIME != (t - world.cg.oldTime) / POWERUP_BLINK_TIME {
            //trap_S_StartSound( NULL, cg.snap->ps.clientNum, CHAN_ITEM, cgs.media.wearOffSound );
        }
    }
}

/// Raven `CG_AddBufferedSound` — queues an announcer sound behind whatever is
/// already playing.
///
/// PORT-NOTE: Raven bumps `soundBufferOut` without the ring modulo, so a full
/// queue can leave it at `MAX_SOUNDBUFFER`; [`CG_PlayBufferedSounds`] says what
/// the port does with that.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1942-1950`
pub fn CG_AddBufferedSound(world: &mut CgWorld, sfx: sfxHandle_t) {
    if sfx == 0 {
        return;
    }
    let soundBufferIn = world.cg.soundBufferIn;
    world.cg.soundBuffer[soundBufferIn as usize] = sfx;
    world.cg.soundBufferIn = (soundBufferIn + 1) % MAX_SOUNDBUFFER as i32;
    if world.cg.soundBufferIn == world.cg.soundBufferOut {
        world.cg.soundBufferOut += 1;
    }
}

/// Raven `CG_PlayBufferedSounds` — pops one queued announcer sound every 750
/// msec.
///
/// PORT-NOTE: `CG_AddBufferedSound` can leave `soundBufferOut` at
/// `MAX_SOUNDBUFFER`, and Raven then indexes one past `cg.soundBuffer` — the
/// read lands on `cg.voiceChatTime`, which self-heals the wedge: Raven plays
/// whatever garbage handle it finds there, zeroes it, and the `% MAX_SOUNDBUFFER`
/// afterward wraps `MAX_SOUNDBUFFER` back to a valid index. The port can't play
/// a garbage handle safely, so it skips the play but still advances/wraps
/// `soundBufferOut` on the out-of-bounds slot (§F19) — the ring heals either way,
/// it just never fires a bogus sound to do it.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1957-1966`
pub fn CG_PlayBufferedSounds(ctx: &mut CgContext) {
    if ctx.world.cg.soundTime < ctx.world.cg.time {
        let soundBufferOut = ctx.world.cg.soundBufferOut;
        if soundBufferOut == ctx.world.cg.soundBufferIn {
            return;
        }

        match ctx
            .world
            .cg
            .soundBuffer
            .get(soundBufferOut as usize)
            .copied()
        {
            Some(queued) if queued != 0 => {
                trap::S_StartLocalSound(ctx.engine, queued, CHAN_ANNOUNCER);
                ctx.world.cg.soundBuffer[soundBufferOut as usize] = 0;
                ctx.world.cg.soundBufferOut = (soundBufferOut + 1) % MAX_SOUNDBUFFER as i32;
                ctx.world.cg.soundTime = ctx.world.cg.time + 750;
            }
            Some(_) => {
                // in-bounds, empty slot - Raven skips this one too
            }
            None => {
                // out of bounds; wrap the ring so the wedge heals without
                // playing whatever garbage sits past the array
                ctx.world.cg.soundBufferOut = (soundBufferOut + 1) % MAX_SOUNDBUFFER as i32;
            }
        }
    }
}

/// Raven `CG_SE_UpdateShake` — jitters the camera origin and angles for the
/// remainder of a `CGCam_Shake`, fading the intensity out over the duration.
///
/// PORT-NOTE: the second loop refills only PITCH and YAW ("Don't do ROLL"), so
/// the angle jitter's ROLL reuses the origin jitter's Z from the first loop.
/// Raven's, kept.
///
/// Source: `oracle/codemp/cgame/cg_view.c:2011-2049`
pub fn CG_SE_UpdateShake(world: &mut CgWorld, origin: &mut vec3_t, angles: &mut vec3_t) {
    if world.view.cgScreenEffects.shake_duration <= 0 {
        return;
    }

    if world.cg.time
        > (world.view.cgScreenEffects.shake_start + world.view.cgScreenEffects.shake_duration)
    {
        world.view.cgScreenEffects.shake_intensity = 0.0;
        world.view.cgScreenEffects.shake_duration = 0;
        world.view.cgScreenEffects.shake_start = 0;
        return;
    }

    world.view.cgScreenEffects.FOV = CAMERA_DEFAULT_FOV;
    world.view.cgScreenEffects.FOV2 = CAMERA_DEFAULT_FOV;

    //intensity_scale now also takes into account FOV with 90.0 as normal
    let intensity_scale = 1.0
        - ((world.cg.time - world.view.cgScreenEffects.shake_start) as f32
            / world.view.cgScreenEffects.shake_duration as f32)
            * (((world.view.cgScreenEffects.FOV + world.view.cgScreenEffects.FOV2) / 2.0) / 90.0);

    let intensity = world.view.cgScreenEffects.shake_intensity * intensity_scale;

    let mut moveDir: vec3_t = [0.0; 3];
    for i in 0..3 {
        moveDir[i] = (world.bg_state.rng.crandom() * intensity as f64) as f32;
    }

    //Move the camera
    let out = *origin;
    _VectorAdd(out, moveDir, origin);

    // Don't do ROLL
    for i in 0..2 {
        moveDir[i] = (world.bg_state.rng.crandom() * intensity as f64) as f32;
    }

    //Move the angles
    let out = *angles;
    _VectorAdd(out, moveDir, angles);
}

/// Raven `CG_SE_UpdateMusic` — walks the music volume multiplier back up to 1.0
/// in 0.1 steps and mirrors it into `s_musicMult`.
///
/// Source: `oracle/codemp/cgame/cg_view.c:2051-2095`
pub fn CG_SE_UpdateMusic(ctx: &mut CgContext) {
    // Raven compares the float against math.h doubles, so the 0.1 steps below
    // stay in f64 and round back into the float field.
    if (ctx.world.view.cgScreenEffects.music_volume_multiplier as f64) < 0.1 {
        ctx.world.view.cgScreenEffects.music_volume_multiplier = 1.0;
        return;
    }

    if ctx.world.view.cgScreenEffects.music_volume_time < ctx.world.cg.time {
        if ctx.world.view.cgScreenEffects.music_volume_multiplier != 1.0
            || ctx.world.view.cgScreenEffects.music_volume_set != qfalse
        {
            let stepped = ctx.world.view.cgScreenEffects.music_volume_multiplier as f64 + 0.1;
            ctx.world.view.cgScreenEffects.music_volume_multiplier = stepped as f32;
            if ctx.world.view.cgScreenEffects.music_volume_multiplier > 1.0 {
                ctx.world.view.cgScreenEffects.music_volume_multiplier = 1.0;
            }

            let musMultStr = format!(
                "{:.6}",
                ctx.world.view.cgScreenEffects.music_volume_multiplier
            );
            trap::Cvar_Set(ctx.engine, "s_musicMult", &musMultStr);

            if ctx.world.view.cgScreenEffects.music_volume_multiplier == 1.0 {
                ctx.world.view.cgScreenEffects.music_volume_set = qfalse;
            } else {
                ctx.world.view.cgScreenEffects.music_volume_time = ctx.world.cg.time + 200;
            }
        }

        return;
    }

    if ctx.world.view.cgScreenEffects.music_volume_set == qfalse {
        // if the volume_time is >= cg.time, we should have a volume multiplier set
        let musMultStr = format!(
            "{:.6}",
            ctx.world.view.cgScreenEffects.music_volume_multiplier
        );
        trap::Cvar_Set(ctx.engine, "s_musicMult", &musMultStr);
        ctx.world.view.cgScreenEffects.music_volume_set = qtrue;
    }
}

/// Raven `CGCam_Shake` — starts a screen shake, clamped to
/// `MAX_SHAKE_INTENSITY`.
///
/// Source: `oracle/codemp/cgame/cg_view.c:2110-2127`
pub fn CGCam_Shake(world: &mut CgWorld, mut intensity: f32, duration: c_int) {
    if intensity > MAX_SHAKE_INTENSITY {
        intensity = MAX_SHAKE_INTENSITY;
    }

    world.view.cgScreenEffects.shake_intensity = intensity;
    world.view.cgScreenEffects.shake_duration = duration;

    world.view.cgScreenEffects.shake_start = world.cg.time;
}

/// Raven `CGCam_SetMusicMult` — ducks the music to `multiplier` for `duration`
/// msec, clamped to 0.1..1.0.
///
/// Source: `oracle/codemp/cgame/cg_view.c:2151-2166`
pub fn CGCam_SetMusicMult(world: &mut CgWorld, mut multiplier: f32, duration: c_int) {
    if multiplier < 0.1 {
        multiplier = 0.1;
    }

    if multiplier > 1.0 {
        multiplier = 1.0;
    }

    world.view.cgScreenEffects.music_volume_multiplier = multiplier;
    world.view.cgScreenEffects.music_volume_time = world.cg.time + duration;
    world.view.cgScreenEffects.music_volume_set = qfalse;
}

/// Raven `CG_EmplacedView` — constrains the view to the emplaced gun's yaw arc,
/// and force-sets the client's angles for 5 seconds when the gun says to.
///
/// Source: `oracle/codemp/cgame/cg_view.c:2185-2211`
pub fn CG_EmplacedView(ctx: &mut CgContext, angles: vec3_t) {
    let mut yaw: f32 = 0.0;

    // Raven derefs `cg.snap` unguarded; a null there is UB, so the port reads
    // the emplaced gun off entity 0 — the worldspawn, whose `origin2` is the
    // zeroed no-constraint value (§F19).
    let emplacedIndex = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.emplacedIndex)
        .unwrap_or(0);
    let constraint = ctx
        .world
        .entity(emplacedIndex as usize)
        .currentState
        .origin2[0];

    let over = BG_EmplacedView(ctx.world.cg.refdef.viewangles, angles, &mut yaw, constraint);

    if over != 0 {
        ctx.world.cg.refdef.viewangles[YAW] = yaw;
        let viewangles = ctx.world.cg.refdef.viewangles;
        AnglesToAxis(viewangles, ctx.world.cg.refdef.viewaxis.as_mut_ptr());

        if over == 2 {
            let time = ctx.world.cg.time + 5000;
            trap::SetClientForceAngle(ctx.engine, time, &viewangles);
        }
    }

    //we want to constrain the predicted player state viewangles as well
    let over = BG_EmplacedView(
        ctx.world.cg.predictedPlayerState.viewangles,
        angles,
        &mut yaw,
        constraint,
    );
    if over != 0 {
        ctx.world.cg.predictedPlayerState.viewangles[YAW] = yaw;
    }
}

/// Raven `CG_AddRefentForAutoMap` — adds one entity to the automap's own scene,
/// flattened to yaw only. Raven's `centity_t *cent` is the entity number (§B5).
///
/// Source: `oracle/codemp/cgame/cg_view.c:2214-2253`
pub fn CG_AddRefentForAutoMap(ctx: &mut CgContext, centNum: usize) {
    let cent = ctx.world.entity(centNum);

    if cent.currentState.eFlags & EF_NODRAW != 0 {
        return;
    }

    let mut ent = refEntity_t::zeroed();
    ent.reType = refEntityType_t::RT_MODEL;

    let mut flat: vec3_t = [0.0; 3];
    _VectorCopy(cent.lerpAngles, &mut flat);
    flat[PITCH] = 0.0;
    flat[ROLL] = 0.0;

    _VectorCopy(cent.lerpOrigin, &mut ent.origin);
    _VectorCopy(flat, &mut ent.angles);
    AnglesToAxis(flat, ent.axis.as_mut_ptr());

    if !cent.ghoul2.is_null()
        && (cent.currentState.eType == entityType_t::ET_PLAYER as c_int
            || cent.currentState.eType == entityType_t::ET_NPC as c_int
            || cent.currentState.modelGhoul2 != 0)
    {
        // using a ghoul2 model
        ent.ghoul2 = cent.ghoul2;
        ent.radius = cent.currentState.g2radius as f32;

        if ent.radius == 0.0 {
            ent.radius = 64.0;
        }
    } else {
        // then assume a standard indexed model
        let modelindex = cent.currentState.modelindex as usize;
        ent.hModel = ctx.world.cgs.gameModels[modelindex];
    }

    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_TestModel_f` — the `testmodel` console command; parks a model 100
/// units in front of the eye so it can be looked at.
///
/// Source: `oracle/codemp/cgame/cg_view.c:60-89`
pub fn CG_TestModel_f(ctx: &mut CgContext) {
    let mut angles: vec3_t = [0.0; 3];

    ctx.world.cg.testModelEntity = refEntity_t::zeroed();
    if trap::Argc(ctx.engine) < 2 {
        return;
    }

    let modelName = CG_Argv(ctx, 1);
    Q_strncpyz(&mut ctx.world.cg.testModelName, &modelName, MAX_QPATH);
    let testModelName = buf_to_string(&ctx.world.cg.testModelName.map(|c| c as u8));
    ctx.world.cg.testModelEntity.hModel = trap::R_RegisterModel(ctx.engine, &testModelName);

    if trap::Argc(ctx.engine) == 3 {
        let backlerp = CG_Argv(ctx, 2);
        ctx.world.cg.testModelEntity.backlerp = atof(&backlerp) as f32;
        ctx.world.cg.testModelEntity.frame = 1;
        ctx.world.cg.testModelEntity.oldframe = 0;
    }
    if ctx.world.cg.testModelEntity.hModel == 0 {
        CG_Printf(ctx, "Can't register model\n");
        return;
    }

    let vieworg = ctx.world.cg.refdef.vieworg;
    let forward = ctx.world.cg.refdef.viewaxis[0];
    _VectorMA(
        vieworg,
        100.0,
        forward,
        &mut ctx.world.cg.testModelEntity.origin,
    );

    angles[PITCH] = 0.0;
    angles[YAW] = 180.0 + ctx.world.cg.refdef.viewangles[1];
    angles[ROLL] = 0.0;

    AnglesToAxis(angles, ctx.world.cg.testModelEntity.axis.as_mut_ptr());
    ctx.world.cg.testGun = qfalse;
}

/// Raven `CG_TestModelNextFrame_f` — the `nextframe` console command.
///
/// Source: `oracle/codemp/cgame/cg_view.c:108-111`
pub fn CG_TestModelNextFrame_f(ctx: &mut CgContext) {
    ctx.world.cg.testModelEntity.frame += 1;
    let frame = ctx.world.cg.testModelEntity.frame;
    CG_Printf(ctx, &format!("frame {}\n", frame));
}

/// Raven `CG_TestModelPrevFrame_f` — the `prevframe` console command; floors at
/// frame 0.
///
/// Source: `oracle/codemp/cgame/cg_view.c:113-119`
pub fn CG_TestModelPrevFrame_f(ctx: &mut CgContext) {
    ctx.world.cg.testModelEntity.frame -= 1;
    if ctx.world.cg.testModelEntity.frame < 0 {
        ctx.world.cg.testModelEntity.frame = 0;
    }
    let frame = ctx.world.cg.testModelEntity.frame;
    CG_Printf(ctx, &format!("frame {}\n", frame));
}

/// Raven `CG_TestModelNextSkin_f` — the `nextskin` console command.
///
/// Source: `oracle/codemp/cgame/cg_view.c:121-124`
pub fn CG_TestModelNextSkin_f(ctx: &mut CgContext) {
    ctx.world.cg.testModelEntity.skinNum += 1;
    let skinNum = ctx.world.cg.testModelEntity.skinNum;
    CG_Printf(ctx, &format!("skin {}\n", skinNum));
}

/// Raven `CG_TestModelPrevSkin_f` — the `prevskin` console command; floors at
/// skin 0.
///
/// Source: `oracle/codemp/cgame/cg_view.c:126-132`
pub fn CG_TestModelPrevSkin_f(ctx: &mut CgContext) {
    ctx.world.cg.testModelEntity.skinNum -= 1;
    if ctx.world.cg.testModelEntity.skinNum < 0 {
        ctx.world.cg.testModelEntity.skinNum = 0;
    }
    let skinNum = ctx.world.cg.testModelEntity.skinNum;
    CG_Printf(ctx, &format!("skin {}\n", skinNum));
}

/// Raven `CG_AddTestModel` — puts the `testmodel` entity into this frame's
/// scene, re-registering it each time in case the level changed under it.
///
/// Source: `oracle/codemp/cgame/cg_view.c:134-160`
pub fn CG_AddTestModel(ctx: &mut CgContext) {
    // re-register the model, because the level may have changed
    let testModelName = buf_to_string(&ctx.world.cg.testModelName.map(|c| c as u8));
    ctx.world.cg.testModelEntity.hModel = trap::R_RegisterModel(ctx.engine, &testModelName);
    if ctx.world.cg.testModelEntity.hModel == 0 {
        CG_Printf(ctx, "Can't register model\n");
        return;
    }

    // if testing a gun, set the origin reletive to the view origin
    if ctx.world.cg.testGun != qfalse {
        let vieworg = ctx.world.cg.refdef.vieworg;
        let viewaxis = ctx.world.cg.refdef.viewaxis;
        _VectorCopy(vieworg, &mut ctx.world.cg.testModelEntity.origin);
        _VectorCopy(viewaxis[0], &mut ctx.world.cg.testModelEntity.axis[0]);
        _VectorCopy(viewaxis[1], &mut ctx.world.cg.testModelEntity.axis[1]);
        _VectorCopy(viewaxis[2], &mut ctx.world.cg.testModelEntity.axis[2]);

        // allow the position to be adjusted
        let gun_x = ctx.world.cvars.cg_gun_x.value;
        let gun_y = ctx.world.cvars.cg_gun_y.value;
        let gun_z = ctx.world.cvars.cg_gun_z.value;
        for i in 0..3 {
            ctx.world.cg.testModelEntity.origin[i] += viewaxis[0][i] * gun_x;
            ctx.world.cg.testModelEntity.origin[i] += viewaxis[1][i] * gun_y;
            ctx.world.cg.testModelEntity.origin[i] += viewaxis[2][i] * gun_z;
        }
    }

    trap::R_AddRefEntityToScene(ctx.engine, &ctx.world.cg.testModelEntity);
}

/// Raven `CG_OffsetFirstPersonView` — everything that nudges the eye off the
/// player's exact origin: weapon and damage kick, run/bob lean, view height,
/// duck/land/step smoothing.
///
/// Raven's `origin`/`angles` locals are pointers straight into `cg.refdef`, and
/// [`CG_StepOffset`] writes the same memory partway through, so the port works
/// the refdef fields in place rather than on copies.
///
/// Source: `oracle/codemp/cgame/cg_view.c:899-1043`
pub fn CG_OffsetFirstPersonView(world: &mut CgWorld) {
    // Raven derefs `cg.snap` unguarded for both the intermission test and the
    // dead test; a null there is UB, so the port leaves the view unoffset (§F19).
    let Some((pm_type, health, deadYaw)) = world.cg.snap_ref().map(|snap| {
        (
            snap.ps.pm_type,
            snap.ps.stats[STAT_HEALTH as usize],
            snap.ps.stats[STAT_DEAD_YAW as usize],
        )
    }) else {
        return;
    };

    if pm_type == pmtype_t::PM_INTERMISSION as c_int {
        return;
    }

    // if dead, fix the angle and don't add any kick
    if health <= 0 {
        world.cg.refdef.viewangles[ROLL] = 40.0;
        world.cg.refdef.viewangles[PITCH] = -15.0;
        world.cg.refdef.viewangles[YAW] = deadYaw as f32;
        world.cg.refdef.vieworg[2] += world.cg.predictedPlayerState.viewheight as f32;
        return;
    }

    // add angles based on weapon kick
    let mut kickTime = world.cg.time - world.cg.kick_time;
    if kickTime < 800 {
        //kicks are always 1 second long.  Deal with it.
        let kickPerc: f32;
        if kickTime <= 200 {
            //winding up
            kickPerc = kickTime as f32 / 200.0;
        } else {
            //returning to normal
            kickTime = 800 - kickTime;
            kickPerc = kickTime as f32 / 600.0;
        }
        let angles = world.cg.refdef.viewangles;
        let kick_angles = world.cg.kick_angles;
        _VectorMA(
            angles,
            kickPerc,
            kick_angles,
            &mut world.cg.refdef.viewangles,
        );
    }

    // add angles based on damage kick
    if world.cg.damageTime != 0.0 {
        // `cg.damageTime` is Raven's float, so the int time widens into it.
        let mut ratio = world.cg.time as f32 - world.cg.damageTime;
        if ratio < DAMAGE_DEFLECT_TIME as f32 {
            ratio /= DAMAGE_DEFLECT_TIME as f32;
            world.cg.refdef.viewangles[PITCH] += ratio * world.cg.v_dmg_pitch;
            world.cg.refdef.viewangles[ROLL] += ratio * world.cg.v_dmg_roll;
        } else {
            // Raven's leading `1.0` is math.h's double, so the whole tail is a
            // double that rounds back into the float `ratio`.
            ratio = (1.0f64
                - ((ratio - DAMAGE_DEFLECT_TIME as f32) / DAMAGE_RETURN_TIME as f32) as f64)
                as f32;
            if ratio > 0.0 {
                world.cg.refdef.viewangles[PITCH] += ratio * world.cg.v_dmg_pitch;
                world.cg.refdef.viewangles[ROLL] += ratio * world.cg.v_dmg_roll;
            }
        }
    }

    // add pitch based on fall kick
    // (Raven `#if 0`'d the fall-kick block out; it never ran.)

    // add angles based on velocity
    let predictedVelocity = world.cg.predictedPlayerState.velocity;

    let delta = _DotProduct(predictedVelocity, world.cg.refdef.viewaxis[0]);
    world.cg.refdef.viewangles[PITCH] += delta * world.cvars.cg_runpitch.value;

    let delta = _DotProduct(predictedVelocity, world.cg.refdef.viewaxis[1]);
    world.cg.refdef.viewangles[ROLL] -= delta * world.cvars.cg_runroll.value;

    // add angles based on bob

    // make sure the bob is visible even at low speeds
    let speed = if world.cg.xyspeed > 200.0 {
        world.cg.xyspeed
    } else {
        200.0
    };

    let ducked = world.cg.predictedPlayerState.pm_flags & PMF_DUCKED != 0;

    let mut delta = world.cg.bobfracsin * world.cvars.cg_bobpitch.value * speed;
    if ducked {
        delta *= 3.0; // crouching
    }
    world.cg.refdef.viewangles[PITCH] += delta;

    let mut delta = world.cg.bobfracsin * world.cvars.cg_bobroll.value * speed;
    if ducked {
        delta *= 3.0; // crouching accentuates roll
    }
    if world.cg.bobcycle & 1 != 0 {
        delta = -delta;
    }
    world.cg.refdef.viewangles[ROLL] += delta;

    //===================================

    // add view height
    world.cg.refdef.vieworg[2] += world.cg.predictedPlayerState.viewheight as f32;

    // smooth out duck height changes
    let timeDelta = world.cg.time - world.cg.duckTime;
    if timeDelta < DUCK_TIME {
        world.cg.refdef.vieworg[2] -=
            world.cg.duckChange * (DUCK_TIME - timeDelta) as f32 / DUCK_TIME as f32;
    }

    // add bob height
    let mut bob = world.cg.bobfracsin * world.cg.xyspeed * world.cvars.cg_bobup.value;
    if bob > 6.0 {
        bob = 6.0;
    }

    world.cg.refdef.vieworg[2] += bob;

    // add fall height
    let mut delta = (world.cg.time - world.cg.landTime) as f32;
    if delta < LAND_DEFLECT_TIME as f32 {
        let f = delta / LAND_DEFLECT_TIME as f32;
        world.cg.refdef.vieworg[2] += world.cg.landChange * f;
    } else if delta < (LAND_DEFLECT_TIME + LAND_RETURN_TIME) as f32 {
        delta -= LAND_DEFLECT_TIME as f32;
        // same double-`1.0` promotion as the damage kick above
        let f = (1.0f64 - (delta / LAND_RETURN_TIME as f32) as f64) as f32;
        world.cg.refdef.vieworg[2] += world.cg.landChange * f;
    }

    // add step offset
    CG_StepOffset(world);

    // add kick offset

    let origin = world.cg.refdef.vieworg;
    let kick_origin = world.cg.kick_origin;
    _VectorAdd(origin, kick_origin, &mut world.cg.refdef.vieworg);

    // pivot the eye based on a neck length
    // (Raven `#if 0`'d the NECK_LENGTH pivot out; it never ran.)
}

/// Raven `CG_CalcFov` — settles this frame's fov, walking the zoom fov in and
/// out and warping it while the eye is underwater. True when the eye is in a
/// liquid.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1191-1334`
pub fn CG_CalcFov(ctx: &mut CgContext) -> bool {
    let mut cgFov = ctx.world.cvars.cg_fov.value;

    if cgFov < 1.0 {
        cgFov = 1.0;
    }
    if cgFov > 97.0 {
        cgFov = 97.0;
    }

    let mut fov_x: f32;

    if ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_INTERMISSION as c_int {
        // if in intermission, use a fixed value
        fov_x = 80.0; //90;
    } else {
        // user selectable
        if ctx.world.cgs.dmflags & DF_FIXED_FOV != 0 {
            // dmflag to prevent wide fov for all clients
            fov_x = 80.0; //90;
        } else {
            fov_x = cgFov;
            if fov_x < 1.0 {
                fov_x = 1.0;
            } else if fov_x > 160.0 {
                fov_x = 160.0;
            }
        }

        if ctx.world.cg.predictedPlayerState.zoomMode == 2 {
            //binoculars
            if ctx.world.view.zoomFov > 40.0 {
                ctx.world.view.zoomFov -= ctx.world.cg.frametime as f32 * 0.075;

                if ctx.world.view.zoomFov < 40.0 {
                    ctx.world.view.zoomFov = 40.0;
                } else if ctx.world.view.zoomFov > cgFov {
                    ctx.world.view.zoomFov = cgFov;
                }
            }

            fov_x = ctx.world.view.zoomFov;
        } else if ctx.world.cg.predictedPlayerState.zoomMode != 0 {
            if ctx.world.cg.predictedPlayerState.zoomLocked == qfalse {
                if ctx.world.view.zoomFov > 50.0 {
                    //Now starting out at nearly half zoomed in
                    ctx.world.view.zoomFov = 50.0;
                }
                ctx.world.view.zoomFov -= ctx.world.cg.frametime as f32 * 0.035; //0.075f;

                if ctx.world.view.zoomFov < MAX_ZOOM_FOV {
                    ctx.world.view.zoomFov = MAX_ZOOM_FOV;
                } else if ctx.world.view.zoomFov > cgFov {
                    ctx.world.view.zoomFov = cgFov;
                } else {
                    // Still zooming
                    if ctx.world.view.zoomSoundTime < ctx.world.cg.time
                        || ctx.world.view.zoomSoundTime > ctx.world.cg.time + 10000
                    {
                        let vieworg = ctx.world.cg.refdef.vieworg;
                        let disruptorZoomLoop = ctx.world.cgs.media.disruptorZoomLoop;
                        trap::S_StartSound(
                            ctx.engine,
                            Some(&vieworg),
                            ENTITYNUM_WORLD,
                            CHAN_LOCAL,
                            disruptorZoomLoop,
                        );
                        ctx.world.view.zoomSoundTime = ctx.world.cg.time + 300;
                    }
                }
            }

            if ctx.world.view.zoomFov < MAX_ZOOM_FOV {
                ctx.world.view.zoomFov = 50.0; // hack to fix zoom during vid restart
            }
            fov_x = ctx.world.view.zoomFov;
        } else {
            ctx.world.view.zoomFov = 80.0;

            let f = (ctx.world.cg.time - ctx.world.cg.predictedPlayerState.zoomTime) as f32
                / ZOOM_OUT_TIME;
            if f > 1.0 {
                // Raven's `fov_x = fov_x;` — the blend is over, keep what we have
            } else {
                fov_x = ctx.world.cg.predictedPlayerState.zoomFov
                    + f * (fov_x - ctx.world.cg.predictedPlayerState.zoomFov);
            }
        }
    }

    // Same widths as `CG_CalcFOVFromX`: `fov_x / 360` is a float divide that
    // only widens to double for the libm call after it.
    let x = (ctx.world.cg.refdef.width as f64 / ((fov_x / 360.0) as f64 * PI).tan()) as f32;
    let mut fov_y = (ctx.world.cg.refdef.height as f64).atan2(x as f64) as f32;
    fov_y = ((fov_y * 360.0) as f64 / PI) as f32;

    // warp if underwater
    let vieworg = ctx.world.cg.refdef.vieworg;
    let viewContents = CG_PointContents(ctx, &vieworg, -1);
    ctx.world.cg.refdef.viewContents = viewContents;
    let inwater;
    if ctx.world.cg.refdef.viewContents & (CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA) != 0 {
        // Raven's `phase` is a float local, so the double expression rounds into
        // it before `sin` widens it back out.
        let phase = (ctx.world.cg.time as f64 / 1000.0 * WAVE_FREQUENCY * PI * 2.0) as f32;
        let v = (WAVE_AMPLITUDE as f64 * (phase as f64).sin()) as f32;
        fov_x += v;
        fov_y -= v;
        inwater = true;
    } else {
        inwater = false;
    }

    // set it
    ctx.world.cg.refdef.fov_x = fov_x;
    ctx.world.cg.refdef.fov_y = fov_y;

    if ctx.world.cg.predictedPlayerState.zoomMode != 0 {
        ctx.world.cg.zoomSensitivity = ctx.world.view.zoomFov / cgFov;
    } else if ctx.world.cg.zoomed == qfalse {
        ctx.world.cg.zoomSensitivity = 1.0;
    } else {
        ctx.world.cg.zoomSensitivity = (ctx.world.cg.refdef.fov_y as f64 / 75.0) as f32;
    }

    inwater
}

/// Raven `CG_UpdateSoundTrackers` — keeps every sound-tracker entity's sound
/// origin glued to the entity it was attached to, and refreshes every looping
/// sound.
///
/// PORT-NOTE: Raven's `cent &&` guard tests the address of an array element, so
/// it is always true; kept as the unconditional walk it already was.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1968-1997`
pub fn CG_UpdateSoundTrackers(ctx: &mut CgContext) {
    for num in 0..ENTITYNUM_NONE as usize {
        let cent = ctx.world.entity(num);
        let eFlags = cent.currentState.eFlags;
        let number = cent.currentState.number;
        let trickedentindex = cent.currentState.trickedentindex;

        //make sure the thing is valid at least.
        if (eFlags & EF_SOUNDTRACKER) != 0 && number == num as c_int {
            //keep sound for this entity updated in accordance with its attached entity at all times
            let clientNum = ctx.world.cg.snap_ref().map(|snap| snap.ps.clientNum);
            if clientNum == Some(trickedentindex) {
                //this is actually the player, so center the sound origin right on top of us
                let vieworg = ctx.world.cg.refdef.vieworg;
                _VectorCopy(vieworg, &mut ctx.world.entity_mut(num).lerpOrigin);
                let lerpOrigin = ctx.world.entity(num).lerpOrigin;
                trap::S_UpdateEntityPosition(ctx.engine, number, &lerpOrigin);
            } else {
                let lerpOrigin = ctx.world.entity(trickedentindex as usize).lerpOrigin;
                trap::S_UpdateEntityPosition(ctx.engine, number, &lerpOrigin);
            }
        }

        if number == num as c_int {
            //update all looping sounds..
            CG_S_UpdateLoopingSounds(ctx, num);
        }
    }
}

/// Raven `CG_CalcScreenEffects` — runs the screen shake over this frame's view
/// and walks the music duck back up.
///
/// Raven hands `CG_SE_UpdateShake` pointers straight into `cg.refdef`; the port
/// copies out and writes back, which is the same thing because the shake only
/// jitters the two vectors it is given.
///
/// Source: `oracle/codemp/cgame/cg_view.c:2104-2108`
pub fn CG_CalcScreenEffects(ctx: &mut CgContext) {
    let mut origin = ctx.world.cg.refdef.vieworg;
    let mut angles = ctx.world.cg.refdef.viewangles;
    CG_SE_UpdateShake(ctx.world, &mut origin, &mut angles);
    ctx.world.cg.refdef.vieworg = origin;
    ctx.world.cg.refdef.viewangles = angles;

    CG_SE_UpdateMusic(ctx);
}

/// Raven `CG_DoCameraShake` — shakes the camera for an explosion at `origin`,
/// falling off linearly to nothing at `radius`.
///
/// Raven: "FIXME: When exactly is the vieworg calculated in relation to the rest
/// of the frame?"
///
/// Source: `oracle/codemp/cgame/cg_view.c:2129-2149`
pub fn CG_DoCameraShake(
    world: &mut CgWorld,
    origin: vec3_t,
    intensity: f32,
    radius: c_int,
    time: c_int,
) {
    let mut dir: vec3_t = [0.0; 3];

    _VectorSubtract(world.cg.refdef.vieworg, origin, &mut dir);
    let dist = VectorNormalize(&mut dir);

    //Use the dir to add kick to the explosion

    if dist > radius as f32 {
        return;
    }

    let intensityScale = 1.0 - (dist / radius as f32);
    let realIntensity = intensity * intensityScale;

    CGCam_Shake(world, realIntensity, time);
}

/// Raven `CG_AddRadarAutomapEnts` — feeds the automap scene with us plus every
/// entity this frame's radar picked up.
///
/// Source: `oracle/codemp/cgame/cg_view.c:2256-2268`
pub fn CG_AddRadarAutomapEnts(ctx: &mut CgContext) {
    //first add yourself
    let clientNum = ctx.world.cg.predictedPlayerState.clientNum as usize;
    CG_AddRefentForAutoMap(ctx, clientNum);

    let mut i: c_int = 0;
    while i < ctx.world.cg.radarEntityCount as c_int {
        let radarEnt = ctx.world.cg.radarEntities[i as usize] as usize;
        CG_AddRefentForAutoMap(ctx, radarEnt);
        i += 1;
    }
}

/// Raven `RF_DEPTHHACK` — for view weapon Z crunching. `cg_ents.rs` has its
/// own private copy beside its own reader; this TU gets its own per §C8.
/// Source: `oracle/codemp/cgame/tr_types.h:21`
const RF_DEPTHHACK: c_int = 0x00008;

/// Raven `CG_TestGun_f` — the `testgun` console command; parks the test model
/// on the view weapon path instead of `CG_TestModel_f`'s free-floating one.
///
/// Source: `oracle/codemp/cgame/cg_view.c:98-105`
pub fn CG_TestGun_f(ctx: &mut CgContext) {
    CG_TestModel_f(ctx);
    ctx.world.cg.testGun = qtrue;
    //cg.testModelEntity.renderfx = RF_MINLIGHT | RF_DEPTHHACK | RF_FIRST_PERSON;

    // rww - 9-13-01 [1-26-01-sof2]
    ctx.world.cg.testModelEntity.renderfx = RF_DEPTHHACK | RF_FIRST_PERSON;
}

/// Raven `CG_ResetThirdPersonViewDamp` — snaps the third-person camera straight
/// onto its ideal target and location with no damping at all, then clips both
/// against the world.
///
/// Source: `oracle/codemp/cgame/cg_view.c:367-410`
pub fn CG_ResetThirdPersonViewDamp(ctx: &mut CgContext) {
    let mut trace = trace_t::zeroed();

    // Cap the pitch within reasonable limits
    if ctx.world.view.cameraFocusAngles[PITCH] > 89.0 {
        ctx.world.view.cameraFocusAngles[PITCH] = 89.0;
    } else if ctx.world.view.cameraFocusAngles[PITCH] < -89.0 {
        ctx.world.view.cameraFocusAngles[PITCH] = -89.0;
    }

    let cameraFocusAngles = ctx.world.view.cameraFocusAngles;
    let mut camerafwd: vec3_t = [0.0; 3];
    let mut cameraup: vec3_t = [0.0; 3];
    AngleVectors(
        cameraFocusAngles,
        Some(&mut camerafwd),
        None,
        Some(&mut cameraup),
    );
    ctx.world.view.camerafwd = camerafwd;
    ctx.world.view.cameraup = cameraup;

    // Set the cameraIdealTarget
    CG_CalcIdealThirdPersonViewTarget(ctx.world);

    // Set the cameraIdealLoc
    CG_CalcIdealThirdPersonViewLocation(ctx.world);

    // Now, we just set everything to the new positions.
    ctx.world.view.cameraCurLoc = ctx.world.view.cameraIdealLoc;
    ctx.world.view.cameraCurTarget = ctx.world.view.cameraIdealTarget;

    // Raven derefs `cg.snap` for the traces' skip entity; a null there is UB, so
    // the port skips client 0 (§F19).
    let clientNum = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.clientNum)
        .unwrap_or(0);

    // First thing we do is trace from the first person viewpoint out to the new target location.
    let cameraFocusLoc = ctx.world.view.cameraFocusLoc;
    let cameraCurTarget = ctx.world.view.cameraCurTarget;
    CG_Trace(
        ctx,
        &mut trace,
        &cameraFocusLoc,
        &cameramins,
        &cameramaxs,
        &cameraCurTarget,
        clientNum,
        MASK_CAMERACLIP,
    );
    if trace.fraction <= 1.0 {
        ctx.world.view.cameraCurTarget = trace.endpos;
    }

    // Now we trace from the new target location to the new view location, to make sure there is nothing in the way.
    let cameraCurTarget = ctx.world.view.cameraCurTarget;
    let cameraCurLoc = ctx.world.view.cameraCurLoc;
    CG_Trace(
        ctx,
        &mut trace,
        &cameraCurTarget,
        &cameramins,
        &cameramaxs,
        &cameraCurLoc,
        clientNum,
        MASK_CAMERACLIP,
    );
    if trace.fraction <= 1.0 {
        ctx.world.view.cameraCurLoc = trace.endpos;
    }

    ctx.world.view.cameraLastFrame = ctx.world.cg.time;
    ctx.world.view.cameraLastYaw = ctx.world.view.cameraFocusAngles[YAW];
    ctx.world.view.cameraStiffFactor = 0.0;
}

/// Raven `CG_UpdateThirdPersonTargetDamp` — walks the camera's look-at point
/// toward the ideal target by `(damp)^(time)` of the distance left, then clips
/// it against the world.
///
/// Raven: "Note that previously there was an upper limit to the number of
/// physics traces that are done through the world for the sake of camera
/// collision, since it wasn't calced per frame. Now it is calculated every
/// frame. This has the benefit that the camera is a lot smoother now (before it
/// lerped between tested points), however two full volume traces each frame is
/// a bit scary to think about."
///
/// Source: `oracle/codemp/cgame/cg_view.c:413-464`
pub fn CG_UpdateThirdPersonTargetDamp(ctx: &mut CgContext) {
    let mut trace = trace_t::zeroed();
    let mut targetdiff: vec3_t = [0.0; 3];

    // Set the cameraIdealTarget
    // Automatically get the ideal target, to avoid jittering.
    CG_CalcIdealThirdPersonViewTarget(ctx.world);

    let hyperSpaceTime = ctx.world.cg.predictedVehicleState.hyperSpaceTime;
    let targetDamp = ctx.world.cvars.cg_thirdPersonTargetDamp.value;

    if hyperSpaceTime != 0 && (ctx.world.cg.time - hyperSpaceTime) < HYPERSPACE_TIME {
        //hyperspacing, no damp
        ctx.world.view.cameraCurTarget = ctx.world.view.cameraIdealTarget;
    } else if targetDamp >= 1.0
        || ctx.world.cg.thisFrameTeleport != qfalse
        || ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0
    {
        // No damping.
        ctx.world.view.cameraCurTarget = ctx.world.view.cameraIdealTarget;
    } else if targetDamp >= 0.0 {
        // Calculate the difference from the current position to the new one.
        let cameraIdealTarget = ctx.world.view.cameraIdealTarget;
        let cameraCurTarget = ctx.world.view.cameraCurTarget;
        _VectorSubtract(cameraIdealTarget, cameraCurTarget, &mut targetdiff);

        // Now we calculate how much of the difference we cover in the time allotted.
        // The equation is (Damp)^(time)
        // Raven's `1.0` and `1.0/(float)CAMERA_DAMP_INTERVAL` are math.h doubles,
        // so both expressions evaluate in f64 and round back into the floats.
        // We must exponent the amount LEFT rather than the amount bled off
        let dampfactor = (1.0f64 - targetDamp as f64) as f32;
        // Our dampfactor is geared towards a time interval equal to "1".
        let dtime = ((ctx.world.cg.time - ctx.world.view.cameraLastFrame) as f32 as f64
            * (1.0f64 / CAMERA_DAMP_INTERVAL as f64)) as f32;

        // Note that since there are a finite number of "practical" delta millisecond values possible,
        // the ratio should be initialized into a chart ultimately.
        let ratio = dampfactor.powf(dtime);

        // This value is how much distance is "left" from the ideal.
        _VectorMA(
            cameraIdealTarget,
            -ratio,
            targetdiff,
            &mut ctx.world.view.cameraCurTarget,
        );
    }

    // Now we trace to see if the new location is cool or not.

    // Raven derefs `cg.snap` for the trace's skip entity; a null there is UB, so
    // the port skips client 0 (§F19).
    let clientNum = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.clientNum)
        .unwrap_or(0);

    // First thing we do is trace from the first person viewpoint out to the new target location.
    let cameraFocusLoc = ctx.world.view.cameraFocusLoc;
    let cameraCurTarget = ctx.world.view.cameraCurTarget;
    CG_Trace(
        ctx,
        &mut trace,
        &cameraFocusLoc,
        &cameramins,
        &cameramaxs,
        &cameraCurTarget,
        clientNum,
        MASK_CAMERACLIP,
    );
    if trace.fraction < 1.0 {
        ctx.world.view.cameraCurTarget = trace.endpos;
    }
}

/// Raven `CG_UpdateThirdPersonCameraDamp` — same damped walk for where the
/// camera itself sits, with the pitch and the yaw-change stiffness folded into
/// the damp factor, then clipped against the world.
///
/// Source: `oracle/codemp/cgame/cg_view.c:468-590`
pub fn CG_UpdateThirdPersonCameraDamp(ctx: &mut CgContext) {
    let mut trace = trace_t::zeroed();
    let mut locdiff: vec3_t = [0.0; 3];

    // Set the cameraIdealLoc
    CG_CalcIdealThirdPersonViewLocation(ctx.world);

    // First thing we do is calculate the appropriate damping factor for the camera.
    let mut dampfactor: f32 = 0.0;
    let hyperSpaceTime = ctx.world.cg.predictedVehicleState.hyperSpaceTime;
    if hyperSpaceTime != 0 && (ctx.world.cg.time - hyperSpaceTime) < HYPERSPACE_TIME {
        //hyperspacing - don't damp camera
        dampfactor = 1.0;
    } else if ctx.world.cvars.cg_thirdPersonCameraDamp.value != 0.0 {
        let dFactor = if ctx.world.cg.predictedPlayerState.m_iVehicleNum == 0 {
            ctx.world.cvars.cg_thirdPersonCameraDamp.value
        } else {
            1.0
        };

        // Note that the camera pitch has already been capped off to 89.
        let mut pitch = Q_fabs(ctx.world.view.cameraFocusAngles[PITCH]);

        // The higher the pitch, the larger the factor, so as you look up, it damps a lot less.
        // Raven's `115.0` and `1.0` are math.h doubles, so each of these lands in
        // f64 before rounding back into the float.
        pitch = (pitch as f64 / 115.0) as f32;
        dampfactor = ((1.0f64 - dFactor as f64) * (pitch * pitch) as f64) as f32;

        dampfactor += dFactor;

        // Now we also multiply in the stiff factor, so that faster yaw changes are stiffer.
        if ctx.world.view.cameraStiffFactor > 0.0 {
            // The cameraStiffFactor is how much of the remaining damp below 1 should be shaved off, i.e. approach 1 as stiffening increases.
            dampfactor = (dampfactor as f64
                + (1.0f64 - dampfactor as f64) * ctx.world.view.cameraStiffFactor as f64)
                as f32;
        }
    }

    if dampfactor >= 1.0 || ctx.world.cg.thisFrameTeleport != qfalse {
        // No damping.
        ctx.world.view.cameraCurLoc = ctx.world.view.cameraIdealLoc;
    } else if dampfactor >= 0.0 {
        // Calculate the difference from the current position to the new one.
        let cameraIdealLoc = ctx.world.view.cameraIdealLoc;
        let cameraCurLoc = ctx.world.view.cameraCurLoc;
        _VectorSubtract(cameraIdealLoc, cameraCurLoc, &mut locdiff);

        // Now we calculate how much of the difference we cover in the time allotted.
        // The equation is (Damp)^(time)
        // We must exponent the amount LEFT rather than the amount bled off
        dampfactor = (1.0f64 - dampfactor as f64) as f32;
        // Our dampfactor is geared towards a time interval equal to "1".
        let dtime = ((ctx.world.cg.time - ctx.world.view.cameraLastFrame) as f32 as f64
            * (1.0f64 / CAMERA_DAMP_INTERVAL as f64)) as f32;

        // Note that since there are a finite number of "practical" delta millisecond values possible,
        // the ratio should be initialized into a chart ultimately.
        let ratio = dampfactor.powf(dtime);

        // This value is how much distance is "left" from the ideal.
        _VectorMA(
            cameraIdealLoc,
            -ratio,
            locdiff,
            &mut ctx.world.view.cameraCurLoc,
        );
    }

    // Raven derefs `cg.snap` for the traces' skip entity; a null there is UB, so
    // the port skips client 0 (§F19).
    let clientNum = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.clientNum)
        .unwrap_or(0);

    // Now we trace from the new target location to the new view location, to make sure there is nothing in the way.
    let cameraCurTarget = ctx.world.view.cameraCurTarget;
    let cameraCurLoc = ctx.world.view.cameraCurLoc;
    CG_Trace(
        ctx,
        &mut trace,
        &cameraCurTarget,
        &cameramins,
        &cameramaxs,
        &cameraCurLoc,
        clientNum,
        MASK_CAMERACLIP,
    );

    if trace.fraction < 1.0 {
        // `trace_t.entityNum` is Raven's `short`; widen for the world test.
        let hitNum = trace.entityNum as c_int;
        let isMover = hitNum < ENTITYNUM_WORLD
            && ctx.world.entity(hitNum as usize).currentState.solid == SOLID_BMODEL
            && ctx.world.entity(hitNum as usize).currentState.eType
                == entityType_t::ET_MOVER as c_int;

        if isMover {
            //get a different position for movers -rww
            let mover = hitNum as usize;

            //this is absolutely hackiful, since we calc view values before we add packet ents and lerp,
            //if we hit a mover we want to update its lerp pos and force it when we do the trace against
            //it.
            let curTr = ctx.world.entity(mover).currentState.pos.trType;
            if curTr != trType_t::TR_STATIONARY && curTr != trType_t::TR_LINEAR {
                let curTrB = ctx.world.entity(mover).currentState.pos.trBase;

                //calc lerporigin for this client frame
                CG_CalcEntityLerpPositions(ctx, mover);

                //force the calc'd lerp to be the base and say we are stationary so we don't try to extrapolate
                //out further.
                ctx.world.entity_mut(mover).currentState.pos.trType = trType_t::TR_STATIONARY;
                let lerpOrigin = ctx.world.entity(mover).lerpOrigin;
                ctx.world.entity_mut(mover).currentState.pos.trBase = lerpOrigin;

                //retrace
                let cameraCurTarget = ctx.world.view.cameraCurTarget;
                let cameraCurLoc = ctx.world.view.cameraCurLoc;
                CG_Trace(
                    ctx,
                    &mut trace,
                    &cameraCurTarget,
                    &cameramins,
                    &cameramaxs,
                    &cameraCurLoc,
                    clientNum,
                    MASK_CAMERACLIP,
                );

                //copy old data back in
                ctx.world.entity_mut(mover).currentState.pos.trType = curTr;
                ctx.world.entity_mut(mover).currentState.pos.trBase = curTrB;
            }
            if trace.fraction < 1.0 {
                //still hit it, so take the proper trace endpos and use that.
                ctx.world.view.cameraCurLoc = trace.endpos;
            }
        } else {
            ctx.world.view.cameraCurLoc = trace.endpos;
        }
    }
}

/// Raven `CG_OffsetFighterView` — pushes the view out sideways/up off the
/// fighter, then pulls it back along the (offset) view angles at range, clipping
/// both hops.
///
/// Raven: "FIXME: do we need to smooth the org?"
///
/// Source: `oracle/codemp/cgame/cg_view.c:1045-1102`
pub fn CG_OffsetFighterView(ctx: &mut CgContext) {
    let mut vehRight: vec3_t = [0.0; 3];
    let mut vehUp: vec3_t = [0.0; 3];
    let mut backDir: vec3_t = [0.0; 3];
    let mut camOrg: vec3_t = [0.0; 3];
    let mut camBackOrg: vec3_t = [0.0; 3];
    // none of these are `mut`: the only arm that would reassign them is the
    // deferred vehicle override below.
    let horzOffset = ctx.world.cvars.cg_thirdPersonHorzOffset.value;
    let vertOffset = ctx.world.cvars.cg_thirdPersonVertOffset.value;
    let pitchOffset = ctx.world.cvars.cg_thirdPersonPitchOffset.value;
    let yawOffset = ctx.world.cvars.cg_thirdPersonAngle.value;
    let range = ctx.world.cvars.cg_thirdPersonRange.value;
    let mut trace = trace_t::zeroed();
    let veh = ctx.world.cg.predictedPlayerState.m_iVehicleNum as usize;

    // Raven fills a `vehFwd` here that nothing in the fn ever reads, so the port
    // asks for no forward vector.
    let viewangles = ctx.world.cg.refdef.viewangles;
    AngleVectors(viewangles, None, Some(&mut vehRight), Some(&mut vehUp));

    if ctx.world.entity(veh).m_pVehicle.is_some() {
        // DEFERRED: `Vehicle_t::m_pVehicleInfo` — `cameraOverride` and the four
        // `camera*Offset`/`cameraRange` values it gates, plus
        // `veh->playerState->hackingTime`, all hang off the `Vehicle_t` referent
        // pool behind `centity_t.m_pVehicle`, which lands with
        // `oracle/codemp/cgame/cg_players.c:7014-7042` (DEC-46.2). Only the
        // presence test is reachable, so the five values keep their cvars — what
        // Raven does for any vehicle that isn't `cameraOverride`.
        // Source: `oracle/codemp/cgame/cg_view.c:1059-1072`
    }

    //Set camera viewing position
    let vieworg = ctx.world.cg.refdef.vieworg;
    _VectorMA(vieworg, horzOffset, vehRight, &mut camOrg);
    let out = camOrg;
    _VectorMA(out, vertOffset, vehUp, &mut camOrg);

    // Raven derefs `cg.snap` for the traces' skip entity; a null there is UB, so
    // the port skips client 0 (§F19).
    let clientNum = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.clientNum)
        .unwrap_or(0);

    //trace to that pos
    CG_Trace(
        ctx,
        &mut trace,
        &vieworg,
        &cameramins,
        &cameramaxs,
        &camOrg,
        clientNum,
        MASK_CAMERACLIP,
    );
    if trace.fraction < 1.0 {
        camOrg = trace.endpos;
    }

    // Set camera viewing direction.
    ctx.world.cg.refdef.viewangles[YAW] += yawOffset;
    ctx.world.cg.refdef.viewangles[PITCH] += pitchOffset;

    //Now bring the cam back from that pos and angles at range
    let viewangles = ctx.world.cg.refdef.viewangles;
    AngleVectors(viewangles, Some(&mut backDir), None, None);
    let out = backDir;
    _VectorScale(out, -1.0, &mut backDir);

    _VectorMA(camOrg, range, backDir, &mut camBackOrg);

    //trace to that pos
    CG_Trace(
        ctx,
        &mut trace,
        &camOrg,
        &cameramins,
        &cameramaxs,
        &camBackOrg,
        clientNum,
        MASK_CAMERACLIP,
    );
    camOrg = trace.endpos;

    //FIXME: do we need to smooth the org?
    // ...and of course we should copy the new view location to the proper spot too.
    ctx.world.cg.refdef.vieworg = camOrg;
}

/// Raven `RDF_NOWORLDMODEL` — used for player configuration screen. `cg_draw.rs`
/// has its own private copy beside its own reader; this TU gets its own per §C8.
/// Source: `oracle/codemp/cgame/tr_types.h:57`
const RDF_NOWORLDMODEL: c_int = 1;

/// Raven `RDF_AUTOMAP` — Raven: "means this scene is to draw the automap -rww".
/// Source: `oracle/codemp/cgame/tr_types.h:63`
const RDF_AUTOMAP: c_int = 32;

/// Raven `RDF_HYPERSPACE` — teleportation effect.
/// Source: `oracle/codemp/cgame/tr_types.h:58`
const RDF_HYPERSPACE: c_int = 4;

/// Raven `CG_DrawAutoMap` — draws the automap scene. -rww
///
/// Source: `oracle/codemp/cgame/cg_view.c:2284-2425`
pub fn CG_DrawAutoMap(ctx: &mut CgContext) {
    let mut tr = trace_t::zeroed();
    let mut fwd: vec3_t = [0.0; 3];

    if ctx.world.cvars.cg_autoMap.integer == 0 {
        //don't do anything then
        return;
    }

    // Raven derefs `cg.snap` for the dead test; a null there is UB, so the port
    // draws no automap at all (§F19).
    let Some(health) = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.stats[STAT_HEALTH as usize])
    else {
        return;
    };
    if health <= 0 {
        //don't show when dead
        return;
    }

    if (ctx.world.cg.predictedPlayerState.pm_flags & PMF_FOLLOW) != 0
        || ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize] == TEAM_SPECTATOR
    {
        //don't show when spec
        return;
    }

    let localNum = ctx.world.cg.predictedPlayerState.clientNum as usize;
    if ctx.world.cgs.clientinfo[localNum].infoValid == qfalse {
        //don't show if bad ci
        return;
    }

    if ctx.world.cgs.gametype < GT_TEAM {
        //don't show in non-team gametypes
        return;
    }

    if ctx.world.view.cg_autoMapInputTime >= ctx.world.cg.time {
        if ctx.world.view.cg_autoMapInput.up != 0.0 {
            ctx.world.view.cg_autoMapZoom -= ctx.world.view.cg_autoMapInput.up;
            if ctx.world.view.cg_autoMapZoom < ctx.world.view.cg_autoMapZoomMainOffset + 64.0 {
                ctx.world.view.cg_autoMapZoom = ctx.world.view.cg_autoMapZoomMainOffset + 64.0;
            }
        }

        if ctx.world.view.cg_autoMapInput.down != 0.0 {
            ctx.world.view.cg_autoMapZoom += ctx.world.view.cg_autoMapInput.down;
            if ctx.world.view.cg_autoMapZoom > ctx.world.view.cg_autoMapZoomMainOffset + 4096.0 {
                ctx.world.view.cg_autoMapZoom = ctx.world.view.cg_autoMapZoomMainOffset + 4096.0;
            }
        }

        if ctx.world.view.cg_autoMapInput.yaw != 0.0 {
            ctx.world.view.cg_autoMapAngle[YAW] += ctx.world.view.cg_autoMapInput.yaw;
        }

        if ctx.world.view.cg_autoMapInput.pitch != 0.0 {
            ctx.world.view.cg_autoMapAngle[PITCH] += ctx.world.view.cg_autoMapInput.pitch;
        }

        if ctx.world.view.cg_autoMapInput.goToDefaults != qfalse {
            ctx.world.view.cg_autoMapZoom = 512.0;
            VectorSet(&mut ctx.world.view.cg_autoMapAngle, 90.0, 0.0, 0.0);
        }
    }

    // Raven's `memset( &refdef, 0, sizeof( refdef ) )` — `refdef_t` is scalars
    // and arrays with no padding, so the zeroed literal is the memset.
    let mut refdef = refdef_t {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        fov_x: 0.0,
        fov_y: 0.0,
        vieworg: [0.0; 3],
        viewangles: [0.0; 3],
        viewaxis: [[0.0; 3]; 3],
        viewContents: 0,
        time: 0,
        rdflags: 0,
        areamask: [0; MAX_MAP_AREA_BYTES],
        text: [[0; MAX_RENDER_STRING_LENGTH]; MAX_RENDER_STRINGS],
    };

    refdef.rdflags = RDF_NOWORLDMODEL | RDF_AUTOMAP;

    let origin = ctx.world.cg.predictedPlayerState.origin;
    _VectorCopy(origin, &mut refdef.vieworg);
    let cg_autoMapAngle = ctx.world.view.cg_autoMapAngle;
    _VectorCopy(cg_autoMapAngle, &mut refdef.viewangles);

    //scale out in the direction of the view angles base on the zoom factor
    AngleVectors(refdef.viewangles, Some(&mut fwd), None, None);
    let vieworg = refdef.vieworg;
    _VectorMA(
        vieworg,
        -ctx.world.view.cg_autoMapZoom,
        fwd,
        &mut refdef.vieworg,
    );

    AnglesToAxis(refdef.viewangles, refdef.viewaxis.as_mut_ptr());

    refdef.fov_x = 50.0;
    refdef.fov_y = 50.0;

    //guess this doesn't need to be done every frame, but eh
    let (vWidth, vHeight) = trap::R_GetRealRes(ctx.engine);

    //set scaling values so that the 640x480 will result at 1.0/1.0
    let hScale = vWidth as f32 / 640.0;
    let vScale = vHeight as f32 / 480.0;

    let x = ctx.world.cvars.cg_autoMapX.value;
    let y = ctx.world.cvars.cg_autoMapY.value;
    let w = ctx.world.cvars.cg_autoMapW.value;
    let h = ctx.world.cvars.cg_autoMapH.value;

    refdef.x = (x * hScale) as c_int;
    refdef.y = (y * vScale) as c_int;
    refdef.width = (w * hScale) as c_int;
    refdef.height = (h * vScale) as c_int;

    let frameLeft = ctx.world.cgs.media.wireframeAutomapFrame_left;
    let frameRight = ctx.world.cgs.media.wireframeAutomapFrame_right;
    let frameTop = ctx.world.cgs.media.wireframeAutomapFrame_top;
    let frameBottom = ctx.world.cgs.media.wireframeAutomapFrame_bottom;
    CG_DrawPic(
        ctx,
        x - SIDEFRAME_WIDTH as f32,
        y,
        SIDEFRAME_WIDTH as f32,
        h,
        frameLeft,
    );
    CG_DrawPic(ctx, x + w, y, SIDEFRAME_WIDTH as f32, h, frameRight);
    CG_DrawPic(
        ctx,
        x - SIDEFRAME_WIDTH as f32,
        y - SIDEFRAME_HEIGHT as f32,
        w + (SIDEFRAME_WIDTH * 2) as f32,
        SIDEFRAME_HEIGHT as f32,
        frameTop,
    );
    CG_DrawPic(
        ctx,
        x - SIDEFRAME_WIDTH as f32,
        y + h,
        w + (SIDEFRAME_WIDTH * 2) as f32,
        SIDEFRAME_HEIGHT as f32,
        frameBottom,
    );

    refdef.time = ctx.world.cg.time;

    trap::R_ClearScene(ctx.engine);
    CG_AddRadarAutomapEnts(ctx);

    // DEFERRED: `Vehicle_t::m_pVehicleInfo->type == VH_FIGHTER` — the last term
    // of Raven's chain hangs off the `Vehicle_t` referent pool behind
    // `centity_t.m_pVehicle` (`oracle/codemp/cgame/cg_players.c:7014-7042`,
    // DEC-46.2), so DEC-46.2's `Option<VehicleId>` can only answer the presence
    // half and this accepts every vehicle class — the same disposition
    // `cg_draw.rs`'s `CG_DrawVehicleHUD` fighter test took.
    // Source: `oracle/codemp/cgame/cg_view.c:2401-2405`
    let vehNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum as usize;
    let inFighter = ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0
        && ctx.world.entity(vehNum).currentState.eType == entityType_t::ET_NPC as c_int
        && ctx.world.entity(vehNum).currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
        && ctx.world.entity(vehNum).m_pVehicle.is_some();

    if inFighter {
        //constantly adjust to current height
        let height = ctx.world.cg.predictedPlayerState.origin[2];
        trap::R_AutomapElevAdj(ctx.engine, height);
    } else {
        //Trace down and set the ground elevation as the main automap elevation point
        let mut playerMins: vec3_t = [0.0; 3];
        let mut playerMaxs: vec3_t = [0.0; 3];
        VectorSet(&mut playerMins, -15.0, -15.0, DEFAULT_MINS_2 as f32);
        VectorSet(&mut playerMaxs, 15.0, 15.0, DEFAULT_MAXS_2 as f32);

        let origin = ctx.world.cg.predictedPlayerState.origin;
        _VectorCopy(origin, &mut fwd);
        fwd[2] -= 4096.0;
        let psClientNum = ctx.world.cg.predictedPlayerState.clientNum;
        CG_Trace(
            ctx,
            &mut tr,
            &origin,
            &playerMins,
            &playerMaxs,
            &fwd,
            psClientNum,
            MASK_SOLID,
        );

        if tr.startsolid == 0 && tr.allsolid == 0 {
            trap::R_AutomapElevAdj(ctx.engine, tr.endpos[2]);
        }
    }
    trap::R_RenderScene(ctx.engine, &refdef);
}

/// Raven `CG_OffsetThirdPersonView` — works out where the third-person camera
/// wants to look this frame, damps its target/location toward that, and
/// copies the result into `cg.refdef`.
///
/// Source: `oracle/codemp/cgame/cg_view.c:603-795`
pub fn CG_OffsetThirdPersonView(ctx: &mut CgContext) {
    let mut diff: vec3_t = [0.0; 3];
    let thirdPersonHorzOffset = ctx.world.cvars.cg_thirdPersonHorzOffset.value;

    let vehNum = ctx
        .world
        .cg
        .snap_ref()
        .map_or(0, |snap| snap.ps.m_iVehicleNum);
    if vehNum != 0 && ctx.world.entity(vehNum as usize).m_pVehicle.is_some() {
        // DEFERRED: `Vehicle_t::m_pVehicleInfo->cameraOverride` /
        // `cameraHorzOffset`, and `veh->playerState->hackingTime` all hang off
        // the `Vehicle_t` referent pool behind `centity_t.m_pVehicle`, which
        // lands with `oracle/codemp/cgame/cg_players.c:7014-7042` (DEC-46.2).
        // Only the presence test is reachable, so `thirdPersonHorzOffset`
        // keeps the cvar value — what Raven does for any vehicle that isn't
        // `cameraOverride`.
        // Source: `oracle/codemp/cgame/cg_view.c:609-621`
    }

    ctx.world.view.cameraStiffFactor = 0.0;

    // Set camera viewing direction.
    let viewangles = ctx.world.cg.refdef.viewangles;
    _VectorCopy(viewangles, &mut ctx.world.view.cameraFocusAngles);

    // if dead, look at killer
    //
    // §F19: Raven's `else if` below unguarded-derefs `cg.snap->ps` once the
    // monster-hold check's own `cg.snap` guard has already failed; a missing
    // snapshot here takes the "alive" arm instead of reproducing that UB.
    // only possibility for now, may add Wampa and sand creature later
    let heldByMonster = ctx.world.cg.snap_ref().map_or(false, |snap| {
        (snap.ps.eFlags2 & EF2_HELD_BY_MONSTER) != 0 && snap.ps.hasLookTarget != qfalse
    }) && {
        let lookTarget = ctx.world.cg.snap_ref().unwrap().ps.lookTarget;
        ctx.world.entity(lookTarget as usize).currentState.NPC_class
            == class_t::CLASS_RANCOR as c_int
    };

    if heldByMonster {
        // being held
        let lookTarget = ctx.world.cg.snap_ref().unwrap().ps.lookTarget;
        let monsterYaw = ctx.world.entity(lookTarget as usize).lerpAngles[YAW];
        //make the look angle the vector from his mouth to me
        VectorSet(
            &mut ctx.world.view.cameraFocusAngles,
            0.0,
            AngleNormalize180(monsterYaw + 180.0),
            0.0,
        );
    } else if ctx
        .world
        .cg
        .snap_ref()
        .map_or(1, |snap| snap.ps.stats[STAT_HEALTH as usize])
        <= 0
    {
        let deadYaw = ctx.world.cg.snap_ref().unwrap().ps.stats[STAT_DEAD_YAW as usize];
        ctx.world.view.cameraFocusAngles[YAW] = deadYaw as f32;
    } else {
        // Add in the third Person Angle.
        ctx.world.view.cameraFocusAngles[YAW] += ctx.world.cvars.cg_thirdPersonAngle.value;
        {
            let pitchOffset = ctx.world.cvars.cg_thirdPersonPitchOffset.value;

            let vehNum2 = ctx
                .world
                .cg
                .snap_ref()
                .map_or(0, |snap| snap.ps.m_iVehicleNum);
            if vehNum2 != 0 && ctx.world.entity(vehNum2 as usize).m_pVehicle.is_some() {
                // DEFERRED: `Vehicle_t::m_pVehicleInfo->cameraPitchDependantVertOffset` /
                // `cameraPitchOffset` hang off the `Vehicle_t` referent pool
                // behind `centity_t.m_pVehicle` (`oracle/codemp/cgame/cg_players.c:7014-7042`,
                // DEC-46.2). Only the presence test is reachable, so
                // `pitchOffset` keeps the cvar value.
                // Source: `oracle/codemp/cgame/cg_view.c:656-679`
            }

            // Raven's `if ( 0 && ... )` is a literal always-false condition
            // (dead code, `VEH_CONTROL_SCHEME_4`-style guard left in), so the
            // else arm is the only one that ever runs.
            // Source: `oracle/codemp/cgame/cg_view.c:681-698`
            ctx.world.view.cameraFocusAngles[PITCH] += pitchOffset;
        }
    }

    // The next thing to do is to see if we need to calculate a new camera target location.

    // If we went back in time for some reason, or if we just started, reset the sample.
    if ctx.world.view.cameraLastFrame == 0 || ctx.world.view.cameraLastFrame > ctx.world.cg.time {
        CG_ResetThirdPersonViewDamp(ctx);
    } else {
        // Cap the pitch within reasonable limits
        //
        // Raven's `BG_UnrestrainedPitchRoll` needs a live `Vehicle_t*`;
        // DEC-46.2 only carries presence through `centity_t.m_pVehicle`, so
        // this passes a null vehicle pointer — the same answer
        // `!pVeh.is_null()` gives inside the fn for the no-vehicle case, and
        // (per §A2) the defined-behavior default when a vehicle's info is
        // unresolvable: the pitch stays clamped rather than assuming
        // unrestricted fighter roll.
        // Source: `oracle/codemp/cgame/cg_view.c:712-716`
        let vehNum3 = ctx.world.cg.predictedPlayerState.m_iVehicleNum;
        let unrestrained = vehNum3 != 0
            && BG_UnrestrainedPitchRoll(
                &mut ctx.world.cg.predictedPlayerState as *mut playerState_t,
                core::ptr::null_mut(),
                &ctx.world.bg_state,
            ) != qfalse;

        if unrestrained {
            //no clamp on pitch
            //FIXME: when pitch >= 90 or <= -90, camera rotates oddly... need to CrossProduct not just vectoangles
        } else {
            if ctx.world.view.cameraFocusAngles[PITCH] > 80.0 {
                ctx.world.view.cameraFocusAngles[PITCH] = 80.0;
            } else if ctx.world.view.cameraFocusAngles[PITCH] < -80.0 {
                ctx.world.view.cameraFocusAngles[PITCH] = -80.0;
            }
        }

        let cameraFocusAngles = ctx.world.view.cameraFocusAngles;
        let mut camerafwd: vec3_t = [0.0; 3];
        let mut cameraup: vec3_t = [0.0; 3];
        AngleVectors(
            cameraFocusAngles,
            Some(&mut camerafwd),
            None,
            Some(&mut cameraup),
        );
        ctx.world.view.camerafwd = camerafwd;
        ctx.world.view.cameraup = cameraup;

        let mut deltayaw = Q_fabs(cameraFocusAngles[YAW] - ctx.world.view.cameraLastYaw);
        if deltayaw > 180.0 {
            // Normalize this angle so that it is between 0 and 180.
            deltayaw = Q_fabs(deltayaw - 360.0);
        }
        ctx.world.view.cameraStiffFactor =
            deltayaw / (ctx.world.cg.time - ctx.world.view.cameraLastFrame) as f32;
        if ctx.world.view.cameraStiffFactor < 1.0 {
            ctx.world.view.cameraStiffFactor = 0.0;
        } else if ctx.world.view.cameraStiffFactor > 2.5 {
            ctx.world.view.cameraStiffFactor = 0.75;
        } else {
            // 1 to 2 scales from 0.0 to 0.5
            ctx.world.view.cameraStiffFactor = (ctx.world.view.cameraStiffFactor - 1.0) * 0.5;
        }
        ctx.world.view.cameraLastYaw = cameraFocusAngles[YAW];

        // Move the target to the new location.
        CG_UpdateThirdPersonTargetDamp(ctx);
        CG_UpdateThirdPersonCameraDamp(ctx);
    }

    // Now interestingly, the Quake method is to calculate a target focus point above the player, and point the camera at it.
    // We won't do that for now.

    // We must now take the angle taken from the camera target and location.
    let cameraCurTarget = ctx.world.view.cameraCurTarget;
    let cameraCurLoc = ctx.world.view.cameraCurLoc;
    _VectorSubtract(cameraCurTarget, cameraCurLoc, &mut diff);
    {
        let dist = VectorNormalize(&mut diff);
        //under normal circumstances, should never be 0.00000 and so on.
        if dist == 0.0 || diff[0] == 0.0 || diff[1] == 0.0 {
            //must be hitting something, need some value to calc angles, so use cam forward
            let camerafwd = ctx.world.view.camerafwd;
            _VectorCopy(camerafwd, &mut diff);
        }
    }

    // Raven's `if ( 0 && ... )` is a literal always-false condition (dead
    // code, same `VEH_CONTROL_SCHEME_4`-style guard as above), so the else
    // arm is the only one that ever runs.
    // Source: `oracle/codemp/cgame/cg_view.c:772-782`
    vectoangles(diff, &mut ctx.world.cg.refdef.viewangles);

    // Temp: just move the camera to the side a bit
    if thirdPersonHorzOffset != 0.0 {
        let viewangles = ctx.world.cg.refdef.viewangles;
        AnglesToAxis(viewangles, ctx.world.cg.refdef.viewaxis.as_mut_ptr());
        let cameraCurLoc = ctx.world.view.cameraCurLoc;
        let viewaxis1 = ctx.world.cg.refdef.viewaxis[1];
        _VectorMA(
            cameraCurLoc,
            thirdPersonHorzOffset,
            viewaxis1,
            &mut ctx.world.view.cameraCurLoc,
        );
    }

    // ...and of course we should copy the new view location to the proper spot too.
    let cameraCurLoc = ctx.world.view.cameraCurLoc;
    _VectorCopy(cameraCurLoc, &mut ctx.world.cg.refdef.vieworg);

    ctx.world.view.cameraLastFrame = ctx.world.cg.time;
}

/// Raven `CG_ThirdPersonActionCam` — sabers-only camera that rides the
/// blade's trail position, lerping toward it and re-tracing so it never
/// clips through geometry.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1400-1474`
pub fn CG_ThirdPersonActionCam(ctx: &mut CgContext) -> bool {
    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot there is no
    // one to aim the action cam at, so report "didn't run".
    let Some(snap) = ctx.world.cg.snap_ref() else {
        return false;
    };
    let clientNum = snap.ps.clientNum as usize;
    let cent = ctx.world.entity(clientNum);

    // if we don't have a g2 instance this frame for whatever reason then do nothing
    if cent.ghoul2.is_null() {
        return false;
    }

    // just being safe, should not ever happen
    if cent.currentState.weapon != WP_SABER {
        return false;
    }

    let ci = &ctx.world.cgs.clientinfo[clientNum];
    // too long since we last got the blade position
    if ctx.world.cg.time - ci.saber[0].blade[0].trail.lastTime > 300 {
        return false;
    }

    let base = ci.saber[0].blade[0].trail.base;
    let lerpOrigin = cent.lerpOrigin;
    let entNumber = cent.currentState.number;
    let smoothFactor = 0.1_f32 * ctx.world.cvars.cg_timescale.value;
    let range = ctx.world.cvars.cg_thirdPersonRange.value;

    // get direction from base to ent origin
    let mut positionDir: vec3_t = [0.0; 3];
    _VectorSubtract(base, lerpOrigin, &mut positionDir);
    VectorNormalize(&mut positionDir);

    // position the cam based on the direction and saber position
    let mut desiredPos: vec3_t = [0.0; 3];
    _VectorMA(lerpOrigin, range * 2.0, positionDir, &mut desiredPos);

    // trace to the desired pos to see how far that way we can actually go before we hit something
    // the endpos will be valid for our desiredpos no matter what
    let mut tr = trace_t::zeroed();
    CG_Trace(
        ctx,
        &mut tr,
        &lerpOrigin,
        &vec3_origin,
        &vec3_origin,
        &desiredPos,
        entNumber,
        MASK_SOLID,
    );
    desiredPos = tr.endpos;

    if ctx.world.cg.time - ctx.world.view.cg_actionCamLastTime > 300 {
        // do a third person offset first and grab the initial point from that
        CG_OffsetThirdPersonView(ctx);
        ctx.world.view.cg_actionCamLastPos = ctx.world.cg.refdef.vieworg;
    }

    ctx.world.view.cg_actionCamLastTime = ctx.world.cg.time;

    // lerp the vieworg to the desired pos from the last valid
    let mut v: vec3_t = [0.0; 3];
    _VectorSubtract(desiredPos, ctx.world.view.cg_actionCamLastPos, &mut v);

    if VectorLength(v) > 64.0 {
        // don't bother moving yet if not far from the last pos
        for i in 0..3 {
            ctx.world.view.cg_actionCamLastPos[i] += v[i] * smoothFactor;
            ctx.world.cg.refdef.vieworg[i] = ctx.world.view.cg_actionCamLastPos[i];
        }
    } else {
        ctx.world.cg.refdef.vieworg = ctx.world.view.cg_actionCamLastPos;
    }

    // Make sure the point is alright
    let vieworg = ctx.world.cg.refdef.vieworg;
    CG_Trace(
        ctx,
        &mut tr,
        &lerpOrigin,
        &vec3_origin,
        &vec3_origin,
        &vieworg,
        entNumber,
        MASK_SOLID,
    );
    ctx.world.cg.refdef.vieworg = tr.endpos;

    let mut positionDir: vec3_t = [0.0; 3];
    _VectorSubtract(lerpOrigin, ctx.world.cg.refdef.vieworg, &mut positionDir);
    let mut desiredAngles: vec3_t = [0.0; 3];
    vectoangles(positionDir, &mut desiredAngles);

    // just set the angles for now
    ctx.world.cg.refdef.viewangles = desiredAngles;
    true
}

/// Raven `CG_CalcViewValues` — settles this frame's `cg.refdef`: intermission
/// override, bob/xyspeed, turret-manning vs normal view angles, camera orbit,
/// predicted-error decay, the emplaced-gun constraint, then hands off to the
/// vehicle/third-person/first-person offset before axis-ing the result.
/// Returns [`CG_CalcFov`]'s liquid-eye flag.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1564-1712`
pub fn CG_CalcViewValues(ctx: &mut CgContext) -> bool {
    // Raven's `memset( &cg.refdef, 0, sizeof( cg.refdef ) )` — `refdef_t` is
    // scalars and arrays with no padding, so the zeroed literal is the memset.
    ctx.world.cg.refdef = refdef_t {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        fov_x: 0.0,
        fov_y: 0.0,
        vieworg: [0.0; 3],
        viewangles: [0.0; 3],
        viewaxis: [[0.0; 3]; 3],
        viewContents: 0,
        time: 0,
        rdflags: 0,
        areamask: [0; MAX_MAP_AREA_BYTES],
        text: [[0; MAX_RENDER_STRING_LENGTH]; MAX_RENDER_STRINGS],
    };

    // calculate size of 3D view
    CG_CalcVrect(ctx);

    let ps = ctx.world.cg.predictedPlayerState;

    // intermission view
    if ps.pm_type == pmtype_t::PM_INTERMISSION as c_int {
        _VectorCopy(ps.origin, &mut ctx.world.cg.refdef.vieworg);
        _VectorCopy(ps.viewangles, &mut ctx.world.cg.refdef.viewangles);
        let viewangles = ctx.world.cg.refdef.viewangles;
        AnglesToAxis(viewangles, ctx.world.cg.refdef.viewaxis.as_mut_ptr());
        return CG_CalcFov(ctx);
    }

    ctx.world.cg.bobcycle = (ps.bobCycle & 128) >> 7;
    // `fabs(sin(...))` runs in double (the `127.0`/`M_PI` promote it), then
    // narrows once at assignment to the float field.
    ctx.world.cg.bobfracsin = (((ps.bobCycle & 127) as f64 / 127.0 * PI).sin()).abs() as f32;
    ctx.world.cg.xyspeed =
        ((ps.velocity[0] * ps.velocity[0] + ps.velocity[1] * ps.velocity[1]) as f64).sqrt() as f32;

    if ctx.world.cg.xyspeed > 270.0 {
        ctx.world.cg.xyspeed = 270.0;
    }

    let manningTurret = CG_CheckPassengerTurretView(ctx.world);
    if !manningTurret {
        // not manning a turret on a vehicle
        _VectorCopy(ps.origin, &mut ctx.world.cg.refdef.vieworg);

        // Raven's `VEH_CONTROL_SCHEME_4` is never defined in this tree, so the
        // `#else` arm below is the only one that ever built.
        // Source: `oracle/codemp/cgame/cg_view.c:1634-1644`
        //
        // `BG_UnrestrainedPitchRoll` needs a live `Vehicle_t*`; DEC-46.2 only
        // carries presence through `centity_t.m_pVehicle`, so this passes a
        // null vehicle pointer — the same answer `!pVeh.is_null()` gives
        // inside the fn for the no-vehicle case (`CG_OffsetThirdPersonView`
        // precedent, `cg_view.c:712-716`).
        let vehNum = ps.m_iVehicleNum;
        let unrestrained = vehNum != 0
            && BG_UnrestrainedPitchRoll(
                &mut ctx.world.cg.predictedPlayerState as *mut playerState_t,
                core::ptr::null_mut(),
                &ctx.world.bg_state,
            ) != qfalse;

        if unrestrained {
            // use the vehicle's viewangles to render view!
            let predictedVehicleViewangles = ctx.world.cg.predictedVehicleState.viewangles;
            ctx.world.cg.refdef.viewangles = predictedVehicleViewangles;
        } else {
            _VectorCopy(ps.viewangles, &mut ctx.world.cg.refdef.viewangles);
        }
    }
    let viewangles = ctx.world.cg.refdef.viewangles;
    ctx.world.view.cg_lastTurretViewAngles = viewangles;

    if ctx.world.cvars.cg_cameraOrbit.integer != 0 {
        if ctx.world.cg.time > ctx.world.cg.nextOrbitTime {
            ctx.world.cg.nextOrbitTime =
                ctx.world.cg.time + ctx.world.cvars.cg_cameraOrbitDelay.integer;
            ctx.world.cvars.cg_thirdPersonAngle.value += ctx.world.cvars.cg_cameraOrbit.value;
        }
    }

    // add error decay
    if ctx.world.cvars.cg_errorDecay.value > 0.0 {
        let t = ctx.world.cg.time - ctx.world.cg.predictedErrorTime;
        let f =
            (ctx.world.cvars.cg_errorDecay.value - t as f32) / ctx.world.cvars.cg_errorDecay.value;
        if f > 0.0 && f < 1.0 {
            let vieworg = ctx.world.cg.refdef.vieworg;
            let predictedError = ctx.world.cg.predictedError;
            _VectorMA(vieworg, f, predictedError, &mut ctx.world.cg.refdef.vieworg);
        } else {
            ctx.world.cg.predictedErrorTime = 0;
        }
    }

    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot the emplaced
    // constraint is simply skipped.
    let emplacedGun = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| (snap.ps.weapon, snap.ps.emplacedIndex));
    if let Some((weapon, emplacedIndex)) = emplacedGun {
        if weapon == WP_EMPLACED_GUN && emplacedIndex != 0 {
            // constrain the view properly for emplaced guns
            let angles = ctx.world.entity(emplacedIndex as usize).currentState.angles;
            CG_EmplacedView(ctx, angles);
        }
    }

    // FIX: okay, if manning a turret, let view turn freely,
    //      and use the vehicle chase camera info to place vieworg
    // if ( !manningTurret )
    {
        let vehNum = ps.m_iVehicleNum;
        let unrestrained = vehNum != 0
            && BG_UnrestrainedPitchRoll(
                &mut ctx.world.cg.predictedPlayerState as *mut playerState_t,
                core::ptr::null_mut(),
                &ctx.world.bg_state,
            ) != qfalse;

        if unrestrained {
            // use the vehicle's viewangles to render view!
            CG_OffsetFighterView(ctx);
        } else if ctx.world.cg.renderingThirdPerson != qfalse {
            // back away from character
            // §F19: null-snap arm reads "not in a special move" (Raven derefs
            // unguarded at cg_view.c:1690)
            let snapSaberMove = ctx.world.cg.snap_ref().map(|snap| snap.ps.saberMove);
            let actionCam = ctx.world.cvars.cg_thirdPersonSpecialCam.integer != 0
                && snapSaberMove.is_some_and(|m| BG_SaberInSpecial(m) != qfalse);
            if actionCam {
                // the action cam
                if !CG_ThirdPersonActionCam(ctx) {
                    // couldn't do it for whatever reason, resort back to third person then
                    CG_OffsetThirdPersonView(ctx);
                }
            } else {
                CG_OffsetThirdPersonView(ctx);
            }
        } else {
            // offset for local bobbing and kicks
            CG_OffsetFirstPersonView(ctx.world);
        }
    }

    // position eye relative to origin
    let viewangles = ctx.world.cg.refdef.viewangles;
    AnglesToAxis(viewangles, ctx.world.cg.refdef.viewaxis.as_mut_ptr());

    if ctx.world.cg.hyperspace != qfalse {
        ctx.world.cg.refdef.rdflags |= RDF_NOWORLDMODEL | RDF_HYPERSPACE;
    }

    // field of view
    CG_CalcFov(ctx)
}

/// Raven `RDF_SKYBOXPORTAL` — marks a scene as a 'portal sky'. `tr_types.h`'s
/// flag; this TU gets its own private copy beside its reader per §C8.
/// Source: `oracle/codemp/cgame/tr_types.h:60`
const RDF_SKYBOXPORTAL: c_int = 8;

/// Raven `RDF_DRAWSKYBOX` — the above (`RDF_SKYBOXPORTAL`) marks a scene as a
/// 'portal sky'; this flag says to draw it.
/// Source: `oracle/codemp/cgame/tr_types.h:61`
const RDF_DRAWSKYBOX: c_int = 16;

/// Raven `RDF_NOFOG` — no global fog in this scene (but still brush fog). -rww
/// Source: `oracle/codemp/cgame/tr_types.h:64`
const RDF_NOFOG: c_int = 64;

/// Raven `CG_DrawSkyBoxPortal` — parses a sky-portal configstring into a
/// portal `refdef_t` and renders it, restoring the normal `cg.refdef`
/// afterward so the caller's own scene build is untouched.
///
/// Source: `oracle/codemp/cgame/cg_view.c:1748-1935`
pub fn CG_DrawSkyBoxPortal(ctx: &mut CgContext, cstr: &str) {
    // for transitions back from zoomed in modes
    ctx.world.view.lastfov = ctx.world.view.zoomFov;

    let mut backuprefdef = ctx.world.cg.refdef;

    let mut qs = QSharedScratch::zeroed();
    let mut cursor: Option<&[u8]> = Some(cstr.as_bytes());

    let (token, rest) = COM_ParseExt(&mut qs, cursor, false);
    cursor = rest;
    if token.is_empty() {
        CG_Error(
            ctx,
            "CG_DrawSkyBoxPortal: error parsing skybox configstring\n",
        );
        return;
    }
    ctx.world.cg.refdef.vieworg[0] = atof(&token) as f32;

    let (token, rest) = COM_ParseExt(&mut qs, cursor, false);
    cursor = rest;
    if token.is_empty() {
        CG_Error(
            ctx,
            "CG_DrawSkyBoxPortal: error parsing skybox configstring\n",
        );
        return;
    }
    ctx.world.cg.refdef.vieworg[1] = atof(&token) as f32;

    let (token, rest) = COM_ParseExt(&mut qs, cursor, false);
    cursor = rest;
    if token.is_empty() {
        CG_Error(
            ctx,
            "CG_DrawSkyBoxPortal: error parsing skybox configstring\n",
        );
        return;
    }
    ctx.world.cg.refdef.vieworg[2] = atof(&token) as f32;

    let (token, rest) = COM_ParseExt(&mut qs, cursor, false);
    cursor = rest;
    if token.is_empty() {
        CG_Error(
            ctx,
            "CG_DrawSkyBoxPortal: error parsing skybox configstring\n",
        );
        return;
    }
    // Raven's `fov_x = atoi(token); if (!fov_x) fov_x = cg_fov.value;` is a
    // dead store — both the intermission and non-intermission arms below
    // unconditionally overwrite fov_x from cg_fov.value before it is ever
    // read, so only the parse (needed to hold cstr's cursor position) and the
    // empty-token error above carry forward.

    // setup fog the first time, ignore this part of the configstring after that
    let (token, rest) = COM_ParseExt(&mut qs, cursor, false);
    cursor = rest;
    if token.is_empty() {
        CG_Error(
            ctx,
            "CG_DrawSkyBoxPortal: error parsing skybox configstring.  No fog state\n",
        );
        return;
    } else if atoi(&token) != 0 {
        // this camera has fog
        // Raven parses fogColor/fogStart/fogEnd here but never stores them
        // anywhere afterward (verified: no further read of any of the three
        // in cg_view.c) — the parse and its error checks are the only
        // observable behavior, kept faithfully; the values themselves are
        // genuinely dead.
        let (token, rest) = COM_ParseExt(&mut qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            CG_Error(
                ctx,
                "CG_DrawSkyBoxPortal: error parsing skybox configstring.  No fog[0]\n",
            );
            return;
        }

        let (token, rest) = COM_ParseExt(&mut qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            CG_Error(
                ctx,
                "CG_DrawSkyBoxPortal: error parsing skybox configstring.  No fog[1]\n",
            );
            return;
        }

        let (token, rest) = COM_ParseExt(&mut qs, cursor, false);
        cursor = rest;
        if token.is_empty() {
            CG_Error(
                ctx,
                "CG_DrawSkyBoxPortal: error parsing skybox configstring.  No fog[2]\n",
            );
            return;
        }

        let (token, rest) = COM_ParseExt(&mut qs, cursor, false);
        cursor = rest;
        let _fogStart = if token.is_empty() { 0 } else { atoi(&token) };

        let (token, rest) = COM_ParseExt(&mut qs, cursor, false);
        cursor = rest;
        let _fogEnd = if token.is_empty() { 0 } else { atoi(&token) };
    }
    let _ = cursor; // cstr isn't parsed any further past this point

    let mut fov_x: f32;
    if ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_INTERMISSION as c_int {
        // if in intermission, use a fixed value
        fov_x = ctx.world.cvars.cg_fov.value;
    } else {
        fov_x = ctx.world.cvars.cg_fov.value;
        if fov_x < 1.0 {
            fov_x = 1.0;
        } else if fov_x > 160.0 {
            fov_x = 160.0;
        }

        if ctx.world.cg.predictedPlayerState.zoomMode != 0 {
            fov_x = ctx.world.view.zoomFov;
        }

        // do smooth transitions for zooming
        if ctx.world.cg.predictedPlayerState.zoomMode != 0 {
            // zoomed/zooming in
            let f = (ctx.world.cg.time - ctx.world.cg.zoomTime) as f32 / ZOOM_OUT_TIME;
            if f > 1.0 {
                fov_x = ctx.world.view.zoomFov;
            } else {
                fov_x += f * (ctx.world.view.zoomFov - fov_x);
            }
            ctx.world.view.lastfov = fov_x;
        } else {
            // zooming out
            let f = (ctx.world.cg.time - ctx.world.cg.zoomTime) as f32 / ZOOM_OUT_TIME;
            if f > 1.0 {
                // Raven's `fov_x = fov_x;` — the blend is over, keep what we have
            } else {
                fov_x = ctx.world.view.zoomFov + f * (fov_x - ctx.world.view.zoomFov);
            }
        }
    }

    // Same widths as `CG_CalcFov`: `fov_x / 360` is a float divide that only
    // widens to double for the libm call after it.
    let x = (ctx.world.cg.refdef.width as f64 / ((fov_x / 360.0) as f64 * PI).tan()) as f32;
    let mut fov_y = (ctx.world.cg.refdef.height as f64).atan2(x as f64) as f32;
    fov_y = ((fov_y * 360.0) as f64 / PI) as f32;

    ctx.world.cg.refdef.fov_x = fov_x;
    ctx.world.cg.refdef.fov_y = fov_y;

    ctx.world.cg.refdef.rdflags |= RDF_SKYBOXPORTAL;
    ctx.world.cg.refdef.rdflags |= RDF_DRAWSKYBOX;

    ctx.world.cg.refdef.time = ctx.world.cg.time;

    if ctx.world.cg.hyperspace == qfalse {
        // rww - also had to add this to add effects being rendered in portal sky
        // areas properly.
        trap::FX_AddScheduledEffects(ctx.engine, true);
    }

    // rww - there was no proper way to put real entities inside the portal view
    // before. this will put specially flagged entities in the render.
    CG_AddPacketEntities(ctx, qtrue);

    if ctx.world.main.cg_skyOri {
        // ok, we want to orient the sky refdef vieworg based on the normal
        // vieworg's relation to the ori pos
        let mut dif: vec3_t = [0.0; 3];
        _VectorSubtract(backuprefdef.vieworg, ctx.world.main.cg_skyOriPos, &mut dif);
        _VectorScale(dif, ctx.world.main.cg_skyOriScale, &mut dif);
        let vieworg = ctx.world.cg.refdef.vieworg;
        _VectorAdd(vieworg, dif, &mut ctx.world.cg.refdef.vieworg);
    }

    if ctx.world.main.cg_noFogOutsidePortal {
        // make sure no fog flag is stripped first, and make sure it is set on
        // the normal refdef
        ctx.world.cg.refdef.rdflags &= !RDF_NOFOG;
        backuprefdef.rdflags |= RDF_NOFOG;
    }

    // draw the skybox
    trap::R_RenderScene(ctx.engine, &ctx.world.cg.refdef);

    ctx.world.cg.refdef = backuprefdef;
}

/// Raven `CG_DrawActiveFrame` — the top of the per-frame draw: pulls in the
/// snapshot, runs prediction, builds the render/sound lists, and issues the
/// actual draw. `menus`/`ds`/`dc` thread through to the fns further down the
/// call chain that need the menu framework (`CG_ProcessSnapshots`,
/// `CG_DrawInformation`, `CG_DrawActive`) per cg_draw.rs precedent.
///
/// Raven's `#ifdef VEH_CONTROL_SCHEME_4` block (fov override for fighter
/// pilots) is dead in the retail build — `VEH_CONTROL_SCHEME_4` is never
/// defined anywhere in the oracle tree — so it, and the `mSensitivityOverride`
/// / `bUseFighterPitch` / `isFighter` locals that only exist to feed it, are
/// dropped.
///
/// Source: `oracle/codemp/cgame/cg_view.c:2447-2762`
#[allow(clippy::too_many_arguments)]
pub fn CG_DrawActiveFrame(
    ctx: &mut CgContext,
    serverTime: c_int,
    stereoView: stereoFrame_t,
    demoPlayback: qboolean,
    menus: &mut MenuSystem,
    ds: &DisplayState,
) {
    let mSensitivity = ctx.world.cg.zoomSensitivity;
    let mPitchOverride = 0.0f32;
    let mYawOverride = 0.0f32;

    if ctx.world.players.cgQueueLoad {
        // do this before you start messing around with adding ghoul2 refents and crap
        CG_ActualLoadDeferredPlayers(ctx);
        ctx.world.players.cgQueueLoad = false;
    }

    ctx.world.cg.time = serverTime;
    ctx.world.cg.demoPlayback = demoPlayback;

    if let Some(snap) = ctx.world.cg.snap_ref() {
        let team = snap.ps.persistant[PERS_TEAM as usize];
        if ctx.world.cvars.ui_myteam.integer != team {
            trap::Cvar_Set(ctx.engine, "ui_myteam", &format!("{team}"));
        }
    }

    if ctx.world.cgs.gametype == GT_SIEGE {
        if let Some(snap) = ctx.world.cg.snap_ref() {
            let clientNum = snap.ps.clientNum as usize;
            let siegeIndex = ctx.world.cgs.clientinfo[clientNum].siegeIndex;
            if ctx.world.view.cg_siegeClassIndex != siegeIndex {
                ctx.world.view.cg_siegeClassIndex = siegeIndex;
                if ctx.world.view.cg_siegeClassIndex == -1 {
                    trap::Cvar_Set(ctx.engine, "ui_mySiegeClass", "<none>");
                } else {
                    // §F19: siegeIndex comes off the server's clientinfo; past
                    // the parsed class count Raven read a zeroed fixed-array
                    // slot (empty name) - we skip the set instead of panicking.
                    let idx = ctx.world.view.cg_siegeClassIndex as usize;
                    if let Some(class) = ctx.world.bg_state.bgSiegeClasses.get(idx) {
                        let name = class.name.clone();
                        trap::Cvar_Set(ctx.engine, "ui_mySiegeClass", &name);
                    }
                }
            }
        }
    }

    // update cvars
    CG_UpdateCvars(ctx);

    // if we are only updating the screen as a loading
    // pacifier, don't even try to read snapshots
    if ctx.world.cg.infoScreenText[0] != 0 {
        CG_DrawInformation(ctx, ds);
        return;
    }

    trap::FX_AdjustTime(ctx.engine, ctx.world.cg.time);

    CG_RunLightStyles(ctx);

    // any looped sounds will be respecified as entities
    // are added to the render list
    trap::S_ClearLoopingSounds(ctx.engine);

    // clear all the render lists
    trap::R_ClearScene(ctx.engine);

    // set up cg.snap and possibly cg.nextSnap
    CG_ProcessSnapshots(ctx, menus, ds);

    trap::ROFF_UpdateEntities(ctx.engine);

    // if we haven't received any snapshots yet, all
    // we can draw is the information screen
    let snapNotActive = match ctx.world.cg.snap_ref() {
        None => true,
        Some(snap) => (snap.snapFlags & SNAPFLAG_NOT_ACTIVE) != 0,
    };
    if snapNotActive {
        // Raven's `#if 0` snapshot-timeout block (cg_view.c:2518-2540) never
        // compiled in retail; not transcribed.
        CG_DrawInformation(ctx, ds);
        return;
    }

    // let the client system know what our weapon and zoom settings are
    let mSensitivity = if ctx
        .world
        .cg
        .snap_ref()
        .is_some_and(|snap| snap.ps.saberLockTime > ctx.world.cg.time)
    {
        0.01f32
    } else if ctx.world.cg.predictedPlayerState.weapon == WP_EMPLACED_GUN as c_int {
        // lower sens for emplaced guns and vehicles
        0.2f32
    } else {
        mSensitivity
    };

    if ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0 {
        ctx.world.view.veh = Some(ctx.world.cg.predictedPlayerState.m_iVehicleNum as usize);
    }
    // Raven's `veh &&` is only the first conjunct of the if - a NULL veh still
    // takes the else arm, so on-foot players push their weapon/force/item
    // selects to the engine every frame too.
    let fighterControls = match ctx.world.view.veh {
        Some(vehIdx) => {
            let veh = ctx.world.entity(vehIdx);
            // DEFERRED: `Vehicle_t::m_pVehicleInfo->type == VH_FIGHTER` hangs off
            // the `Vehicle_t` referent pool behind `centity_t.m_pVehicle`
            // (`oracle/codemp/cgame/cg_players.c:7014-7042`, DEC-46.2), so the
            // presence test stands in for the fighter-type check — the same
            // disposition `CG_DrawAutoMap`'s automap-elevation branch took.
            // Source: `oracle/codemp/cgame/cg_view.c:2579-2588`
            veh.currentState.eType == entityType_t::ET_NPC as c_int
                && veh.currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
                && veh.m_pVehicle.is_some()
                && ctx.world.cvars.bg_fighterAltControl.integer != 0
        }
        None => false,
    };
    if fighterControls {
        trap::SetUserCmdValue(
            ctx.engine,
            ctx.world.cg.weaponSelect,
            mSensitivity,
            mPitchOverride,
            mYawOverride,
            0.0,
            ctx.world.cg.forceSelect,
            ctx.world.cg.itemSelect,
            true,
        );
        // this is done because I don't want an extra assign each frame
        // because I am so perfect and super efficient.
        ctx.world.view.veh = None;
    } else {
        trap::SetUserCmdValue(
            ctx.engine,
            ctx.world.cg.weaponSelect,
            mSensitivity,
            mPitchOverride,
            mYawOverride,
            0.0,
            ctx.world.cg.forceSelect,
            ctx.world.cg.itemSelect,
            false,
        );
    }

    // this counter will be bumped for every valid scene we generate
    ctx.world.cg.clientFrame += 1;

    // update cg.predictedPlayerState
    CG_PredictPlayerState(ctx);

    // decide on third person view
    let snapHealthPersistant = ctx.world.cg.snap_ref().map(|snap| {
        (
            snap.ps.stats[STAT_HEALTH as usize],
            snap.ps.persistant[PERS_TEAM as usize],
        )
    });
    let (snapHealth, snapTeam) = snapHealthPersistant.unwrap();
    ctx.world.cg.renderingThirdPerson =
        (ctx.world.cvars.cg_thirdPerson.integer != 0 || snapHealth <= 0) as c_int;

    if snapHealth > 0 {
        if ctx.world.cg.predictedPlayerState.weapon == WP_EMPLACED_GUN as c_int
            && ctx.world.cg.predictedPlayerState.emplacedIndex != 0
        {
            // force third person for e-web and emplaced use
            // (the commented-out `cg_entities[emplacedIndex]` weapon check
            // Raven left in cg_view.c:2607 never ran; not transcribed)
            ctx.world.cg.renderingThirdPerson = 1;
        } else if ctx.world.cg.predictedPlayerState.weapon == WP_SABER as c_int
            || ctx.world.cg.predictedPlayerState.weapon == WP_MELEE
            || BG_InGrappleMove(ctx.world.cg.predictedPlayerState.torsoAnim) != 0
            || BG_InGrappleMove(ctx.world.cg.predictedPlayerState.legsAnim) != 0
            || ctx.world.cg.predictedPlayerState.forceHandExtend == HANDEXTEND_KNOCKDOWN as c_int
            || ctx.world.cg.predictedPlayerState.fallingToDeath != 0
            || ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0
            || PM_InKnockDown(&mut ctx.world.cg.predictedPlayerState) != qfalse
        {
            if ctx.world.cvars.cg_fpls.integer != 0
                && ctx.world.cg.predictedPlayerState.weapon == WP_SABER as c_int
            {
                // force to first person for fpls
                ctx.world.cg.renderingThirdPerson = 0;
            } else {
                ctx.world.cg.renderingThirdPerson = 1;
            }
        } else if ctx
            .world
            .cg
            .snap_ref()
            // Raven reads the SNAPSHOT zoomMode here (not the predicted one the
            // fog chain below uses); §F19: the fn already returned before this
            // point if no snap, so the None arm can't fire - it reads as 0.
            .is_some_and(|snap| snap.ps.zoomMode != 0)
        {
            // always force first person when zoomed
            ctx.world.cg.renderingThirdPerson = 0;
        }
    }

    if ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_SPECTATOR as c_int {
        // always first person for spec
        ctx.world.cg.renderingThirdPerson = 0;
    }

    if snapTeam == TEAM_SPECTATOR {
        ctx.world.cg.renderingThirdPerson = 0;
    }

    // build cg.refdef
    let inwater = CG_CalcViewValues(ctx);

    if ctx.world.view.cg_linearFogOverride != 0.0 {
        trap::R_SetRangeFog(ctx.engine, -ctx.world.view.cg_linearFogOverride);
    } else if ctx.world.cg.predictedPlayerState.zoomMode != 0 {
        // zooming with binoculars or sniper, set the fog range based on the
        // zoom level -rww
        ctx.world.view.cg_rangedFogging = true;
        // smaller the fov the less fog we have between the view and cull dist
        trap::R_SetRangeFog(ctx.engine, ctx.world.cg.refdef.fov_x * 64.0);
    } else if ctx.world.view.cg_rangedFogging {
        // disable it
        ctx.world.view.cg_rangedFogging = false;
        trap::R_SetRangeFog(ctx.engine, 0.0);
    }

    let cstr = CG_ConfigString(ctx, CS_SKYBOXORG);
    if !cstr.is_empty() {
        // we have a skyportal
        CG_DrawSkyBoxPortal(ctx, &cstr);
    }

    CG_CalcScreenEffects(ctx);

    // first person blend blobs, done after AnglesToAxis
    if ctx.world.cg.renderingThirdPerson == 0
        && ctx.world.cg.predictedPlayerState.pm_type != pmtype_t::PM_SPECTATOR as c_int
    {
        CG_DamageBlendBlob(ctx);
    }

    // build the render lists
    if ctx.world.cg.hyperspace == qfalse {
        CG_AddPacketEntities(ctx, qfalse); // adter calcViewValues, so predicted player state is correct
        CG_AddMarks(ctx);
        CG_AddParticles(ctx);
        CG_AddLocalEntities(ctx);
        CG_DrawMiscEnts(ctx);
    }
    let predictedPs = ctx.world.cg.predictedPlayerState;
    CG_AddViewWeapon(ctx, &predictedPs);

    if ctx.world.cg.hyperspace == qfalse {
        trap::FX_AddScheduledEffects(ctx.engine, false);
    }

    // add buffered sounds
    CG_PlayBufferedSounds(ctx);

    // finish up the rest of the refdef
    if ctx.world.cg.testModelEntity.hModel != 0 {
        CG_AddTestModel(ctx);
    }
    ctx.world.cg.refdef.time = ctx.world.cg.time;
    let areamask = ctx.world.cg.snap_ref().unwrap().areamask;
    ctx.world.cg.refdef.areamask = areamask;

    // warning sounds when powerup is wearing off
    CG_PowerupTimerSounds(ctx.world);

    // if there are any entities flagged as sound trackers and attached to
    // other entities, update their sound pos
    CG_UpdateSoundTrackers(ctx);

    if ctx.world.draw.gCGHasFallVector {
        let mut lookAng: vec3_t = [0.0; 3];
        let snapOrigin = ctx.world.cg.snap_ref().unwrap().ps.origin;
        let vieworg = ctx.world.cg.refdef.vieworg;
        _VectorSubtract(snapOrigin, vieworg, &mut lookAng);
        VectorNormalize(&mut lookAng);
        vectoangles(lookAng, &mut lookAng);

        let fallVector = ctx.world.draw.gCGFallVector;
        _VectorCopy(fallVector, &mut ctx.world.cg.refdef.vieworg);
        AnglesToAxis(lookAng, ctx.world.cg.refdef.viewaxis.as_mut_ptr());
    }

    // This is done from the vieworg to get origin for non-attenuated sounds
    let cstr = CG_ConfigString(ctx, CS_GLOBAL_AMBIENT_SET);
    if !cstr.is_empty() {
        let vieworg = ctx.world.cg.refdef.vieworg;
        trap::S_UpdateAmbientSet(ctx.engine, &cstr, &vieworg);
    }

    // update audio positions
    let clientNum = ctx.world.cg.snap_ref().unwrap().ps.clientNum;
    let vieworg = ctx.world.cg.refdef.vieworg;
    let viewaxis = ctx.world.cg.refdef.viewaxis;
    trap::S_Respatialize(ctx.engine, clientNum, &vieworg, &viewaxis, inwater as c_int);

    // make sure the lagometerSample and frame timing isn't done twice when in stereo
    if stereoView != STEREO_RIGHT {
        ctx.world.cg.frametime = ctx.world.cg.time - ctx.world.cg.oldTime;
        if ctx.world.cg.frametime < 0 {
            ctx.world.cg.frametime = 0;
        }
        ctx.world.cg.oldTime = ctx.world.cg.time;
        CG_AddLagometerFrameInfo(ctx.world);
    }

    if ctx.world.cvars.cg_timescale.value != ctx.world.cvars.cg_timescaleFadeEnd.value {
        if ctx.world.cvars.cg_timescale.value < ctx.world.cvars.cg_timescaleFadeEnd.value {
            ctx.world.cvars.cg_timescale.value += ctx.world.cvars.cg_timescaleFadeSpeed.value
                * (ctx.world.cg.frametime as f32)
                / 1000.0;
            if ctx.world.cvars.cg_timescale.value > ctx.world.cvars.cg_timescaleFadeEnd.value {
                ctx.world.cvars.cg_timescale.value = ctx.world.cvars.cg_timescaleFadeEnd.value;
            }
        } else {
            ctx.world.cvars.cg_timescale.value -= ctx.world.cvars.cg_timescaleFadeSpeed.value
                * (ctx.world.cg.frametime as f32)
                / 1000.0;
            if ctx.world.cvars.cg_timescale.value < ctx.world.cvars.cg_timescaleFadeEnd.value {
                ctx.world.cvars.cg_timescale.value = ctx.world.cvars.cg_timescaleFadeEnd.value;
            }
        }
        if ctx.world.cvars.cg_timescaleFadeSpeed.value != 0.0 {
            let value = ctx.world.cvars.cg_timescale.value;
            trap::Cvar_Set(ctx.engine, "timescale", &format!("{value:.6}"));
        }
    }

    // actually issue the rendering calls
    CG_DrawActive(ctx, stereoView, menus, ds);

    CG_DrawAutoMap(ctx);

    if ctx.world.cvars.cg_stats.integer != 0 {
        let clientFrame = ctx.world.cg.clientFrame;
        CG_Printf(ctx, &format!("cg.clientFrame:{clientFrame}\n"));
    }
}
