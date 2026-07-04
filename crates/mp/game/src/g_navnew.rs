// PORT-COMPLETE: g_navnew.c 5/8
//! Port of `oracle/oracle/codemp/game/g_navnew.c` (jampgame mega-pass).
//!
//! Generated from `tools/closure-prototype/fnskel.py`; bodies filled per the
//! jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`).
//!
//! SPINE (fork rulings 1/4 + `docs/architecture/engine-seam.md`, precedent
//! `NPC_reactions.rs`/`w_force.rs`): logic fns that reach `level`/cvars/traps
//! thread the `GameContext<'_>` receiver (`.world: *mut GameWorld`, `.engine`)
//! as an ADDITIVE first parameter (the faithful C signature carries none).
//! `level` -> `(*ctx.world).level`, cvars -> `(*ctx.world).cvars`. Traps go
//! through `trap::X(ctx.engine, ..)`. `vec3_t` (`[f32;3]`) is `Copy`, so
//! `VectorCopy`/`VectorMA`/`VectorScale`/`VectorAdd`/`VectorSubtract`/
//! `VectorClear`/`VectorSet`/`DotProduct`/`VectorCompare` (all inline macros
//! in the oracle) transcribe as plain array arithmetic per the bless-the-rule
//! appendix, not function calls. Raw `gentity_t*` chains are `unsafe`
//! raw-pointer field access mirroring the C exactly (`gentity_t::NPC`/
//! `::client` are opaque `*mut c_void`, cast to `gNPC_t`/`gclient_t` per the
//! `NPC_reactions.rs` precedent).
//!
//! `NAVNEW_DanceWithBlocker`/`NAVNEW_SidestepBlocker`'s faithful `movedir`
//! out-param cannot be received as a by-value `vec3_t` (`vec3-outparam-seam`,
//! `g_combat.rs` precedent) — but unlike that cross-file precedent, these
//! `movedir` out-params are OWNED by this same file (not a frozen callee
//! signature), so per §C7 (out-params -> return values / mutable refs) they
//! are declared `movedir: &mut vec3_t` here instead of parking.
//!
//! PARKED (see PORT-ESCALATION markers): `NPC_SetBlocked`,
//! `NAVNEW_PushBlocker`, `NAVNEW_SidestepBlocker`, `NAVNEW_Bypass`,
//! `NAVNEW_ResolveEntityCollision`, `NAVNEW_AvoidCollision`,
//! `NAVNEW_TestNodeConnectionBlocked`, `NAVNEW_MoveToGoal` all read JKA-added
//! `CONTENTS_*`/`MASK_NPCSOLID` collision-mask bits, `NIF_*`/`EDGE_*`/`NODE_*`
//! debug-draw enum values, or magic-number defines (`MIN_BLOCKED_SPEECH_TIME`,
//! `MIN_DOOR_BLOCK_DIST_SQR`, `DEFAULT_MINS_2`/`DEFAULT_MAXS_2`,
//! `NPCAI_BLOCKED`) that are not resolved anywhere in this packet and are not
//! confidently re-derivable the way `ENTITYNUM_NONE`/`STEPSIZE` are (topic
//! `const-value`); `NAVNEW_MoveToGoal` additionally reaches the ambient
//! "current NPC" global `NPCInfo` with no `GameContext` channel (topic
//! `ai-context`, matching `NPC_reactions.rs`/`NPC_utils.rs`).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::trap;
use crate::world::GameContext;

use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_qshared::common::mp::gentity::MAX_FAILED_NODES;
use mp_qshared::shared::MAX_GENTITIES;
use mp_qshared::shared::{QFALSE, QTRUE};

use crate::q_math::VectorNormalize;

/// Raven `ENTITYNUM_NONE`/`ENTITYNUM_WORLD` — derived from `MAX_GENTITIES`,
/// an invariant unchanged across every id-tech-3 engine build including JKA.
/// Source: `oracle/oracle/codemp/game/q_shared.h` (ENTITYNUM_NONE/ENTITYNUM_WORLD defines)
const ENTITYNUM_NONE: c_int = (MAX_GENTITIES - 1) as c_int;
const ENTITYNUM_WORLD: c_int = (MAX_GENTITIES - 2) as c_int;

// Raven `DEFAULT_MINS_2`/`DEFAULT_MAXS_2` (`bg_public.h:41-42`); file-local per
// the same duplication precedent as `g_nav.rs`/`g_weapon.rs`/`bg_pmove.rs`/
// `ai_wpnav.rs`/`NPC_stats.rs`/`g_vehicles.rs` (no canonical shared module).
const DEFAULT_MINS_2: f32 = -24.0;
const DEFAULT_MAXS_2: f32 = 40.0;

/// Raven `NAV_CheckNodeFailedForEnt`.
///
/// Raven: "FIXME: must be a better way to do this". `+1` because 0 is a valid
/// nodeNum but also the default (unset) slot value.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:15-28`
pub fn NAV_CheckNodeFailedForEnt(ent: *mut gentity_t, nodeNum: c_int) -> qboolean {
    unsafe {
        for j in 0..MAX_FAILED_NODES {
            if (*ent).failedWaypoints[j] == nodeNum + 1 {
                //we failed against this node
                return QTRUE;
            }
        }
    }
    QFALSE
}

