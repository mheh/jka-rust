#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_assignments,
    unused_parens
)]

//! MP botlib `be_ai_move.cpp` movement-AI function bodies.
//!
//! Blind C-track transcription of `oracle/codemp/botlib/be_ai_move.cpp`.
//! Source-cited per fn. Vector macros expand to the ported q_math surface
//! (`_VectorSubtract`/`_VectorCopy`/… mirror the mp_game names, NAV-D6 home).

use core::ffi::{c_char, c_int, c_ulong};

use crate::aasfile::aas_reachability_s::aas_reachability_t;
use crate::aasfile::presence_type::{PRESENCE_CROUCH, PRESENCE_NORMAL};
use crate::aasfile::travel_type::{
    TRAVELTYPE_MASK, TRAVEL_BARRIERJUMP, TRAVEL_BFGJUMP, TRAVEL_CROUCH, TRAVEL_ELEVATOR,
    TRAVEL_FUNCBOB, TRAVEL_GRAPPLEHOOK, TRAVEL_JUMP, TRAVEL_JUMPPAD, TRAVEL_LADDER,
    TRAVEL_ROCKETJUMP, TRAVEL_SWIM, TRAVEL_TELEPORT, TRAVEL_WALK, TRAVEL_WALKOFFLEDGE,
    TRAVEL_WATERJUMP,
};
use crate::be_aas_bsp::be_aas_bsp_consts::MAX_EPAIRKEY;
use crate::be_ai_move::be_ai_move_cpp_consts::{
    AVOIDREACH_TIME, AVOIDREACH_TRIES, MODELTYPE_FUNC_BOB, MODELTYPE_FUNC_DOOR,
    MODELTYPE_FUNC_PLAT, MODELTYPE_FUNC_STATIC, PREDICTIONTIME_JUMP,
};
use crate::be_ai_move::bot_movestate_s::{
    bot_movestate_s, bot_movestate_t, MAX_AVOIDREACH, MAX_AVOIDSPOTS,
};
use crate::be_ai_move::move_consts::{
    AVOID_ALWAYS, AVOID_CLEAR, MFL_ACTIVEGRAPPLE, MFL_AGAINSTLADDER, MFL_BARRIERJUMP,
    MFL_GRAPPLEPULL, MFL_GRAPPLERESET, MFL_ONGROUND, MFL_SWIMMING, MFL_TELEPORTED, MFL_WALK,
    MFL_WATERJUMP, MOVERESULT_BLOCKEDBYAVOIDSPOT, MOVERESULT_MOVEMENTVIEW,
    MOVERESULT_MOVEMENTVIEWSET, MOVERESULT_MOVEMENTWEAPON, MOVERESULT_ONTOPOFOBSTACLE,
    MOVERESULT_ONTOPOF_ELEVATOR, MOVERESULT_ONTOPOF_FUNCBOB, MOVERESULT_SWIMVIEW,
    MOVERESULT_WAITING, MOVE_CROUCH, MOVE_JUMP, RESULTTYPE_ELEVATORUP, RESULTTYPE_INSOLIDAREA,
    RESULTTYPE_WAITFORFUNCBOBBING,
};
use crate::BotLib;

use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::common_fns::Com_Memset;
use mp_game::q_shared::Q_stricmp;
use mp_qshared::common::mp::botlib::aas_clientmove_s::aas_clientmove_t;
use mp_qshared::common::mp::botlib::aas_entityinfo_s::aas_entityinfo_t;
use mp_qshared::common::mp::botlib::aas_stop_event::{
    SE_ENTERLAVA, SE_ENTERSLIME, SE_ENTERWATER, SE_HITGROUND, SE_HITGROUNDDAMAGE,
};
use mp_qshared::common::mp::botlib::aas_trace_s::aas_trace_t;
use mp_qshared::common::mp::botlib::bot_avoidspot_s::{bot_avoidspot_s, bot_avoidspot_t};
use mp_qshared::common::mp::botlib::bot_initmove_s::bot_initmove_t;
use mp_qshared::common::mp::botlib::bot_moveresult_s::bot_moveresult_t;
use mp_qshared::common::mp::botlib::botlib_error::BLERR_NOERROR;
use mp_qshared::common::mp::botlib::bsp_trace_s::bsp_trace_t;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL, PRT_MESSAGE};
use mp_qshared::common::mp::botlib::travel_flags::{TFL_DONOTENTER, TFL_JUMPPAD};
use mp_qshared::common::mp::qcommon::bot_goal::bot_goal_t;
use mp_qshared::shared::limits::MAX_MODELS;
use mp_qshared::shared::limits::{ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_CLIENTS};
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vectoangles,
    DistanceSquared, VectorLength, VectorLengthSquared, VectorNormalize, VectorNormalize2,
    VectorSet, PITCH,
};
use mp_qshared::shared::surface_flags::{
    CONTENTS_BODY, CONTENTS_LAVA, CONTENTS_PLAYERCLIP, CONTENTS_SLIME, CONTENTS_SOLID,
    CONTENTS_WATER,
};
use mp_qshared::shared::{qfalse, qtrue, vec2_t, vec3_t};

use crate::be_aas_bspq3_fns::{
    AAS_BSPModelMinsMaxsOrigin, AAS_NextBSPEntity, AAS_PointContents, AAS_Trace,
    AAS_ValueForBSPEpairKey,
};
use crate::be_aas_entity::{
    AAS_EntityInfo, AAS_EntityModelNum, AAS_EntityModelindex, AAS_EntityType, AAS_NextEntity,
    AAS_OriginOfMoverWithModelNum,
};
use crate::be_aas_main::{AAS_ProjectPointOntoVector, AAS_Time};
use crate::be_aas_move::{
    AAS_AgainstLadder, AAS_HorizontalVelocityForJump, AAS_JumpReachRunStart, AAS_OnGround,
    AAS_PredictClientMovement, AAS_Swimming,
};
use crate::be_aas_reach_fns::{AAS_AreaDoNotEnter, AAS_AreaJumpPad, AAS_AreaReachability};
use crate::be_aas_route_fns::{
    AAS_AreaContentsTravelFlags, AAS_AreaTravelTimeToGoalArea, AAS_NextAreaReachability,
    AAS_NextModelReachability, AAS_ReachabilityFromNum, AAS_TravelFlagForType,
};
use crate::be_aas_sample_fns::{
    AAS_AreaPresenceType, AAS_PointAreaNum, AAS_PresenceTypeBoundingBox, AAS_TraceAreas,
    AAS_TraceClientBBox,
};
use crate::be_ea_fns::{
    EA_Attack, EA_Command, EA_Crouch, EA_DelayedJump, EA_Jump, EA_Move, EA_MoveForward, EA_MoveUp,
    EA_SelectWeapon, EA_View, EA_Walk,
};
use crate::l_libvar_fns::LibVar;
use crate::l_memory_fns::{FreeMemory, GetClearedMemory};

/// Raven `BotMoveStateFromHandle`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:151-164`
pub fn BotMoveStateFromHandle(bot: &mut BotLib, handle: c_int) -> *mut bot_movestate_t {
    unsafe {
        if handle <= 0 || handle > MAX_CLIENTS as c_int {
            (bot.botimport.Print.unwrap())(
                PRT_FATAL,
                c"move state handle %d out of range\n".as_ptr() as *mut c_char,
                handle,
            );
            return core::ptr::null_mut();
        }
        if bot.botmovestates[handle as usize].is_null() {
            (bot.botimport.Print.unwrap())(
                PRT_FATAL,
                c"invalid move state %d\n".as_ptr() as *mut c_char,
                handle,
            );
            return core::ptr::null_mut();
        }
        bot.botmovestates[handle as usize]
    }
}

/// Raven `AngleDiff`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:203-217`
pub fn AngleDiff(ang1: f32, ang2: f32) -> f32 {
    let mut diff: f32;

    diff = ang1 - ang2;
    if ang1 > ang2 {
        if diff > 180.0 {
            diff -= 360.0;
        }
    } else {
        if diff < -180.0 {
            diff += 360.0;
        }
    }
    diff
}

/// Raven `BotAddToTarget`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:805-824`
pub fn BotAddToTarget(
    start: vec3_t,
    end: vec3_t,
    maxdist: f32,
    dist: *mut f32,
    target: *mut vec3_t,
) -> c_int {
    unsafe {
        let mut dir: vec3_t = [0.0; 3];
        let curdist: f32;

        _VectorSubtract(end, start, &mut dir);
        curdist = VectorNormalize(&mut dir);
        if *dist + curdist < maxdist {
            _VectorCopy(end, &mut *target);
            *dist += curdist;
            qfalse
        } else {
            _VectorMA(start, maxdist - *dist, dir, &mut *target);
            *dist = maxdist;
            qtrue
        }
    }
}

/// Raven `Intersection`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1236-1258`
pub fn Intersection(p1: vec2_t, p2: vec2_t, p3: vec2_t, p4: vec2_t, mut out: vec2_t) -> c_int {
    let (x1, dx1, dy1, x2, dx2, dy2, d);

    dx1 = p2[0] - p1[0];
    dy1 = p2[1] - p1[1];
    dx2 = p4[0] - p3[0];
    dy2 = p4[1] - p3[1];

    d = dy1 * dx2 - dx1 * dy2;
    if d != 0.0 {
        x1 = p1[1] * dx1 - p1[0] * dy1;
        x2 = p3[1] * dx2 - p3[0] * dy2;
        out[0] = ((dx1 * x2 - dx2 * x1) / d) as c_int as f32;
        out[1] = ((dy1 * x2 - dy2 * x1) / d) as c_int as f32;
        qtrue
    } else {
        qfalse
    }
}

/// Raven `BotClearMoveResult`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1313-1321`
pub fn BotClearMoveResult(moveresult: *mut bot_moveresult_t) {
    unsafe {
        (*moveresult).failure = qfalse;
        (*moveresult).r#type = 0;
        (*moveresult).blocked = qfalse;
        (*moveresult).blockentity = 0;
        (*moveresult).traveltype = 0;
        (*moveresult).flags = 0;
    }
}

/// Raven `BotAirControl`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1675-1705`
pub fn BotAirControl(
    bot: &mut BotLib,
    origin: vec3_t,
    velocity: vec3_t,
    goal: vec3_t,
    dir: *mut vec3_t,
    speed: *mut f32,
) -> c_int {
    unsafe {
        let mut org: vec3_t = [0.0; 3];
        let mut vel: vec3_t = [0.0; 3];
        let mut dist: f32;
        let mut i: c_int;

        _VectorCopy(origin, &mut org);
        _VectorScale(velocity, 0.1, &mut vel);
        i = 0;
        while i < 50 {
            vel[2] -= (*bot.sv_gravity).value * 0.01;
            //if going down and next position would be below the goal
            if vel[2] < 0.0 && org[2] + vel[2] < goal[2] {
                _VectorScale(vel, (goal[2] - org[2]) / vel[2], &mut vel);
                _VectorAdd(org, vel, &mut org);
                _VectorSubtract(goal, org, &mut *dir);
                dist = VectorNormalize(&mut *dir);
                if dist > 32.0 {
                    dist = 32.0;
                }
                *speed = 400.0 - (400.0 - 13.0 * dist);
                return qtrue;
            } else {
                _VectorAdd(org, vel, &mut org);
            }
            i += 1;
        }
        VectorSet(&mut *dir, 0.0, 0.0, 0.0);
        *speed = 400.0;
        qfalse
    }
}

/// Raven `BotReachabilityTime`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2949-2974`
pub fn BotReachabilityTime(bot: &mut BotLib, reach: *mut aas_reachability_t) -> c_int {
    unsafe {
        match (*reach).traveltype & TRAVELTYPE_MASK {
            TRAVEL_WALK => 5,
            TRAVEL_CROUCH => 5,
            TRAVEL_BARRIERJUMP => 5,
            TRAVEL_LADDER => 6,
            TRAVEL_WALKOFFLEDGE => 5,
            TRAVEL_JUMP => 5,
            TRAVEL_SWIM => 5,
            TRAVEL_WATERJUMP => 5,
            TRAVEL_TELEPORT => 5,
            TRAVEL_ELEVATOR => 10,
            TRAVEL_GRAPPLEHOOK => 8,
            TRAVEL_ROCKETJUMP => 6,
            TRAVEL_BFGJUMP => 6,
            TRAVEL_JUMPPAD => 10,
            TRAVEL_FUNCBOB => 10,
            _ => {
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"travel type %d not implemented yet\n".as_ptr() as *mut c_char,
                    (*reach).traveltype,
                );
                8
            }
        }
    }
}

/// Raven `BotFreeMoveState`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:130-144`
pub fn BotFreeMoveState(bot: &mut BotLib, handle: c_int) {
    unsafe {
        if handle <= 0 || handle > MAX_CLIENTS as c_int {
            (bot.botimport.Print.unwrap())(
                PRT_FATAL,
                c"move state handle %d out of range\n".as_ptr() as *mut c_char,
                handle,
            );
            return;
        }
        if bot.botmovestates[handle as usize].is_null() {
            (bot.botimport.Print.unwrap())(
                PRT_FATAL,
                c"invalid move state %d\n".as_ptr() as *mut c_char,
                handle,
            );
            return;
        }
        let p = bot.botmovestates[handle as usize];
        FreeMemory(bot, p as *mut ());
        bot.botmovestates[handle as usize] = core::ptr::null_mut();
    }
}

/// Raven `BotInitMoveState`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:171-196`
pub fn BotInitMoveState(bot: &mut BotLib, handle: c_int, initmove: *mut bot_initmove_t) {
    unsafe {
        let ms: *mut bot_movestate_t;

        ms = BotMoveStateFromHandle(bot, handle);
        if ms.is_null() {
            return;
        }
        _VectorCopy((*initmove).origin, &mut (*ms).origin);
        _VectorCopy((*initmove).velocity, &mut (*ms).velocity);
        _VectorCopy((*initmove).viewoffset, &mut (*ms).viewoffset);
        (*ms).entitynum = (*initmove).entitynum;
        (*ms).client = (*initmove).client;
        (*ms).thinktime = (*initmove).thinktime;
        (*ms).presencetype = (*initmove).presencetype;
        _VectorCopy((*initmove).viewangles, &mut (*ms).viewangles);
        //
        (*ms).moveflags &= !MFL_ONGROUND;
        if (*initmove).or_moveflags & MFL_ONGROUND != 0 {
            (*ms).moveflags |= MFL_ONGROUND;
        }
        (*ms).moveflags &= !MFL_TELEPORTED;
        if (*initmove).or_moveflags & MFL_TELEPORTED != 0 {
            (*ms).moveflags |= MFL_TELEPORTED;
        }
        (*ms).moveflags &= !MFL_WATERJUMP;
        if (*initmove).or_moveflags & MFL_WATERJUMP != 0 {
            (*ms).moveflags |= MFL_WATERJUMP;
        }
        (*ms).moveflags &= !MFL_WALK;
        if (*initmove).or_moveflags & MFL_WALK != 0 {
            (*ms).moveflags |= MFL_WALK;
        }
        (*ms).moveflags &= !MFL_GRAPPLEPULL;
        if (*initmove).or_moveflags & MFL_GRAPPLEPULL != 0 {
            (*ms).moveflags |= MFL_GRAPPLEPULL;
        }
    }
}

