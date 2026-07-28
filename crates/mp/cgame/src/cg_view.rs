//! Port of `oracle/codemp/cgame/cg_view.c` — view/camera placement, fov, and the per-frame scene build. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case, non_upper_case_globals)]

use core::f64::consts::PI;
use core::ffi::c_int;

use mp_bg::bg_misc::BG_EmplacedView;
use mp_bg::public::entity_effects::EF2_HELD_BY_MONSTER;
use mp_bg::public::entity_flags::EF_NODRAW;
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::pmtype::pmtype_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::common::mp::qcommon::player_state::MAX_POWERUPS;
use mp_qshared::shared::q_math::{
    _VectorAdd, _VectorCopy, _VectorMA, AnglesToAxis, PITCH, ROLL, YAW,
};
use mp_qshared::shared::sound_channel::CHAN_ANNOUNCER;
use mp_qshared::shared::surface_flags::{CONTENTS_PLAYERCLIP, MASK_SOLID};
use mp_qshared::shared::{qfalse, qtrue, sfxHandle_t, vec3_t};

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

/// Raven `DAMAGE_TIME` — how long the damage blend blob lives, in msec.
/// Source: `oracle/codemp/cgame/cg_local.h:30`
pub(crate) const DAMAGE_TIME: c_int = 500;

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
