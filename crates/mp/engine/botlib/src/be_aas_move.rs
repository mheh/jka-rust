#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_assignments,
    unused_parens,
    clippy::too_many_arguments
)]

//! MP botlib `be_aas_move.cpp` — AAS client-movement prediction (accelerate/
//! friction/air-control physics, ladder/swim/ground checks, weapon-jump
//! velocity, bbox-clip trace, and the frame-stepped `AAS_ClientMovementPrediction`
//! core the reachability/jump code predicts through).
//!
//! Source: `oracle/codemp/botlib/be_aas_move.cpp`

use core::ffi::c_int;

use mp_engine_qcommon::common::Common;
use mp_qshared::common::mp::botlib::aas_clientmove_s::{aas_clientmove_s, aas_clientmove_t};
use mp_qshared::common::mp::botlib::aas_stop_event::{
    SE_ENTERAREA, SE_ENTERLAVA, SE_ENTERSLIME, SE_ENTERWATER, SE_GAP, SE_HITBOUNDINGBOX,
    SE_HITGROUND, SE_HITGROUNDAREA, SE_HITGROUNDDAMAGE, SE_LEAVEGROUND, SE_NONE,
    SE_TOUCHCLUSTERPORTAL, SE_TOUCHJUMPPAD, SE_TOUCHTELEPORTER,
};
use mp_qshared::common::mp::botlib::aas_trace_s::aas_trace_t;
use mp_qshared::common::mp::botlib::line_color::{LINECOLOR_BLUE, LINECOLOR_RED};
use mp_qshared::common::mp::botlib::print_type::PRT_MESSAGE;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin,
    AngleVectors, VectorClear, VectorCompare, VectorLength, VectorNormalize, PITCH, ROLL, YAW,
};
use mp_qshared::shared::surface_flags::{
    CONTENTS_LAVA, CONTENTS_SLIME, CONTENTS_SOLID, CONTENTS_WATER,
};
use mp_qshared::shared::vec3_t;

use crate::aasfile::aas_area_s::aas_area_t;
use crate::aasfile::aas_face_s::aas_face_t;
use crate::aasfile::aas_plane_s::aas_plane_t;
use crate::aasfile::aas_reachability_s::aas_reachability_t;
use crate::aasfile::area_contents::{
    AREACONTENTS_CLUSTERPORTAL, AREACONTENTS_JUMPPAD, AREACONTENTS_LAVA, AREACONTENTS_SLIME,
    AREACONTENTS_TELEPORTER, AREACONTENTS_WATER,
};
use crate::aasfile::area_flags::AREA_LADDER;
use crate::aasfile::face_flags::FACE_LADDER;
use crate::aasfile::presence_type::{PRESENCE_CROUCH, PRESENCE_NORMAL};
use crate::BotLib;

use crate::be_aas_bspq3_fns::{AAS_PointContents, AAS_Trace};
use crate::be_aas_debug_fns::{AAS_ClearShownDebugLines, AAS_DebugLine};
use crate::l_libvar_fns::LibVarValue;

use mp_engine_qcommon::common_fns::Com_Memset;

/// Raven `AAS_SetMovedir`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:218-232`
// `MOVEDIR_DOWN`/`MOVEDIR_UP`/`VEC_DOWN`/`VEC_UP` are private consts local to
// `g_utils.rs`'s own `AAS_SetMovedir`, not importable; redefined here inline.
pub fn AAS_SetMovedir(bot: &mut BotLib, angles: vec3_t, movedir: *mut vec3_t) {
    const VEC_UP: vec3_t = [0.0, -1.0, 0.0];
    const MOVEDIR_UP: vec3_t = [0.0, 0.0, 1.0];
    const VEC_DOWN: vec3_t = [0.0, -2.0, 0.0];
    const MOVEDIR_DOWN: vec3_t = [0.0, 0.0, -1.0];

    unsafe {
        if VectorCompare(angles, VEC_UP) {
            _VectorCopy(MOVEDIR_UP, &mut *movedir);
        } else if VectorCompare(angles, VEC_DOWN) {
            _VectorCopy(MOVEDIR_DOWN, &mut *movedir);
        } else {
            AngleVectors(angles, Some(&mut *movedir), None, None);
        }
    }
}

/// Raven `AAS_Accelerate`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:347-366`
// q2 style
pub fn AAS_Accelerate(
    velocity: *mut vec3_t,
    frametime: f32,
    wishdir: vec3_t,
    wishspeed: f32,
    accel: f32,
) {
    unsafe {
        let currentspeed = _DotProduct(*velocity, wishdir);
        let addspeed = wishspeed - currentspeed;
        if addspeed <= 0.0 {
            return;
        }
        let mut accelspeed = accel * frametime * wishspeed;
        if accelspeed > addspeed {
            accelspeed = addspeed;
        }

        for i in 0..3 {
            (*velocity)[i] += accelspeed * wishdir[i];
        }
    }
}

/// Raven `AAS_AirControl`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:373-378`
pub fn AAS_AirControl(start: vec3_t, end: vec3_t, velocity: vec3_t, cmdmove: vec3_t) {
    let mut dir: vec3_t = [0.0; 3];
    _VectorSubtract(end, start, &mut dir);
}

/// Raven `AAS_ApplyFriction`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:386-402`
pub fn AAS_ApplyFriction(vel: *mut vec3_t, friction: f32, stopspeed: f32, frametime: f32) {
    unsafe {
        // horizontal speed
        let speed = ((*vel)[0] * (*vel)[0] + (*vel)[1] * (*vel)[1]).sqrt();
        if speed != 0.0 {
            let control = if speed < stopspeed { stopspeed } else { speed };
            let mut newspeed = speed - frametime * control * friction;
            if newspeed < 0.0 {
                newspeed = 0.0;
            }
            newspeed /= speed;
            (*vel)[0] *= newspeed;
            (*vel)[1] *= newspeed;
        }
    }
}