/// Raven `BotOnMover`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:425-464`
pub fn BotOnMover(
    bot: &mut BotLib,
    origin: vec3_t,
    entnum: c_int,
    reach: *mut aas_reachability_t,
) -> c_int {
    unsafe {
        let mut i: c_int;
        let modelnum: c_int;
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut modelorigin: vec3_t = [0.0; 3];
        let mut org: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0, 0.0, 0.0];
        let boxmins: vec3_t = [-16.0, -16.0, -8.0];
        let boxmaxs: vec3_t = [16.0, 16.0, 8.0];
        let trace: bsp_trace_t;

        modelnum = (*reach).facenum & 0x0000FFFF;
        //get some bsp model info
        AAS_BSPModelMinsMaxsOrigin(bot, modelnum, angles, &mut mins, &mut maxs, &mut [0.0; 3]);
        //
        if AAS_OriginOfMoverWithModelNum(bot, modelnum, &mut modelorigin) == 0 {
            (bot.botimport.Print.unwrap())(
                PRT_MESSAGE,
                c"no entity with model %d\n".as_ptr() as *mut c_char,
                modelnum,
            );
            return qfalse;
        }
        //
        i = 0;
        while i < 2 {
            if origin[i as usize] > modelorigin[i as usize] + maxs[i as usize] + 16.0 {
                return qfalse;
            }
            if origin[i as usize] < modelorigin[i as usize] + mins[i as usize] - 16.0 {
                return qfalse;
            }
            i += 1;
        }
        //
        _VectorCopy(origin, &mut org);
        org[2] += 24.0;
        _VectorCopy(origin, &mut end);
        end[2] -= 48.0;
        //
        trace = AAS_Trace(
            bot,
            org,
            boxmins,
            boxmaxs,
            end,
            entnum,
            CONTENTS_SOLID | CONTENTS_PLAYERCLIP,
        );
        if trace.startsolid == 0 && trace.allsolid == 0 {
            //NOTE: the reachability face number is the model number of the elevator
            if trace.ent != ENTITYNUM_NONE && AAS_EntityModelNum(bot, trace.ent) == modelnum {
                return qtrue;
            }
        }
        qfalse
    }
}