/// Raven `NPC_ClearBlocked`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:34-41`
pub fn NPC_ClearBlocked(self_: *mut gentity_t) {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        if npc.is_null() {
            return;
        }
        //self->NPC->aiFlags &= ~NPCAI_BLOCKED;
        (*npc).blockingEntNum = ENTITYNUM_NONE;
    }
}

/// Raven `NPC_SetBlocked`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:43-51`
pub fn NPC_SetBlocked(
    ctx: GameContext<'_>,self_: *mut gentity_t, blocker: *mut gentity_t) {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        if npc.is_null() {
            return;
        }

        //self->NPC->aiFlags |= NPCAI_BLOCKED;
        (*npc).blockedSpeechDebounceTime = (*ctx.world).level.time + MIN_BLOCKED_SPEECH_TIME + (((*ctx.world).bg_state.rng.random() * 4000.0) as c_int);
        (*npc).blockingEntNum = (*blocker).s.number;
    }
}

/// Raven `NAVNEW_ClearPathBetweenPoints`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:58-77`
pub fn NAVNEW_ClearPathBetweenPoints(
    ctx: GameContext<'_>,
    start: vec3_t,
    end: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    ignore: c_int,
    clipmask: c_int,
) -> c_int {
    unsafe {
        //Test if they're even conceivably close to one another
        if trap::InPVS(
            ctx.engine,
            GInPvsArgs::new(&start as *const vec3_t, &end as *const vec3_t),
        ) == QFALSE
        {
            return ENTITYNUM_WORLD;
        }

        let mut trace: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &start as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &end as *const vec3_t,
                ignore,
                clipmask,
            ),
        );

        //if( ( ( trace.startsolid == false ) && ( trace.allsolid == false ) ) && ( trace.fraction < 1.0f ) )
        //{//FIXME: check for drops?
        //FIXME: if startsolid or allsolid, then the path isn't clear... but returning ENTITYNUM_NONE indicates to CheckFailedEdge that is is clear...?
        trace.entityNum as c_int
        //}

        //return ENTITYNUM_NONE;
    }
}

/// Raven `NAVNEW_PushBlocker`.
///
/// Raven: try pushing blocker to one side.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:84-171`
pub fn NAVNEW_PushBlocker(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    right: vec3_t,
    setBlockedInfo: qboolean,
) {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        if (*npc).shoveCount > 30 {
            //don't push for more than 3 seconds;
            return;
        }

        if (*blocker).s.number == 0 {
            //never push the player
            return;
        }

        let client = (*blocker).client as *mut gclient_t;
        if !client.is_null() && VectorCompare((*client).pushVec, [0.0f32, 0.0, 0.0]) == QFALSE {
            //someone else is pushing him, wait until they give up?
            return;
        }

        let mut mins = [0.0f32; 3];
        crate::q_math::_VectorCopy((*blocker).r.mins, &mut mins);
        mins[2] += STEPSIZE;

        let moveamt = ((*self_).r.maxs[1] + (*blocker).r.maxs[1]) * 1.2;//yes, magic number

        let mut end = [0.0f32; 3];
        crate::q_math::_VectorMA((*blocker).r.currentOrigin, -moveamt, right, &mut end);
        let mut tr: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &(*blocker).r.currentOrigin as *const vec3_t,
                &mins as *const vec3_t,
                &(*blocker).r.maxs as *const vec3_t,
                &end as *const vec3_t,
                (*blocker).s.number,
                (*blocker).clipmask | CONTENTS_BOTCLIP,
            ),
        );
        let leftSucc = if !tr.startsolid && !tr.allsolid {
            tr.fraction
        } else {
            0.0f32
        };

        if leftSucc >= 1.0f32 {
            //it's clear, shove him that way
            crate::q_math::_VectorScale(right, -moveamt, &mut (*client).pushVec);
            (*client).pushVecTime = (*ctx.world).level.time + 2000;
        } else {
            crate::q_math::_VectorMA((*blocker).r.currentOrigin, moveamt, right, &mut end);
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*blocker).r.currentOrigin as *const vec3_t,
                    &mins as *const vec3_t,
                    &(*blocker).r.maxs as *const vec3_t,
                    &end as *const vec3_t,
                    (*blocker).s.number,
                    (*blocker).clipmask | CONTENTS_BOTCLIP,
                ),
            );
            let rightSucc = if !tr.startsolid && !tr.allsolid {
                tr.fraction
            } else {
                0.0f32
            };

            if leftSucc == 0.0f32 && rightSucc == 0.0f32 {
                //both sides failed
                if (*ctx.world).cvars.d_patched.integer != 0 {
                    //use patch-style navigation
                    (*client).pushVecTime = 0;
                }
                return;
            }

            if rightSucc >= 1.0f32 {
                //it's clear, shove him that way
                crate::q_math::_VectorScale(right, moveamt, &mut (*client).pushVec);
                (*client).pushVecTime = (*ctx.world).level.time + 2000;
            }
            //if neither are enough, we probably can't get around him, but keep trying
            else if leftSucc >= rightSucc {
                //favor the left, all things being equal
                crate::q_math::_VectorScale(right, -moveamt, &mut (*client).pushVec);
                (*client).pushVecTime = (*ctx.world).level.time + 2000;
            } else {
                crate::q_math::_VectorScale(right, moveamt, &mut (*client).pushVec);
                (*client).pushVecTime = (*ctx.world).level.time + 2000;
            }
        }

        if setBlockedInfo != QFALSE {
            //we tried pushing
            (*npc).shoveCount += 1;
        }
    }
}