/// Raven `AAS_HorizontalVelocityForJump`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:1045-1084`
pub fn AAS_HorizontalVelocityForJump(
    bot: &mut BotLib,
    zvel: f32,
    start: vec3_t,
    end: vec3_t,
    velocity: *mut f32,
) -> c_int {
    unsafe {
        let phys_gravity = bot.aassettings.phys_gravity;
        let phys_maxvelocity = bot.aassettings.phys_maxvelocity;

        // maximum height a player can jump with the given initial z velocity
        let maxjump = 0.5 * phys_gravity * (zvel / phys_gravity) * (zvel / phys_gravity);
        // top of the parabolic jump
        let top = start[2] + maxjump;
        // height the bot will fall from the top
        let height2fall = top - end[2];
        // if the goal is to high to jump to
        if height2fall < 0.0 {
            *velocity = phys_maxvelocity;
            return 0;
        }
        // time a player takes to fall the height
        let t = (height2fall / (0.5 * phys_gravity)).sqrt();
        // direction from start to end
        let mut dir: vec3_t = [0.0; 3];
        _VectorSubtract(end, start, &mut dir);
        //
        if (t + zvel / phys_gravity) == 0.0f32 {
            *velocity = phys_maxvelocity;
            return 0;
        }
        // calculate horizontal speed
        *velocity = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt() / (t + zvel / phys_gravity);
        // the horizontal speed must be lower than the max speed
        if *velocity > phys_maxvelocity {
            *velocity = phys_maxvelocity;
            return 0;
        }
        1
    }
}

/// Raven `AAS_DropToFloor`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:39-50`
pub fn AAS_DropToFloor(bot: &mut BotLib, origin: *mut vec3_t, mins: vec3_t, maxs: vec3_t) -> c_int {
    unsafe {
        let mut end: vec3_t = [0.0; 3];
        _VectorCopy(*origin, &mut end);
        end[2] -= 100.0;
        let trace = AAS_Trace(bot, *origin, mins, maxs, end, 0, CONTENTS_SOLID);
        if trace.startsolid != 0 {
            return 0;
        }
        _VectorCopy(trace.endpos, &mut *origin);
        1
    }
}

/// Raven `AAS_AgainstLadder`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:106-160`
pub fn AAS_AgainstLadder(bot: &mut BotLib, origin: vec3_t) -> c_int {
    unsafe {
        let mut org: vec3_t = [0.0; 3];
        _VectorCopy(origin, &mut org);
        let mut areanum = crate::be_aas_sample_fns::AAS_PointAreaNum(bot, org);
        if areanum == 0 {
            org[0] += 1.0;
            areanum = crate::be_aas_sample_fns::AAS_PointAreaNum(bot, org);
            if areanum == 0 {
                org[1] += 1.0;
                areanum = crate::be_aas_sample_fns::AAS_PointAreaNum(bot, org);
                if areanum == 0 {
                    org[0] -= 2.0;
                    areanum = crate::be_aas_sample_fns::AAS_PointAreaNum(bot, org);
                    if areanum == 0 {
                        org[1] -= 2.0;
                        areanum = crate::be_aas_sample_fns::AAS_PointAreaNum(bot, org);
                    }
                }
            }
        }
        // if in solid... wrrr shouldn't happen
        if areanum == 0 {
            return 0;
        }
        // if not in a ladder area
        if (*bot.aasworld.areasettings.add(areanum as usize)).areaflags & AREA_LADDER == 0 {
            return 0;
        }
        // if a crouch only area
        if (*bot.aasworld.areasettings.add(areanum as usize)).presencetype & PRESENCE_NORMAL == 0 {
            return 0;
        }
        //
        let area: *mut aas_area_t = bot.aasworld.areas.add(areanum as usize);
        for i in 0..(*area).numfaces {
            let facenum = *bot.aasworld.faceindex.add(((*area).firstface + i) as usize);
            let side = (facenum < 0) as c_int;
            let face: *mut aas_face_t = bot.aasworld.faces.add(facenum.unsigned_abs() as usize);
            // if the face isn't a ladder face
            if (*face).faceflags & FACE_LADDER == 0 {
                continue;
            }
            // get the plane the face is in
            let plane: *mut aas_plane_t =
                bot.aasworld.planes.add(((*face).planenum ^ side) as usize);
            // if the origin is pretty close to the plane
            if (_DotProduct((*plane).normal, origin) - (*plane).dist).abs() < 3.0 {
                if crate::be_aas_sample_fns::AAS_PointInsideFace(
                    bot,
                    facenum.unsigned_abs() as c_int,
                    origin,
                    0.1,
                ) != 0
                {
                    return 1;
                }
            }
        }
        0
    }
}

/// Raven `AAS_Swimming`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:198-206`
pub fn AAS_Swimming(bot: &mut BotLib, origin: vec3_t) -> c_int {
    unsafe {
        let mut testorg: vec3_t = [0.0; 3];
        _VectorCopy(origin, &mut testorg);
        testorg[2] -= 2.0;
        if AAS_PointContents(bot, testorg) & (CONTENTS_LAVA | CONTENTS_SLIME | CONTENTS_WATER) != 0
        {
            return 1;
        }
        0
    }
}

