//! Port of `oracle/codemp/cgame/cg_view.c` — view/camera placement, fov, and the per-frame scene build. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case, non_upper_case_globals)]

use core::f64::consts::PI;
use core::ffi::c_int;

use mp_bg::bg_misc::BG_EmplacedView;
use mp_bg::public::dm_flags::DF_FIXED_FOV;
use mp_bg::public::entity_effects::EF2_HELD_BY_MONSTER;
use mp_bg::public::entity_flags::{EF_NODRAW, EF_SOUNDTRACKER};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::pmtype::pmtype_t;
use mp_bg::public::stat_index::statIndex_t::{STAT_DEAD_YAW, STAT_HEALTH};
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::common::mp::qcommon::player_state::MAX_POWERUPS;
use mp_qshared::common::mp::qcommon::pm_flags::PMF_DUCKED;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorSubtract, AnglesToAxis,
    VectorNormalize, PITCH, ROLL, YAW,
};
use mp_qshared::shared::sound_channel::{CHAN_ANNOUNCER, CHAN_LOCAL};
use mp_qshared::shared::surface_flags::{
    CONTENTS_LAVA, CONTENTS_PLAYERCLIP, CONTENTS_SLIME, CONTENTS_WATER, MASK_SOLID,
};
use mp_qshared::shared::{
    qfalse, qtrue, sfxHandle_t, vec3_t, ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_QPATH,
};
use native_string::{atof, buf_to_string, Q_strncpyz};

use crate::cg_ents::CG_S_UpdateLoopingSounds;
use crate::cg_main::{CG_Argv, CG_Printf};
use crate::cg_predict::CG_PointContents;
use crate::cg_weapons::{LAND_DEFLECT_TIME, LAND_RETURN_TIME};
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