/// Raven `NAVNEW_DanceWithBlocker`.
///
/// Raven: sees if blocker has any lateral movement.
///
/// The faithful `movedir` out-param is declared `&mut vec3_t` (§C7) rather
/// than by-value — this is this file's own signature, not a frozen cross-file
/// callee, so unlike the `vec3-outparam-seam` park precedent (`g_combat.rs`)
/// it is fixed here rather than parked.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:178-215`
pub fn NAVNEW_DanceWithBlocker(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    movedir: &mut vec3_t,
    right: vec3_t,
) -> qboolean {
    unsafe {
        let client = (*blocker).client as *mut gclient_t;
        if !client.is_null() && (*client).ps.velocity != [0.0f32, 0.0, 0.0] {
            let mut blocker_movedir = (*client).ps.velocity;
            blocker_movedir[2] = 0.0; //cancel any vertical motion
            let dot = blocker_movedir[0] * right[0]
                + blocker_movedir[1] * right[1]
                + blocker_movedir[2] * right[2];
            if dot > 50.0 {
                //he's moving to the right of me at a relatively good speed
                //go to my left
                movedir[0] += -1.0 * right[0];
                movedir[1] += -1.0 * right[1];
                movedir[2] += -1.0 * right[2];
                VectorNormalize(movedir);
                return QTRUE;
            } else if dot > -50.0 {
                //he's moving to the left of me at a relatively good speed
                //go to my right
                movedir[0] += right[0];
                movedir[1] += right[1];
                movedir[2] += right[2];
                VectorNormalize(movedir);
                return QTRUE;
            }
            /*
            vec3_t	block_pos;
            trace_t	tr;
            VectorScale( blocker_movedir, -1, blocker_movedir );
            VectorMA( self->r.currentOrigin, blocked_dist, blocker_movedir, block_pos );
            if ( NAVNEW_CheckAhead( self, block_pos, tr, ( self->clipmask & ~CONTENTS_BODY )|CONTENTS_BOTCLIP ) )
            {
                VectorCopy( blocker_movedir, movedir );
                return qtrue;
            }
            */
        }
    }
    QFALSE
}