/// Raven `AAS_WeaponJumpZVelocity`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:273-317`
pub fn AAS_WeaponJumpZVelocity(bot: &mut BotLib, origin: vec3_t, radiusdamage: f32) -> f32 {
    unsafe {
        let rocketoffset: vec3_t = [8.0, 8.0, -8.0];
        let botmins: vec3_t = [-16.0, -16.0, -24.0];
        let botmaxs: vec3_t = [16.0, 16.0, 32.0];

        // look down (90 degrees)
        let mut viewangles: vec3_t = [0.0; 3];
        viewangles[PITCH as usize] = 90.0;
        viewangles[YAW as usize] = 0.0;
        viewangles[ROLL as usize] = 0.0;
        // get the start point shooting from
        let mut start: vec3_t = [0.0; 3];
        _VectorCopy(origin, &mut start);
        start[2] += 8.0; // view offset Z
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        AngleVectors(viewangles, Some(&mut forward), Some(&mut right), None);
        start[0] += forward[0] * rocketoffset[0] + right[0] * rocketoffset[1];
        start[1] += forward[1] * rocketoffset[0] + right[1] * rocketoffset[1];
        start[2] += forward[2] * rocketoffset[0] + right[2] * rocketoffset[1] + rocketoffset[2];
        // end point of the trace
        let mut end: vec3_t = [0.0; 3];
        _VectorMA(start, 500.0, forward, &mut end);
        // trace a line to get the impact point
        let bsptrace = AAS_Trace(bot, start, [0.0; 3], [0.0; 3], end, 1, CONTENTS_SOLID);
        // calculate the damage the bot will get from the rocket impact
        let mut v: vec3_t = [0.0; 3];
        _VectorAdd(botmins, botmaxs, &mut v);
        _VectorMA(origin, 0.5, v, &mut v);
        _VectorSubtract(bsptrace.endpos, v, &mut v);
        //
        let mut points = radiusdamage - 0.5 * VectorLength(v);
        if points < 0.0 {
            points = 0.0;
        }
        // the owner of the rocket gets half the damage
        points *= 0.5;
        // mass of the bot (p_client.c: PutClientInServer)
        let mass = 200.0f32;
        // knockback is the same as the damage points
        let knockback = points;
        // direction of the damage (from trace.endpos to bot origin)
        let mut dir: vec3_t = [0.0; 3];
        _VectorSubtract(origin, bsptrace.endpos, &mut dir);
        VectorNormalize(&mut dir);
        // damage velocity
        let mut kvel: vec3_t = [0.0; 3];
        _VectorScale(dir, 1600.0 * knockback / mass, &mut kvel); // the rocket jump hack...
                                                                 // rocket impact velocity + jump velocity
        kvel[2] + bot.aassettings.phys_jumpvel
    }
}

/// Raven `AAS_ClipToBBox`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:409-469`
pub fn AAS_ClipToBBox(
    bot: &mut BotLib,
    trace: *mut aas_trace_t,
    start: vec3_t,
    end: vec3_t,
    presencetype: c_int,
    mins: vec3_t,
    maxs: vec3_t,
) -> c_int {
    unsafe {
        let mut bboxmins: vec3_t = [0.0; 3];
        let mut bboxmaxs: vec3_t = [0.0; 3];
        crate::be_aas_sample_fns::AAS_PresenceTypeBoundingBox(
            bot,
            presencetype,
            &mut bboxmins,
            &mut bboxmaxs,
        );
        let mut absmins: vec3_t = [0.0; 3];
        let mut absmaxs: vec3_t = [0.0; 3];
        _VectorSubtract(mins, bboxmaxs, &mut absmins);
        _VectorSubtract(maxs, bboxmins, &mut absmaxs);
        //
        _VectorCopy(end, &mut (*trace).endpos);
        (*trace).fraction = 1.0;
        for i in 0..3 {
            if start[i] < absmins[i] && end[i] < absmins[i] {
                return 0;
            }
            if start[i] > absmaxs[i] && end[i] > absmaxs[i] {
                return 0;
            }
        }
        // check bounding box collision
        let mut dir: vec3_t = [0.0; 3];
        _VectorSubtract(end, start, &mut dir);
        let mut frac = 1.0f32;
        let mut mid: vec3_t = [0.0; 3];
        let mut i = 0usize;
        let mut planedist;
        while i < 3 {
            // get plane to test collision with for the current axis direction
            if dir[i] > 0.0 {
                planedist = absmins[i];
            } else {
                planedist = absmaxs[i];
            }
            // calculate collision fraction
            let front = start[i] - planedist;
            let back = end[i] - planedist;
            frac = front / (front - back);
            // check if between bounding planes of next axis
            let mut side = i + 1;
            if side > 2 {
                side = 0;
            }
            mid[side] = start[side] + dir[side] * frac;
            if mid[side] > absmins[side] && mid[side] < absmaxs[side] {
                // check if between bounding planes of next axis
                side += 1;
                if side > 2 {
                    side = 0;
                }
                mid[side] = start[side] + dir[side] * frac;
                if mid[side] > absmins[side] && mid[side] < absmaxs[side] {
                    mid[i] = planedist;
                    break;
                }
            }
            i += 1;
        }
        // if there was a collision
        if i != 3 {
            (*trace).startsolid = 0;
            (*trace).fraction = frac;
            (*trace).ent = 0;
            (*trace).planenum = 0;
            (*trace).area = 0;
            (*trace).lastarea = 0;
            // trace endpos
            for j in 0..3 {
                (*trace).endpos[j] = start[j] + dir[j] * frac;
            }
            return 1;
        }
        0
    }
}

/// Raven `AAS_RocketJumpZVelocity`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:324-328`
pub fn AAS_RocketJumpZVelocity(bot: &mut BotLib, origin: vec3_t) -> f32 {
    // rocket radius damage is 120 (p_weapon.c: Weapon_RocketLauncher_Fire)
    AAS_WeaponJumpZVelocity(bot, origin, 120.0)
}

/// Raven `AAS_BFGJumpZVelocity`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:335-339`
pub fn AAS_BFGJumpZVelocity(bot: &mut BotLib, origin: vec3_t) -> f32 {
    // bfg radius damage is 1000 (p_weapon.c: weapon_bfg_fire)
    // Raven: the oracle itself passes 120, not 1000, despite the comment.
    AAS_WeaponJumpZVelocity(bot, origin, 120.0)
}