/// Raven `MoverDown`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:471-489`
pub fn MoverDown(bot: &mut BotLib, reach: *mut aas_reachability_t) -> c_int {
    unsafe {
        let modelnum: c_int;
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut origin: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0, 0.0, 0.0];

        modelnum = (*reach).facenum & 0x0000FFFF;
        //get some bsp model info
        AAS_BSPModelMinsMaxsOrigin(bot, modelnum, angles, &mut mins, &mut maxs, &mut origin);
        //
        if AAS_OriginOfMoverWithModelNum(bot, modelnum, &mut origin) == 0 {
            (bot.botimport.Print.unwrap())(
                PRT_MESSAGE,
                c"no entity with model %d\n".as_ptr() as *mut c_char,
                modelnum,
            );
            return qfalse;
        }
        //if the top of the plat is below the reachability start point
        if origin[2] + maxs[2] < (*reach).start[2] {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `BotOnTopOfEntity`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:532-545`
pub fn BotOnTopOfEntity(bot: &mut BotLib, ms: *mut bot_movestate_t) -> c_int {
    unsafe {
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let up: vec3_t = [0.0, 0.0, 1.0];
        let trace: bsp_trace_t;

        AAS_PresenceTypeBoundingBox(bot, (*ms).presencetype, &mut mins, &mut maxs);
        _VectorMA((*ms).origin, -3.0, up, &mut end);
        trace = AAS_Trace(
            bot,
            (*ms).origin,
            mins,
            maxs,
            end,
            (*ms).entitynum,
            CONTENTS_SOLID | CONTENTS_PLAYERCLIP,
        );
        if trace.startsolid == 0 && (trace.ent != ENTITYNUM_WORLD && trace.ent != ENTITYNUM_NONE) {
            return trace.ent;
        }
        -1
    }
}

/// Raven `BotAddToAvoidReach`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:566-591`
pub fn BotAddToAvoidReach(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    number: c_int,
    avoidtime: f32,
) {
    unsafe {
        let mut i: usize;

        i = 0;
        while i < MAX_AVOIDREACH {
            if (*ms).avoidreach[i] == number {
                if (*ms).avoidreachtimes[i] > AAS_Time(bot) {
                    (*ms).avoidreachtries[i] += 1;
                } else {
                    (*ms).avoidreachtries[i] = 1;
                }
                (*ms).avoidreachtimes[i] = AAS_Time(bot) + avoidtime;
                return;
            }
            i += 1;
        }
        //add the reachability to the reachabilities to avoid for a while
        i = 0;
        while i < MAX_AVOIDREACH {
            if (*ms).avoidreachtimes[i] < AAS_Time(bot) {
                (*ms).avoidreach[i] = number;
                (*ms).avoidreachtimes[i] = AAS_Time(bot) + avoidtime;
                (*ms).avoidreachtries[i] = 1;
                return;
            }
            i += 1;
        }
    }
}

/// Raven `DistanceFromLineSquared`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:598-617`
pub fn DistanceFromLineSquared(p: vec3_t, lp1: vec3_t, lp2: vec3_t) -> f32 {
    let mut proj: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut j: usize;

    AAS_ProjectPointOntoVector(p, lp1, lp2, &mut proj);
    j = 0;
    while j < 3 {
        if (proj[j] > lp1[j] && proj[j] > lp2[j]) || (proj[j] < lp1[j] && proj[j] < lp2[j]) {
            break;
        }
        j += 1;
    }
    if j < 3 {
        if (proj[j] - lp1[j]).abs() < (proj[j] - lp2[j]).abs() {
            _VectorSubtract(p, lp1, &mut dir);
        } else {
            _VectorSubtract(p, lp2, &mut dir);
        }
        return VectorLengthSquared(dir);
    }
    _VectorSubtract(p, proj, &mut dir);
    VectorLengthSquared(dir)
}

/// Raven `BotAddAvoidSpot`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:702-720`
pub fn BotAddAvoidSpot(
    bot: &mut BotLib,
    movestate: c_int,
    origin: vec3_t,
    radius: f32,
    r#type: c_int,
) {
    unsafe {
        let ms: *mut bot_movestate_t;

        ms = BotMoveStateFromHandle(bot, movestate);
        if ms.is_null() {
            return;
        }
        if r#type == AVOID_CLEAR {
            (*ms).numavoidspots = 0;
            return;
        }

        if (*ms).numavoidspots >= MAX_AVOIDSPOTS as c_int {
            return;
        }
        _VectorCopy(
            origin,
            &mut (*ms).avoidspots[(*ms).numavoidspots as usize].origin,
        );
        (*ms).avoidspots[(*ms).numavoidspots as usize].radius = radius;
        (*ms).avoidspots[(*ms).numavoidspots as usize].r#type = r#type;
        (*ms).numavoidspots += 1;
    }
}

/// Raven `BotVisible`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:881-888`
pub fn BotVisible(bot: &mut BotLib, ent: c_int, eye: vec3_t, target: vec3_t) -> c_int {
    unsafe {
        let trace: bsp_trace_t;

        trace = AAS_Trace(
            bot,
            eye,
            [0.0; 3],
            [0.0; 3],
            target,
            ent,
            CONTENTS_SOLID | CONTENTS_PLAYERCLIP,
        );
        if trace.fraction >= 1.0 {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `MoverBottomCenter`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:958-976`
pub fn MoverBottomCenter(
    bot: &mut BotLib,
    reach: *mut aas_reachability_t,
    bottomcenter: *mut vec3_t,
) {
    unsafe {
        let modelnum: c_int;
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut origin: vec3_t = [0.0; 3];
        let mut mids: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0, 0.0, 0.0];

        modelnum = (*reach).facenum & 0x0000FFFF;
        //get some bsp model info
        AAS_BSPModelMinsMaxsOrigin(bot, modelnum, angles, &mut mins, &mut maxs, &mut origin);
        //
        if AAS_OriginOfMoverWithModelNum(bot, modelnum, &mut origin) == 0 {
            (bot.botimport.Print.unwrap())(
                PRT_MESSAGE,
                c"no entity with model %d\n".as_ptr() as *mut c_char,
                modelnum,
            );
        }
        //get a point just above the plat in the bottom position
        _VectorAdd(mins, maxs, &mut mids);
        _VectorMA(origin, 0.5, mids, &mut *bottomcenter);
        (*bottomcenter)[2] = (*reach).start[2];
    }
}

/// Raven `BotSwimInDirection`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1082-1090`
pub fn BotSwimInDirection(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    dir: vec3_t,
    speed: f32,
    r#type: c_int,
) -> c_int {
    unsafe {
        let mut normdir: vec3_t = [0.0; 3];

        _VectorCopy(dir, &mut normdir);
        VectorNormalize(&mut normdir);
        EA_Move(bot, (*ms).client, normdir, speed);
        qtrue
    }
}

/// Raven `BotFinishTravel_Walk`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1383-1414`
pub fn BotFinishTravel_Walk(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut hordir: vec3_t = [0.0; 3];
        let dist: f32;
        let speed: f32;
        // §19: Raven's `result` is stack-uninitialized; zero it before BotClearMoveResult.
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //go straight to the reachability end
        hordir[0] = (*reach).end[0] - (*ms).origin[0];
        hordir[1] = (*reach).end[1] - (*ms).origin[1];
        hordir[2] = 0.0;
        let mut dist = VectorNormalize(&mut hordir);
        //
        if dist > 100.0 {
            dist = 100.0;
        }
        speed = 400.0 - (400.0 - 3.0 * dist);
        //
        EA_Move(bot, (*ms).client, hordir, speed);
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotTravel_WaterJump`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1541-1568`
pub fn BotTravel_WaterJump(
    common: &mut Common,
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut dir: vec3_t = [0.0; 3];
        let mut hordir: vec3_t = [0.0; 3];
        let dist: f32;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //swim straight to reachability end
        _VectorSubtract((*reach).end, (*ms).origin, &mut dir);
        _VectorCopy(dir, &mut hordir);
        hordir[2] = 0.0;
        dir[2] += 15.0 + common.qrand.crandom() * 40.0;
        VectorNormalize(&mut dir);
        let dist = VectorNormalize(&mut hordir);
        //elemantary actions
        EA_MoveForward(bot, (*ms).client);
        //move up if close to the actual out of water jump spot
        if dist < 40.0 {
            EA_MoveUp(bot, (*ms).client);
        }
        //set the ideal view angles
        vectoangles(dir, &mut result.ideal_viewangles);
        result.flags |= MOVERESULT_MOVEMENTVIEW;
        //
        _VectorCopy(dir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotFinishTravel_WaterJump`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1575-1605`
pub fn BotFinishTravel_WaterJump(
    common: &mut Common,
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut dir: vec3_t = [0.0; 3];
        let mut pnt: vec3_t = [0.0; 3];
        let dist: f32;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //if waterjumping there's nothing to do
        if (*ms).moveflags & MFL_WATERJUMP != 0 {
            return result;
        }
        //if not touching any water anymore don't do anything
        _VectorCopy((*ms).origin, &mut pnt);
        pnt[2] -= 32.0; //extra for q2dm4 near red armor/mega health
        if AAS_PointContents(bot, pnt) & (CONTENTS_LAVA | CONTENTS_SLIME | CONTENTS_WATER) == 0 {
            return result;
        }
        //swim straight to reachability end
        _VectorSubtract((*reach).end, (*ms).origin, &mut dir);
        dir[0] += common.qrand.crandom() * 10.0;
        dir[1] += common.qrand.crandom() * 10.0;
        dir[2] += 70.0 + common.qrand.crandom() * 10.0;
        let dist = VectorNormalize(&mut dir);
        //elemantary actions
        EA_Move(bot, (*ms).client, dir, 400.0);
        //set the ideal view angles
        vectoangles(dir, &mut result.ideal_viewangles);
        result.flags |= MOVERESULT_MOVEMENTVIEW;
        //
        _VectorCopy(dir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotFinishTravel_Jump`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1943-1971`
pub fn BotFinishTravel_Jump(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut hordir: vec3_t = [0.0; 3];
        let mut hordir2: vec3_t = [0.0; 3];
        let speed: f32;
        let dist: f32;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //if not jumped yet
        if (*ms).jumpreach == 0 {
            return result;
        }
        //go straight to the reachability end
        hordir[0] = (*reach).end[0] - (*ms).origin[0];
        hordir[1] = (*reach).end[1] - (*ms).origin[1];
        hordir[2] = 0.0;
        let dist = VectorNormalize(&mut hordir);
        //
        hordir2[0] = (*reach).end[0] - (*reach).start[0];
        hordir2[1] = (*reach).end[1] - (*reach).start[1];
        hordir2[2] = 0.0;
        VectorNormalize(&mut hordir2);
        //
        if _DotProduct(hordir, hordir2) < -0.5 && dist < 24.0 {
            return result;
        }
        //always use max speed when traveling through the air
        let speed = 800.0;
        //
        EA_Move(bot, (*ms).client, hordir, speed);
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotTravel_Ladder`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1978-2027`
pub fn BotTravel_Ladder(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut dir: vec3_t = [0.0; 3];
        let mut viewdir: vec3_t = [0.0; 3];
        let origin: vec3_t = [0.0, 0.0, 0.0];
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //
        {
            _VectorSubtract((*reach).end, (*ms).origin, &mut dir);
            VectorNormalize(&mut dir);
            //set the ideal view angles, facing the ladder up or down
            viewdir[0] = dir[0];
            viewdir[1] = dir[1];
            viewdir[2] = 3.0 * dir[2];
            vectoangles(viewdir, &mut result.ideal_viewangles);
            //elemantary action
            EA_Move(bot, (*ms).client, origin, 0.0);
            EA_MoveForward(bot, (*ms).client);
            //set movement view flag so the AI can see the view is focussed
            result.flags |= MOVERESULT_MOVEMENTVIEW;
        }
        //save the movement direction
        _VectorCopy(dir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotFuncBobStartEnd`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2246-2297`
pub fn BotFuncBobStartEnd(
    bot: &mut BotLib,
    reach: *mut aas_reachability_t,
    start: *mut vec3_t,
    end: *mut vec3_t,
    origin: *mut vec3_t,
) {
    unsafe {
        let spawnflags: c_int;
        let modelnum: c_int;
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut mid: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0, 0.0, 0.0];
        let mut num0: c_int;
        let mut num1: c_int;

        modelnum = (*reach).facenum & 0x0000FFFF;
        if AAS_OriginOfMoverWithModelNum(bot, modelnum, &mut *origin) == 0 {
            (bot.botimport.Print.unwrap())(
                PRT_MESSAGE,
                c"BotFuncBobStartEnd: no entity with model %d\n".as_ptr() as *mut c_char,
                modelnum,
            );
            VectorSet(&mut *start, 0.0, 0.0, 0.0);
            VectorSet(&mut *end, 0.0, 0.0, 0.0);
            return;
        }
        AAS_BSPModelMinsMaxsOrigin(bot, modelnum, angles, &mut mins, &mut maxs, &mut [0.0; 3]);
        _VectorAdd(mins, maxs, &mut mid);
        _VectorScale(mid, 0.5, &mut mid);
        _VectorCopy(mid, &mut *start);
        _VectorCopy(mid, &mut *end);
        spawnflags = (*reach).facenum >> 16;
        num0 = (*reach).edgenum >> 16;
        if num0 > 0x00007FFF {
            num0 |= 0xFFFF0000u32 as c_int;
        }
        num1 = (*reach).edgenum & 0x0000FFFF;
        if num1 > 0x00007FFF {
            num1 |= 0xFFFF0000u32 as c_int;
        }
        if spawnflags & 1 != 0 {
            (*start)[0] = num0 as f32;
            (*end)[0] = num1 as f32;
            //
            (*origin)[0] += mid[0];
            (*origin)[1] = mid[1];
            (*origin)[2] = mid[2];
        } else if spawnflags & 2 != 0 {
            (*start)[1] = num0 as f32;
            (*end)[1] = num1 as f32;
            //
            (*origin)[0] = mid[0];
            (*origin)[1] += mid[1];
            (*origin)[2] = mid[2];
        } else {
            (*start)[2] = num0 as f32;
            (*end)[2] = num1 as f32;
            //
            (*origin)[0] = mid[0];
            (*origin)[1] = mid[1];
            (*origin)[2] += mid[2];
        }
    }
}

/// Raven `BotTravel_RocketJump`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2727-2785`
pub fn BotTravel_RocketJump(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut hordir: vec3_t = [0.0; 3];
        let mut dist: f32;
        let mut speed: f32;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //
        hordir[0] = (*reach).start[0] - (*ms).origin[0];
        hordir[1] = (*reach).start[1] - (*ms).origin[1];
        hordir[2] = 0.0;
        //
        dist = VectorNormalize(&mut hordir);
        //look in the movement direction
        vectoangles(hordir, &mut result.ideal_viewangles);
        //look straight down
        result.ideal_viewangles[PITCH as usize] = 90.0;
        //
        if dist < 5.0
            && AngleDiff(result.ideal_viewangles[0], (*ms).viewangles[0]).abs() < 5.0
            && AngleDiff(result.ideal_viewangles[1], (*ms).viewangles[1]).abs() < 5.0
        {
            hordir[0] = (*reach).end[0] - (*ms).origin[0];
            hordir[1] = (*reach).end[1] - (*ms).origin[1];
            hordir[2] = 0.0;
            VectorNormalize(&mut hordir);
            //elemantary action jump
            EA_Jump(bot, (*ms).client);
            EA_Attack(bot, (*ms).client);
            EA_Move(bot, (*ms).client, hordir, 800.0);
            //
            (*ms).jumpreach = (*ms).lastreachnum;
        } else {
            if dist > 80.0 {
                dist = 80.0;
            }
            speed = 400.0 - (400.0 - 5.0 * dist);
            EA_Move(bot, (*ms).client, hordir, speed);
        }
        //look in the movement direction
        vectoangles(hordir, &mut result.ideal_viewangles);
        //look straight down
        result.ideal_viewangles[PITCH as usize] = 90.0;
        //set the view angles directly
        EA_View(bot, (*ms).client, result.ideal_viewangles);
        //view is important for the movment
        result.flags |= MOVERESULT_MOVEMENTVIEWSET;
        //select the rocket launcher
        EA_SelectWeapon(
            bot,
            (*ms).client,
            (*bot.weapindex_rocketlauncher).value as c_int,
        );
        //weapon is used for movement
        result.weapon = (*bot.weapindex_rocketlauncher).value as c_int;
        result.flags |= MOVERESULT_MOVEMENTWEAPON;
        //
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotTravel_BFGJump`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2792-2846`
pub fn BotTravel_BFGJump(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut hordir: vec3_t = [0.0; 3];
        let mut dist: f32;
        let mut speed: f32;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //
        hordir[0] = (*reach).start[0] - (*ms).origin[0];
        hordir[1] = (*reach).start[1] - (*ms).origin[1];
        hordir[2] = 0.0;
        //
        dist = VectorNormalize(&mut hordir);
        //
        if dist < 5.0
            && AngleDiff(result.ideal_viewangles[0], (*ms).viewangles[0]).abs() < 5.0
            && AngleDiff(result.ideal_viewangles[1], (*ms).viewangles[1]).abs() < 5.0
        {
            hordir[0] = (*reach).end[0] - (*ms).origin[0];
            hordir[1] = (*reach).end[1] - (*ms).origin[1];
            hordir[2] = 0.0;
            VectorNormalize(&mut hordir);
            //elemantary action jump
            EA_Jump(bot, (*ms).client);
            EA_Attack(bot, (*ms).client);
            EA_Move(bot, (*ms).client, hordir, 800.0);
            //
            (*ms).jumpreach = (*ms).lastreachnum;
        } else {
            if dist > 80.0 {
                dist = 80.0;
            }
            speed = 400.0 - (400.0 - 5.0 * dist);
            EA_Move(bot, (*ms).client, hordir, speed);
        }
        //look in the movement direction
        vectoangles(hordir, &mut result.ideal_viewangles);
        //look straight down
        result.ideal_viewangles[PITCH as usize] = 90.0;
        //set the view angles directly
        EA_View(bot, (*ms).client, result.ideal_viewangles);
        //view is important for the movment
        result.flags |= MOVERESULT_MOVEMENTVIEWSET;
        //select the rocket launcher
        EA_SelectWeapon(bot, (*ms).client, (*bot.weapindex_bfg10k).value as c_int);
        //weapon is used for movement
        result.weapon = (*bot.weapindex_bfg10k).value as c_int;
        result.flags |= MOVERESULT_MOVEMENTWEAPON;
        //
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotFinishTravel_WeaponJump`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2853-2886`
pub fn BotFinishTravel_WeaponJump(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut hordir: vec3_t = [0.0; 3];
        let mut speed: f32 = 0.0;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //if not jumped yet
        if (*ms).jumpreach == 0 {
            return result;
        }
        //
        if BotAirControl(
            bot,
            (*ms).origin,
            (*ms).velocity,
            (*reach).end,
            &mut hordir,
            &mut speed,
        ) == 0
        {
            //go straight to the reachability end
            _VectorSubtract((*reach).end, (*ms).origin, &mut hordir);
            hordir[2] = 0.0;
            VectorNormalize(&mut hordir);
            speed = 400.0;
        }
        //
        EA_Move(bot, (*ms).client, hordir, speed);
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotResetAvoidReach`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:3498-3507`
pub fn BotResetAvoidReach(bot: &mut BotLib, movestate: c_int) {
    unsafe {
        let ms: *mut bot_movestate_t;

        ms = BotMoveStateFromHandle(bot, movestate);
        if ms.is_null() {
            return;
        }
        Com_Memset(
            (*ms).avoidreach.as_mut_ptr() as *mut (),
            0,
            MAX_AVOIDREACH * core::mem::size_of::<c_int>(),
        );
        Com_Memset(
            (*ms).avoidreachtimes.as_mut_ptr() as *mut (),
            0,
            MAX_AVOIDREACH * core::mem::size_of::<f32>(),
        );
        Com_Memset(
            (*ms).avoidreachtries.as_mut_ptr() as *mut (),
            0,
            MAX_AVOIDREACH * core::mem::size_of::<c_int>(),
        );
    }
}

/// Raven `BotResetLastAvoidReach`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:3514-3537`
pub fn BotResetLastAvoidReach(bot: &mut BotLib, movestate: c_int) {
    unsafe {
        let mut i: usize;
        let mut latest: usize;
        let mut latesttime: f32;
        let ms: *mut bot_movestate_t;

        ms = BotMoveStateFromHandle(bot, movestate);
        if ms.is_null() {
            return;
        }
        latesttime = 0.0;
        latest = 0;
        i = 0;
        while i < MAX_AVOIDREACH {
            if (*ms).avoidreachtimes[i] > latesttime {
                latesttime = (*ms).avoidreachtimes[i];
                latest = i;
            }
            i += 1;
        }
        if latesttime != 0.0 {
            (*ms).avoidreachtimes[latest] = 0.0;
            // §19: Raven reads `avoidreachtries[i]` with loop-terminal
            // `i == MAX_AVOIDREACH` (OOB, garbage guard); defined pick: `latest`.
            if (*ms).avoidreachtries[latest] > 0 {
                (*ms).avoidreachtries[latest] -= 1;
            }
        }
    }
}

/// Raven `BotResetMoveState`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:3544-3551`
pub fn BotResetMoveState(bot: &mut BotLib, movestate: c_int) {
    unsafe {
        let ms: *mut bot_movestate_t;

        ms = BotMoveStateFromHandle(bot, movestate);
        if ms.is_null() {
            return;
        }
        Com_Memset(ms as *mut (), 0, core::mem::size_of::<bot_movestate_t>());
    }
}

/// Raven `BotShutdownMoveAI`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:3579-3591`
pub fn BotShutdownMoveAI(bot: &mut BotLib) {
    unsafe {
        let mut i: c_int;

        i = 1;
        while i <= MAX_CLIENTS as c_int {
            if !bot.botmovestates[i as usize].is_null() {
                let p = bot.botmovestates[i as usize];
                FreeMemory(bot, p as *mut ());
                bot.botmovestates[i as usize] = core::ptr::null_mut();
            }
            i += 1;
        }
    }
}

/// Raven `BotAllocMoveState`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:110-123`
pub fn BotAllocMoveState(bot: &mut BotLib) -> c_int {
    unsafe {
        let mut i: c_int;

        i = 1;
        while i <= MAX_CLIENTS as c_int {
            if bot.botmovestates[i as usize].is_null() {
                bot.botmovestates[i as usize] =
                    GetClearedMemory(bot, core::mem::size_of::<bot_movestate_t>() as c_ulong)
                        as *mut bot_movestate_s;
                return i;
            }
            i += 1;
        }
        0
    }
}

/// Raven `BotFuzzyPointReachabilityArea`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:224-277`
pub fn BotFuzzyPointReachabilityArea(bot: &mut BotLib, origin: vec3_t) -> c_int {
    unsafe {
        let mut firstareanum: c_int;
        let mut j: c_int;
        let mut x: c_int;
        let mut y: c_int;
        let mut z: c_int;
        let mut areas: [c_int; 10] = [0; 10];
        let mut numareas: c_int;
        let areanum: c_int;
        let mut bestareanum: c_int;
        let mut dist: f32;
        let mut bestdist: f32;
        let mut points: [vec3_t; 10] = [[0.0; 3]; 10];
        let mut v: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];

        firstareanum = 0;
        areanum = AAS_PointAreaNum(bot, origin);
        if areanum != 0 {
            firstareanum = areanum;
            if AAS_AreaReachability(bot, areanum) != 0 {
                return areanum;
            }
        }
        _VectorCopy(origin, &mut end);
        end[2] += 4.0;
        numareas = AAS_TraceAreas(
            bot,
            origin,
            end,
            areas.as_mut_ptr(),
            points.as_mut_ptr(),
            10,
        );
        j = 0;
        while j < numareas {
            if AAS_AreaReachability(bot, areas[j as usize]) != 0 {
                return areas[j as usize];
            }
            j += 1;
        }
        bestdist = 999999.0;
        bestareanum = 0;
        z = 1;
        while z >= -1 {
            x = 1;
            while x >= -1 {
                y = 1;
                while y >= -1 {
                    _VectorCopy(origin, &mut end);
                    end[0] += (x * 8) as f32;
                    end[1] += (y * 8) as f32;
                    end[2] += (z * 12) as f32;
                    numareas = AAS_TraceAreas(
                        bot,
                        origin,
                        end,
                        areas.as_mut_ptr(),
                        points.as_mut_ptr(),
                        10,
                    );
                    j = 0;
                    while j < numareas {
                        if AAS_AreaReachability(bot, areas[j as usize]) != 0 {
                            _VectorSubtract(points[j as usize], origin, &mut v);
                            dist = VectorLength(v);
                            if dist < bestdist {
                                bestareanum = areas[j as usize];
                                bestdist = dist;
                            }
                        }
                        if firstareanum == 0 {
                            firstareanum = areas[j as usize];
                        }
                        j += 1;
                    }
                    y -= 1;
                }
                x -= 1;
            }
            if bestareanum != 0 {
                return bestareanum;
            }
            z -= 1;
        }
        firstareanum
    }
}

/// Raven `BotSetBrushModelTypes`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:496-525`
pub fn BotSetBrushModelTypes(bot: &mut BotLib) {
    unsafe {
        let mut ent: c_int;
        let mut modelnum: c_int;
        let mut classname: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];
        let mut model: [c_char; MAX_EPAIRKEY as usize] = [0; MAX_EPAIRKEY as usize];

        Com_Memset(
            bot.modeltypes.as_mut_ptr() as *mut (),
            0,
            MAX_MODELS as usize * core::mem::size_of::<c_int>(),
        );
        //
        ent = AAS_NextBSPEntity(bot, 0);
        while ent != 0 {
            'cont: {
                if AAS_ValueForBSPEpairKey(
                    bot,
                    ent,
                    c"classname".as_ptr() as *mut c_char,
                    classname.as_mut_ptr(),
                    MAX_EPAIRKEY,
                ) == 0
                {
                    break 'cont;
                }
                if AAS_ValueForBSPEpairKey(
                    bot,
                    ent,
                    c"model".as_ptr() as *mut c_char,
                    model.as_mut_ptr(),
                    MAX_EPAIRKEY,
                ) == 0
                {
                    break 'cont;
                }
                if model[0] != 0 {
                    modelnum = libc::atoi(model.as_ptr().add(1));
                } else {
                    modelnum = 0;
                }

                if modelnum < 0 || modelnum > MAX_MODELS {
                    (bot.botimport.Print.unwrap())(
                        PRT_MESSAGE,
                        c"entity %s model number out of range\n".as_ptr() as *mut c_char,
                        classname.as_ptr(),
                    );
                    break 'cont;
                }

                if Q_stricmp(classname.as_ptr(), c"func_bobbing".as_ptr()) == 0 {
                    bot.modeltypes[modelnum as usize] = MODELTYPE_FUNC_BOB;
                } else if Q_stricmp(classname.as_ptr(), c"func_plat".as_ptr()) == 0 {
                    bot.modeltypes[modelnum as usize] = MODELTYPE_FUNC_PLAT;
                } else if Q_stricmp(classname.as_ptr(), c"func_door".as_ptr()) == 0 {
                    bot.modeltypes[modelnum as usize] = MODELTYPE_FUNC_DOOR;
                } else if Q_stricmp(classname.as_ptr(), c"func_static".as_ptr()) == 0 {
                    bot.modeltypes[modelnum as usize] = MODELTYPE_FUNC_STATIC;
                }
            }
            ent = AAS_NextBSPEntity(bot, ent);
        }
    }
}

/// Raven `BotValidTravel`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:552-559`
pub fn BotValidTravel(
    bot: &mut BotLib,
    origin: vec3_t,
    reach: *mut aas_reachability_t,
    travelflags: c_int,
) -> c_int {
    unsafe {
        //if the reachability uses an unwanted travel type
        if AAS_TravelFlagForType(bot, (*reach).traveltype) & !travelflags != 0 {
            return qfalse;
        }
        //don't go into areas with bad travel types
        if AAS_AreaContentsTravelFlags(bot, (*reach).areanum) & !travelflags != 0 {
            return qfalse;
        }
        qtrue
    }
}

/// Raven `BotAvoidSpots`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:636-695`
pub fn BotAvoidSpots(
    origin: vec3_t,
    reach: *mut aas_reachability_t,
    avoidspots: *mut bot_avoidspot_t,
    numavoidspots: c_int,
) -> c_int {
    unsafe {
        let checkbetween: c_int;
        let mut i: c_int;
        let mut r#type: c_int;
        let mut squareddist: f32;
        let mut squaredradius: f32;

        checkbetween = match (*reach).traveltype & TRAVELTYPE_MASK {
            TRAVEL_WALK => qtrue,
            TRAVEL_CROUCH => qtrue,
            TRAVEL_BARRIERJUMP => qtrue,
            TRAVEL_LADDER => qtrue,
            TRAVEL_WALKOFFLEDGE => qfalse,
            TRAVEL_JUMP => qfalse,
            TRAVEL_SWIM => qtrue,
            TRAVEL_WATERJUMP => qtrue,
            TRAVEL_TELEPORT => qfalse,
            TRAVEL_ELEVATOR => qfalse,
            TRAVEL_GRAPPLEHOOK => qfalse,
            TRAVEL_ROCKETJUMP => qfalse,
            TRAVEL_BFGJUMP => qfalse,
            TRAVEL_JUMPPAD => qfalse,
            TRAVEL_FUNCBOB => qfalse,
            _ => qtrue,
        };

        r#type = AVOID_CLEAR;
        i = 0;
        while i < numavoidspots {
            squaredradius =
                (*avoidspots.add(i as usize)).radius * (*avoidspots.add(i as usize)).radius;
            squareddist = DistanceFromLineSquared(
                (*avoidspots.add(i as usize)).origin,
                origin,
                (*reach).start,
            );
            // if moving towards the avoid spot
            if squareddist < squaredradius
                && DistanceSquared((*avoidspots.add(i as usize)).origin, origin) > squareddist
            {
                r#type = (*avoidspots.add(i as usize)).r#type;
            } else if checkbetween != 0 {
                squareddist = DistanceFromLineSquared(
                    (*avoidspots.add(i as usize)).origin,
                    (*reach).start,
                    (*reach).end,
                );
                // if moving towards the avoid spot
                if squareddist < squaredradius
                    && DistanceSquared((*avoidspots.add(i as usize)).origin, (*reach).start)
                        > squareddist
                {
                    r#type = (*avoidspots.add(i as usize)).r#type;
                }
            } else {
                DistanceSquared((*avoidspots.add(i as usize)).origin, (*reach).end);
                // if the reachability leads closer to the avoid spot
                if squareddist < squaredradius
                    && DistanceSquared((*avoidspots.add(i as usize)).origin, (*reach).start)
                        > squareddist
                {
                    r#type = (*avoidspots.add(i as usize)).r#type;
                }
            }
            if r#type == AVOID_ALWAYS {
                return r#type;
            }
            i += 1;
        }
        r#type
    }
}

/// Raven `BotCheckBlocked`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1265-1306`
pub fn BotCheckBlocked(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    dir: vec3_t,
    checkbottom: c_int,
    result: *mut bot_moveresult_t,
) {
    unsafe {
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let up: vec3_t = [0.0, 0.0, 1.0];
        let mut trace: bsp_trace_t;

        //test for entities obstructing the bot's path
        AAS_PresenceTypeBoundingBox(bot, (*ms).presencetype, &mut mins, &mut maxs);
        //
        if _DotProduct(dir, up).abs() < 0.7 {
            mins[2] += (*bot.sv_maxstep).value; //if the bot can step on
            maxs[2] -= 10.0; //a little lower to avoid low ceiling
        }
        _VectorMA((*ms).origin, 3.0, dir, &mut end);
        trace = AAS_Trace(
            bot,
            (*ms).origin,
            mins,
            maxs,
            end,
            (*ms).entitynum,
            CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_BODY,
        );
        //if not started in solid and not hitting the world entity
        if trace.startsolid == 0 && (trace.ent != ENTITYNUM_WORLD && trace.ent != ENTITYNUM_NONE) {
            (*result).blocked = qtrue;
            (*result).blockentity = trace.ent;
        }
        //if not in an area with reachability
        else if checkbottom != 0 && AAS_AreaReachability(bot, (*ms).areanum) == 0 {
            //check if the bot is standing on something
            AAS_PresenceTypeBoundingBox(bot, (*ms).presencetype, &mut mins, &mut maxs);
            _VectorMA((*ms).origin, -3.0, up, &mut end);
            trace = AAS_Trace(
                bot,
                (*ms).origin,
                mins,
                maxs,
                end,
                (*ms).entitynum,
                CONTENTS_SOLID | CONTENTS_PLAYERCLIP,
            );
            if trace.startsolid == 0
                && (trace.ent != ENTITYNUM_WORLD && trace.ent != ENTITYNUM_NONE)
            {
                (*result).blocked = qtrue;
                (*result).blockentity = trace.ent;
                (*result).flags |= MOVERESULT_ONTOPOFOBSTACLE;
            }
        }
    }
}

/// Raven `BotFinishTravel_Elevator`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2216-2239`
pub fn BotFinishTravel_Elevator(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut bottomcenter: vec3_t = [0.0; 3];
        let mut bottomdir: vec3_t = [0.0; 3];
        let mut topdir: vec3_t = [0.0; 3];
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //
        MoverBottomCenter(bot, reach, &mut bottomcenter);
        _VectorSubtract(bottomcenter, (*ms).origin, &mut bottomdir);
        //
        _VectorSubtract((*reach).end, (*ms).origin, &mut topdir);
        //
        if bottomdir[2].abs() < topdir[2].abs() {
            VectorNormalize(&mut bottomdir);
            EA_Move(bot, (*ms).client, bottomdir, 300.0);
        } else {
            VectorNormalize(&mut topdir);
            EA_Move(bot, (*ms).client, topdir, 300.0);
        }
        result
    }
}

/// Raven `BotFinishTravel_FuncBobbing`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2462-2507`
pub fn BotFinishTravel_FuncBobbing(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut bob_origin: vec3_t = [0.0; 3];
        let mut bob_start: vec3_t = [0.0; 3];
        let mut bob_end: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let mut hordir: vec3_t = [0.0; 3];
        let mut bottomcenter: vec3_t = [0.0; 3];
        let mut result: bot_moveresult_t = core::mem::zeroed();
        let mut dist: f32;
        let mut speed: f32;

        BotClearMoveResult(&mut result);
        //
        BotFuncBobStartEnd(bot, reach, &mut bob_start, &mut bob_end, &mut bob_origin);
        //
        _VectorSubtract(bob_origin, bob_end, &mut dir);
        dist = VectorLength(dir);
        //if the func_bobbing is near the end
        if dist < 16.0 {
            _VectorSubtract((*reach).end, (*ms).origin, &mut hordir);
            if (*ms).moveflags & MFL_SWIMMING == 0 {
                hordir[2] = 0.0;
            }
            dist = VectorNormalize(&mut hordir);
            //
            if dist > 60.0 {
                dist = 60.0;
            }
            speed = 360.0 - (360.0 - 6.0 * dist);
            //
            if speed > 5.0 {
                EA_Move(bot, (*ms).client, dir, speed);
            }
            _VectorCopy(dir, &mut result.movedir);
            //
            if (*ms).moveflags & MFL_SWIMMING != 0 {
                result.flags |= MOVERESULT_SWIMVIEW;
            }
        } else {
            MoverBottomCenter(bot, reach, &mut bottomcenter);
            _VectorSubtract(bottomcenter, (*ms).origin, &mut hordir);
            if (*ms).moveflags & MFL_SWIMMING == 0 {
                hordir[2] = 0.0;
            }
            dist = VectorNormalize(&mut hordir);
            //
            if dist > 5.0 {
                //move to the center of the plat
                if dist > 100.0 {
                    dist = 100.0;
                }
                speed = 400.0 - (400.0 - 4.0 * dist);
                //
                EA_Move(bot, (*ms).client, hordir, speed);
                _VectorCopy(hordir, &mut result.movedir);
            }
        }
        result
    }
}

/// Raven `GrappleState`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2517-2540`
pub fn GrappleState(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> c_int {
    unsafe {
        let mut i: c_int;
        let mut entinfo: aas_entityinfo_t = core::mem::zeroed();

        //if the grapple hook is pulling
        if (*ms).moveflags & MFL_GRAPPLEPULL != 0 {
            return 2;
        }
        //check for a visible grapple missile entity
        //or visible grapple entity
        i = AAS_NextEntity(bot, 0);
        while i != 0 {
            if AAS_EntityType(bot, i) == (*bot.entitytypemissile).value as c_int {
                AAS_EntityInfo(bot, i, &mut entinfo);
                if entinfo.weapon == (*bot.weapindex_grapple).value as c_int {
                    return 1;
                }
            }
            i = AAS_NextEntity(bot, i);
        }
        //no valid grapple at all
        0
    }
}

/// Raven `BotResetGrapple`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2547-2566`
pub fn BotResetGrapple(bot: &mut BotLib, ms: *mut bot_movestate_t) {
    unsafe {
        let mut reach: aas_reachability_t = core::mem::zeroed();

        AAS_ReachabilityFromNum(bot, (*ms).lastreachnum, &mut reach);
        //if not using the grapple hook reachability anymore
        if (reach.traveltype & TRAVELTYPE_MASK) != TRAVEL_GRAPPLEHOOK {
            if (*ms).moveflags & MFL_ACTIVEGRAPPLE != 0 || (*ms).grapplevisible_time != 0.0 {
                if (*bot.offhandgrapple).value != 0.0 {
                    EA_Command(bot, (*ms).client, (*bot.cmd_grappleoff).string);
                }
                (*ms).moveflags &= !MFL_ACTIVEGRAPPLE;
                (*ms).grapplevisible_time = 0.0;
            }
        }
    }
}

/// Raven `BotTravel_Crouch`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1421-1444`
pub fn BotTravel_Crouch(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let speed: f32;
        let mut hordir: vec3_t = [0.0; 3];
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //
        speed = 400.0;
        //walk straight to reachability end
        hordir[0] = (*reach).end[0] - (*ms).origin[0];
        hordir[1] = (*reach).end[1] - (*ms).origin[1];
        hordir[2] = 0.0;
        VectorNormalize(&mut hordir);
        //
        BotCheckBlocked(bot, ms, hordir, qtrue, &mut result);
        //elemantary actions
        EA_Crouch(bot, (*ms).client);
        EA_Move(bot, (*ms).client, hordir, speed);
        //
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotTravel_BarrierJump`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1451-1479`
pub fn BotTravel_BarrierJump(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut dist: f32;
        let mut speed: f32;
        let mut hordir: vec3_t = [0.0; 3];
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //walk straight to reachability start
        hordir[0] = (*reach).start[0] - (*ms).origin[0];
        hordir[1] = (*reach).start[1] - (*ms).origin[1];
        hordir[2] = 0.0;
        dist = VectorNormalize(&mut hordir);
        //
        BotCheckBlocked(bot, ms, hordir, qtrue, &mut result);
        //if pretty close to the barrier
        if dist < 9.0 {
            EA_Jump(bot, (*ms).client);
        } else {
            if dist > 60.0 {
                dist = 60.0;
            }
            speed = 360.0 - (360.0 - 6.0 * dist);
            EA_Move(bot, (*ms).client, hordir, speed);
        }
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotFinishTravel_BarrierJump`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1486-1508`
pub fn BotFinishTravel_BarrierJump(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let dist: f32;
        let mut hordir: vec3_t = [0.0; 3];
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //if near the top or going down
        if (*ms).velocity[2] < 250.0 {
            hordir[0] = (*reach).end[0] - (*ms).origin[0];
            hordir[1] = (*reach).end[1] - (*ms).origin[1];
            hordir[2] = 0.0;
            let dist = VectorNormalize(&mut hordir);
            //
            BotCheckBlocked(bot, ms, hordir, qtrue, &mut result);
            //
            EA_Move(bot, (*ms).client, hordir, 400.0);
            _VectorCopy(hordir, &mut result.movedir);
        }
        //
        result
    }
}

/// Raven `BotTravel_Swim`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1515-1534`
pub fn BotTravel_Swim(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut dir: vec3_t = [0.0; 3];
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //swim straight to reachability end
        _VectorSubtract((*reach).start, (*ms).origin, &mut dir);
        VectorNormalize(&mut dir);
        //
        BotCheckBlocked(bot, ms, dir, qtrue, &mut result);
        //elemantary actions
        EA_Move(bot, (*ms).client, dir, 400.0);
        //
        _VectorCopy(dir, &mut result.movedir);
        vectoangles(dir, &mut result.ideal_viewangles);
        result.flags |= MOVERESULT_SWIMVIEW;
        //
        result
    }
}

/// Raven `BotTravel_WalkOffLedge`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1612-1668`
pub fn BotTravel_WalkOffLedge(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut hordir: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let mut dist: f32;
        let mut speed: f32 = 0.0;
        let reachhordist: f32;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //check if the bot is blocked by anything
        _VectorSubtract((*reach).start, (*ms).origin, &mut dir);
        VectorNormalize(&mut dir);
        BotCheckBlocked(bot, ms, dir, qtrue, &mut result);
        //if the reachability start and end are practially above each other
        _VectorSubtract((*reach).end, (*reach).start, &mut dir);
        dir[2] = 0.0;
        reachhordist = VectorLength(dir);
        //walk straight to the reachability start
        hordir[0] = (*reach).start[0] - (*ms).origin[0];
        hordir[1] = (*reach).start[1] - (*ms).origin[1];
        hordir[2] = 0.0;
        dist = VectorNormalize(&mut hordir);
        //if pretty close to the start focus on the reachability end
        if dist < 48.0 {
            hordir[0] = (*reach).end[0] - (*ms).origin[0];
            hordir[1] = (*reach).end[1] - (*ms).origin[1];
            hordir[2] = 0.0;
            VectorNormalize(&mut hordir);
            //
            if reachhordist < 20.0 {
                speed = 100.0;
            } else if AAS_HorizontalVelocityForJump(
                bot,
                0.0,
                (*reach).start,
                (*reach).end,
                &mut speed,
            ) == 0
            {
                speed = 400.0;
            }
        } else {
            if reachhordist < 20.0 {
                if dist > 64.0 {
                    dist = 64.0;
                }
                speed = 400.0 - (256.0 - 4.0 * dist);
            } else {
                speed = 400.0;
            }
        }
        //
        BotCheckBlocked(bot, ms, hordir, qtrue, &mut result);
        //elemantary action
        EA_Move(bot, (*ms).client, hordir, speed);
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotFinishTravel_WalkOffLedge`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1712-1743`
pub fn BotFinishTravel_WalkOffLedge(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut dir: vec3_t = [0.0; 3];
        let mut hordir: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut v: vec3_t = [0.0; 3];
        let mut dist: f32;
        let mut speed: f32 = 0.0;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //
        _VectorSubtract((*reach).end, (*ms).origin, &mut dir);
        BotCheckBlocked(bot, ms, dir, qtrue, &mut result);
        //
        _VectorSubtract((*reach).end, (*ms).origin, &mut v);
        v[2] = 0.0;
        dist = VectorNormalize(&mut v);
        if dist > 16.0 {
            _VectorMA((*reach).end, 16.0, v, &mut end);
        } else {
            _VectorCopy((*reach).end, &mut end);
        }
        //
        if BotAirControl(
            bot,
            (*ms).origin,
            (*ms).velocity,
            end,
            &mut hordir,
            &mut speed,
        ) == 0
        {
            //go straight to the reachability end
            _VectorCopy(dir, &mut hordir);
            hordir[2] = 0.0;
            //
            dist = VectorNormalize(&mut hordir);
            speed = 400.0;
        }
        //
        EA_Move(bot, (*ms).client, hordir, speed);
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotTravel_Teleport`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2034-2058`
pub fn BotTravel_Teleport(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut hordir: vec3_t = [0.0; 3];
        let dist: f32;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //if the bot is being teleported
        if (*ms).moveflags & MFL_TELEPORTED != 0 {
            return result;
        }

        //walk straight to center of the teleporter
        _VectorSubtract((*reach).start, (*ms).origin, &mut hordir);
        if (*ms).moveflags & MFL_SWIMMING == 0 {
            hordir[2] = 0.0;
        }
        let dist = VectorNormalize(&mut hordir);
        //
        BotCheckBlocked(bot, ms, hordir, qtrue, &mut result);

        if dist < 30.0 {
            EA_Move(bot, (*ms).client, hordir, 200.0);
        } else {
            EA_Move(bot, (*ms).client, hordir, 400.0);
        }

        if (*ms).moveflags & MFL_SWIMMING != 0 {
            result.flags |= MOVERESULT_SWIMVIEW;
        }

        _VectorCopy(hordir, &mut result.movedir);
        result
    }
}

/// Raven `BotTravel_Grapple`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2573-2720`
pub fn BotTravel_Grapple(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut result: bot_moveresult_t = core::mem::zeroed();
        let mut dist: f32;
        let mut speed: f32;
        let mut dir: vec3_t = [0.0; 3];
        let mut viewdir: vec3_t = [0.0; 3];
        let mut org: vec3_t = [0.0; 3];
        let state: c_int;
        let areanum: c_int;
        let trace: bsp_trace_t;

        BotClearMoveResult(&mut result);
        //
        if (*ms).moveflags & MFL_GRAPPLERESET != 0 {
            if (*bot.offhandgrapple).value != 0.0 {
                EA_Command(bot, (*ms).client, (*bot.cmd_grappleoff).string);
            }
            (*ms).moveflags &= !MFL_ACTIVEGRAPPLE;
            return result;
        }
        //
        if (*bot.offhandgrapple).value as c_int == 0 {
            result.weapon = (*bot.weapindex_grapple).value as c_int;
            result.flags |= MOVERESULT_MOVEMENTWEAPON;
        }
        //
        if (*ms).moveflags & MFL_ACTIVEGRAPPLE != 0 {
            //
            state = GrappleState(bot, ms, reach);
            //
            _VectorSubtract((*reach).end, (*ms).origin, &mut dir);
            dir[2] = 0.0;
            dist = VectorLength(dir);
            //if very close to the grapple end or the grappled is hooked and
            //the bot doesn't get any closer
            if state != 0 && dist < 48.0 {
                if (*ms).lastgrappledist - dist < 1.0 {
                    if (*bot.offhandgrapple).value != 0.0 {
                        EA_Command(bot, (*ms).client, (*bot.cmd_grappleoff).string);
                    }
                    (*ms).moveflags &= !MFL_ACTIVEGRAPPLE;
                    (*ms).moveflags |= MFL_GRAPPLERESET;
                    (*ms).reachability_time = 0.0; //end the reachability
                    return result;
                }
            }
            //if no valid grapple at all, or the grapple hooked and the bot
            //isn't moving anymore
            else if state == 0 || (state == 2 && dist > (*ms).lastgrappledist - 2.0) {
                if (*ms).grapplevisible_time < AAS_Time(bot) - 0.4 {
                    if (*bot.offhandgrapple).value != 0.0 {
                        EA_Command(bot, (*ms).client, (*bot.cmd_grappleoff).string);
                    }
                    (*ms).moveflags &= !MFL_ACTIVEGRAPPLE;
                    (*ms).moveflags |= MFL_GRAPPLERESET;
                    (*ms).reachability_time = 0.0; //end the reachability
                    return result;
                }
            } else {
                (*ms).grapplevisible_time = AAS_Time(bot);
            }
            //
            if (*bot.offhandgrapple).value as c_int == 0 {
                EA_Attack(bot, (*ms).client);
            }
            //remember the current grapple distance
            (*ms).lastgrappledist = dist;
        } else {
            //
            (*ms).grapplevisible_time = AAS_Time(bot);
            //
            _VectorSubtract((*reach).start, (*ms).origin, &mut dir);
            if (*ms).moveflags & MFL_SWIMMING == 0 {
                dir[2] = 0.0;
            }
            _VectorAdd((*ms).origin, (*ms).viewoffset, &mut org);
            _VectorSubtract((*reach).end, org, &mut viewdir);
            //
            dist = VectorNormalize(&mut dir);
            vectoangles(viewdir, &mut result.ideal_viewangles);
            result.flags |= MOVERESULT_MOVEMENTVIEW;
            //
            if dist < 5.0
                && AngleDiff(result.ideal_viewangles[0], (*ms).viewangles[0]).abs() < 2.0
                && AngleDiff(result.ideal_viewangles[1], (*ms).viewangles[1]).abs() < 2.0
            {
                //check if the grapple missile path is clear
                _VectorAdd((*ms).origin, (*ms).viewoffset, &mut org);
                trace = AAS_Trace(
                    bot,
                    org,
                    [0.0; 3],
                    [0.0; 3],
                    (*reach).end,
                    (*ms).entitynum,
                    CONTENTS_SOLID,
                );
                _VectorSubtract((*reach).end, trace.endpos, &mut dir);
                if VectorLength(dir) > 16.0 {
                    result.failure = qtrue;
                    return result;
                }
                //activate the grapple
                if (*bot.offhandgrapple).value != 0.0 {
                    EA_Command(bot, (*ms).client, (*bot.cmd_grappleon).string);
                } else {
                    EA_Attack(bot, (*ms).client);
                }
                (*ms).moveflags |= MFL_ACTIVEGRAPPLE;
                (*ms).lastgrappledist = 999999.0;
            } else {
                if dist < 70.0 {
                    speed = 300.0 - (300.0 - 4.0 * dist);
                } else {
                    speed = 400.0;
                }
                //
                BotCheckBlocked(bot, ms, dir, qtrue, &mut result);
                //elemantary action move in direction
                EA_Move(bot, (*ms).client, dir, speed);
                _VectorCopy(dir, &mut result.movedir);
            }
            //if in another area before actually grappling
            areanum = AAS_PointAreaNum(bot, (*ms).origin);
            if areanum != 0 && areanum != (*ms).reachareanum {
                (*ms).reachability_time = 0.0;
            }
        }
        result
    }
}

/// Raven `BotTravel_JumpPad`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2893-2913`
pub fn BotTravel_JumpPad(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let dist: f32;
        let speed: f32;
        let mut hordir: vec3_t = [0.0; 3];
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //first walk straight to the reachability start
        hordir[0] = (*reach).start[0] - (*ms).origin[0];
        hordir[1] = (*reach).start[1] - (*ms).origin[1];
        hordir[2] = 0.0;
        let dist = VectorNormalize(&mut hordir);
        //
        BotCheckBlocked(bot, ms, hordir, qtrue, &mut result);
        let speed = 400.0;
        //elemantary action move in direction
        EA_Move(bot, (*ms).client, hordir, speed);
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotFinishTravel_JumpPad`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2920-2941`
pub fn BotFinishTravel_JumpPad(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut speed: f32 = 0.0;
        let mut hordir: vec3_t = [0.0; 3];
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        if BotAirControl(
            bot,
            (*ms).origin,
            (*ms).velocity,
            (*reach).end,
            &mut hordir,
            &mut speed,
        ) == 0
        {
            hordir[0] = (*reach).end[0] - (*ms).origin[0];
            hordir[1] = (*reach).end[1] - (*ms).origin[1];
            hordir[2] = 0.0;
            VectorNormalize(&mut hordir);
            speed = 400.0;
        }
        BotCheckBlocked(bot, ms, hordir, qtrue, &mut result);
        //elemantary action move in direction
        EA_Move(bot, (*ms).client, hordir, speed);
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotMoveInGoalArea`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2981-3031`
pub fn BotMoveInGoalArea(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    goal: *mut bot_goal_t,
) -> bot_moveresult_t {
    unsafe {
        let mut result: bot_moveresult_t = core::mem::zeroed();
        let mut dir: vec3_t = [0.0; 3];
        let mut dist: f32;
        let mut speed: f32;

        BotClearMoveResult(&mut result);
        //walk straight to the goal origin
        dir[0] = (*goal).origin[0] - (*ms).origin[0];
        dir[1] = (*goal).origin[1] - (*ms).origin[1];
        if (*ms).moveflags & MFL_SWIMMING != 0 {
            dir[2] = (*goal).origin[2] - (*ms).origin[2];
            result.traveltype = TRAVEL_SWIM;
        } else {
            dir[2] = 0.0;
            result.traveltype = TRAVEL_WALK;
        }
        //
        dist = VectorNormalize(&mut dir);
        if dist > 100.0 {
            dist = 100.0;
        }
        speed = 400.0 - (400.0 - 4.0 * dist);
        if speed < 10.0 {
            speed = 0.0;
        }
        //
        BotCheckBlocked(bot, ms, dir, qtrue, &mut result);
        //elemantary action move in direction
        EA_Move(bot, (*ms).client, dir, speed);
        _VectorCopy(dir, &mut result.movedir);
        //
        if (*ms).moveflags & MFL_SWIMMING != 0 {
            vectoangles(dir, &mut result.ideal_viewangles);
            result.flags |= MOVERESULT_SWIMVIEW;
        }
        //
        (*ms).lastreachnum = 0;
        (*ms).lastareanum = 0;
        (*ms).lastgoalareanum = (*goal).areanum;
        _VectorCopy((*ms).origin, &mut (*ms).lastorigin);
        //
        result
    }
}

/// Raven `BotSetupMoveAI`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:3558-3572`
pub fn BotSetupMoveAI(bot: &mut BotLib) -> c_int {
    BotSetBrushModelTypes(bot);
    bot.sv_maxstep = LibVar(
        bot,
        c"sv_step".as_ptr() as *mut c_char,
        c"18".as_ptr() as *mut c_char,
    );
    bot.sv_maxbarrier = LibVar(
        bot,
        c"sv_maxbarrier".as_ptr() as *mut c_char,
        c"32".as_ptr() as *mut c_char,
    );
    bot.sv_gravity = LibVar(
        bot,
        c"sv_gravity".as_ptr() as *mut c_char,
        c"800".as_ptr() as *mut c_char,
    );
    bot.weapindex_rocketlauncher = LibVar(
        bot,
        c"weapindex_rocketlauncher".as_ptr() as *mut c_char,
        c"5".as_ptr() as *mut c_char,
    );
    bot.weapindex_bfg10k = LibVar(
        bot,
        c"weapindex_bfg10k".as_ptr() as *mut c_char,
        c"9".as_ptr() as *mut c_char,
    );
    bot.weapindex_grapple = LibVar(
        bot,
        c"weapindex_grapple".as_ptr() as *mut c_char,
        c"10".as_ptr() as *mut c_char,
    );
    bot.entitytypemissile = LibVar(
        bot,
        c"entitytypemissile".as_ptr() as *mut c_char,
        c"3".as_ptr() as *mut c_char,
    );
    bot.offhandgrapple = LibVar(
        bot,
        c"offhandgrapple".as_ptr() as *mut c_char,
        c"0".as_ptr() as *mut c_char,
    );
    bot.cmd_grappleon = LibVar(
        bot,
        c"cmd_grappleon".as_ptr() as *mut c_char,
        c"grappleon".as_ptr() as *mut c_char,
    );
    bot.cmd_grappleoff = LibVar(
        bot,
        c"cmd_grappleoff".as_ptr() as *mut c_char,
        c"grappleoff".as_ptr() as *mut c_char,
    );
    BLERR_NOERROR
}

/// Raven `BotReachabilityArea`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:284-342`
pub fn BotReachabilityArea(bot: &mut BotLib, origin: vec3_t, client: c_int) -> c_int {
    unsafe {
        let modelnum: c_int;
        let modeltype: c_int;
        let reachnum: c_int;
        let areanum: c_int;
        let mut reach: aas_reachability_t = core::mem::zeroed();
        let mut org: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let up: vec3_t = [0.0, 0.0, 1.0];
        let bsptrace: bsp_trace_t;
        let trace: aas_trace_t;

        //check if the bot is standing on something
        AAS_PresenceTypeBoundingBox(bot, PRESENCE_CROUCH, &mut mins, &mut maxs);
        _VectorMA(origin, -3.0, up, &mut end);
        bsptrace = AAS_Trace(
            bot,
            origin,
            mins,
            maxs,
            end,
            client,
            CONTENTS_SOLID | CONTENTS_PLAYERCLIP,
        );
        if bsptrace.startsolid == 0 && bsptrace.fraction < 1.0 && bsptrace.ent != ENTITYNUM_NONE {
            //if standing on the world the bot should be in a valid area
            if bsptrace.ent == ENTITYNUM_WORLD {
                return BotFuzzyPointReachabilityArea(bot, origin);
            }

            modelnum = AAS_EntityModelindex(bot, bsptrace.ent);
            modeltype = bot.modeltypes[modelnum as usize];

            //if standing on a func_plat or func_bobbing then the bot is assumed to be
            //in the area the reachability points to
            if modeltype == MODELTYPE_FUNC_PLAT || modeltype == MODELTYPE_FUNC_BOB {
                reachnum = AAS_NextModelReachability(bot, 0, modelnum);
                if reachnum != 0 {
                    AAS_ReachabilityFromNum(bot, reachnum, &mut reach);
                    return reach.areanum;
                }
            }

            //if the bot is swimming the bot should be in a valid area
            if AAS_Swimming(bot, origin) != 0 {
                return BotFuzzyPointReachabilityArea(bot, origin);
            }
            //
            areanum = BotFuzzyPointReachabilityArea(bot, origin);
            //if the bot is in an area with reachabilities
            if areanum != 0 && AAS_AreaReachability(bot, areanum) != 0 {
                return areanum;
            }
            //trace down till the ground is hit because the bot is standing on some other entity
            _VectorCopy(origin, &mut org);
            _VectorCopy(org, &mut end);
            end[2] -= 800.0;
            trace = AAS_TraceClientBBox(bot, org, end, PRESENCE_CROUCH, -1);
            if trace.startsolid == 0 {
                _VectorCopy(trace.endpos, &mut org);
            }
            //
            return BotFuzzyPointReachabilityArea(bot, org);
        }
        //
        BotFuzzyPointReachabilityArea(bot, origin)
    }
}

/// Raven `BotGapDistance`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:983-1025`
pub fn BotGapDistance(bot: &mut BotLib, origin: vec3_t, hordir: vec3_t, entnum: c_int) -> f32 {
    unsafe {
        let mut dist: f32;
        let mut startz: f32;
        let mut start: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut trace: aas_trace_t;

        //do gap checking
        startz = origin[2];
        //this enables walking down stairs more fluidly
        {
            _VectorCopy(origin, &mut start);
            _VectorCopy(origin, &mut end);
            end[2] -= 60.0;
            trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_CROUCH, entnum);
            if trace.fraction >= 1.0 {
                return 1.0;
            }
            startz = trace.endpos[2] + 1.0;
        }
        //
        dist = 8.0;
        while dist <= 100.0 {
            _VectorMA(origin, dist, hordir, &mut start);
            start[2] = startz + 24.0;
            _VectorCopy(start, &mut end);
            end[2] -= 48.0 + (*bot.sv_maxbarrier).value;
            trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_CROUCH, entnum);
            //if solid is found the bot can't walk any further and fall into a gap
            if trace.startsolid == 0 {
                //if it is a gap
                if trace.endpos[2] < startz - (*bot.sv_maxstep).value - 8.0 {
                    _VectorCopy(trace.endpos, &mut end);
                    end[2] -= 20.0;
                    if AAS_PointContents(bot, end) & CONTENTS_WATER != 0 {
                        break;
                    }
                    //if a gap is found slow down
                    return dist;
                }
                startz = trace.endpos[2];
            }
            dist += 8.0;
        }
        0.0
    }
}

/// Raven `BotCheckBarrierJump`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1032-1075`
pub fn BotCheckBarrierJump(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    dir: vec3_t,
    speed: f32,
) -> c_int {
    unsafe {
        let mut start: vec3_t = [0.0; 3];
        let mut hordir: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut trace: aas_trace_t;

        _VectorCopy((*ms).origin, &mut end);
        end[2] += (*bot.sv_maxbarrier).value;
        //trace right up
        trace = AAS_TraceClientBBox(bot, (*ms).origin, end, PRESENCE_NORMAL, (*ms).entitynum);
        //this shouldn't happen... but we check anyway
        if trace.startsolid != 0 {
            return qfalse;
        }
        //if very low ceiling it isn't possible to jump up to a barrier
        if trace.endpos[2] - (*ms).origin[2] < (*bot.sv_maxstep).value {
            return qfalse;
        }
        //
        hordir[0] = dir[0];
        hordir[1] = dir[1];
        hordir[2] = 0.0;
        VectorNormalize(&mut hordir);
        _VectorMA(
            (*ms).origin,
            (*ms).thinktime * speed * 0.5,
            hordir,
            &mut end,
        );
        _VectorCopy(trace.endpos, &mut start);
        end[2] = trace.endpos[2];
        //trace from previous trace end pos horizontally in the move direction
        trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_NORMAL, (*ms).entitynum);
        //again this shouldn't happen
        if trace.startsolid != 0 {
            return qfalse;
        }
        //
        _VectorCopy(trace.endpos, &mut start);
        _VectorCopy(trace.endpos, &mut end);
        end[2] = (*ms).origin[2];
        //trace down from the previous trace end pos
        trace = AAS_TraceClientBBox(bot, start, end, PRESENCE_NORMAL, (*ms).entitynum);
        //if solid
        if trace.startsolid != 0 {
            return qfalse;
        }
        //if no obstacle at all
        if trace.fraction >= 1.0 {
            return qfalse;
        }
        //if less than the maximum step height
        if trace.endpos[2] - (*ms).origin[2] < (*bot.sv_maxstep).value {
            return qfalse;
        }
        //
        EA_Jump(bot, (*ms).client);
        EA_Move(bot, (*ms).client, hordir, speed);
        (*ms).moveflags |= MFL_BARRIERJUMP;
        //there is a barrier
        qtrue
    }
}

/// Raven `BotTravel_Walk`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1328-1376`
pub fn BotTravel_Walk(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut dist: f32;
        let mut speed: f32 = 0.0;
        let mut hordir: vec3_t = [0.0; 3];
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //first walk straight to the reachability start
        hordir[0] = (*reach).start[0] - (*ms).origin[0];
        hordir[1] = (*reach).start[1] - (*ms).origin[1];
        hordir[2] = 0.0;
        dist = VectorNormalize(&mut hordir);
        //
        BotCheckBlocked(bot, ms, hordir, qtrue, &mut result);
        //
        if dist < 10.0 {
            //walk straight to the reachability end
            hordir[0] = (*reach).end[0] - (*ms).origin[0];
            hordir[1] = (*reach).end[1] - (*ms).origin[1];
            hordir[2] = 0.0;
            dist = VectorNormalize(&mut hordir);
        }
        //if going towards a crouch area
        if AAS_AreaPresenceType(bot, (*reach).areanum) & PRESENCE_NORMAL == 0 {
            //if pretty close to the reachable area
            if dist < 20.0 {
                EA_Crouch(bot, (*ms).client);
            }
        }
        //
        dist = BotGapDistance(bot, (*ms).origin, hordir, (*ms).entitynum);
        //
        if (*ms).moveflags & MFL_WALK != 0 {
            if dist > 0.0 {
                speed = 200.0 - (180.0 - 1.0 * dist);
            } else {
                speed = 200.0;
            }
            EA_Walk(bot, (*ms).client);
        } else {
            if dist > 0.0 {
                speed = 400.0 - (360.0 - 2.0 * dist);
            } else {
                speed = 400.0;
            }
        }
        //elemantary action move in direction
        EA_Move(bot, (*ms).client, hordir, speed);
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotTravel_Elevator`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2065-2209`
pub fn BotTravel_Elevator(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut dir: vec3_t = [0.0; 3];
        let mut dir1: vec3_t = [0.0; 3];
        let mut dir2: vec3_t = [0.0; 3];
        let mut hordir: vec3_t = [0.0; 3];
        let mut bottomcenter: vec3_t = [0.0; 3];
        let mut dist: f32;
        let mut dist1: f32;
        let mut dist2: f32;
        let mut speed: f32;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //if standing on the plat
        if BotOnMover(bot, (*ms).origin, (*ms).entitynum, reach) != 0 {
            //if vertically not too far from the end point
            if (((*ms).origin[2] - (*reach).end[2]) as c_int).abs()
                < (*bot.sv_maxbarrier).value as c_int
            {
                //move to the end point
                _VectorSubtract((*reach).end, (*ms).origin, &mut hordir);
                hordir[2] = 0.0;
                VectorNormalize(&mut hordir);
                if BotCheckBarrierJump(bot, ms, hordir, 100.0) == 0 {
                    EA_Move(bot, (*ms).client, hordir, 400.0);
                }
                _VectorCopy(hordir, &mut result.movedir);
            }
            //if not really close to the center of the elevator
            else {
                MoverBottomCenter(bot, reach, &mut bottomcenter);
                _VectorSubtract(bottomcenter, (*ms).origin, &mut hordir);
                hordir[2] = 0.0;
                dist = VectorNormalize(&mut hordir);
                //
                if dist > 10.0 {
                    //move to the center of the plat
                    if dist > 100.0 {
                        dist = 100.0;
                    }
                    speed = 400.0 - (400.0 - 4.0 * dist);
                    //
                    EA_Move(bot, (*ms).client, hordir, speed);
                    _VectorCopy(hordir, &mut result.movedir);
                }
            }
        } else {
            //if very near the reachability end
            _VectorSubtract((*reach).end, (*ms).origin, &mut dir);
            dist = VectorLength(dir);
            if dist < 64.0 {
                if dist > 60.0 {
                    dist = 60.0;
                }
                speed = 360.0 - (360.0 - 6.0 * dist);
                //
                if (*ms).moveflags & MFL_SWIMMING != 0
                    || BotCheckBarrierJump(bot, ms, dir, 50.0) == 0
                {
                    if speed > 5.0 {
                        EA_Move(bot, (*ms).client, dir, speed);
                    }
                }
                _VectorCopy(dir, &mut result.movedir);
                //
                if (*ms).moveflags & MFL_SWIMMING != 0 {
                    result.flags |= MOVERESULT_SWIMVIEW;
                }
                //stop using this reachability
                (*ms).reachability_time = 0.0;
                return result;
            }
            //get direction and distance to reachability start
            _VectorSubtract((*reach).start, (*ms).origin, &mut dir1);
            if (*ms).moveflags & MFL_SWIMMING == 0 {
                dir1[2] = 0.0;
            }
            dist1 = VectorNormalize(&mut dir1);
            //if the elevator isn't down
            if MoverDown(bot, reach) == 0 {
                dist = dist1;
                _VectorCopy(dir1, &mut dir);
                //
                BotCheckBlocked(bot, ms, dir, qfalse, &mut result);
                //
                if dist > 60.0 {
                    dist = 60.0;
                }
                speed = 360.0 - (360.0 - 6.0 * dist);
                //
                if (*ms).moveflags & MFL_SWIMMING == 0
                    && BotCheckBarrierJump(bot, ms, dir, 50.0) == 0
                {
                    if speed > 5.0 {
                        EA_Move(bot, (*ms).client, dir, speed);
                    }
                }
                _VectorCopy(dir, &mut result.movedir);
                //
                if (*ms).moveflags & MFL_SWIMMING != 0 {
                    result.flags |= MOVERESULT_SWIMVIEW;
                }
                //this isn't a failure... just wait till the elevator comes down
                result.r#type = RESULTTYPE_ELEVATORUP;
                result.flags |= MOVERESULT_WAITING;
                return result;
            }
            //get direction and distance to elevator bottom center
            MoverBottomCenter(bot, reach, &mut bottomcenter);
            _VectorSubtract(bottomcenter, (*ms).origin, &mut dir2);
            if (*ms).moveflags & MFL_SWIMMING == 0 {
                dir2[2] = 0.0;
            }
            dist2 = VectorNormalize(&mut dir2);
            //if very close to the reachability start or
            //closer to the elevator center or
            //between reachability start and elevator center
            if dist1 < 20.0 || dist2 < dist1 || _DotProduct(dir1, dir2) < 0.0 {
                dist = dist2;
                _VectorCopy(dir2, &mut dir);
            } else
            //closer to the reachability start
            {
                dist = dist1;
                _VectorCopy(dir1, &mut dir);
            }
            //
            BotCheckBlocked(bot, ms, dir, qfalse, &mut result);
            //
            if dist > 60.0 {
                dist = 60.0;
            }
            speed = 400.0 - (400.0 - 6.0 * dist);
            //
            if (*ms).moveflags & MFL_SWIMMING == 0 && BotCheckBarrierJump(bot, ms, dir, 50.0) == 0 {
                EA_Move(bot, (*ms).client, dir, speed);
            }
            _VectorCopy(dir, &mut result.movedir);
            //
            if (*ms).moveflags & MFL_SWIMMING != 0 {
                result.flags |= MOVERESULT_SWIMVIEW;
            }
        }
        result
    }
}

/// Raven `BotTravel_FuncBobbing`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:2304-2455`
pub fn BotTravel_FuncBobbing(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut dir: vec3_t = [0.0; 3];
        let mut dir1: vec3_t = [0.0; 3];
        let mut dir2: vec3_t = [0.0; 3];
        let mut hordir: vec3_t = [0.0; 3];
        let mut bottomcenter: vec3_t = [0.0; 3];
        let mut bob_start: vec3_t = [0.0; 3];
        let mut bob_end: vec3_t = [0.0; 3];
        let mut bob_origin: vec3_t = [0.0; 3];
        let mut dist: f32;
        let mut dist1: f32;
        let mut dist2: f32;
        let mut speed: f32;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //
        BotFuncBobStartEnd(bot, reach, &mut bob_start, &mut bob_end, &mut bob_origin);
        //if standing ontop of the func_bobbing
        if BotOnMover(bot, (*ms).origin, (*ms).entitynum, reach) != 0 {
            //if near end point of reachability
            _VectorSubtract(bob_origin, bob_end, &mut dir);
            if VectorLength(dir) < 24.0 {
                //move to the end point
                _VectorSubtract((*reach).end, (*ms).origin, &mut hordir);
                hordir[2] = 0.0;
                VectorNormalize(&mut hordir);
                if BotCheckBarrierJump(bot, ms, hordir, 100.0) == 0 {
                    EA_Move(bot, (*ms).client, hordir, 400.0);
                }
                _VectorCopy(hordir, &mut result.movedir);
            }
            //if not really close to the center of the elevator
            else {
                MoverBottomCenter(bot, reach, &mut bottomcenter);
                _VectorSubtract(bottomcenter, (*ms).origin, &mut hordir);
                hordir[2] = 0.0;
                dist = VectorNormalize(&mut hordir);
                //
                if dist > 10.0 {
                    //move to the center of the plat
                    if dist > 100.0 {
                        dist = 100.0;
                    }
                    speed = 400.0 - (400.0 - 4.0 * dist);
                    //
                    EA_Move(bot, (*ms).client, hordir, speed);
                    _VectorCopy(hordir, &mut result.movedir);
                }
            }
        } else {
            //if very near the reachability end
            _VectorSubtract((*reach).end, (*ms).origin, &mut dir);
            dist = VectorLength(dir);
            if dist < 64.0 {
                if dist > 60.0 {
                    dist = 60.0;
                }
                speed = 360.0 - (360.0 - 6.0 * dist);
                //if swimming or no barrier jump
                if (*ms).moveflags & MFL_SWIMMING != 0
                    || BotCheckBarrierJump(bot, ms, dir, 50.0) == 0
                {
                    if speed > 5.0 {
                        EA_Move(bot, (*ms).client, dir, speed);
                    }
                }
                _VectorCopy(dir, &mut result.movedir);
                //
                if (*ms).moveflags & MFL_SWIMMING != 0 {
                    result.flags |= MOVERESULT_SWIMVIEW;
                }
                //stop using this reachability
                (*ms).reachability_time = 0.0;
                return result;
            }
            //get direction and distance to reachability start
            _VectorSubtract((*reach).start, (*ms).origin, &mut dir1);
            if (*ms).moveflags & MFL_SWIMMING == 0 {
                dir1[2] = 0.0;
            }
            dist1 = VectorNormalize(&mut dir1);
            //if func_bobbing is Not it's start position
            _VectorSubtract(bob_origin, bob_start, &mut dir);
            if VectorLength(dir) > 16.0 {
                dist = dist1;
                _VectorCopy(dir1, &mut dir);
                //
                BotCheckBlocked(bot, ms, dir, qfalse, &mut result);
                //
                if dist > 60.0 {
                    dist = 60.0;
                }
                speed = 360.0 - (360.0 - 6.0 * dist);
                //
                if (*ms).moveflags & MFL_SWIMMING == 0
                    && BotCheckBarrierJump(bot, ms, dir, 50.0) == 0
                {
                    if speed > 5.0 {
                        EA_Move(bot, (*ms).client, dir, speed);
                    }
                }
                _VectorCopy(dir, &mut result.movedir);
                //
                if (*ms).moveflags & MFL_SWIMMING != 0 {
                    result.flags |= MOVERESULT_SWIMVIEW;
                }
                //this isn't a failure... just wait till the func_bobbing arrives
                result.r#type = RESULTTYPE_WAITFORFUNCBOBBING;
                result.flags |= MOVERESULT_WAITING;
                return result;
            }
            //get direction and distance to func_bob bottom center
            MoverBottomCenter(bot, reach, &mut bottomcenter);
            _VectorSubtract(bottomcenter, (*ms).origin, &mut dir2);
            if (*ms).moveflags & MFL_SWIMMING == 0 {
                dir2[2] = 0.0;
            }
            dist2 = VectorNormalize(&mut dir2);
            //if very close to the reachability start or
            //closer to the elevator center or
            //between reachability start and func_bobbing center
            if dist1 < 20.0 || dist2 < dist1 || _DotProduct(dir1, dir2) < 0.0 {
                dist = dist2;
                _VectorCopy(dir2, &mut dir);
            } else
            //closer to the reachability start
            {
                dist = dist1;
                _VectorCopy(dir1, &mut dir);
            }
            //
            BotCheckBlocked(bot, ms, dir, qfalse, &mut result);
            //
            if dist > 60.0 {
                dist = 60.0;
            }
            speed = 400.0 - (400.0 - 6.0 * dist);
            //
            if (*ms).moveflags & MFL_SWIMMING == 0 && BotCheckBarrierJump(bot, ms, dir, 50.0) == 0 {
                EA_Move(bot, (*ms).client, dir, speed);
            }
            _VectorCopy(dir, &mut result.movedir);
            //
            if (*ms).moveflags & MFL_SWIMMING != 0 {
                result.flags |= MOVERESULT_SWIMVIEW;
            }
        }
        result
    }
}

/// Raven `BotWalkInDirection`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1097-1207`
pub fn BotWalkInDirection(
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    dir: vec3_t,
    speed: f32,
    mut r#type: c_int,
) -> c_int {
    unsafe {
        let mut hordir: vec3_t = [0.0; 3];
        let mut cmdmove: vec3_t = [0.0; 3];
        let mut velocity: vec3_t = [0.0; 3];
        let mut tmpdir: vec3_t = [0.0; 3];
        let mut origin: vec3_t = [0.0; 3];
        let presencetype: c_int;
        let maxframes: c_int;
        let cmdframes: c_int;
        let stopevent: c_int;
        let mut r#move: aas_clientmove_t = core::mem::zeroed();
        let mut dist: f32;

        if AAS_OnGround(bot, (*ms).origin, (*ms).presencetype, (*ms).entitynum) != 0 {
            (*ms).moveflags |= MFL_ONGROUND;
        }
        //if the bot is on the ground
        if (*ms).moveflags & MFL_ONGROUND != 0 {
            //if there is a barrier the bot can jump on
            if BotCheckBarrierJump(bot, ms, dir, speed) != 0 {
                return qtrue;
            }
            //remove barrier jump flag
            (*ms).moveflags &= !MFL_BARRIERJUMP;
            //get the presence type for the movement
            if (r#type & MOVE_CROUCH) != 0 && (r#type & MOVE_JUMP) == 0 {
                presencetype = PRESENCE_CROUCH;
            } else {
                presencetype = PRESENCE_NORMAL;
            }
            //horizontal direction
            hordir[0] = dir[0];
            hordir[1] = dir[1];
            hordir[2] = 0.0;
            VectorNormalize(&mut hordir);
            //if the bot is not supposed to jump
            if (r#type & MOVE_JUMP) == 0 {
                //if there is a gap, try to jump over it
                if BotGapDistance(bot, (*ms).origin, hordir, (*ms).entitynum) > 0.0 {
                    r#type |= MOVE_JUMP;
                }
            }
            //get command movement
            _VectorScale(hordir, speed, &mut cmdmove);
            _VectorCopy((*ms).velocity, &mut velocity);
            //
            if r#type & MOVE_JUMP != 0 {
                cmdmove[2] = 400.0;
                maxframes = (PREDICTIONTIME_JUMP / 0.1) as c_int;
                cmdframes = 1;
                stopevent = SE_HITGROUND
                    | SE_HITGROUNDDAMAGE
                    | SE_ENTERWATER
                    | SE_ENTERSLIME
                    | SE_ENTERLAVA;
            } else {
                maxframes = 2;
                cmdframes = 2;
                stopevent = SE_HITGROUNDDAMAGE | SE_ENTERWATER | SE_ENTERSLIME | SE_ENTERLAVA;
            }
            //
            _VectorCopy((*ms).origin, &mut origin);
            origin[2] += 0.5;
            AAS_PredictClientMovement(
                bot,
                &mut r#move,
                (*ms).entitynum,
                origin,
                presencetype,
                qtrue,
                velocity,
                cmdmove,
                cmdframes,
                maxframes,
                0.1f32,
                stopevent,
                0,
                qfalse,
            );
            //if prediction time wasn't enough to fully predict the movement
            if r#move.frames >= maxframes && (r#type & MOVE_JUMP) != 0 {
                return qfalse;
            }
            //don't enter slime or lava and don't fall from too high
            if r#move.stopevent & (SE_ENTERSLIME | SE_ENTERLAVA | SE_HITGROUNDDAMAGE) != 0 {
                return qfalse;
            }
            //if ground was hit
            if r#move.stopevent & SE_HITGROUND != 0 {
                //check for nearby gap
                VectorNormalize2(r#move.velocity, &mut tmpdir);
                dist = BotGapDistance(bot, r#move.endpos, tmpdir, (*ms).entitynum);
                if dist > 0.0 {
                    return qfalse;
                }
                //
                dist = BotGapDistance(bot, r#move.endpos, hordir, (*ms).entitynum);
                if dist > 0.0 {
                    return qfalse;
                }
            }
            //get horizontal movement
            tmpdir[0] = r#move.endpos[0] - (*ms).origin[0];
            tmpdir[1] = r#move.endpos[1] - (*ms).origin[1];
            tmpdir[2] = 0.0;
            //
            //the bot is blocked by something
            if VectorLength(tmpdir) < speed * (*ms).thinktime * 0.5 {
                return qfalse;
            }
            //perform the movement
            if r#type & MOVE_JUMP != 0 {
                EA_Jump(bot, (*ms).client);
            }
            if r#type & MOVE_CROUCH != 0 {
                EA_Crouch(bot, (*ms).client);
            }
            EA_Move(bot, (*ms).client, hordir, speed);
            //movement was succesfull
            qtrue
        } else {
            if (*ms).moveflags & MFL_BARRIERJUMP != 0 {
                //if near the top or going down
                if (*ms).velocity[2] < 50.0 {
                    EA_Move(bot, (*ms).client, dir, speed);
                }
            }
            //FIXME: do air control to avoid hazards
            qtrue
        }
    }
}

/// Raven `BotGetReachabilityToGoal`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:727-798`
pub fn BotGetReachabilityToGoal(
    bot: &mut BotLib,
    origin: vec3_t,
    areanum: c_int,
    lastgoalareanum: c_int,
    lastareanum: c_int,
    avoidreach: *mut c_int,
    avoidreachtimes: *mut f32,
    avoidreachtries: *mut c_int,
    goal: *mut bot_goal_t,
    mut travelflags: c_int,
    mut movetravelflags: c_int,
    avoidspots: *mut bot_avoidspot_s,
    numavoidspots: c_int,
    flags: *mut c_int,
) -> c_int {
    unsafe {
        let mut i: usize;
        let mut t: c_int;
        let mut besttime: c_int;
        let mut bestreachnum: c_int;
        let mut reachnum: c_int;
        let mut reach: aas_reachability_t = core::mem::zeroed();

        //if not in a valid area
        if areanum == 0 {
            return 0;
        }
        //
        if AAS_AreaDoNotEnter(bot, areanum) != 0 || AAS_AreaDoNotEnter(bot, (*goal).areanum) != 0 {
            travelflags |= TFL_DONOTENTER;
            movetravelflags |= TFL_DONOTENTER;
        }
        //use the routing to find the next area to go to
        besttime = 0;
        bestreachnum = 0;
        //
        reachnum = AAS_NextAreaReachability(bot, areanum, 0);
        while reachnum != 0 {
            'cont: {
                // AVOIDREACH is defined; keep the avoidance block.
                //check if it isn't an reachability to avoid
                i = 0;
                while i < MAX_AVOIDREACH {
                    if *avoidreach.add(i) == reachnum && *avoidreachtimes.add(i) >= AAS_Time(bot) {
                        break;
                    }
                    i += 1;
                }
                if i != MAX_AVOIDREACH && *avoidreachtries.add(i) > AVOIDREACH_TRIES {
                    break 'cont;
                }
                //get the reachability from the number
                AAS_ReachabilityFromNum(bot, reachnum, &mut reach);
                //NOTE: do not go back to the previous area if the goal didn't change
                if lastgoalareanum == (*goal).areanum && reach.areanum == lastareanum {
                    break 'cont;
                }
                //if the travel isn't valid
                if BotValidTravel(bot, origin, &mut reach, movetravelflags) == 0 {
                    break 'cont;
                }
                //get the travel time
                t = AAS_AreaTravelTimeToGoalArea(
                    bot,
                    reach.areanum,
                    &reach.end,
                    (*goal).areanum,
                    travelflags,
                );
                //if the goal area isn't reachable from the reachable area
                if t == 0 {
                    break 'cont;
                }
                //if the bot should not use this reachability to avoid bad spots
                if BotAvoidSpots(origin, &mut reach, avoidspots, numavoidspots) != 0 {
                    if !flags.is_null() {
                        *flags |= MOVERESULT_BLOCKEDBYAVOIDSPOT;
                    }
                    break 'cont;
                }
                //add the travel time towards the area
                t += reach.traveltime as c_int;
                //if the travel time is better than the ones already found
                if besttime == 0 || t < besttime {
                    besttime = t;
                    bestreachnum = reachnum;
                }
            }
            reachnum = AAS_NextAreaReachability(bot, areanum, reachnum);
        }
        //
        bestreachnum
    }
}

/// Raven `BotMoveInDirection`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1214-1229`
pub fn BotMoveInDirection(
    bot: &mut BotLib,
    movestate: c_int,
    dir: vec3_t,
    speed: f32,
    r#type: c_int,
) -> c_int {
    unsafe {
        let ms: *mut bot_movestate_t;

        ms = BotMoveStateFromHandle(bot, movestate);
        if ms.is_null() {
            return qfalse;
        }
        //if swimming
        if AAS_Swimming(bot, (*ms).origin) != 0 {
            BotSwimInDirection(bot, ms, dir, speed, r#type)
        } else {
            BotWalkInDirection(bot, ms, dir, speed, r#type)
        }
    }
}

/// Raven `BotTravel_Jump`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:1872-1936`
pub fn BotTravel_Jump(
    common: &mut Common,
    bot: &mut BotLib,
    ms: *mut bot_movestate_t,
    reach: *mut aas_reachability_t,
) -> bot_moveresult_t {
    unsafe {
        let mut hordir: vec3_t = [0.0; 3];
        let mut dir1: vec3_t = [0.0; 3];
        let mut dir2: vec3_t = [0.0; 3];
        let mut start: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut runstart: vec3_t = [0.0; 3];
        let mut dist1: f32;
        let mut dist2: f32;
        let speed: f32;
        let mut result: bot_moveresult_t = core::mem::zeroed();

        BotClearMoveResult(&mut result);
        //
        AAS_JumpReachRunStart(common, bot, reach, &mut runstart);
        //
        hordir[0] = runstart[0] - (*reach).start[0];
        hordir[1] = runstart[1] - (*reach).start[1];
        hordir[2] = 0.0;
        VectorNormalize(&mut hordir);
        //
        _VectorCopy((*reach).start, &mut start);
        start[2] += 1.0;
        _VectorMA((*reach).start, 80.0, hordir, &mut runstart);
        //check for a gap
        dist1 = 0.0;
        while dist1 < 80.0 {
            _VectorMA(start, dist1 + 10.0, hordir, &mut end);
            end[2] += 1.0;
            if AAS_PointAreaNum(bot, end) != (*ms).reachareanum {
                break;
            }
            dist1 += 10.0;
        }
        if dist1 < 80.0 {
            _VectorMA((*reach).start, dist1, hordir, &mut runstart);
        }
        //
        _VectorSubtract((*ms).origin, (*reach).start, &mut dir1);
        dir1[2] = 0.0;
        dist1 = VectorNormalize(&mut dir1);
        _VectorSubtract((*ms).origin, runstart, &mut dir2);
        dir2[2] = 0.0;
        dist2 = VectorNormalize(&mut dir2);
        //if just before the reachability start
        if _DotProduct(dir1, dir2) < -0.8 || dist2 < 5.0 {
            hordir[0] = (*reach).end[0] - (*ms).origin[0];
            hordir[1] = (*reach).end[1] - (*ms).origin[1];
            hordir[2] = 0.0;
            VectorNormalize(&mut hordir);
            //elemantary action jump
            if dist1 < 24.0 {
                EA_Jump(bot, (*ms).client);
            } else if dist1 < 32.0 {
                EA_DelayedJump(bot, (*ms).client);
            }
            EA_Move(bot, (*ms).client, hordir, 600.0);
            //
            (*ms).jumpreach = (*ms).lastreachnum;
        } else {
            hordir[0] = runstart[0] - (*ms).origin[0];
            hordir[1] = runstart[1] - (*ms).origin[1];
            hordir[2] = 0.0;
            VectorNormalize(&mut hordir);
            //
            if dist2 > 80.0 {
                dist2 = 80.0;
            }
            let speed = 400.0 - (400.0 - 5.0 * dist2);
            EA_Move(bot, (*ms).client, hordir, speed);
        }
        _VectorCopy(hordir, &mut result.movedir);
        //
        result
    }
}

/// Raven `BotMovementViewTarget`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:826-874`
pub fn BotMovementViewTarget(
    bot: &mut BotLib,
    movestate: c_int,
    goal: *mut bot_goal_t,
    travelflags: c_int,
    lookahead: f32,
    target: *mut vec3_t,
) -> c_int {
    unsafe {
        let mut reach: aas_reachability_t = core::mem::zeroed();
        let mut reachnum: c_int;
        let mut lastareanum: c_int;
        let ms: *mut bot_movestate_t;
        let mut end: vec3_t = [0.0; 3];
        let mut dist: f32;

        ms = BotMoveStateFromHandle(bot, movestate);
        if ms.is_null() {
            return qfalse;
        }
        reachnum = 0;
        //if the bot has no goal or no last reachability
        if (*ms).lastreachnum == 0 || goal.is_null() {
            return qfalse;
        }

        reachnum = (*ms).lastreachnum;
        _VectorCopy((*ms).origin, &mut end);
        lastareanum = (*ms).lastareanum;
        dist = 0.0;
        while reachnum != 0 && dist < lookahead {
            AAS_ReachabilityFromNum(bot, reachnum, &mut reach);
            if BotAddToTarget(end, reach.start, lookahead, &mut dist, target) != 0 {
                return qtrue;
            }
            //never look beyond teleporters
            if (reach.traveltype & TRAVELTYPE_MASK) == TRAVEL_TELEPORT {
                return qtrue;
            }
            //never look beyond the weapon jump point
            if (reach.traveltype & TRAVELTYPE_MASK) == TRAVEL_ROCKETJUMP {
                return qtrue;
            }
            if (reach.traveltype & TRAVELTYPE_MASK) == TRAVEL_BFGJUMP {
                return qtrue;
            }
            //don't add jump pad distances
            if (reach.traveltype & TRAVELTYPE_MASK) != TRAVEL_JUMPPAD
                && (reach.traveltype & TRAVELTYPE_MASK) != TRAVEL_ELEVATOR
                && (reach.traveltype & TRAVELTYPE_MASK) != TRAVEL_FUNCBOB
            {
                if BotAddToTarget(reach.start, reach.end, lookahead, &mut dist, target) != 0 {
                    return qtrue;
                }
            }
            reachnum = BotGetReachabilityToGoal(
                bot,
                reach.end,
                reach.areanum,
                (*ms).lastgoalareanum,
                lastareanum,
                (*ms).avoidreach.as_mut_ptr(),
                (*ms).avoidreachtimes.as_mut_ptr(),
                (*ms).avoidreachtries.as_mut_ptr(),
                goal,
                travelflags,
                travelflags,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
            _VectorCopy(reach.end, &mut end);
            lastareanum = reach.areanum;
            if lastareanum == (*goal).areanum {
                BotAddToTarget(reach.end, (*goal).origin, lookahead, &mut dist, target);
                return qtrue;
            }
        }
        //
        qfalse
    }
}

/// Raven `BotPredictVisiblePosition`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:895-951`
pub fn BotPredictVisiblePosition(
    bot: &mut BotLib,
    origin: vec3_t,
    mut areanum: c_int,
    goal: *mut bot_goal_t,
    travelflags: c_int,
    target: *mut vec3_t,
) -> c_int {
    unsafe {
        let mut reach: aas_reachability_t = core::mem::zeroed();
        let mut reachnum: c_int;
        let lastgoalareanum: c_int;
        let mut lastareanum: c_int;
        let mut i: c_int;
        let mut avoidreach: [c_int; MAX_AVOIDREACH] = [0; MAX_AVOIDREACH];
        let mut avoidreachtimes: [f32; MAX_AVOIDREACH] = [0.0; MAX_AVOIDREACH];
        let mut avoidreachtries: [c_int; MAX_AVOIDREACH] = [0; MAX_AVOIDREACH];
        let mut end: vec3_t = [0.0; 3];

        //if the bot has no goal or no last reachability
        if goal.is_null() {
            return qfalse;
        }
        //if the areanum is not valid
        if areanum == 0 {
            return qfalse;
        }
        //if the goal areanum is not valid
        if (*goal).areanum == 0 {
            return qfalse;
        }

        Com_Memset(
            avoidreach.as_mut_ptr() as *mut (),
            0,
            MAX_AVOIDREACH * core::mem::size_of::<c_int>(),
        );
        lastgoalareanum = (*goal).areanum;
        lastareanum = areanum;
        _VectorCopy(origin, &mut end);
        //only do 20 hops
        i = 0;
        while i < 20 && (areanum != (*goal).areanum) {
            //
            reachnum = BotGetReachabilityToGoal(
                bot,
                end,
                areanum,
                lastgoalareanum,
                lastareanum,
                avoidreach.as_mut_ptr(),
                avoidreachtimes.as_mut_ptr(),
                avoidreachtries.as_mut_ptr(),
                goal,
                travelflags,
                travelflags,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
            if reachnum == 0 {
                return qfalse;
            }
            AAS_ReachabilityFromNum(bot, reachnum, &mut reach);
            //
            if BotVisible(bot, (*goal).entitynum, (*goal).origin, reach.start) != 0 {
                _VectorCopy(reach.start, &mut *target);
                return qtrue;
            }
            //
            if BotVisible(bot, (*goal).entitynum, (*goal).origin, reach.end) != 0 {
                _VectorCopy(reach.end, &mut *target);
                return qtrue;
            }
            //
            if reach.areanum == (*goal).areanum {
                _VectorCopy(reach.end, &mut *target);
                return qtrue;
            }
            //
            lastareanum = areanum;
            areanum = reach.areanum;
            _VectorCopy(reach.end, &mut end);
            //
            i += 1;
        }
        //
        qfalse
    }
}

/// Raven `BotMoveToGoal`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:3038-3491`
pub fn BotMoveToGoal(
    common: &mut Common,
    bot: &mut BotLib,
    result: *mut bot_moveresult_t,
    movestate: c_int,
    goal: *mut bot_goal_t,
    travelflags: c_int,
) {
    unsafe {
        let mut reachnum: c_int;
        let mut lastreachnum: c_int;
        let mut foundjumppad: c_int;
        let mut ent: c_int;
        let mut resultflags: c_int;
        let mut reach: aas_reachability_t = core::mem::zeroed();
        let mut lastreach: aas_reachability_t = core::mem::zeroed();
        let ms: *mut bot_movestate_t;

        BotClearMoveResult(result);
        //
        ms = BotMoveStateFromHandle(bot, movestate);
        if ms.is_null() {
            return;
        }
        //reset the grapple before testing if the bot has a valid goal
        BotResetGrapple(bot, ms);
        //
        if goal.is_null() {
            (*result).failure = qtrue;
            return;
        }
        //remove some of the move flags
        (*ms).moveflags &= !(MFL_SWIMMING | MFL_AGAINSTLADDER);
        //set some of the move flags
        //NOTE: the MFL_ONGROUND flag is also set in the higher AI
        if AAS_OnGround(bot, (*ms).origin, (*ms).presencetype, (*ms).entitynum) != 0 {
            (*ms).moveflags |= MFL_ONGROUND;
        }
        //
        if (*ms).moveflags & MFL_ONGROUND != 0 {
            let modeltype: c_int;
            let modelnum: c_int;

            ent = BotOnTopOfEntity(bot, ms);

            if ent != -1 {
                modelnum = AAS_EntityModelindex(bot, ent);
                if modelnum >= 0 && modelnum < MAX_MODELS {
                    modeltype = bot.modeltypes[modelnum as usize];

                    if modeltype == MODELTYPE_FUNC_PLAT {
                        AAS_ReachabilityFromNum(bot, (*ms).lastreachnum, &mut reach);
                        //if the bot is Not using the elevator
                        if (reach.traveltype & TRAVELTYPE_MASK) != TRAVEL_ELEVATOR ||
                            //NOTE: the face number is the plat model number
                            (reach.facenum & 0x0000FFFF) != modelnum
                        {
                            reachnum = AAS_NextModelReachability(bot, 0, modelnum);
                            if reachnum != 0 {
                                AAS_ReachabilityFromNum(bot, reachnum, &mut reach);
                                (*ms).lastreachnum = reachnum;
                                (*ms).reachability_time =
                                    AAS_Time(bot) + BotReachabilityTime(bot, &mut reach) as f32;
                            } else {
                                if bot.bot_developer != 0 {
                                    (bot.botimport.Print.unwrap())(
                                        PRT_MESSAGE,
                                        c"client %d: on func_plat without reachability\n".as_ptr()
                                            as *mut c_char,
                                        (*ms).client,
                                    );
                                }
                                (*result).blocked = qtrue;
                                (*result).blockentity = ent;
                                (*result).flags |= MOVERESULT_ONTOPOFOBSTACLE;
                                return;
                            }
                        }
                        (*result).flags |= MOVERESULT_ONTOPOF_ELEVATOR;
                    } else if modeltype == MODELTYPE_FUNC_BOB {
                        AAS_ReachabilityFromNum(bot, (*ms).lastreachnum, &mut reach);
                        //if the bot is Not using the func bobbing
                        if (reach.traveltype & TRAVELTYPE_MASK) != TRAVEL_FUNCBOB ||
                            //NOTE: the face number is the func_bobbing model number
                            (reach.facenum & 0x0000FFFF) != modelnum
                        {
                            reachnum = AAS_NextModelReachability(bot, 0, modelnum);
                            if reachnum != 0 {
                                AAS_ReachabilityFromNum(bot, reachnum, &mut reach);
                                (*ms).lastreachnum = reachnum;
                                (*ms).reachability_time =
                                    AAS_Time(bot) + BotReachabilityTime(bot, &mut reach) as f32;
                            } else {
                                if bot.bot_developer != 0 {
                                    (bot.botimport.Print.unwrap())(
                                        PRT_MESSAGE,
                                        c"client %d: on func_bobbing without reachability\n"
                                            .as_ptr()
                                            as *mut c_char,
                                        (*ms).client,
                                    );
                                }
                                (*result).blocked = qtrue;
                                (*result).blockentity = ent;
                                (*result).flags |= MOVERESULT_ONTOPOFOBSTACLE;
                                return;
                            }
                        }
                        (*result).flags |= MOVERESULT_ONTOPOF_FUNCBOB;
                    } else if modeltype == MODELTYPE_FUNC_STATIC || modeltype == MODELTYPE_FUNC_DOOR
                    {
                        // check if ontop of a door bridge ?
                        (*ms).areanum = BotFuzzyPointReachabilityArea(bot, (*ms).origin);
                        // if not in a reachability area
                        if AAS_AreaReachability(bot, (*ms).areanum) == 0 {
                            (*result).blocked = qtrue;
                            (*result).blockentity = ent;
                            (*result).flags |= MOVERESULT_ONTOPOFOBSTACLE;
                            return;
                        }
                    } else {
                        (*result).blocked = qtrue;
                        (*result).blockentity = ent;
                        (*result).flags |= MOVERESULT_ONTOPOFOBSTACLE;
                        return;
                    }
                }
            }
        }
        //if swimming
        if AAS_Swimming(bot, (*ms).origin) != 0 {
            (*ms).moveflags |= MFL_SWIMMING;
        }
        //if against a ladder
        if AAS_AgainstLadder(bot, (*ms).origin) != 0 {
            (*ms).moveflags |= MFL_AGAINSTLADDER;
        }
        //if the bot is on the ground, swimming or against a ladder
        if (*ms).moveflags & (MFL_ONGROUND | MFL_SWIMMING | MFL_AGAINSTLADDER) != 0 {
            //
            AAS_ReachabilityFromNum(bot, (*ms).lastreachnum, &mut lastreach);
            //reachability area the bot is in
            (*ms).areanum = BotFuzzyPointReachabilityArea(bot, (*ms).origin);
            //
            if (*ms).areanum == 0 {
                (*result).failure = qtrue;
                (*result).blocked = qtrue;
                (*result).blockentity = 0;
                (*result).r#type = RESULTTYPE_INSOLIDAREA;
                return;
            }
            //if the bot is in the goal area
            if (*ms).areanum == (*goal).areanum {
                *result = BotMoveInGoalArea(bot, ms, goal);
                return;
            }
            //assume we can use the reachability from the last frame
            reachnum = (*ms).lastreachnum;
            //if there is a last reachability
            if reachnum != 0 {
                AAS_ReachabilityFromNum(bot, reachnum, &mut reach);
                //check if the reachability is still valid
                if AAS_TravelFlagForType(bot, reach.traveltype) & travelflags == 0 {
                    reachnum = 0;
                }
                //special grapple hook case
                else if (reach.traveltype & TRAVELTYPE_MASK) == TRAVEL_GRAPPLEHOOK {
                    if (*ms).reachability_time < AAS_Time(bot)
                        || ((*ms).moveflags & MFL_GRAPPLERESET) != 0
                    {
                        reachnum = 0;
                    }
                }
                //special elevator case
                else if (reach.traveltype & TRAVELTYPE_MASK) == TRAVEL_ELEVATOR
                    || (reach.traveltype & TRAVELTYPE_MASK) == TRAVEL_FUNCBOB
                {
                    if ((*result).flags & MOVERESULT_ONTOPOF_FUNCBOB) != 0
                        || ((*result).flags & MOVERESULT_ONTOPOF_FUNCBOB) != 0
                    {
                        (*ms).reachability_time = AAS_Time(bot) + 5.0;
                    }
                    //if the bot was going for an elevator and reached the reachability area
                    if (*ms).areanum == reach.areanum || (*ms).reachability_time < AAS_Time(bot) {
                        reachnum = 0;
                    }
                } else {
                    //if the goal area changed or the reachability timed out
                    //or the area changed
                    if (*ms).lastgoalareanum != (*goal).areanum
                        || (*ms).reachability_time < AAS_Time(bot)
                        || (*ms).lastareanum != (*ms).areanum
                    {
                        reachnum = 0;
                    }
                }
            }
            resultflags = 0;
            //if the bot needs a new reachability
            if reachnum == 0 {
                //if the area has no reachability links
                if AAS_AreaReachability(bot, (*ms).areanum) == 0 {}
                //get a new reachability leading towards the goal
                reachnum = BotGetReachabilityToGoal(
                    bot,
                    (*ms).origin,
                    (*ms).areanum,
                    (*ms).lastgoalareanum,
                    (*ms).lastareanum,
                    (*ms).avoidreach.as_mut_ptr(),
                    (*ms).avoidreachtimes.as_mut_ptr(),
                    (*ms).avoidreachtries.as_mut_ptr(),
                    goal,
                    travelflags,
                    travelflags,
                    (*ms).avoidspots.as_mut_ptr(),
                    (*ms).numavoidspots,
                    &mut resultflags,
                );
                //the area number the reachability starts in
                (*ms).reachareanum = (*ms).areanum;
                //reset some state variables
                (*ms).jumpreach = 0; //for TRAVEL_JUMP
                (*ms).moveflags &= !MFL_GRAPPLERESET; //for TRAVEL_GRAPPLEHOOK
                                                      //if there is a reachability to the goal
                if reachnum != 0 {
                    AAS_ReachabilityFromNum(bot, reachnum, &mut reach);
                    //set a timeout for this reachability
                    (*ms).reachability_time =
                        AAS_Time(bot) + BotReachabilityTime(bot, &mut reach) as f32;
                    //
                    // AVOIDREACH is defined; add the reachability to avoid for a while
                    BotAddToAvoidReach(bot, ms, reachnum, AVOIDREACH_TIME as f32);
                }
            }
            //
            (*ms).lastreachnum = reachnum;
            (*ms).lastgoalareanum = (*goal).areanum;
            (*ms).lastareanum = (*ms).areanum;
            //if the bot has a reachability
            if reachnum != 0 {
                //get the reachability from the number
                AAS_ReachabilityFromNum(bot, reachnum, &mut reach);
                (*result).traveltype = reach.traveltype;
                //
                match reach.traveltype & TRAVELTYPE_MASK {
                    TRAVEL_WALK => *result = BotTravel_Walk(bot, ms, &mut reach),
                    TRAVEL_CROUCH => *result = BotTravel_Crouch(bot, ms, &mut reach),
                    TRAVEL_BARRIERJUMP => *result = BotTravel_BarrierJump(bot, ms, &mut reach),
                    TRAVEL_LADDER => *result = BotTravel_Ladder(bot, ms, &mut reach),
                    TRAVEL_WALKOFFLEDGE => *result = BotTravel_WalkOffLedge(bot, ms, &mut reach),
                    TRAVEL_JUMP => *result = BotTravel_Jump(common, bot, ms, &mut reach),
                    TRAVEL_SWIM => *result = BotTravel_Swim(bot, ms, &mut reach),
                    TRAVEL_WATERJUMP => *result = BotTravel_WaterJump(common, bot, ms, &mut reach),
                    TRAVEL_TELEPORT => *result = BotTravel_Teleport(bot, ms, &mut reach),
                    TRAVEL_ELEVATOR => *result = BotTravel_Elevator(bot, ms, &mut reach),
                    TRAVEL_GRAPPLEHOOK => *result = BotTravel_Grapple(bot, ms, &mut reach),
                    TRAVEL_ROCKETJUMP => *result = BotTravel_RocketJump(bot, ms, &mut reach),
                    TRAVEL_BFGJUMP => *result = BotTravel_BFGJump(bot, ms, &mut reach),
                    TRAVEL_JUMPPAD => *result = BotTravel_JumpPad(bot, ms, &mut reach),
                    TRAVEL_FUNCBOB => *result = BotTravel_FuncBobbing(bot, ms, &mut reach),
                    _ => {
                        (bot.botimport.Print.unwrap())(
                            PRT_FATAL,
                            c"travel type %d not implemented yet\n".as_ptr() as *mut c_char,
                            reach.traveltype & TRAVELTYPE_MASK,
                        );
                    }
                }
                (*result).traveltype = reach.traveltype;
                (*result).flags |= resultflags;
            } else {
                (*result).failure = qtrue;
                (*result).flags |= resultflags;
                Com_Memset(
                    &mut reach as *mut aas_reachability_t as *mut (),
                    0,
                    core::mem::size_of::<aas_reachability_t>(),
                );
            }
        } else {
            let mut i: c_int;
            let mut numareas: c_int;
            let mut areas: [c_int; 16] = [0; 16];
            let mut end: vec3_t = [0.0; 3];

            //special handling of jump pads when the bot uses a jump pad without knowing it
            foundjumppad = qfalse;
            _VectorMA(
                (*ms).origin,
                -2.0 * (*ms).thinktime,
                (*ms).velocity,
                &mut end,
            );
            numareas = AAS_TraceAreas(
                bot,
                (*ms).origin,
                end,
                areas.as_mut_ptr(),
                core::ptr::null_mut(),
                16,
            );
            i = numareas - 1;
            while i >= 0 {
                if AAS_AreaJumpPad(bot, areas[i as usize]) != 0 {
                    foundjumppad = qtrue;
                    lastreachnum = BotGetReachabilityToGoal(
                        bot,
                        end,
                        areas[i as usize],
                        (*ms).lastgoalareanum,
                        (*ms).lastareanum,
                        (*ms).avoidreach.as_mut_ptr(),
                        (*ms).avoidreachtimes.as_mut_ptr(),
                        (*ms).avoidreachtries.as_mut_ptr(),
                        goal,
                        travelflags,
                        TFL_JUMPPAD,
                        (*ms).avoidspots.as_mut_ptr(),
                        (*ms).numavoidspots,
                        core::ptr::null_mut(),
                    );
                    if lastreachnum != 0 {
                        (*ms).lastreachnum = lastreachnum;
                        (*ms).lastareanum = areas[i as usize];
                        break;
                    } else {
                        lastreachnum = AAS_NextAreaReachability(bot, areas[i as usize], 0);
                        while lastreachnum != 0 {
                            //get the reachability from the number
                            AAS_ReachabilityFromNum(bot, lastreachnum, &mut reach);
                            if (reach.traveltype & TRAVELTYPE_MASK) == TRAVEL_JUMPPAD {
                                (*ms).lastreachnum = lastreachnum;
                                (*ms).lastareanum = areas[i as usize];
                                break;
                            }
                            lastreachnum =
                                AAS_NextAreaReachability(bot, areas[i as usize], lastreachnum);
                        }
                        if lastreachnum != 0 {
                            break;
                        }
                    }
                }
                i -= 1;
            }
            if bot.bot_developer != 0 {
                //if a jumppad is found with the trace but no reachability is found
                if foundjumppad != 0 && (*ms).lastreachnum == 0 {
                    (bot.botimport.Print.unwrap())(
                        PRT_MESSAGE,
                        c"client %d didn't find jumppad reachability\n".as_ptr() as *mut c_char,
                        (*ms).client,
                    );
                }
            }
            //
            if (*ms).lastreachnum != 0 {
                AAS_ReachabilityFromNum(bot, (*ms).lastreachnum, &mut reach);
                (*result).traveltype = reach.traveltype;
                //
                match reach.traveltype & TRAVELTYPE_MASK {
                    TRAVEL_WALK => *result = BotTravel_Walk(bot, ms, &mut reach),
                    TRAVEL_CROUCH => { /*do nothing*/ }
                    TRAVEL_BARRIERJUMP => {
                        *result = BotFinishTravel_BarrierJump(bot, ms, &mut reach)
                    }
                    TRAVEL_LADDER => *result = BotTravel_Ladder(bot, ms, &mut reach),
                    TRAVEL_WALKOFFLEDGE => {
                        *result = BotFinishTravel_WalkOffLedge(bot, ms, &mut reach)
                    }
                    TRAVEL_JUMP => *result = BotFinishTravel_Jump(bot, ms, &mut reach),
                    TRAVEL_SWIM => *result = BotTravel_Swim(bot, ms, &mut reach),
                    TRAVEL_WATERJUMP => {
                        *result = BotFinishTravel_WaterJump(common, bot, ms, &mut reach)
                    }
                    TRAVEL_TELEPORT => { /*do nothing*/ }
                    TRAVEL_ELEVATOR => *result = BotFinishTravel_Elevator(bot, ms, &mut reach),
                    TRAVEL_GRAPPLEHOOK => *result = BotTravel_Grapple(bot, ms, &mut reach),
                    TRAVEL_ROCKETJUMP | TRAVEL_BFGJUMP => {
                        *result = BotFinishTravel_WeaponJump(bot, ms, &mut reach)
                    }
                    TRAVEL_JUMPPAD => *result = BotFinishTravel_JumpPad(bot, ms, &mut reach),
                    TRAVEL_FUNCBOB => *result = BotFinishTravel_FuncBobbing(bot, ms, &mut reach),
                    _ => {
                        (bot.botimport.Print.unwrap())(
                            PRT_FATAL,
                            c"(last) travel type %d not implemented yet\n".as_ptr() as *mut c_char,
                            reach.traveltype & TRAVELTYPE_MASK,
                        );
                    }
                }
                (*result).traveltype = reach.traveltype;
            }
        }
        //FIXME: is it right to do this here?
        if (*result).blocked != 0 {
            (*ms).reachability_time -= 10.0 * (*ms).thinktime;
        }
        //copy the last origin
        _VectorCopy((*ms).origin, &mut (*ms).lastorigin);
        //return the movement result
    }
}