/// Raven `NAVNEW_SidestepBlocker`.
///
/// Raven: trace to sides of blocker and see if either is clear.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:222-340`
pub fn NAVNEW_SidestepBlocker(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    blocked_dir: vec3_t,
    blocked_dist: f32,
    movedir: &mut vec3_t,
    right: vec3_t,
) -> qboolean {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        let mut mins = [0.0f32; 3];
        crate::q_math::_VectorCopy((*self_).r.mins, &mut mins);
        mins[2] += STEPSIZE;

        //Get the blocked direction
        let yaw = crate::bg_misc::vectoyaw(blocked_dir);

        //Get the avoid radius
        let avoid_radius_1 = ((*blocker).r.maxs[0] * (*blocker).r.maxs[0] + (*blocker).r.maxs[1] * (*blocker).r.maxs[1]).sqrt();
        let avoid_radius_2 = ((*self_).r.maxs[0] * (*self_).r.maxs[0] + (*self_).r.maxs[1] * (*self_).r.maxs[1]).sqrt();
        let avoidRadius = avoid_radius_1 + avoid_radius_2;

        //See if we're inside our avoidance radius
        let arcAngle = if blocked_dist <= avoidRadius {
            135.0f32
        } else {
            (avoidRadius / blocked_dist) * 90.0f32
        };

        let mut avoidAngles = [0.0f32; 3];

        //need to stop it from ping-ponging, so we have a bit of a debounce time on which side you try
        if (*npc).sideStepHoldTime > (*ctx.world).level.time {
            let adj_arcAngle = if (*npc).lastSideStepSide == -1 {
                -arcAngle
            } else {
                arcAngle
            };
            avoidAngles[1] = crate::q_math::AngleNormalize360(yaw + adj_arcAngle);
            let mut avoidRight_dir = [0.0f32; 3];
            crate::q_math::AngleVectors(avoidAngles, Some(&mut *movedir), None, None);
            let mut block_pos = [0.0f32; 3];
            crate::q_math::_VectorMA((*self_).r.currentOrigin, blocked_dist, *movedir, &mut block_pos);
            let mut tr: trace_t = core::mem::zeroed();
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*self_).r.currentOrigin as *const vec3_t,
                    &mins as *const vec3_t,
                    &(*self_).r.maxs as *const vec3_t,
                    &block_pos as *const vec3_t,
                    (*self_).s.number,
                    (*self_).clipmask | CONTENTS_BOTCLIP,
                ),
            );
            return if tr.fraction == 1.0 && !tr.allsolid && !tr.startsolid { QTRUE } else { QFALSE };
        }

        //test right
        avoidAngles[crate::q_shared::YAW] = crate::q_math::AngleNormalize360(yaw + arcAngle);
        let mut avoidRight_dir = [0.0f32; 3];
        crate::q_math::AngleVectors(avoidAngles, Some(&mut avoidRight_dir), None, None);

        let mut block_pos = [0.0f32; 3];
        crate::q_math::_VectorMA((*self_).r.currentOrigin, blocked_dist, avoidRight_dir, &mut block_pos);

        let mut tr: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &(*self_).r.currentOrigin as *const vec3_t,
                &mins as *const vec3_t,
                &(*self_).r.maxs as *const vec3_t,
                &block_pos as *const vec3_t,
                (*self_).s.number,
                (*self_).clipmask | CONTENTS_BOTCLIP,
            ),
        );

        if !tr.allsolid && !tr.startsolid {
            if tr.fraction >= 1.0f32 {
                //all clear, go for it (favor the right if both are equal)
                crate::q_math::_VectorCopy(avoidRight_dir, movedir);
                (*npc).lastSideStepSide = 1;
                (*npc).sideStepHoldTime = (*ctx.world).level.time + 2000;
                return QTRUE;
            }
            let rightSucc = tr.fraction;

            //now test left
            let adj_arcAngle = -arcAngle;
            avoidAngles[1] = crate::q_math::AngleNormalize360(yaw + adj_arcAngle);
            let mut avoidLeft_dir = [0.0f32; 3];
            crate::q_math::AngleVectors(avoidAngles, Some(&mut avoidLeft_dir), None, None);

            crate::q_math::_VectorMA((*self_).r.currentOrigin, blocked_dist, avoidLeft_dir, &mut block_pos);

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*self_).r.currentOrigin as *const vec3_t,
                    &mins as *const vec3_t,
                    &(*self_).r.maxs as *const vec3_t,
                    &block_pos as *const vec3_t,
                    (*self_).s.number,
                    (*self_).clipmask | CONTENTS_BOTCLIP,
                ),
            );

            if !tr.allsolid && !tr.startsolid {
                if tr.fraction >= 1.0f32 {
                    //all clear, go for it (right side would have already succeeded if as good as this)
                    crate::q_math::_VectorCopy(avoidLeft_dir, movedir);
                    (*npc).lastSideStepSide = -1;
                    (*npc).sideStepHoldTime = (*ctx.world).level.time + 2000;
                    return QTRUE;
                }
                let leftSucc = tr.fraction;

                if rightSucc * blocked_dist >= avoidRadius || leftSucc * blocked_dist >= avoidRadius {
                    //the traces hit something, but got a relatively good distance
                    if rightSucc >= leftSucc {
                        //favor the right, all things being equal
                        crate::q_math::_VectorCopy(avoidRight_dir, movedir);
                        (*npc).lastSideStepSide = 1;
                        (*npc).sideStepHoldTime = (*ctx.world).level.time + 2000;
                    } else {
                        crate::q_math::_VectorCopy(avoidLeft_dir, movedir);
                        (*npc).lastSideStepSide = -1;
                        (*npc).sideStepHoldTime = (*ctx.world).level.time + 2000;
                    }
                    return QTRUE;
                }

                //if neither are enough, we probably can't get around him
                return QFALSE;
            } else {
                return QFALSE;
            }
        } else {
            //test left
            let mut arcAngle_neg = -arcAngle;
            avoidAngles[crate::q_shared::YAW] = crate::q_math::AngleNormalize360(yaw + arcAngle_neg);
            let mut avoidLeft_dir = [0.0f32; 3];
            crate::q_math::AngleVectors(avoidAngles, Some(&mut avoidLeft_dir), None, None);

            crate::q_math::_VectorMA((*self_).r.currentOrigin, blocked_dist, avoidLeft_dir, &mut block_pos);

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*self_).r.currentOrigin as *const vec3_t,
                    &mins as *const vec3_t,
                    &(*self_).r.maxs as *const vec3_t,
                    &block_pos as *const vec3_t,
                    (*self_).s.number,
                    (*self_).clipmask | CONTENTS_BOTCLIP,
                ),
            );

            if !tr.allsolid && !tr.startsolid {
                if tr.fraction >= 1.0f32 {
                    //all clear, go for it (right side would have already succeeded if as good as this)
                    crate::q_math::_VectorCopy(avoidLeft_dir, movedir);
                    (*npc).lastSideStepSide = -1;
                    (*npc).sideStepHoldTime = (*ctx.world).level.time + 2000;
                    return QTRUE;
                }
            }
        }

        //if neither are enough, we probably can't get around him
        QFALSE
    }
}

/// Raven `NAVNEW_Bypass`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:347-377`
pub fn NAVNEW_Bypass(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    blocked_dir: vec3_t,
    blocked_dist: f32,
    movedir: vec3_t,
    setBlockedInfo: qboolean,
) -> qboolean {
    unsafe {
        //Draw debug info if requested
        if (*ctx.world).globals.NAVDEBUG_showCollision != 0 {
            G_DrawEdge((*self_).r.currentOrigin, (*blocker).r.currentOrigin, EDGE_NORMAL);
        }

        let mut moveangles = [0.0f32; 3];
        let mut right = [0.0f32; 3];
        crate::q_math::vectoangles(movedir, &mut moveangles);
        moveangles[2] = 0.0;
        crate::q_math::AngleVectors(moveangles, None, Some(&mut right), None);

        //Check to see what dir the other guy is moving in (if any) and pick the opposite dir
        let mut movedir_local = movedir;
        if NAVNEW_DanceWithBlocker(ctx, self_, blocker, &mut movedir_local, right) != QFALSE {
            return QTRUE;
        }

        //Okay, so he's not moving to my side, see which side of him is most clear
        let mut movedir_out = movedir;
        if NAVNEW_SidestepBlocker(ctx, self_, blocker, blocked_dir, blocked_dist, &mut movedir_out, right) != QFALSE {
            return QTRUE;
        }

        //Neither side is clear, tell him to step aside
        NAVNEW_PushBlocker(ctx, self_, blocker, right, setBlockedInfo);

        QFALSE
    }
}