/// Raven `AAS_InitSettings`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:57-98`
pub fn AAS_InitSettings(bot: &mut BotLib) {
    unsafe {
        bot.aassettings.phys_gravitydirection[0] = 0.0;
        bot.aassettings.phys_gravitydirection[1] = 0.0;
        bot.aassettings.phys_gravitydirection[2] = -1.0;
        bot.aassettings.phys_friction = LibVarValue(bot, "phys_friction", "6");
        bot.aassettings.phys_stopspeed = LibVarValue(bot, "phys_stopspeed", "100");
        bot.aassettings.phys_gravity = LibVarValue(bot, "phys_gravity", "800");
        bot.aassettings.phys_waterfriction = LibVarValue(bot, "phys_waterfriction", "1");
        bot.aassettings.phys_watergravity = LibVarValue(bot, "phys_watergravity", "400");
        bot.aassettings.phys_maxvelocity = LibVarValue(bot, "phys_maxvelocity", "320");
        bot.aassettings.phys_maxwalkvelocity = LibVarValue(bot, "phys_maxwalkvelocity", "320");
        bot.aassettings.phys_maxcrouchvelocity =
            LibVarValue(bot, "phys_maxcrouchvelocity", "100");
        bot.aassettings.phys_maxswimvelocity = LibVarValue(bot, "phys_maxswimvelocity", "150");
        bot.aassettings.phys_walkaccelerate = LibVarValue(bot, "phys_walkaccelerate", "10");
        bot.aassettings.phys_airaccelerate = LibVarValue(bot, "phys_airaccelerate", "1");
        bot.aassettings.phys_swimaccelerate = LibVarValue(bot, "phys_swimaccelerate", "4");
        bot.aassettings.phys_maxstep = LibVarValue(bot, "phys_maxstep", "19");
        bot.aassettings.phys_maxsteepness = LibVarValue(bot, "phys_maxsteepness", "0.7");
        bot.aassettings.phys_maxwaterjump = LibVarValue(bot, "phys_maxwaterjump", "18");
        bot.aassettings.phys_maxbarrier = LibVarValue(bot, "phys_maxbarrier", "33");
        bot.aassettings.phys_jumpvel = LibVarValue(bot, "phys_jumpvel", "270");
        bot.aassettings.phys_falldelta5 = LibVarValue(bot, "phys_falldelta5", "40");
        bot.aassettings.phys_falldelta10 = LibVarValue(bot, "phys_falldelta10", "60");
        bot.aassettings.rs_waterjump = LibVarValue(bot, "rs_waterjump", "400");
        bot.aassettings.rs_teleport = LibVarValue(bot, "rs_teleport", "50");
        bot.aassettings.rs_barrierjump = LibVarValue(bot, "rs_barrierjump", "100");
        bot.aassettings.rs_startcrouch = LibVarValue(bot, "rs_startcrouch", "300");
        bot.aassettings.rs_startgrapple = LibVarValue(bot, "rs_startgrapple", "500");
        bot.aassettings.rs_startwalkoffledge = LibVarValue(bot, "rs_startwalkoffledge", "70");
        bot.aassettings.rs_startjump = LibVarValue(bot, "rs_startjump", "300");
        bot.aassettings.rs_rocketjump = LibVarValue(bot, "rs_rocketjump", "500");
        bot.aassettings.rs_bfgjump = LibVarValue(bot, "rs_bfgjump", "500");
        bot.aassettings.rs_jumppad = LibVarValue(bot, "rs_jumppad", "250");
        bot.aassettings.rs_aircontrolledjumppad =
            LibVarValue(bot, "rs_aircontrolledjumppad", "300");
        bot.aassettings.rs_funcbob = LibVarValue(bot, "rs_funcbob", "300");
        bot.aassettings.rs_startelevator = LibVarValue(bot, "rs_startelevator", "50");
        bot.aassettings.rs_falldamage5 = LibVarValue(bot, "rs_falldamage5", "300");
        bot.aassettings.rs_falldamage10 = LibVarValue(bot, "rs_falldamage10", "500");
        bot.aassettings.rs_maxfallheight = LibVarValue(bot, "rs_maxfallheight", "0");
        bot.aassettings.rs_maxjumpfallheight = LibVarValue(bot, "rs_maxjumpfallheight", "450");
    }
}

/// Raven `AAS_OnGround`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:168-190`
pub fn AAS_OnGround(
    bot: &mut BotLib,
    origin: vec3_t,
    presencetype: c_int,
    passent: c_int,
) -> c_int {
    unsafe {
        let up: vec3_t = [0.0, 0.0, 1.0];
        let mut end: vec3_t = [0.0; 3];
        _VectorCopy(origin, &mut end);
        end[2] -= 10.0;

        let trace =
            crate::be_aas_sample_fns::AAS_TraceClientBBox(bot, origin, end, presencetype, passent);

        // if in solid
        if trace.startsolid != 0 {
            return 0;
        }
        // if nothing hit at all
        if trace.fraction >= 1.0 {
            return 0;
        }
        // if too far from the hit plane
        if origin[2] - trace.endpos[2] > 10.0 {
            return 0;
        }
        // check if the plane isn't too steep
        let plane = crate::be_aas_sample_fns::AAS_PlaneFromNum(bot, trace.planenum);
        if _DotProduct((*plane).normal, up) < bot.aassettings.phys_maxsteepness {
            return 0;
        }
        // the bot is on the ground
        1
    }
}