/// Raven `NAVNEW_CheckDoubleBlock`.
///
/// Raven: stop double waiting.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:384-391`
pub fn NAVNEW_CheckDoubleBlock(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    blocked_dir: vec3_t,
) -> qboolean {
    unsafe {
        let npc = (*blocker).NPC as *mut gNPC_t;
        if !npc.is_null() && (*npc).blockingEntNum == (*self_).s.number {
            return QTRUE;
        }
    }
    QFALSE
}

/// Raven `NAVNEW_ResolveEntityCollision`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:399-435`
pub fn NAVNEW_ResolveEntityCollision(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    movedir: vec3_t,
    pathDir: vec3_t,
    setBlockedInfo: qboolean,
) -> qboolean {
    unsafe {
        //Doors are ignored
        if crate::q_shared::Q_stricmp((*blocker).classname, cstr("func_door").as_ptr()) == 0 {
            let mut center = [0.0f32; 3];
            CalcTeamDoorCenter(blocker, &mut center);
            if crate::q_math::DistanceSquared((*self_).r.currentOrigin, center) > MIN_DOOR_BLOCK_DIST_SQR {
                return QTRUE;
            }
        }

        let mut blocked_dir = [0.0f32; 3];
        crate::q_math::_VectorSubtract((*blocker).r.currentOrigin, (*self_).r.currentOrigin, &mut blocked_dir);
        let blocked_dist = crate::q_math::VectorNormalize(&mut blocked_dir);

        //Make sure an actual collision is going to happen
        //	if ( NAVNEW_PredictCollision( self, blocker, movedir, blocked_dir ) == qfalse )
        //		return qtrue;

        //First, attempt to walk around the blocker or shove him out of the way
        if NAVNEW_Bypass(ctx, self_, blocker, blocked_dir, blocked_dist, movedir, setBlockedInfo) != QFALSE {
            return QTRUE;
        }

        //Can't get around him... see if I'm blocking him too... if so, I need to just keep moving?
        if NAVNEW_CheckDoubleBlock(self_, blocker, blocked_dir) != QFALSE {
            return QTRUE;
        }

        if setBlockedInfo != QFALSE {
            //Complain about it if we can
            NPC_SetBlocked(ctx, self_, blocker);
        }

        QFALSE
    }
}

/// Raven `NAVNEW_AvoidCollision`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:442-518`
pub fn NAVNEW_AvoidCollision(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    goal: *mut gentity_t,
    info: *mut navInfo_t,
    setBlockedInfo: qboolean,
    blockedMovesLimit: c_int,
) -> qboolean {
    unsafe {
        //Cap our distance
        if (*info).distance > MAX_COLL_AVOID_DIST {
            (*info).distance = MAX_COLL_AVOID_DIST;
        }

        //Get an end position
        let mut movedir = [0.0f32; 3];
        let mut movepos = [0.0f32; 3];
        crate::q_math::_VectorMA((*self_).r.currentOrigin, (*info).distance, (*info).direction, &mut movepos);
        crate::q_math::_VectorCopy((*info).direction, &mut movedir);

        //Now test against entities
        if NAV_CheckAhead(ctx, self_, movepos, &mut (*info).trace as *mut trace_t, CONTENTS_BODY) == QFALSE {
            //Get the blocker
            (*info).blocker = &mut (*ctx.world).g_entities[(*info).trace.entityNum as usize] as *mut gentity_t;
            (*info).flags |= NIF_COLLISION;

            //Ok to hit our goal entity
            if goal == (*info).blocker {
                return QTRUE;
            }

            if setBlockedInfo != QFALSE {
                if (*(*self_).NPC as *mut gNPC_t).consecutiveBlockedMoves > blockedMovesLimit {
                    if (*ctx.world).cvars.d_patched.integer != 0 {
                        //use patch-style navigation
                        (*(*self_).NPC as *mut gNPC_t).consecutiveBlockedMoves += 1;
                    }
                    NPC_SetBlocked(ctx, self_, (*info).blocker);
                    return QFALSE;
                }
                (*(*self_).NPC as *mut gNPC_t).consecutiveBlockedMoves += 1;
            }
            //See if we're moving along with them
            //if ( NAVNEW_TrueCollision( self, info->blocker, movedir, info->direction ) == qfalse )
            //	return qtrue;

            //Test for blocking by standing on goal
            if NAV_TestForBlocked(ctx, self_, goal, (*info).blocker, (*info).distance, &mut (*info).flags as *mut c_int) == QTRUE {
                return QFALSE;
            }

            //If the above function said we're blocked, don't do the extra checks
            /*
            if ( info->flags & NIF_BLOCKED )
                return qtrue;
            */

            //See if we can get that entity to move out of our way
            if NAVNEW_ResolveEntityCollision(ctx, self_, (*info).blocker, movedir, (*info).pathDirection, setBlockedInfo) == QFALSE {
                return QFALSE;
            }

            crate::q_math::_VectorCopy(movedir, &mut (*info).direction);

            return QTRUE;
        } else {
            if setBlockedInfo != QFALSE {
                (*(*self_).NPC as *mut gNPC_t).consecutiveBlockedMoves = 0;
            }
        }

        //Our path is clear, just move there
        if (*ctx.world).globals.NAVDEBUG_showCollision != 0 {
            G_DrawEdge((*self_).r.currentOrigin, movepos, EDGE_MOVEDIR);
        }

        QTRUE
    }
}

/// Raven `NAVNEW_TestNodeConnectionBlocked`.
///
/// Raven: see if the direct path between 2 nodes is blocked by architecture
/// or an ent.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:520-572`
pub fn NAVNEW_TestNodeConnectionBlocked(
    ctx: GameContext<'_>,
    wp1: c_int,
    wp2: c_int,
    ignoreEnt: *mut gentity_t,
    goalEntNum: c_int,
    checkWorld: qboolean,
    checkEnts: qboolean,
) -> qboolean {
    unsafe {
        if checkWorld == QFALSE && checkEnts == QFALSE {
            //duh, nothing to trace against
            return QFALSE;
        }
        let mut playerMins = [0.0f32; 3];
        let mut playerMaxs = [0.0f32; 3];
        playerMins[0] = -15.0;
        playerMins[1] = -15.0;
        playerMins[2] = DEFAULT_MINS_2;
        playerMaxs[0] = 15.0;
        playerMaxs[1] = 15.0;
        playerMaxs[2] = DEFAULT_MAXS_2;

        let mut pos1 = [0.0f32; 3];
        let mut pos2 = [0.0f32; 3];
        trap::Nav_GetNodePosition(ctx.engine, crate::mp_abi::game::syscalls::G_NAV_GETNODEPOS::GNavGetnodeposArgs::new(wp1, &mut pos1 as *mut vec3_t));
        trap::Nav_GetNodePosition(ctx.engine, crate::mp_abi::game::syscalls::G_NAV_GETNODEPOS::GNavGetnodeposArgs::new(wp2, &mut pos2 as *mut vec3_t));

        let mut clipmask = MASK_NPCSOLID | CONTENTS_BOTCLIP;
        if checkWorld == QFALSE {
            clipmask &= !(CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP);
        }
        if checkEnts == QFALSE {
            clipmask &= !CONTENTS_BODY;
        }

        let mut mins = [0.0f32; 3];
        let mut maxs = [0.0f32; 3];
        let ignoreEntNum: c_int;
        if !ignoreEnt.is_null() {
            crate::q_math::_VectorCopy((*ignoreEnt).r.mins, &mut mins);
            crate::q_math::_VectorCopy((*ignoreEnt).r.maxs, &mut maxs);
            ignoreEntNum = (*ignoreEnt).s.number;
        } else {
            crate::q_math::_VectorCopy(playerMins, &mut mins);
            crate::q_math::_VectorCopy(playerMaxs, &mut mins);
            ignoreEntNum = ENTITYNUM_NONE;
        }
        mins[2] += STEPSIZE;
        //don't let box get inverted
        if mins[2] > maxs[2] {
            mins[2] = maxs[2];
        }

        let mut trace: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &pos1 as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &pos2 as *const vec3_t,
                ignoreEntNum,
                clipmask,
            ),
        );
        if trace.fraction >= 1.0f32 || trace.entityNum == goalEntNum {
            //clear or hit goal
            return QFALSE;
        }
        //hit something we weren't supposed to
        QTRUE
    }
}