/// Raven `AAS_ClientMovementPrediction`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:488-970`
pub fn AAS_ClientMovementPrediction(
    bot: &mut BotLib,
    r#move: *mut aas_clientmove_s,
    entnum: c_int,
    origin: vec3_t,
    mut presencetype: c_int,
    mut onground: c_int,
    velocity: vec3_t,
    cmdmove: vec3_t,
    cmdframes: c_int,
    maxframes: c_int,
    mut frametime: f32,
    stopevent: c_int,
    stopareanum: c_int,
    mins: vec3_t,
    maxs: vec3_t,
    visualize: c_int,
) -> c_int {
    unsafe {
        if frametime <= 0.0 {
            frametime = 0.1;
        }
        //
        let phys_friction = bot.aassettings.phys_friction;
        let phys_stopspeed = bot.aassettings.phys_stopspeed;
        let phys_gravity = bot.aassettings.phys_gravity;
        let phys_waterfriction = bot.aassettings.phys_waterfriction;
        let phys_watergravity = bot.aassettings.phys_watergravity;
        let phys_maxwalkvelocity = bot.aassettings.phys_maxwalkvelocity; // * frametime;
        let phys_maxcrouchvelocity = bot.aassettings.phys_maxcrouchvelocity; // * frametime;
        let phys_maxswimvelocity = bot.aassettings.phys_maxswimvelocity; // * frametime;
        let phys_walkaccelerate = bot.aassettings.phys_walkaccelerate;
        let phys_airaccelerate = bot.aassettings.phys_airaccelerate;
        let phys_swimaccelerate = bot.aassettings.phys_swimaccelerate;
        let phys_maxstep = bot.aassettings.phys_maxstep;
        let phys_maxsteepness = bot.aassettings.phys_maxsteepness;
        let phys_jumpvel = bot.aassettings.phys_jumpvel * frametime;
        //
        Com_Memset(
            r#move as *mut (),
            0,
            core::mem::size_of::<aas_clientmove_t>(),
        );
        let mut trace: aas_trace_t = core::mem::zeroed();
        Com_Memset(
            &mut trace as *mut aas_trace_t as *mut (),
            0,
            core::mem::size_of::<aas_trace_t>(),
        );
        // start at the current origin
        let mut org: vec3_t = [0.0; 3];
        _VectorCopy(origin, &mut org);
        org[2] += 0.25;
        // velocity to test for the first frame
        let mut frame_test_vel: vec3_t = [0.0; 3];
        _VectorScale(velocity, frametime, &mut frame_test_vel);
        //
        let mut jump_frame: c_int = -1;
        let up: vec3_t = [0.0, 0.0, 1.0];
        let mut n: c_int = 0;
        // predict a maximum of 'maxframes' ahead
        while n < maxframes {
            let swimming = AAS_Swimming(bot, org);
            // get gravity depending on swimming or not
            let gravity = if swimming != 0 {
                phys_watergravity
            } else {
                phys_gravity
            };
            // apply gravity at the START of the frame
            frame_test_vel[2] -= gravity * 0.1 * frametime;
            // if on the ground or swimming
            if onground != 0 || swimming != 0 {
                let friction = if swimming != 0 {
                    phys_friction
                } else {
                    phys_waterfriction
                };
                // apply friction
                _VectorScale(frame_test_vel, 1.0 / frametime, &mut frame_test_vel);
                AAS_ApplyFriction(&mut frame_test_vel, friction, phys_stopspeed, frametime);
                _VectorScale(frame_test_vel, frametime, &mut frame_test_vel);
            }
            let mut crouch = 0;
            // apply command movement
            if n < cmdframes {
                let mut maxvel = phys_maxwalkvelocity;
                let mut accelerate = phys_airaccelerate;
                let mut wishdir: vec3_t = [0.0; 3];
                _VectorCopy(cmdmove, &mut wishdir);
                if onground != 0 {
                    if cmdmove[2] < -300.0 {
                        crouch = 1;
                        maxvel = phys_maxcrouchvelocity;
                    }
                    // if not swimming and upmove is positive then jump
                    if swimming == 0 && cmdmove[2] > 1.0 {
                        // jump velocity minus the gravity for one frame + 5 for safety
                        frame_test_vel[2] = phys_jumpvel - (gravity * 0.1 * frametime) + 5.0;
                        jump_frame = n;
                        // jumping so air accelerate
                        accelerate = phys_airaccelerate;
                    } else {
                        accelerate = phys_walkaccelerate;
                    }
                }
                if swimming != 0 {
                    maxvel = phys_maxswimvelocity;
                    accelerate = phys_swimaccelerate;
                } else {
                    wishdir[2] = 0.0;
                }
                //
                let mut wishspeed = VectorNormalize(&mut wishdir);
                if wishspeed > maxvel {
                    wishspeed = maxvel;
                }
                _VectorScale(frame_test_vel, 1.0 / frametime, &mut frame_test_vel);
                AAS_Accelerate(
                    &mut frame_test_vel,
                    frametime,
                    wishdir,
                    wishspeed,
                    accelerate,
                );
                _VectorScale(frame_test_vel, frametime, &mut frame_test_vel);
            }
            if crouch != 0 {
                presencetype = PRESENCE_CROUCH;
            } else if presencetype == PRESENCE_CROUCH {
                if crate::be_aas_sample_fns::AAS_PointPresenceType(bot, org) & PRESENCE_NORMAL != 0
                {
                    presencetype = PRESENCE_NORMAL;
                }
            }
            // save the current origin
            let mut lastorg: vec3_t = [0.0; 3];
            _VectorCopy(org, &mut lastorg);
            // move linear during one frame
            let mut left_test_vel: vec3_t = [0.0; 3];
            _VectorCopy(frame_test_vel, &mut left_test_vel);
            let mut j: c_int = 0;
            let mut end: vec3_t;
            loop {
                end = [0.0; 3];
                _VectorAdd(org, left_test_vel, &mut end);
                // trace a bounding box
                trace = crate::be_aas_sample_fns::AAS_TraceClientBBox(
                    bot,
                    org,
                    end,
                    presencetype,
                    entnum,
                );
                //
                if visualize != 0 {
                    if trace.startsolid != 0 {
                        bot.botimport.Print.unwrap()(
                            PRT_MESSAGE,
                            c"PredictMovement: start solid\n".as_ptr() as *mut _,
                        );
                    }
                    AAS_DebugLine(bot, org, trace.endpos, LINECOLOR_RED);
                }
                //
                if stopevent
                    & (SE_ENTERAREA | SE_TOUCHJUMPPAD | SE_TOUCHTELEPORTER | SE_TOUCHCLUSTERPORTAL)
                    != 0
                {
                    let mut areas: [c_int; 20] = [0; 20];
                    let mut points: [vec3_t; 20] = [[0.0; 3]; 20];
                    let numareas = crate::be_aas_sample_fns::AAS_TraceAreas(
                        bot,
                        org,
                        trace.endpos,
                        areas.as_mut_ptr(),
                        points.as_mut_ptr(),
                        20,
                    );
                    for i in 0..numareas {
                        let ai = areas[i as usize];
                        if stopevent & SE_ENTERAREA != 0 && ai == stopareanum {
                            _VectorCopy(points[i as usize], &mut (*r#move).endpos);
                            _VectorScale(frame_test_vel, 1.0 / frametime, &mut (*r#move).velocity);
                            (*r#move).endarea = ai;
                            (*r#move).trace = trace;
                            (*r#move).stopevent = SE_ENTERAREA;
                            (*r#move).presencetype = presencetype;
                            (*r#move).endcontents = 0;
                            (*r#move).time = n as f32 * frametime;
                            (*r#move).frames = n;
                            return 1;
                        }
                        // NOTE: if not the first frame
                        if (stopevent & SE_TOUCHJUMPPAD != 0) && n != 0 {
                            if (*bot.aasworld.areasettings.add(ai as usize)).contents
                                & AREACONTENTS_JUMPPAD
                                != 0
                            {
                                _VectorCopy(points[i as usize], &mut (*r#move).endpos);
                                _VectorScale(
                                    frame_test_vel,
                                    1.0 / frametime,
                                    &mut (*r#move).velocity,
                                );
                                (*r#move).endarea = ai;
                                (*r#move).trace = trace;
                                (*r#move).stopevent = SE_TOUCHJUMPPAD;
                                (*r#move).presencetype = presencetype;
                                (*r#move).endcontents = 0;
                                (*r#move).time = n as f32 * frametime;
                                (*r#move).frames = n;
                                return 1;
                            }
                        }
                        if stopevent & SE_TOUCHTELEPORTER != 0
                            && (*bot.aasworld.areasettings.add(ai as usize)).contents
                                & AREACONTENTS_TELEPORTER
                                != 0
                        {
                            _VectorCopy(points[i as usize], &mut (*r#move).endpos);
                            (*r#move).endarea = ai;
                            _VectorScale(frame_test_vel, 1.0 / frametime, &mut (*r#move).velocity);
                            (*r#move).trace = trace;
                            (*r#move).stopevent = SE_TOUCHTELEPORTER;
                            (*r#move).presencetype = presencetype;
                            (*r#move).endcontents = 0;
                            (*r#move).time = n as f32 * frametime;
                            (*r#move).frames = n;
                            return 1;
                        }
                        if stopevent & SE_TOUCHCLUSTERPORTAL != 0
                            && (*bot.aasworld.areasettings.add(ai as usize)).contents
                                & AREACONTENTS_CLUSTERPORTAL
                                != 0
                        {
                            _VectorCopy(points[i as usize], &mut (*r#move).endpos);
                            (*r#move).endarea = ai;
                            _VectorScale(frame_test_vel, 1.0 / frametime, &mut (*r#move).velocity);
                            (*r#move).trace = trace;
                            (*r#move).stopevent = SE_TOUCHCLUSTERPORTAL;
                            (*r#move).presencetype = presencetype;
                            (*r#move).endcontents = 0;
                            (*r#move).time = n as f32 * frametime;
                            (*r#move).frames = n;
                            return 1;
                        }
                    }
                }
                //
                if stopevent & SE_HITBOUNDINGBOX != 0 {
                    if AAS_ClipToBBox(
                        bot,
                        &mut trace as *mut aas_trace_t,
                        org,
                        trace.endpos,
                        presencetype,
                        mins,
                        maxs,
                    ) != 0
                    {
                        _VectorCopy(trace.endpos, &mut (*r#move).endpos);
                        (*r#move).endarea =
                            crate::be_aas_sample_fns::AAS_PointAreaNum(bot, (*r#move).endpos);
                        _VectorScale(frame_test_vel, 1.0 / frametime, &mut (*r#move).velocity);
                        (*r#move).trace = trace;
                        (*r#move).stopevent = SE_HITBOUNDINGBOX;
                        (*r#move).presencetype = presencetype;
                        (*r#move).endcontents = 0;
                        (*r#move).time = n as f32 * frametime;
                        (*r#move).frames = n;
                        return 1;
                    }
                }
                // move the entity to the trace end point
                _VectorCopy(trace.endpos, &mut org);
                // if there was a collision
                if trace.fraction < 1.0 {
                    // get the plane the bounding box collided with
                    let plane = crate::be_aas_sample_fns::AAS_PlaneFromNum(bot, trace.planenum);
                    //
                    if stopevent & SE_HITGROUNDAREA != 0
                        && _DotProduct((*plane).normal, up) > phys_maxsteepness
                    {
                        let mut start: vec3_t = [0.0; 3];
                        _VectorCopy(org, &mut start);
                        start[2] += 0.5;
                        if crate::be_aas_sample_fns::AAS_PointAreaNum(bot, start) == stopareanum {
                            _VectorCopy(start, &mut (*r#move).endpos);
                            (*r#move).endarea = stopareanum;
                            _VectorScale(frame_test_vel, 1.0 / frametime, &mut (*r#move).velocity);
                            (*r#move).trace = trace;
                            (*r#move).stopevent = SE_HITGROUNDAREA;
                            (*r#move).presencetype = presencetype;
                            (*r#move).endcontents = 0;
                            (*r#move).time = n as f32 * frametime;
                            (*r#move).frames = n;
                            return 1;
                        }
                    }
                    // assume there's no step
                    let mut step = false;
                    // if it is a vertical plane and the bot didn't jump recently
                    if (*plane).normal[2] == 0.0 && (jump_frame < 0 || n - jump_frame > 2) {
                        // check for a step
                        let mut start: vec3_t = [0.0; 3];
                        _VectorMA(org, -0.25, (*plane).normal, &mut start);
                        let mut stepend: vec3_t = [0.0; 3];
                        _VectorCopy(start, &mut stepend);
                        start[2] += phys_maxstep;
                        let steptrace = crate::be_aas_sample_fns::AAS_TraceClientBBox(
                            bot,
                            start,
                            stepend,
                            presencetype,
                            entnum,
                        );
                        //
                        if steptrace.startsolid == 0 {
                            let plane2 =
                                crate::be_aas_sample_fns::AAS_PlaneFromNum(bot, steptrace.planenum);
                            if _DotProduct((*plane2).normal, up) > phys_maxsteepness {
                                _VectorSubtract(end, steptrace.endpos, &mut left_test_vel);
                                left_test_vel[2] = 0.0;
                                frame_test_vel[2] = 0.0;
                                if visualize != 0 && steptrace.endpos[2] - org[2] > 0.125 {
                                    let mut dbg_start: vec3_t = [0.0; 3];
                                    _VectorCopy(org, &mut dbg_start);
                                    dbg_start[2] = steptrace.endpos[2];
                                    AAS_DebugLine(bot, org, dbg_start, LINECOLOR_BLUE);
                                }
                                org[2] = steptrace.endpos[2];
                                step = true;
                            }
                        }
                    }
                    //
                    if !step {
                        // velocity left to test for this frame is the projection
                        // of the current test velocity into the hit plane
                        let dp = _DotProduct(left_test_vel, (*plane).normal);
                        _VectorMA(left_test_vel, -dp, (*plane).normal, &mut left_test_vel);
                        // store the old velocity for landing check
                        let mut old_frame_test_vel: vec3_t = [0.0; 3];
                        _VectorCopy(frame_test_vel, &mut old_frame_test_vel);
                        // test velocity for the next frame is the projection
                        // of the velocity of the current frame into the hit plane
                        let dp2 = _DotProduct(frame_test_vel, (*plane).normal);
                        _VectorMA(frame_test_vel, -dp2, (*plane).normal, &mut frame_test_vel);
                        // check for a landing on an almost horizontal floor
                        if _DotProduct((*plane).normal, up) > phys_maxsteepness {
                            onground = 1;
                        }
                        if stopevent & SE_HITGROUNDDAMAGE != 0 {
                            let mut delta = 0.0f32;
                            if old_frame_test_vel[2] < 0.0
                                && frame_test_vel[2] > old_frame_test_vel[2]
                                && onground == 0
                            {
                                delta = old_frame_test_vel[2];
                            } else if onground != 0 {
                                delta = frame_test_vel[2] - old_frame_test_vel[2];
                            }
                            if delta != 0.0 {
                                delta *= 10.0;
                                delta = delta * delta * 0.0001;
                                if swimming != 0 {
                                    delta = 0.0;
                                }
                                // never take falling damage if completely underwater
                                if delta > 40.0 {
                                    _VectorCopy(org, &mut (*r#move).endpos);
                                    (*r#move).endarea =
                                        crate::be_aas_sample_fns::AAS_PointAreaNum(bot, org);
                                    _VectorCopy(frame_test_vel, &mut (*r#move).velocity);
                                    (*r#move).trace = trace;
                                    (*r#move).stopevent = SE_HITGROUNDDAMAGE;
                                    (*r#move).presencetype = presencetype;
                                    (*r#move).endcontents = 0;
                                    (*r#move).time = n as f32 * frametime;
                                    (*r#move).frames = n;
                                    return 1;
                                }
                            }
                        }
                    }
                }
                // extra check to prevent endless loop
                j += 1;
                if j > 20 {
                    return 0;
                }
                // while there is a plane hit
                if !(trace.fraction < 1.0) {
                    break;
                }
            }
            // if going down
            if frame_test_vel[2] <= 10.0 {
                // check for a liquid at the feet of the bot
                let mut feet: vec3_t = [0.0; 3];
                _VectorCopy(org, &mut feet);
                feet[2] -= 22.0;
                let pc = AAS_PointContents(bot, feet);
                // get event from pc
                let mut event = SE_NONE;
                if pc & CONTENTS_LAVA != 0 {
                    event |= SE_ENTERLAVA;
                }
                if pc & CONTENTS_SLIME != 0 {
                    event |= SE_ENTERSLIME;
                }
                if pc & CONTENTS_WATER != 0 {
                    event |= SE_ENTERWATER;
                }
                //
                let areanum = crate::be_aas_sample_fns::AAS_PointAreaNum(bot, org);
                if (*bot.aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_LAVA
                    != 0
                {
                    event |= SE_ENTERLAVA;
                }
                if (*bot.aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_SLIME
                    != 0
                {
                    event |= SE_ENTERSLIME;
                }
                if (*bot.aasworld.areasettings.add(areanum as usize)).contents & AREACONTENTS_WATER
                    != 0
                {
                    event |= SE_ENTERWATER;
                }
                // if in lava or slime
                if event & stopevent != 0 {
                    _VectorCopy(org, &mut (*r#move).endpos);
                    (*r#move).endarea = areanum;
                    _VectorScale(frame_test_vel, 1.0 / frametime, &mut (*r#move).velocity);
                    (*r#move).stopevent = event & stopevent;
                    (*r#move).presencetype = presencetype;
                    (*r#move).endcontents = pc;
                    (*r#move).time = n as f32 * frametime;
                    (*r#move).frames = n;
                    return 1;
                }
            }
            //
            onground = AAS_OnGround(bot, org, presencetype, entnum);
            // if onground and on the ground for at least one whole frame
            if onground != 0 {
                if stopevent & SE_HITGROUND != 0 {
                    _VectorCopy(org, &mut (*r#move).endpos);
                    (*r#move).endarea = crate::be_aas_sample_fns::AAS_PointAreaNum(bot, org);
                    _VectorScale(frame_test_vel, 1.0 / frametime, &mut (*r#move).velocity);
                    (*r#move).trace = trace;
                    (*r#move).stopevent = SE_HITGROUND;
                    (*r#move).presencetype = presencetype;
                    (*r#move).endcontents = 0;
                    (*r#move).time = n as f32 * frametime;
                    (*r#move).frames = n;
                    return 1;
                }
            } else if stopevent & SE_LEAVEGROUND != 0 {
                _VectorCopy(org, &mut (*r#move).endpos);
                (*r#move).endarea = crate::be_aas_sample_fns::AAS_PointAreaNum(bot, org);
                _VectorScale(frame_test_vel, 1.0 / frametime, &mut (*r#move).velocity);
                (*r#move).trace = trace;
                (*r#move).stopevent = SE_LEAVEGROUND;
                (*r#move).presencetype = presencetype;
                (*r#move).endcontents = 0;
                (*r#move).time = n as f32 * frametime;
                (*r#move).frames = n;
                return 1;
            } else if stopevent & SE_GAP != 0 {
                let mut start: vec3_t = [0.0; 3];
                _VectorCopy(org, &mut start);
                let mut gend: vec3_t = [0.0; 3];
                _VectorCopy(start, &mut gend);
                gend[2] -= 48.0 + bot.aassettings.phys_maxbarrier;
                let gaptrace = crate::be_aas_sample_fns::AAS_TraceClientBBox(
                    bot,
                    start,
                    gend,
                    PRESENCE_CROUCH,
                    -1,
                );
                // if solid is found the bot cannot walk any further and will not fall into a gap
                if gaptrace.startsolid == 0 {
                    // if it is a gap (lower than one step height)
                    if gaptrace.endpos[2] < org[2] - bot.aassettings.phys_maxstep - 1.0 {
                        if AAS_PointContents(bot, gend) & CONTENTS_WATER == 0 {
                            _VectorCopy(lastorg, &mut (*r#move).endpos);
                            (*r#move).endarea =
                                crate::be_aas_sample_fns::AAS_PointAreaNum(bot, lastorg);
                            _VectorScale(frame_test_vel, 1.0 / frametime, &mut (*r#move).velocity);
                            (*r#move).trace = trace;
                            (*r#move).stopevent = SE_GAP;
                            (*r#move).presencetype = presencetype;
                            (*r#move).endcontents = 0;
                            (*r#move).time = n as f32 * frametime;
                            (*r#move).frames = n;
                            return 1;
                        }
                    }
                }
            }
            n += 1;
        }
        //
        _VectorCopy(org, &mut (*r#move).endpos);
        (*r#move).endarea = crate::be_aas_sample_fns::AAS_PointAreaNum(bot, org);
        _VectorScale(frame_test_vel, 1.0 / frametime, &mut (*r#move).velocity);
        (*r#move).stopevent = SE_NONE;
        (*r#move).presencetype = presencetype;
        (*r#move).endcontents = 0;
        (*r#move).time = n as f32 * frametime;
        (*r#move).frames = n;
        //
        1
    }
}

/// Raven `AAS_PredictClientMovement`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:977-990`
pub fn AAS_PredictClientMovement(
    bot: &mut BotLib,
    r#move: *mut aas_clientmove_s,
    entnum: c_int,
    origin: vec3_t,
    presencetype: c_int,
    onground: c_int,
    velocity: vec3_t,
    cmdmove: vec3_t,
    cmdframes: c_int,
    maxframes: c_int,
    frametime: f32,
    stopevent: c_int,
    stopareanum: c_int,
    visualize: c_int,
) -> c_int {
    let mins: vec3_t = [0.0; 3];
    let maxs: vec3_t = [0.0; 3];
    AAS_ClientMovementPrediction(
        bot,
        r#move,
        entnum,
        origin,
        presencetype,
        onground,
        velocity,
        cmdmove,
        cmdframes,
        maxframes,
        frametime,
        stopevent,
        stopareanum,
        mins,
        maxs,
        visualize,
    )
}

/// Raven `AAS_ClientMovementHitBBox`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:997-1009`
pub fn AAS_ClientMovementHitBBox(
    bot: &mut BotLib,
    r#move: *mut aas_clientmove_s,
    entnum: c_int,
    origin: vec3_t,
    presencetype: c_int,
    onground: c_int,
    velocity: vec3_t,
    cmdmove: vec3_t,
    cmdframes: c_int,
    maxframes: c_int,
    frametime: f32,
    mins: vec3_t,
    maxs: vec3_t,
    visualize: c_int,
) -> c_int {
    AAS_ClientMovementPrediction(
        bot,
        r#move,
        entnum,
        origin,
        presencetype,
        onground,
        velocity,
        cmdmove,
        cmdframes,
        maxframes,
        frametime,
        SE_HITBOUNDINGBOX,
        0,
        mins,
        maxs,
        visualize,
    )
}

/// Raven `AAS_JumpReachRunStart`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:239-265`
pub fn AAS_JumpReachRunStart(
    common: &mut Common,
    bot: &mut BotLib,
    reach: *mut aas_reachability_t,
    runstart: *mut vec3_t,
) {
    let _ = common;
    unsafe {
        let mut hordir: vec3_t = [0.0; 3];
        hordir[0] = (*reach).start[0] - (*reach).end[0];
        hordir[1] = (*reach).start[1] - (*reach).end[1];
        hordir[2] = 0.0;
        VectorNormalize(&mut hordir);
        // start point
        let mut start: vec3_t = [0.0; 3];
        _VectorCopy((*reach).start, &mut start);
        start[2] += 1.0;
        // get command movement
        let mut cmdmove: vec3_t = [0.0; 3];
        _VectorScale(hordir, 400.0, &mut cmdmove);
        //
        let mut r#move: aas_clientmove_t = core::mem::zeroed();
        AAS_PredictClientMovement(
            bot,
            &mut r#move as *mut aas_clientmove_t,
            -1,
            start,
            PRESENCE_NORMAL,
            1,
            vec3_origin,
            cmdmove,
            1,
            2,
            0.1,
            SE_ENTERWATER | SE_ENTERSLIME | SE_ENTERLAVA | SE_HITGROUNDDAMAGE | SE_GAP,
            0,
            0,
        );
        _VectorCopy(r#move.endpos, &mut *runstart);
        // don't enter slime or lava and don't fall from too high
        if r#move.stopevent & (SE_ENTERSLIME | SE_ENTERLAVA | SE_HITGROUNDDAMAGE) != 0 {
            _VectorCopy(start, &mut *runstart);
        }
    }
}

/// Raven `AAS_TestMovementPrediction`.
///
/// Source: `oracle/codemp/botlib/be_aas_move.cpp:1016-1033`
pub fn AAS_TestMovementPrediction(bot: &mut BotLib, entnum: c_int, origin: vec3_t, dir: vec3_t) {
    unsafe {
        let mut velocity: vec3_t = [0.0; 3];
        VectorClear(&mut velocity);
        let mut dir = dir;
        if AAS_Swimming(bot, origin) == 0 {
            dir[2] = 0.0;
        }
        VectorNormalize(&mut dir);
        let mut cmdmove: vec3_t = [0.0; 3];
        _VectorScale(dir, 400.0, &mut cmdmove);
        cmdmove[2] = 224.0;
        AAS_ClearShownDebugLines(bot);
        let mut r#move: aas_clientmove_t = core::mem::zeroed();
        AAS_PredictClientMovement(
            bot,
            &mut r#move as *mut aas_clientmove_t,
            entnum,
            origin,
            PRESENCE_NORMAL,
            1,
            velocity,
            cmdmove,
            13,
            13,
            0.1,
            SE_HITGROUND,
            0,
            1,
        );
        if r#move.stopevent & SE_LEAVEGROUND != 0 {
            bot.botimport.Print.unwrap()(PRT_MESSAGE, c"leave ground\n".as_ptr() as *mut _);
        }
    }
}