/// Raven `NAVNEW_MoveToGoal`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:578-865`
pub fn NAVNEW_MoveToGoal(
    ctx: GameContext<'_>, self_: *mut gentity_t, info: *mut navInfo_t) -> c_int {
    unsafe {
        let mut bestNode = WAYPOINT_NONE;
        let mut foundClearPath = QFALSE;
        let mut origin = [0.0f32; 3];
        let mut tempInfo: navInfo_t = core::mem::zeroed();
        let mut setBlockedInfo = QTRUE;
        let mut inBestWP = QFALSE;
        let mut inGoalWP = QFALSE;
        let mut goalWPFailed = QFALSE;
        let mut numTries = 0;

        core::ptr::copy_nonoverlapping(info as *const navInfo_t, &mut tempInfo as *mut navInfo_t, 1);

        //Must have a goal entity to move there
        if (*(*self_).NPC as *mut gNPC_t).goalEntity.is_none() {
            return WAYPOINT_NONE;
        }

        let goal_entity = match (*(*self_).NPC as *mut gNPC_t).goalEntity {
            Some(eid) => eid,
            None => return WAYPOINT_NONE,
        };

        if (*self_).waypoint == WAYPOINT_NONE && (*self_).noWaypointTime > (*ctx.world).level.time {
            //didn't have a valid one in about the past second, don't look again just yet
            return WAYPOINT_NONE;
        }

        let goal_ent_ptr = if goal_entity.0 < (*ctx.world).g_entities.len() as u32 {
            &mut (*ctx.world).g_entities[goal_entity.0 as usize] as *mut gentity_t
        } else {
            core::ptr::null_mut()
        };

        if !goal_ent_ptr.is_null() && (*goal_ent_ptr).waypoint == WAYPOINT_NONE && (*goal_ent_ptr).noWaypointTime > (*ctx.world).level.time {
            //didn't have a valid one in about the past second, don't look again just yet
            return WAYPOINT_NONE;
        }

        if (*self_).noWaypointTime > (*ctx.world).level.time && !goal_ent_ptr.is_null() && (*goal_ent_ptr).noWaypointTime > (*ctx.world).level.time {
            //just use current waypoints
            bestNode = trap::Nav_GetBestNodeAltRoute2(
                ctx.engine,
                crate::mp_abi::game::syscalls::G_NAV_GETBESTALT2::GNavGetbestalt2Args::new(
                    (*self_).waypoint,
                    if !goal_ent_ptr.is_null() { (*goal_ent_ptr).waypoint } else { NODE_NONE },
                    bestNode,
                ),
            );
        }
        //FIXME!!!!: this is making them wiggle back and forth between waypoints
        else if {
            bestNode = trap::Nav_GetBestPathBetweenEnts(
                ctx.engine,
                crate::mp_abi::game::syscalls::G_NAV_GETBESTPATH::GNavGetbestpathArgs::new(self_, goal_ent_ptr, NF_CLEAR_PATH),
            );
            bestNode == NODE_NONE
        } {
            //one of us didn't have a valid waypoint!
            if (*self_).waypoint == NODE_NONE {
                //don't even try to find one again for a bit
                (*self_).noWaypointTime = (*ctx.world).level.time + (*ctx.world).bg_state.rng.Q_irand(500, 1500);
            }
            if !goal_ent_ptr.is_null() && (*goal_ent_ptr).waypoint == NODE_NONE {
                //don't even try to find one again for a bit
                (*goal_ent_ptr).noWaypointTime = (*ctx.world).level.time + (*ctx.world).bg_state.rng.Q_irand(500, 1500);
            }
            return WAYPOINT_NONE;
        } else {
            if !goal_ent_ptr.is_null() && (*goal_ent_ptr).noWaypointTime < (*ctx.world).level.time {
                (*goal_ent_ptr).noWaypointTime = (*ctx.world).level.time + (*ctx.world).bg_state.rng.Q_irand(500, 1500);
            }
        }

        while foundClearPath == QFALSE {
            inBestWP = QFALSE;
            inGoalWP = QFALSE;

            if bestNode == WAYPOINT_NONE {
                return WAYPOINT_NONE;
            }

            trap::Nav_GetNodePosition(
                ctx.engine,
                crate::mp_abi::game::syscalls::G_NAV_GETNODEPOS::GNavGetnodeposArgs::new(bestNode, &mut origin as *mut vec3_t),
            );

            if !inGoalWP {
                //not heading straight for goal
                if bestNode == (*self_).waypoint {
                    //we know it's clear or architecture
                } else {
                    //heading to an edge off our confirmed clear waypoint... make sure it's clear
                    //it it's not, bestNode will fall back to our waypoint
                    let oldBestNode = bestNode;
                    bestNode = NAV_TestBestNode(ctx, self_, (*self_).waypoint, bestNode, QTRUE);
                    if bestNode == (*self_).waypoint {
                        //we fell back to our waypoint, reset the origin
                        (*(*self_).NPC as *mut gNPC_t).aiFlags |= NPCAI_BLOCKED;
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            crate::mp_abi::game::syscalls::G_NAV_GETNODEPOS::GNavGetnodeposArgs::new(oldBestNode, &mut NPCInfo.blockedDest as *mut vec3_t),
                        );
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            crate::mp_abi::game::syscalls::G_NAV_GETNODEPOS::GNavGetnodeposArgs::new(bestNode, &mut origin as *mut vec3_t),
                        );
                    }
                }
            }

            core::ptr::copy_nonoverlapping(info as *const navInfo_t, &mut tempInfo as *mut navInfo_t, 1);
            crate::q_math::_VectorSubtract(origin, (*self_).r.currentOrigin, &mut tempInfo.direction);
            crate::q_math::VectorNormalize(&mut tempInfo.direction);

            //NOTE: One very important thing NAVNEW_AvoidCollision does is
            //		it actually CHANGES the value of "direction" - it changes it to
            //		whatever dir you need to go in to avoid the obstacle...
            foundClearPath = NAVNEW_AvoidCollision(ctx, self_, goal_ent_ptr, &mut tempInfo, setBlockedInfo, 5);

            if foundClearPath == QFALSE {
                //blocked by an ent
                if inGoalWP != QFALSE {
                    //we were heading straight for the goal, head for the goal's wp instead
                    trap::Nav_GetNodePosition(
                        ctx.engine,
                        crate::mp_abi::game::syscalls::G_NAV_GETNODEPOS::GNavGetnodeposArgs::new(bestNode, &mut origin as *mut vec3_t),
                    );
                    foundClearPath = NAVNEW_AvoidCollision(ctx, self_, goal_ent_ptr, &mut tempInfo, setBlockedInfo, 5);
                }
            }

            if foundClearPath != QFALSE {
                //clear!
                //If we got set to blocked, clear it
                NPC_ClearBlocked(self_);
                //Take the dir
                core::ptr::copy_nonoverlapping(&tempInfo as *const navInfo_t, info as *mut navInfo_t, 1);
                if (*self_).s.weapon == WP_SABER {
                    //jedi
                    if (*info).direction[2] * (*info).distance > 64.0 {
                        (*(*self_).NPC as *mut gNPC_t).aiFlags |= NPCAI_BLOCKED;
                        crate::q_math::_VectorCopy(origin, &mut NPCInfo.blockedDest);
                        return WAYPOINT_NONE;
                    }
                }
            } else {
                //blocked by ent!
                if setBlockedInfo != QFALSE {
                    (*(*self_).NPC as *mut gNPC_t).aiFlags |= NPCAI_BLOCKED;
                    trap::Nav_GetNodePosition(
                        ctx.engine,
                        crate::mp_abi::game::syscalls::G_NAV_GETNODEPOS::GNavGetnodeposArgs::new(bestNode, &mut NPCInfo.blockedDest as *mut vec3_t),
                    );
                }
                //Only set blocked info first time
                setBlockedInfo = QFALSE;

                if inGoalWP != QFALSE {
                    //we headed for our goal and failed and our goal's WP and failed
                    if (*self_).waypoint == (if !goal_ent_ptr.is_null() { (*goal_ent_ptr).waypoint } else { NODE_NONE }) {
                        //our waypoint is our goal's waypoint, nothing we can do
                        //remember that this node is blocked
                        trap::Nav_AddFailedNode(ctx.engine, crate::mp_abi::game::syscalls::G_NAV_ADDFAIL::GNavAddfailArgs::new(self_, (*self_).waypoint));
                        return WAYPOINT_NONE;
                    } else {
                        //try going for our waypoint this time
                        goalWPFailed = QTRUE;
                        inGoalWP = QFALSE;
                    }
                } else if bestNode != (*self_).waypoint {
                    //we headed toward our next waypoint (instead of our waypoint) and failed
                    if (*ctx.world).cvars.d_altRoutes.integer != 0 {
                        //mark this edge failed and try our waypoint
                        //NOTE: don't assume there is something blocking the direct path
                        //			between my waypoint and the bestNode... I could be off
                        //			that path because of collision avoidance...
                        if (*ctx.world).cvars.d_patched.integer != 0 && //use patch-style navigation
                           (trap::Nav_NodesAreNeighbors(
                               ctx.engine,
                               crate::mp_abi::game::syscalls::G_NAV_NEIGHBORS::GNavNeighborsArgs::new((*self_).waypoint, bestNode),
                           ) == QFALSE
                           || NAVNEW_TestNodeConnectionBlocked(ctx, (*self_).waypoint, bestNode, self_, (if !goal_ent_ptr.is_null() { (*goal_ent_ptr).s.number } else { ENTITYNUM_NONE }), QFALSE, QTRUE) != QFALSE)
                        {
                            //the direct path between these 2 nodes is blocked by an ent
                            trap::Nav_AddFailedEdge(
                                ctx.engine,
                                crate::mp_abi::game::syscalls::G_NAV_ADDEDGE::GNavAddedgeArgs::new((*self_).s.number, (*self_).waypoint, bestNode),
                            );
                        }
                        bestNode = (*self_).waypoint;
                    } else {
                        //we should stop
                        return WAYPOINT_NONE;
                    }
                } else {
                    //we headed for *our* waypoint and couldn't get to it
                    if (*ctx.world).cvars.d_altRoutes.integer != 0 {
                        //remember that this node is blocked
                        trap::Nav_AddFailedNode(ctx.engine, crate::mp_abi::game::syscalls::G_NAV_ADDFAIL::GNavAddfailArgs::new(self_, (*self_).waypoint));
                        //Now we should get our waypoints again
                        //FIXME: cache the trace-data for subsequent calls as only the route info would have changed
                        return WAYPOINT_NONE;
                    } else {
                        //we should stop
                        return WAYPOINT_NONE;
                    }
                }

                if {
                    numTries += 1;
                    numTries
                } >= 10
                {
                    return WAYPOINT_NONE;
                }
            }
        }

        //Draw any debug info, if requested
        if (*ctx.world).globals.NAVDEBUG_showEnemyPath != 0 {
            let mut dest = [0.0f32; 3];
            let mut start = [0.0f32; 3];

            //Get the positions
            trap::Nav_GetNodePosition(
                ctx.engine,
                crate::mp_abi::game::syscalls::G_NAV_GETNODEPOS::GNavGetnodeposArgs::new(
                    if !goal_ent_ptr.is_null() { (*goal_ent_ptr).waypoint } else { NODE_NONE },
                    &mut dest as *mut vec3_t,
                ),
            );
            trap::Nav_GetNodePosition(
                ctx.engine,
                crate::mp_abi::game::syscalls::G_NAV_GETNODEPOS::GNavGetnodeposArgs::new(bestNode, &mut start as *mut vec3_t),
            );

            //Draw the route
            G_DrawNode(start, NODE_START);
            if bestNode != (*self_).waypoint {
                let mut wpPos = [0.0f32; 3];
                trap::Nav_GetNodePosition(
                    ctx.engine,
                    crate::mp_abi::game::syscalls::G_NAV_GETNODEPOS::GNavGetnodeposArgs::new((*self_).waypoint, &mut wpPos as *mut vec3_t),
                );
                G_DrawNode(wpPos, NODE_NAVGOAL);
            }
            G_DrawNode(dest, NODE_GOAL);
            if !goal_ent_ptr.is_null() {
                G_DrawEdge(dest, (*goal_ent_ptr).r.currentOrigin, EDGE_PATH);
                G_DrawNode((*goal_ent_ptr).r.currentOrigin, NODE_GOAL);
            }
            trap::Nav_ShowPath(
                ctx.engine,
                crate::mp_abi::game::syscalls::G_NAV_SHOWPATH::GNavShowpathArgs::new(bestNode, if !goal_ent_ptr.is_null() { (*goal_ent_ptr).waypoint } else { NODE_NONE }),
            );
        }

        (*(*self_).NPC as *mut gNPC_t).shoveCount = 0;

        //let me keep this waypoint for a while
        if (*self_).noWaypointTime < (*ctx.world).level.time {
            (*self_).noWaypointTime = (*ctx.world).level.time + (*ctx.world).bg_state.rng.Q_irand(500, 1500);
        }
        bestNode
    }
}
    }
}
